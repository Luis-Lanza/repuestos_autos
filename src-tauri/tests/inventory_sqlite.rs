use std::time::Instant;

use repuestos_autos::application::inventory::InventoryRepository;
use repuestos_autos::domain::inventory::{InventoryError, InventoryOperation};
use repuestos_autos::domain::RequestId;
use repuestos_autos::infrastructure::sqlite::{
    open_database, open_seeded_catalog, production_database_config, SqliteInventoryRepository,
};
use rusqlite::params;
use sha2::{Digest, Sha256};

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
            InventoryOperation::stock_entry_with_prices(
                1,
                request("550e8400-e29b-41d4-a716-446655440103"),
                2,
                1_250,
                None,
                None,
                Some("delivery".into()),
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(result.resulting_quantity, 10);
    assert_eq!(result.note.as_deref(), Some("delivery"));
    let identity = connection
        .query_row(
            "SELECT operation_kind, payload_version, canonical_payload, payload_sha256 FROM inventory_movements WHERE request_id = ?1",
            ["550e8400-e29b-41d4-a716-446655440103"],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .unwrap();
    assert_eq!(identity.0, "stock_entry");
    assert_eq!(identity.1, 2);
    assert!(!identity.2.is_empty());
    assert_eq!(identity.3.len(), 64);
    assert_eq!(
        SqliteInventoryRepository::new(&mut connection)
            .confirm(
                InventoryOperation::stock_entry_with_prices(
                    1,
                    request("550e8400-e29b-41d4-a716-446655440103"),
                    2,
                    1_250,
                    None,
                    None,
                    Some("delivery".into()),
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
    assert!(connection
        .execute(
            "UPDATE inventory_movements SET unit_purchase_price_centavos = 1 WHERE request_id = ?1",
            ["550e8400-e29b-41d4-a716-446655440103"],
        )
        .is_err());
}

#[test]
fn conflicting_stock_reuse_rejects_changed_quantity_note_product_and_operation() {
    let mut connection = open_seeded_catalog().unwrap();
    let request_id = request("550e8400-e29b-41d4-a716-446655440111");
    let first =
        InventoryOperation::stock_entry_with_prices(1, request_id.clone(), 2, 1_250, None, None, Some("delivery".into())).unwrap();
    SqliteInventoryRepository::new(&mut connection)
        .confirm(first)
        .unwrap();

    assert_eq!(
        SqliteInventoryRepository::new(&mut connection).confirm(
            InventoryOperation::stock_entry_with_prices(1, request_id.clone(), 3, 1_250, None, None, Some("delivery".into()))
                .unwrap(),
        ),
        Err(InventoryError::REQUEST_CONFLICT)
    );
    assert_eq!(
        SqliteInventoryRepository::new(&mut connection).confirm(
            InventoryOperation::stock_entry_with_prices(1, request_id.clone(), 2, 1_250, None, None, Some("other".into()))
                .unwrap(),
        ),
        Err(InventoryError::REQUEST_CONFLICT)
    );
    assert_eq!(
        SqliteInventoryRepository::new(&mut connection).confirm(
            InventoryOperation::stock_entry_with_prices(2, request_id.clone(), 2, 1_250, None, None, Some("delivery".into()))
                .unwrap(),
        ),
        Err(InventoryError::REQUEST_CONFLICT)
    );
    assert_eq!(
        SqliteInventoryRepository::new(&mut connection)
            .confirm(InventoryOperation::physical_count(1, request_id, 10, "delivery").unwrap(),),
        Err(InventoryError::REQUEST_CONFLICT)
    );
    assert_eq!(
        scalar(
            &connection,
            "SELECT quantity FROM stock_balances WHERE product_id = 1"
        ),
        10
    );
    assert_eq!(
        scalar(
            &connection,
            "SELECT COUNT(*) FROM inventory_movements WHERE request_id IS NOT NULL"
        ),
        1
    );
}

#[test]
fn stock_entry_atomically_updates_current_prices_and_keeps_cost_on_immutable_movement() {
    let mut connection = open_seeded_catalog().unwrap();
    let request_id = request("550e8400-e29b-41d4-a716-446655440120");
    let operation = InventoryOperation::stock_entry_with_prices(
        1, request_id.clone(), 2, 1_750, Some(5_000), Some(4_000), None,
    ).unwrap();
    SqliteInventoryRepository::new(&mut connection).confirm(operation.clone()).unwrap();
    assert_eq!(scalar(&connection, "SELECT purchase_price_centavos FROM products WHERE id = 1"), 1_750);
    assert_eq!(scalar(&connection, "SELECT list_price_centavos FROM products WHERE id = 1"), 5_000);
    assert_eq!(scalar(&connection, "SELECT minimum_unit_price_centavos FROM products WHERE id = 1"), 4_000);
    assert_eq!(scalar(&connection, "SELECT unit_purchase_price_centavos FROM inventory_movements WHERE request_id = '550e8400-e29b-41d4-a716-446655440120'"), 1_750);
    assert_eq!(SqliteInventoryRepository::new(&mut connection).confirm(
        InventoryOperation::stock_entry_with_prices(1, request_id.clone(), 2, 1_750, Some(5_000), Some(4_000), None).unwrap()
    ).unwrap().resulting_quantity, 10);
    for (cost, sale, minimum) in [(1_751, Some(5_000), Some(4_000)), (1_750, Some(5_001), Some(4_000)), (1_750, Some(5_000), Some(3_999))] {
        assert_eq!(SqliteInventoryRepository::new(&mut connection).confirm(
            InventoryOperation::stock_entry_with_prices(1, request_id.clone(), 2, cost, sale, minimum, None).unwrap()
        ), Err(InventoryError::REQUEST_CONFLICT));
    }
    SqliteInventoryRepository::new(&mut connection).confirm(
        InventoryOperation::physical_count(1, request("550e8400-e29b-41d4-a716-446655440121"), 9, "counted").unwrap()
    ).unwrap();
    assert_eq!(scalar(&connection, "SELECT purchase_price_centavos FROM products WHERE id = 1"), 1_750);
    assert_eq!(scalar(&connection, "SELECT list_price_centavos FROM products WHERE id = 1"), 5_000);
    assert_eq!(scalar(&connection, "SELECT minimum_unit_price_centavos FROM products WHERE id = 1"), 4_000);
}

#[test]
fn conflicting_physical_count_reuse_rejects_changed_count_and_reason() {
    let mut connection = open_seeded_catalog().unwrap();
    let request_id = request("550e8400-e29b-41d4-a716-446655440112");
    SqliteInventoryRepository::new(&mut connection)
        .confirm(InventoryOperation::physical_count(1, request_id.clone(), 6, " counted ").unwrap())
        .unwrap();

    assert_eq!(
        SqliteInventoryRepository::new(&mut connection).confirm(
            InventoryOperation::physical_count(1, request_id.clone(), 7, "counted").unwrap(),
        ),
        Err(InventoryError::REQUEST_CONFLICT)
    );
    assert_eq!(
        SqliteInventoryRepository::new(&mut connection)
            .confirm(InventoryOperation::physical_count(1, request_id, 6, "different").unwrap(),),
        Err(InventoryError::REQUEST_CONFLICT)
    );
    assert_eq!(
        scalar(
            &connection,
            "SELECT quantity FROM stock_balances WHERE product_id = 1"
        ),
        6
    );
    assert_eq!(
        scalar(
            &connection,
            "SELECT COUNT(*) FROM inventory_movements WHERE request_id IS NOT NULL"
        ),
        1
    );
}

#[test]
fn legacy_inventory_request_ids_fail_closed_without_changing_stock() {
    let mut connection = open_seeded_catalog().unwrap();
    let request_id = "550e8400-e29b-41d4-a716-446655440113";
    connection
        .execute(
            "INSERT INTO inventory_movements (product_id, movement_type, quantity_delta, occurred_at, request_id, resulting_quantity) VALUES (?1, 'stock_entry', ?2, ?3, ?4, ?5)",
            params![1, 2, "2025-01-01T00:00:00Z", request_id, 10],
        )
        .unwrap();

    assert_eq!(
        SqliteInventoryRepository::new(&mut connection)
            .confirm(InventoryOperation::stock_entry_with_prices(1, request(request_id), 2, 1_250, None, None, None).unwrap(),),
        Err(InventoryError::REQUEST_CONFLICT)
    );
    assert_eq!(
        scalar(
            &connection,
            "SELECT quantity FROM stock_balances WHERE product_id = 1"
        ),
        8
    );
    assert_eq!(
        scalar(
            &connection,
            "SELECT COUNT(*) FROM inventory_movements WHERE request_id IS NOT NULL"
        ),
        1
    );
}

#[test]
fn valid_legacy_v1_stock_entry_identity_conflicts_with_price_aware_retry() {
    let mut connection = open_seeded_catalog().unwrap();
    let request_id = "550e8400-e29b-41d4-a716-446655440122";
    let canonical_payload = b"12:inventory/v111:stock_entry1:11:24:null";
    connection.execute(
        "INSERT INTO inventory_movements (product_id, movement_type, quantity_delta, occurred_at, request_id, resulting_quantity, operation_kind, payload_version, canonical_payload, payload_sha256) VALUES (1, 'stock_entry', 2, '2025-01-01T00:00:00Z', ?1, 10, 'stock_entry', 1, ?2, ?3)",
        params![request_id, canonical_payload, format!("{:x}", Sha256::digest(canonical_payload))],
    ).unwrap();

    assert_eq!(
        SqliteInventoryRepository::new(&mut connection).confirm(
            InventoryOperation::stock_entry_with_prices(1, request(request_id), 2, 1_250, None, None, None).unwrap(),
        ),
        Err(InventoryError::REQUEST_CONFLICT)
    );
    assert_eq!(scalar(&connection, "SELECT quantity FROM stock_balances WHERE product_id = 1"), 8);
    let movement_count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM inventory_movements WHERE request_id = ?1",
        [request_id],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(movement_count, 1);
}

#[test]
fn partial_or_malformed_inventory_identity_fails_without_new_effects() {
    let mut connection = open_seeded_catalog().unwrap();
    let request_id = "550e8400-e29b-41d4-a716-446655440114";
    connection
        .execute(
            "INSERT INTO inventory_movements (product_id, movement_type, quantity_delta, occurred_at, request_id, resulting_quantity, operation_kind) VALUES (?1, 'stock_entry', ?2, ?3, ?4, ?5, 'stock_entry')",
            params![1, 2, "2025-01-01T00:00:00Z", request_id, 10],
        )
        .unwrap();
    assert_eq!(
        SqliteInventoryRepository::new(&mut connection)
            .confirm(InventoryOperation::stock_entry_with_prices(1, request(request_id), 2, 1_250, None, None, None).unwrap(),),
        Err(InventoryError::PERSISTENCE_FAILURE)
    );

    let malformed_request_id = "550e8400-e29b-41d4-a716-446655440115";
    connection
        .execute(
            "INSERT INTO inventory_movements (product_id, movement_type, quantity_delta, occurred_at, request_id, resulting_quantity, operation_kind, payload_version, canonical_payload, payload_sha256) VALUES (?1, 'stock_entry', ?2, ?3, ?4, ?5, 'stock_entry', 1, x'01', ?6)",
            params![
                1,
                2,
                "2025-01-01T00:00:00Z",
                malformed_request_id,
                10,
                format!("{:x}", Sha256::digest([1_u8])),
            ],
        )
        .unwrap();
    assert_eq!(
        SqliteInventoryRepository::new(&mut connection).confirm(
            InventoryOperation::stock_entry_with_prices(1, request(malformed_request_id), 2, 1_250, None, None, None).unwrap(),
        ),
        Err(InventoryError::PERSISTENCE_FAILURE)
    );
    assert_eq!(
        scalar(
            &connection,
            "SELECT quantity FROM stock_balances WHERE product_id = 1"
        ),
        8
    );
    assert_eq!(
        scalar(
            &connection,
            "SELECT COUNT(*) FROM inventory_movements WHERE request_id IS NOT NULL"
        ),
        2
    );
}

#[test]
fn noncanonical_persisted_identity_fails_without_new_effects() {
    let mut connection = open_seeded_catalog().unwrap();
    let request_id = "550e8400-e29b-41d4-a716-446655440116";
    let operation = InventoryOperation::stock_entry_with_prices(1, request(request_id), 2, 1_250, None, None, None).unwrap();
    let identity = operation.identity();
    let canonical_payload = identity.canonical_payload();
    assert!(canonical_payload.starts_with(b"12:"));
    let mut noncanonical_payload = b"012:".to_vec();
    noncanonical_payload.extend_from_slice(&canonical_payload[3..]);
    connection
        .execute(
            "INSERT INTO inventory_movements (product_id, movement_type, quantity_delta, occurred_at, request_id, resulting_quantity, operation_kind, payload_version, canonical_payload, payload_sha256) VALUES (?1, 'stock_entry', ?2, ?3, ?4, ?5, 'stock_entry', 1, ?6, ?7)",
            params![
                1,
                2,
                "2025-01-01T00:00:00Z",
                request_id,
                10,
                noncanonical_payload,
                format!("{:x}", Sha256::digest(&noncanonical_payload)),
            ],
        )
        .unwrap();

    assert_eq!(
        SqliteInventoryRepository::new(&mut connection).confirm(operation),
        Err(InventoryError::PERSISTENCE_FAILURE)
    );
    assert_eq!(
        scalar(
            &connection,
            "SELECT quantity FROM stock_balances WHERE product_id = 1"
        ),
        8
    );
    assert_eq!(
        scalar(
            &connection,
            "SELECT COUNT(*) FROM inventory_movements WHERE request_id IS NOT NULL"
        ),
        1
    );
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
            InventoryOperation::stock_entry_with_prices(
                2,
                request("550e8400-e29b-41d4-a716-446655440106"),
                1,
                1_250,
                None,
                None,
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
            InventoryOperation::stock_entry_with_prices(
                1,
                request("550e8400-e29b-41d4-a716-446655440108"),
                1,
                1_250,
                None,
                None,
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
    let operation = InventoryOperation::stock_entry_with_prices(
        1,
        request("550e8400-e29b-41d4-a716-446655440107"),
        1,
        1_250,
        None,
        None,
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
            InventoryOperation::stock_entry_with_prices(
                1,
                request("550e8400-e29b-41d4-a716-446655440107"),
                1,
                1_250,
                None,
                None,
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
    let operation = InventoryOperation::stock_entry_with_prices(
        1,
        request("550e8400-e29b-41d4-a716-446655440109"),
        2,
        1_250,
        None,
        None,
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
fn product_price_update_failure_rolls_back_movement_and_balance() {
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute_batch("CREATE TRIGGER reject_product_price_update BEFORE UPDATE ON products WHEN new.id = 1 BEGIN SELECT RAISE(ABORT, 'forced failure'); END;").unwrap();
    let previous_purchase: Option<i64> = connection.query_row("SELECT purchase_price_centavos FROM products WHERE id = 1", [], |row| row.get(0)).unwrap();
    let previous_sale = scalar(&connection, "SELECT list_price_centavos FROM products WHERE id = 1");
    let previous_minimum = scalar(&connection, "SELECT minimum_unit_price_centavos FROM products WHERE id = 1");

    assert_eq!(
        SqliteInventoryRepository::new(&mut connection).confirm(
            InventoryOperation::stock_entry_with_prices(1, request("550e8400-e29b-41d4-a716-446655440123"), 2, 1_750, None, None, None).unwrap(),
        ),
        Err(InventoryError::PERSISTENCE_FAILURE)
    );
    assert_eq!(scalar(&connection, "SELECT quantity FROM stock_balances WHERE product_id = 1"), 8);
    assert_eq!(scalar(&connection, "SELECT COUNT(*) FROM inventory_movements WHERE request_id = '550e8400-e29b-41d4-a716-446655440123'"), 0);
    let purchase_after: Option<i64> = connection.query_row("SELECT purchase_price_centavos FROM products WHERE id = 1", [], |row| row.get(0)).unwrap();
    assert_eq!(purchase_after, previous_purchase);
    assert_eq!(scalar(&connection, "SELECT list_price_centavos FROM products WHERE id = 1"), previous_sale);
    assert_eq!(scalar(&connection, "SELECT minimum_unit_price_centavos FROM products WHERE id = 1"), previous_minimum);
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
    let operation = InventoryOperation::stock_entry_with_prices(
        1,
        request("550e8400-e29b-41d4-a716-446655440110"),
        1,
        1_250,
        None,
        None,
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
