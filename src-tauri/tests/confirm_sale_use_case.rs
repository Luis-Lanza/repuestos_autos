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
