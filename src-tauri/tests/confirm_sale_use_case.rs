use repuestos_autos::application::sales::{confirm_sale, ConfirmSaleRequest, RequestedLine};
use repuestos_autos::catalog::open_seeded_catalog;
use repuestos_autos::domain::sales::Payment;
use repuestos_autos::domain::{MoneyCentavos, Quantity, RequestId};

fn request() -> ConfirmSaleRequest {
    ConfirmSaleRequest {
        request_id: RequestId::parse("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        lines: vec![(1, 2, 2_500), (3, 1, 3_000)]
            .into_iter()
            .map(|(product_id, quantity, price)| RequestedLine {
                product_id,
                quantity: Quantity::new(quantity).unwrap(),
                negotiated_unit_price: MoneyCentavos::new(price).unwrap(),
            })
            .collect(),
        payments: vec![Payment::cash(
            MoneyCentavos::new(8_000).unwrap(),
            MoneyCentavos::new(9_000).unwrap(),
            MoneyCentavos::new(1_000).unwrap(),
        )
        .unwrap()],
    }
}

#[test]
fn confirms_a_multi_line_cash_sale_with_persisted_stock_movements_and_summary() {
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute("INSERT INTO products (id, category_id, sku, name, active, minimum_unit_price_centavos) VALUES (3, 1, 'FLT-002', 'Filtro de aire', 1, 3000)", []).unwrap();
    connection
        .execute(
            "INSERT INTO stock_balances (product_id, quantity) VALUES (3, 4)",
            [],
        )
        .unwrap();
    let sale = confirm_sale(&mut connection, request()).unwrap();

    assert_eq!(sale.total, MoneyCentavos::new(8_000).unwrap());
    assert_eq!(sale.lines.len(), 2);
    assert_eq!(
        sale.lines[0].minimum_unit_price_snapshot,
        MoneyCentavos::new(2_500).unwrap()
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT quantity FROM stock_balances WHERE product_id = 1",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        6
    );
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM inventory_movements", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        2
    );
}

#[test]
fn rolls_back_every_effect_when_a_later_line_has_insufficient_stock() {
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute("INSERT INTO products (id, category_id, sku, name, active, minimum_unit_price_centavos) VALUES (3, 1, 'FLT-002', 'Filtro de aire', 1, 3000)", []).unwrap();
    connection
        .execute(
            "INSERT INTO stock_balances (product_id, quantity) VALUES (3, 0)",
            [],
        )
        .unwrap();

    assert_eq!(
        confirm_sale(&mut connection, request()),
        Err("insufficient stock".into())
    );
    for table in [
        "sales",
        "sale_lines",
        "sale_payments",
        "inventory_movements",
    ] {
        assert_eq!(
            connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    assert_eq!(
        connection
            .query_row(
                "SELECT quantity FROM stock_balances WHERE product_id = 1",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        8
    );
}

#[test]
fn returns_the_original_summary_without_reapplying_a_changed_retry() {
    let mut connection = open_seeded_catalog().unwrap();
    let first = confirm_sale(
        &mut connection,
        ConfirmSaleRequest {
            request_id: RequestId::parse("550e8400-e29b-41d4-a716-446655440001").unwrap(),
            lines: vec![RequestedLine {
                product_id: 1,
                quantity: Quantity::new(1).unwrap(),
                negotiated_unit_price: MoneyCentavos::new(2_500).unwrap(),
            }],
            payments: vec![Payment::qr(MoneyCentavos::new(2_500).unwrap())],
        },
    )
    .unwrap();
    let retry = confirm_sale(
        &mut connection,
        ConfirmSaleRequest {
            request_id: RequestId::parse("550e8400-e29b-41d4-a716-446655440001").unwrap(),
            lines: vec![RequestedLine {
                product_id: 1,
                quantity: Quantity::new(2).unwrap(),
                negotiated_unit_price: MoneyCentavos::new(9_999).unwrap(),
            }],
            payments: vec![Payment::qr(MoneyCentavos::new(19_998).unwrap())],
        },
    )
    .unwrap();

    assert_eq!(retry, first);
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM sales", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM sale_lines", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM inventory_movements", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT quantity FROM stock_balances WHERE product_id = 1",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        7
    );
}

#[test]
fn reports_persistence_integrity_for_an_incomplete_reserved_sale() {
    let mut connection = open_seeded_catalog().unwrap();
    connection
        .execute(
            "INSERT INTO sales (request_id, status, total_centavos) VALUES (?1, 'pending', 0)",
            ["550e8400-e29b-41d4-a716-446655440000"],
        )
        .unwrap();

    assert_eq!(
        confirm_sale(&mut connection, request()),
        Err("persistence integrity failure".into())
    );
}
