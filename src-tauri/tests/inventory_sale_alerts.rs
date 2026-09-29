use repuestos_autos::application::inventory::list_inventory_alerts;
use repuestos_autos::domain::inventory::AlertClassification;
use repuestos_autos::commands::confirm_sale::{
    confirm_sale, ConfirmSaleRequest, PaymentInputRequest, RequestedLine,
};
use repuestos_autos::infrastructure::sqlite::{open_seeded_catalog, SqliteInventoryRepository};

#[test]
fn inventory_alert_listing_uses_persisted_threshold_and_keeps_zero_out_of_stock() {
    let mut connection = open_seeded_catalog().unwrap();
    connection
        .execute(
            "UPDATE products SET low_stock_threshold = 3 WHERE id = 1",
            [],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE stock_balances SET quantity = 2 WHERE product_id = 1",
            [],
        )
        .unwrap();

    let repository = SqliteInventoryRepository::new(&mut connection);
    let low_stock_alert = list_inventory_alerts(&repository)
        .unwrap()
        .into_iter()
        .find(|alert| alert.product_id == 1)
        .unwrap();
    assert_eq!(low_stock_alert.quantity, 2);
    assert_eq!(low_stock_alert.classification, AlertClassification::LowStock);

    connection
        .execute(
            "UPDATE stock_balances SET quantity = 0 WHERE product_id = 1",
            [],
        )
        .unwrap();
    let repository = SqliteInventoryRepository::new(&mut connection);
    let out_of_stock_alert = list_inventory_alerts(&repository)
        .unwrap()
        .into_iter()
        .find(|alert| alert.product_id == 1)
        .unwrap();
    assert_eq!(out_of_stock_alert.quantity, 0);
    assert_eq!(out_of_stock_alert.classification, AlertClassification::OutOfStock);
}

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
                captured_unit_price_centavos: 2_500,
                captured_revision: 0,
                final_unit_price_centavos: None,
                acknowledged_price_centavos: None,
                acknowledged_revision: None,
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
