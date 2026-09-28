use repuestos_autos::application::sales::{
    ApplicationConfirmSaleRequest, ApplicationRequestedLine, ConfirmSaleUseCase,
};
use repuestos_autos::domain::sales::{Payment, PaymentInput};
use repuestos_autos::domain::{MoneyCentavos, Quantity, RequestId};
use repuestos_autos::infrastructure::sqlite::open_seeded_catalog;
use repuestos_autos::infrastructure::sqlite::sale_repository::SqliteSaleRepository;

fn confirm_request(request_id: &str, unit_price: i64) -> ApplicationConfirmSaleRequest {
    ApplicationConfirmSaleRequest {
        request_id: RequestId::parse(request_id).unwrap(),
        lines: vec![ApplicationRequestedLine {
            product_id: 1,
            quantity: Quantity::new(1).unwrap(),
            captured_unit_price: MoneyCentavos::new(unit_price).unwrap(),
            captured_revision: 0,
            final_unit_price: None,
            acknowledged_price: None,
            acknowledged_revision: None,
        }],
        payment: PaymentInput {
            amount_tendered: None,
            qr_applied: Some(MoneyCentavos::new(unit_price).unwrap()),
        },
    }
}

#[test]
fn confirmation_snapshots_purchase_cost_and_cost_edits_cannot_rewrite_history() {
    let mut connection = open_seeded_catalog().unwrap();
    connection
        .execute(
            "UPDATE products SET purchase_price_centavos = 1_250 WHERE id = 1",
            [],
        )
        .unwrap();

    let sale = ConfirmSaleUseCase::new(&mut connection, &SqliteSaleRepository)
        .confirm(confirm_request(
            "550e8400-e29b-41d4-a716-446655440201",
            2_500,
        ))
        .unwrap();

    assert_eq!(
        connection
            .query_row(
                "SELECT unit_cost_snapshot_centavos FROM sale_lines WHERE sale_id = ?1",
                [sale.sale_id],
                |row| row.get::<_, Option<i64>>(0),
            )
            .unwrap(),
        Some(1_250),
    );
    connection
        .execute(
            "UPDATE products SET purchase_price_centavos = 1_500 WHERE id = 1",
            [],
        )
        .unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT unit_cost_snapshot_centavos FROM sale_lines WHERE sale_id = ?1",
                [sale.sale_id],
                |row| row.get::<_, Option<i64>>(0),
            )
            .unwrap(),
        Some(1_250),
    );
    assert!(connection
        .execute(
            "UPDATE sale_lines SET unit_cost_snapshot_centavos = 1_500 WHERE sale_id = ?1",
            [sale.sale_id],
        )
        .is_err());
}

#[test]
fn confirmation_preserves_unknown_cost_when_product_has_no_purchase_price() {
    let mut connection = open_seeded_catalog().unwrap();

    let sale = ConfirmSaleUseCase::new(&mut connection, &SqliteSaleRepository)
        .confirm(confirm_request(
            "550e8400-e29b-41d4-a716-446655440202",
            2_500,
        ))
        .unwrap();

    assert_eq!(
        connection
            .query_row(
                "SELECT unit_cost_snapshot_centavos FROM sale_lines WHERE sale_id = ?1",
                [sale.sale_id],
                |row| row.get::<_, Option<i64>>(0),
            )
            .unwrap(),
        None,
    );
}

#[test]
fn direct_sale_confirmation_also_captures_cost_inside_its_transaction() {
    use repuestos_autos::application::sales::{confirm_sale, ConfirmSaleRequest, RequestedLine};

    let mut connection = open_seeded_catalog().unwrap();
    connection
        .execute(
            "UPDATE products SET purchase_price_centavos = 900 WHERE id = 1",
            [],
        )
        .unwrap();
    let sale = confirm_sale(
        &mut connection,
        ConfirmSaleRequest {
            request_id: RequestId::parse("550e8400-e29b-41d4-a716-446655440203").unwrap(),
            lines: vec![RequestedLine {
                product_id: 1,
                quantity: Quantity::new(1).unwrap(),
                negotiated_unit_price: MoneyCentavos::new(2_500).unwrap(),
            }],
            payments: vec![Payment::qr(MoneyCentavos::new(2_500).unwrap())],
        },
    )
    .unwrap();

    assert_eq!(
        connection
            .query_row(
                "SELECT unit_cost_snapshot_centavos FROM sale_lines WHERE sale_id = ?1",
                [sale.sale_id],
                |row| row.get::<_, Option<i64>>(0),
            )
            .unwrap(),
        Some(900),
    );
}
