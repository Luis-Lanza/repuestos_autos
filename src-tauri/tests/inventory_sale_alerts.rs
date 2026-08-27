use repuestos_autos::application::inventory::list_inventory_alerts;
use repuestos_autos::commands::confirm_sale::{
    confirm_sale, ConfirmSaleRequest, PaymentInputRequest, RequestedLine,
};
use repuestos_autos::infrastructure::sqlite::{open_seeded_catalog, SqliteInventoryRepository};

#[test]
fn confirmed_sale_immediately_surfaces_a_derived_low_stock_alert() {
    let mut connection = open_seeded_catalog().unwrap();
    connection
        .execute(
            "UPDATE stock_balances SET quantity = 2 WHERE product_id = 1",
            [],
        )
        .unwrap();
    confirm_sale(
        &mut connection,
        ConfirmSaleRequest {
            request_id: "550e8400-e29b-41d4-a716-446655440301".into(),
            lines: vec![RequestedLine {
                product_id: 1,
                quantity: 1,
            }],
            payment: PaymentInputRequest {
                amount_tendered_centavos: None,
                qr_applied_centavos: Some(2_500),
            },
        },
    )
    .unwrap();
    let repository = SqliteInventoryRepository::new(&mut connection);
    assert_eq!(list_inventory_alerts(&repository).unwrap()[0].quantity, 1);
}
