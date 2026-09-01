use std::time::Instant;

use repuestos_autos::application::inventory::InventoryRepository;
use repuestos_autos::domain::inventory::{InventoryError, InventoryOperation};
use repuestos_autos::domain::RequestId;
use repuestos_autos::infrastructure::sqlite::{
    open_database, open_seeded_catalog, production_database_config, SqliteInventoryRepository,
};

fn request(value: &str) -> RequestId {
    RequestId::parse(value).unwrap()
}

fn scalar(connection: &rusqlite::Connection, query: &str) -> i64 {
    connection.query_row(query, [], |row| row.get(0)).unwrap()
}

#[test]
fn stock_entry_updates_balance_once_and_persists_an_immutable_movement() {
    let mut connection = open_seeded_catalog().unwrap();
    let result = SqliteInventoryRepository::new(&mut connection)
        .confirm(
            InventoryOperation::stock_entry(
                1,
                request("550e8400-e29b-41d4-a716-446655440103"),
                2,
                Some("delivery".into()),
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(result.resulting_quantity, 10);
    assert_eq!(result.note.as_deref(), Some("delivery"));
    assert_eq!(
        SqliteInventoryRepository::new(&mut connection)
            .confirm(
                InventoryOperation::stock_entry(
                    1,
                    request("550e8400-e29b-41d4-a716-446655440103"),
                    99,
                    None,
                )
                .unwrap(),
            )
            .unwrap()
            .note
            .as_deref(),
        Some("delivery")
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM inventory_movements WHERE request_id IS NOT NULL",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert!(connection
        .execute(
            "UPDATE inventory_movements SET quantity_delta = 1 WHERE request_id IS NOT NULL",
            []
        )
        .is_err());
}

#[test]
fn adjustment_uses_current_balance_and_invalid_requests_leave_no_movement() {
    let mut connection = open_seeded_catalog().unwrap();
    connection
        .execute(
            "UPDATE stock_balances SET quantity = 10 WHERE product_id = 1",
            [],
        )
        .unwrap();
    let mut repository = SqliteInventoryRepository::new(&mut connection);
    let result = repository
        .confirm(
            InventoryOperation::physical_count(
                1,
                request("550e8400-e29b-41d4-a716-446655440104"),
                7,
                "counted",
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(result.quantity_delta, -3);
    assert_eq!(
        repository.confirm(
            InventoryOperation::physical_count(
                1,
                request("550e8400-e29b-41d4-a716-446655440105"),
                7,
                "counted"
            )
            .unwrap()
        ),
        Err(InventoryError::UNCHANGED_COUNT)
    );
    assert_eq!(
        repository.confirm(
            InventoryOperation::stock_entry(
                2,
                request("550e8400-e29b-41d4-a716-446655440106"),
                1,
                None
            )
            .unwrap()
        ),
        Err(InventoryError::INACTIVE_PRODUCT)
    );
    connection
        .execute(
            "UPDATE stock_balances SET quantity = 9223372036854775807 WHERE product_id = 1",
            [],
        )
        .unwrap();
    assert_eq!(
        SqliteInventoryRepository::new(&mut connection).confirm(
            InventoryOperation::stock_entry(
                1,
                request("550e8400-e29b-41d4-a716-446655440108"),
                1,
                None
            )
            .unwrap()
        ),
        Err(InventoryError::QUANTITY_OVERFLOW)
    );
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM inventory_movements", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn retry_returns_original_result_and_alerts_are_active_ordered_and_indexed() {
    let directory = std::env::temp_dir().join(format!("inventory-sqlite-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    let config = production_database_config(&directory);
    let mut connection = open_database(&config).unwrap();
    let operation = InventoryOperation::stock_entry(
        1,
        request("550e8400-e29b-41d4-a716-446655440107"),
        1,
        None,
    )
    .unwrap();
    let first = SqliteInventoryRepository::new(&mut connection)
        .confirm(operation.clone())
        .unwrap();
    drop(connection);
    let mut connection = open_database(&config).unwrap();
    let retry = SqliteInventoryRepository::new(&mut connection)
        .confirm(
            InventoryOperation::stock_entry(
                1,
                request("550e8400-e29b-41d4-a716-446655440107"),
                99,
                None,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(first, retry);
    connection.execute_batch("UPDATE products SET active = 1 WHERE id = 2; UPDATE stock_balances SET quantity = 0 WHERE product_id = 1; UPDATE stock_balances SET quantity = 1 WHERE product_id = 2;").unwrap();
    let started = Instant::now();
    let alerts = SqliteInventoryRepository::new(&mut connection)
        .list_alerts()
        .unwrap();
    assert!(started.elapsed().as_millis() < 100);
    assert_eq!(
        alerts
            .iter()
            .map(|alert| alert.product_id)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    let plan: String = connection
        .query_row(
            "EXPLAIN QUERY PLAN SELECT id FROM inventory_movements WHERE request_id = ?1",
            ["550e8400-e29b-41d4-a716-446655440107"],
            |row| row.get(3),
        )
        .unwrap();
    assert!(plan.contains("inventory_movements_request_id_idx"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn post_insert_balance_failure_rolls_back_the_inventory_operation() {
    let mut connection = open_seeded_catalog().unwrap();
    connection
        .execute_batch("CREATE TRIGGER reject_inventory_balance_update BEFORE UPDATE ON stock_balances WHEN new.product_id = 1 BEGIN SELECT RAISE(ABORT, 'forced failure'); END;")
        .unwrap();
    let operation = InventoryOperation::stock_entry(
        1,
        request("550e8400-e29b-41d4-a716-446655440109"),
        2,
        None,
    )
    .unwrap();
    assert_eq!(
        SqliteInventoryRepository::new(&mut connection).confirm(operation.clone()),
        Err(InventoryError::PERSISTENCE_FAILURE)
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT quantity FROM stock_balances WHERE product_id = 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        8
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM inventory_movements WHERE request_id IS NOT NULL",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        0
    );
    connection
        .execute_batch("DROP TRIGGER reject_inventory_balance_update;")
        .unwrap();
    assert_eq!(
        SqliteInventoryRepository::new(&mut connection)
            .confirm(operation)
            .unwrap()
            .resulting_quantity,
        10
    );
}

#[test]
fn archived_categories_exclude_active_products_from_operations_and_alerts_without_mutation() {
    let mut connection = open_seeded_catalog().unwrap();
    connection
        .execute_batch("UPDATE categories SET active = 0 WHERE id = 1")
        .unwrap();
    let balance = scalar(
        &connection,
        "SELECT quantity FROM stock_balances WHERE product_id = 1",
    );
    let operation = InventoryOperation::stock_entry(
        1,
        request("550e8400-e29b-41d4-a716-446655440110"),
        1,
        None,
    )
    .unwrap();
    assert_eq!(
        SqliteInventoryRepository::new(&mut connection).confirm(operation),
        Err(InventoryError::INACTIVE_PRODUCT)
    );
    assert_eq!(
        scalar(
            &connection,
            "SELECT quantity FROM stock_balances WHERE product_id = 1"
        ),
        balance
    );
    assert_eq!(
        scalar(
            &connection,
            "SELECT COUNT(*) FROM inventory_movements WHERE request_id IS NOT NULL"
        ),
        0
    );
    assert!(SqliteInventoryRepository::new(&mut connection)
        .list_alerts()
        .unwrap()
        .is_empty());
}
