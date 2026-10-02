use repuestos_autos::application::catalog::access::CatalogAccessSession;
use repuestos_autos::infrastructure::filesystem::catalog_access::CatalogAccessStore;
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
        unit_purchase_price_centavos: Some(1_250),
        sale_price_centavos: None,
        minimum_sale_price_centavos: None,
        note: Some("delivery".into()),
    }
}

#[test]
fn physical_count_request_debug_redacts_catalog_password() {
    let request: PhysicalCountRequest = serde_json::from_value(serde_json::json!({
        "request_id": "550e8400-e29b-41d4-a716-446655440250",
        "product_id": 1,
        "count": 7,
        "reason": "counted",
        "catalog_password": "plaintext-secret"
    }))
    .unwrap();

    let debug = format!("{request:?}");
    assert!(!debug.contains("plaintext-secret"));
    assert!(debug.contains("[REDACTED]"));
}

#[test]
fn omitted_stock_entry_prices_are_accepted_and_preserved() {
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute("UPDATE products SET purchase_price_centavos = NULL WHERE id = 1", []).unwrap();
    let request: StockEntryRequest = serde_json::from_value(serde_json::json!({
        "request_id": "550e8400-e29b-41d4-a716-446655440230",
        "product_id": 1,
        "quantity": 2,
        "note": null
    })).unwrap();
    assert!(matches!(confirm_stock_entry_command(&mut connection, request).unwrap(), InventoryCommandResponse::Success(_)));
    let prices = connection.query_row("SELECT purchase_price_centavos, list_price_centavos, minimum_unit_price_centavos FROM products WHERE id = 1", [], |row| Ok((row.get::<_, Option<i64>>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?))).unwrap();
    assert_eq!(prices, (None, 2_500, 2_500));
}

#[test]
fn physical_count_command_requires_current_catalog_password_before_mutation() {
    let directory = std::env::temp_dir().join(format!("inventory-auth-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&directory).unwrap();
    let access = CatalogAccessSession::open(CatalogAccessStore::new(&directory));
    access.begin_setup("old-password").unwrap();
    access.finish_setup(true).unwrap();
    let mut connection = open_seeded_catalog().unwrap();
    let before_movements: i64 = connection.query_row("SELECT COUNT(*) FROM inventory_movements", [], |row| row.get(0)).unwrap();
    for password in [String::new(), "wrong-password".into()] {
        let response = confirm_physical_count_command(&mut connection, &access, PhysicalCountRequest {
            request_id: "550e8400-e29b-41d4-a716-446655440240".into(), product_id: 1, count: 7,
            reason: "counted".into(), catalog_password: password,
        }).unwrap();
        assert!(matches!(response, InventoryCommandResponse::Error(ref error) if error.code == "catalog_password_invalid"));
        assert_eq!(connection.query_row("SELECT quantity FROM stock_balances WHERE product_id = 1", [], |row| row.get::<_, i64>(0)).unwrap(), 8);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM inventory_movements", [], |row| row.get::<_, i64>(0)).unwrap(), before_movements);
    }
    assert!(matches!(confirm_physical_count_command(&mut connection, &access, PhysicalCountRequest {
        request_id: "550e8400-e29b-41d4-a716-446655440241".into(), product_id: 1, count: 7,
        reason: "counted".into(), catalog_password: "old-password".into(),
    }).unwrap(), InventoryCommandResponse::Success(_)));
    access.change_password("old-password", "new-password").unwrap();
    for (password, count) in [("old-password", 6), ("new-password", 6)] {
        let response = confirm_physical_count_command(&mut connection, &access, PhysicalCountRequest {
            request_id: if password == "old-password" { "550e8400-e29b-41d4-a716-446655440242" } else { "550e8400-e29b-41d4-a716-446655440243" }.into(),
            product_id: 1, count, reason: "counted".into(), catalog_password: password.into(),
        }).unwrap();
        if password == "old-password" {
            assert!(matches!(response, InventoryCommandResponse::Error(ref error) if error.code == "catalog_password_invalid"));
            assert_eq!(connection.query_row("SELECT quantity FROM stock_balances WHERE product_id = 1", [], |row| row.get::<_, i64>(0)).unwrap(), 7);
        } else {
            assert!(matches!(response, InventoryCommandResponse::Success(_)));
            assert_eq!(connection.query_row("SELECT quantity FROM stock_balances WHERE product_id = 1", [], |row| row.get::<_, i64>(0)).unwrap(), 6);
        }
    }
    drop(access);
    std::fs::remove_dir_all(directory).unwrap();
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
    assert_eq!(first.note.as_deref(), Some("delivery"));
    let InventoryCommandResponse::Error(error) = confirm_stock_entry_command(
        &mut connection,
        StockEntryRequest {
            quantity: 99,
            ..entry("550e8400-e29b-41d4-a716-446655440201")
        },
    )
    .unwrap() else {
        panic!("expected a stable request conflict")
    };
    assert_eq!(
        (error.code, error.message),
        (
            "request_conflict",
            "This request ID was already used with different inventory data."
        )
    );
    let auth_directory = std::env::temp_dir().join(format!("inventory-invalid-request-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&auth_directory).unwrap();
    let access = CatalogAccessSession::open(CatalogAccessStore::new(&auth_directory));
    access.begin_setup("valid-password").unwrap();
    access.finish_setup(true).unwrap();
    let InventoryCommandResponse::Error(error) = confirm_physical_count_command(
        &mut connection,
        &access,
        PhysicalCountRequest {
            request_id: "bad".into(),
            product_id: 1,
            count: 7,
            reason: "counted".into(),
            catalog_password: "valid-password".into(),
        },
    )
    .unwrap() else {
        panic!("expected error")
    };
    assert_eq!(
        (error.code, error.message),
        ("invalid_request", "The request shape is invalid.")
    );
    drop(access);
    std::fs::remove_dir_all(auth_directory).unwrap();
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
