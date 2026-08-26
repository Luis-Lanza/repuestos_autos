use repuestos_autos::commands::inventory::{
    confirm_physical_count_command, confirm_stock_entry_command, list_inventory_alerts_command,
    InventoryCommandResponse, PhysicalCountRequest, StockEntryRequest,
};
use repuestos_autos::infrastructure::sqlite::open_seeded_catalog;

fn entry(request_id: &str) -> StockEntryRequest {
    StockEntryRequest {
        request_id: request_id.into(),
        product_id: 1,
        quantity: 2,
        note: Some("delivery".into()),
    }
}

#[test]
fn inventory_commands_preserve_persisted_results_and_only_expose_stable_errors() {
    let mut connection = open_seeded_catalog().unwrap();
    let InventoryCommandResponse::Success(first) = confirm_stock_entry_command(
        &mut connection,
        entry("550e8400-e29b-41d4-a716-446655440201"),
    )
    .unwrap() else {
        panic!("expected success")
    };
    assert_eq!(
        (
            first.request_id.as_str(),
            first.previous_quantity,
            first.resulting_quantity
        ),
        ("550e8400-e29b-41d4-a716-446655440201", 8, 10)
    );
    let retry = confirm_stock_entry_command(
        &mut connection,
        StockEntryRequest {
            quantity: 99,
            ..entry("550e8400-e29b-41d4-a716-446655440201")
        },
    )
    .unwrap();
    assert_eq!(retry, InventoryCommandResponse::Success(first));
    let InventoryCommandResponse::Error(error) = confirm_physical_count_command(
        &mut connection,
        PhysicalCountRequest {
            request_id: "bad".into(),
            product_id: 1,
            count: 7,
            reason: "counted".into(),
        },
    )
    .unwrap() else {
        panic!("expected error")
    };
    assert_eq!(
        (error.code, error.message),
        ("invalid_request", "The request shape is invalid.")
    );
    connection.execute_batch("UPDATE products SET active = 1 WHERE id = 2; UPDATE stock_balances SET quantity = 0 WHERE product_id = 1; UPDATE stock_balances SET quantity = 1 WHERE product_id = 2;").unwrap();
    let InventoryCommandResponse::Alerts(alerts) =
        list_inventory_alerts_command(&mut connection).unwrap()
    else {
        panic!("expected alerts")
    };
    assert_eq!(
        alerts
            .alerts
            .iter()
            .map(|alert| alert.product_id)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
}
