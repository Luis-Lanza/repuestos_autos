use repuestos_autos::infrastructure::sqlite::{
    migration_compatibility, open_database, production_database_config, MigrationCompatibility,
};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
const LEGACY_FIXTURE: &str = include_str!("fixtures/version1_fixed_price_legacy.sql");
const VERSION_TWO_MIGRATION: &str =
    include_str!("../src/infrastructure/sqlite/migrations/0002_fixed_price_checkout.sql");
const VERSION_THREE_MIGRATION: &str =
    include_str!("../src/infrastructure/sqlite/migrations/0003_sale_line_product_snapshots.sql");
const VERSION_FOUR_MIGRATION: &str =
    include_str!("../src/infrastructure/sqlite/migrations/0004_product_onboarding.sql");
const VERSION_FIVE_MIGRATION: &str =
    include_str!("../src/infrastructure/sqlite/migrations/0005_catalog_onboarding_hardening.sql");
const VERSION_SIX_MIGRATION: &str =
    include_str!("../src/infrastructure/sqlite/migrations/0006_operational_inventory_control.sql");
const VERSION_SEVEN_MIGRATION: &str =
    include_str!("../src/infrastructure/sqlite/migrations/0007_catalog_maintenance.sql");
const VERSION_EIGHT_MIGRATION: &str = include_str!(
    "../src/infrastructure/sqlite/migrations/0008_catalog_metadata_name_uniqueness.sql"
);
fn temporary_directory(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "repuestos-autos-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
fn create_legacy_database(directory: &Path) -> PathBuf {
    std::fs::create_dir_all(directory).unwrap();
    let path = directory.join("repuestos-autos.sqlite3");
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch(LEGACY_FIXTURE).unwrap();
    path
}
fn create_version_four_database(directory: &Path) -> PathBuf {
    let path = create_legacy_database(directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch(VERSION_TWO_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 2).unwrap();
    connection.execute_batch(VERSION_THREE_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 3).unwrap();
    connection.execute_batch(VERSION_FOUR_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 4).unwrap();
    path
}
fn create_version_five_database(directory: &Path) -> PathBuf {
    let path = create_version_four_database(directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch(VERSION_FIVE_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 5).unwrap();
    path
}
fn create_version_six_database(directory: &Path) -> PathBuf {
    let path = create_version_five_database(directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch(VERSION_SIX_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 6).unwrap();
    path
}
fn create_version_seven_database(directory: &Path) -> PathBuf {
    let path = create_version_six_database(directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch(VERSION_SEVEN_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 7).unwrap();
    path
}
fn create_version_eight_database(directory: &Path) -> PathBuf {
    let path = create_version_seven_database(directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch(VERSION_EIGHT_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 8).unwrap();
    path
}
fn user_version(path: &Path) -> i64 {
    Connection::open(path)
        .unwrap()
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap()
}
fn legacy_facts(path: &Path) -> Vec<Vec<String>> {
    let connection = Connection::open(path).unwrap();
    let queries = [
        "SELECT id, request_id, status, total_centavos, confirmed_at FROM sales ORDER BY id",
        "SELECT id, sale_id, product_id, quantity, negotiated_unit_price_centavos, line_total_centavos FROM sale_lines ORDER BY id",
        "SELECT id, sale_id, method, amount_applied_centavos, COALESCE(amount_tendered_centavos, 'NULL'), COALESCE(change_given_centavos, 'NULL') FROM sale_payments ORDER BY id",
        "SELECT product_id, quantity FROM stock_balances ORDER BY product_id",
        "SELECT id, product_id, sale_id, sale_line_id, quantity_delta FROM inventory_movements ORDER BY id",
    ];
    queries
        .iter()
        .map(|query| {
            let mut statement = connection.prepare(query).unwrap();
            statement
                .query_map([], |row| {
                    (0..row.as_ref().column_count())
                        .map(|index| {
                            row.get::<_, rusqlite::types::Value>(index)
                                .map(|value| format!("{value:?}"))
                        })
                        .collect::<rusqlite::Result<Vec<_>>>()
                })
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap()
                .into_iter()
                .flatten()
                .collect()
        })
        .collect()
}
#[test]
fn migrates_version_one_without_rewriting_legacy_facts_and_reopens_idempotently() {
    let directory = temporary_directory("migration-success");
    let path = create_legacy_database(&directory);
    let before = legacy_facts(&path);
    let config = production_database_config(&directory);
    let connection = open_database(&config).unwrap();
    assert_eq!(
        connection
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        10
    );
    drop(connection);
    assert_eq!(legacy_facts(&path), before);
    let connection = Connection::open(&path).unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos FROM sale_lines WHERE id = 20",
                [],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )
            .unwrap(),
        (5_000, 2_500)
    );
    assert_eq!(
        connection
            .query_row("SELECT request_id FROM sales WHERE id = 10", [], |row| {
                row.get::<_, String>(0)
            })
            .unwrap(),
        "550e8400-e29b-41d4-a716-446655440099"
    );
    drop(connection);
    let reopened = open_database(&config).unwrap();
    assert_eq!(
        reopened
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        10
    );
    drop(reopened);
    assert_eq!(legacy_facts(&path), before);
    let connection = Connection::open(&path).unwrap();
    assert_eq!(
            connection
                .query_row(
                    "SELECT sku_snapshot IS NULL, product_name_snapshot IS NULL FROM sale_lines WHERE id = 20",
                    [],
                    |row| Ok((row.get::<_, bool>(0)?, row.get::<_, bool>(1)?)),
                )
                .unwrap(),
            (true, true)
        );
    connection.execute_batch("INSERT INTO sales (id, request_id, status, total_centavos) VALUES (11, 'legacy-write-shape', 'confirmed', 2500); INSERT INTO sale_lines (sale_id, product_id, quantity, negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos, line_total_centavos) VALUES (11, 1, 1, 2500, 2500, 2500);").unwrap();
    drop(connection);
    std::fs::remove_dir_all(directory).unwrap();
}
#[test]
fn rejects_failed_preflight_without_changing_legacy_rows_or_version() {
    let directory = temporary_directory("migration-missing-column");
    let path = create_legacy_database(&directory);
    let before = legacy_facts(&path);
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch("ALTER TABLE sale_lines DROP COLUMN minimum_unit_price_snapshot_centavos;")
        .unwrap();
    drop(connection);
    assert!(open_database(&production_database_config(&directory)).is_err());
    assert_eq!(user_version(&path), 1);
    assert_eq!(legacy_facts(&path), before);
    std::fs::remove_dir_all(directory).unwrap();
}
#[test]
fn rejects_foreign_key_corruption_without_changing_legacy_rows_or_version() {
    let directory = temporary_directory("migration-foreign-key");
    let path = create_legacy_database(&directory);
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch("PRAGMA foreign_keys = OFF; DELETE FROM products WHERE id = 1;")
        .unwrap();
    drop(connection);
    let before = legacy_facts(&path);
    assert!(open_database(&production_database_config(&directory)).is_err());
    assert_eq!(user_version(&path), 1);
    assert_eq!(legacy_facts(&path), before);
    std::fs::remove_dir_all(directory).unwrap();
}
#[test]
fn migrates_a_new_version_zero_database_through_version_ten() {
    let directory = temporary_directory("migration-version-zero");
    let connection = open_database(&production_database_config(&directory)).unwrap();
    assert_eq!(
        connection
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        10
    );
    drop(connection);
    std::fs::remove_dir_all(directory).unwrap();
}
#[test]
fn rejects_unknown_future_schema_versions_without_mutation() {
    let directory = temporary_directory("migration-future-version");
    let path = create_legacy_database(&directory);
    let connection = Connection::open(&path).unwrap();
    connection.pragma_update(None, "user_version", 11).unwrap();
    drop(connection);
    let before = legacy_facts(&path);
    assert!(open_database(&production_database_config(&directory)).is_err());
    assert_eq!(user_version(&path), 11);
    assert_eq!(legacy_facts(&path), before);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn upgrades_version_four_preserving_legacy_movement_identity_and_foreign_keys() {
    let directory = temporary_directory("migration-version-four");
    let path = create_version_four_database(&directory);
    let before = legacy_facts(&path);

    let connection = open_database(&production_database_config(&directory)).unwrap();

    assert_eq!(
        connection
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        10
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT movement_type, occurred_at, sale_id, sale_line_id FROM inventory_movements WHERE id = 40",
                [],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?, row.get::<_, i64>(3)?)),
            )
            .unwrap(),
        ("sale".into(), "2025-01-01T12:00:00Z".into(), 10, 20)
    );
    let mut foreign_key_check = connection.prepare("PRAGMA foreign_key_check").unwrap();
    assert!(foreign_key_check
        .query([])
        .unwrap()
        .next()
        .unwrap()
        .is_none());
    drop(foreign_key_check);
    assert_eq!(legacy_facts(&path), before);
    drop(connection);
    assert_eq!(user_version(&path), 10);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejects_corrupt_version_four_before_the_forward_migration() {
    let directory = temporary_directory("migration-version-four-corrupt");
    let path = create_version_four_database(&directory);
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch("PRAGMA foreign_keys = OFF; DELETE FROM sales WHERE id = 10;")
        .unwrap();
    drop(connection);

    assert!(open_database(&production_database_config(&directory)).is_err());
    assert_eq!(user_version(&path), 4);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn migrates_valid_v5_history_verbatim_and_reopens_at_version_ten() {
    let directory = temporary_directory("migration-version-five");
    let path = create_version_five_database(&directory);
    let before = legacy_facts(&path);
    let config = production_database_config(&directory);
    let connection = open_database(&config).unwrap();
    assert_eq!(user_version(&path), 10);
    assert_eq!(
        connection
            .query_row(
                "SELECT movement_type, occurred_at FROM inventory_movements WHERE id = 40",
                [],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .unwrap(),
        ("sale".into(), "2025-01-01T12:00:00Z".into())
    );
    drop(connection);
    assert_eq!(legacy_facts(&path), before);
    assert_eq!(user_version(&path), 10);
    drop(open_database(&config).unwrap());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn migrates_version_six_additively_with_immutable_sale_prices_and_audits() {
    let directory = temporary_directory("migration-version-six");
    let path = create_version_five_database(&directory);
    let before = legacy_facts(&path);
    let connection = open_database(&production_database_config(&directory)).unwrap();

    assert_eq!(user_version(&path), 10);
    assert_eq!(legacy_facts(&path), before);
    assert_eq!(
        connection
            .query_row(
                "SELECT active, revision FROM categories WHERE id = 1",
                [],
                |row| Ok((row.get::<_, bool>(0)?, row.get::<_, i64>(1)?)),
            )
            .unwrap(),
        (true, 0)
    );
    assert!(connection
        .execute(
            "UPDATE sale_lines SET negotiated_unit_price_centavos = 1 WHERE id = 20",
            [],
        )
        .is_err());
    assert!(connection
        .execute(
            "INSERT INTO catalog_audit (entity_type, entity_id, operation, before_json, after_json, revision) VALUES ('product', 1, 'edit', '{}', '{}', 1)",
            [],
        )
        .is_ok());
    assert!(connection.execute("DELETE FROM catalog_audit", []).is_err());
    drop(connection);
    drop(open_database(&production_database_config(&directory)).unwrap());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejects_invalid_version_six_preflight_without_schema_advancement() {
    let directory = temporary_directory("migration-v6-preflight");
    let path = create_version_six_database(&directory);
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch("DROP TRIGGER inventory_movements_immutable_update;")
        .unwrap();
    drop(connection);

    assert!(open_database(&production_database_config(&directory)).is_err());
    assert_eq!(user_version(&path), 6);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn version_six_enforces_each_movement_type_composite_links_and_immutability() {
    let directory = temporary_directory("migration-v6-invariants");
    let path = create_version_five_database(&directory);
    let connection = open_database(&production_database_config(&directory)).unwrap();
    connection.execute_batch("INSERT INTO inventory_movements (id, product_id, movement_type, quantity_delta) VALUES (41, 1, 'opening_stock', 1); INSERT INTO inventory_movements (id, product_id, movement_type, quantity_delta, request_id, resulting_quantity) VALUES (42, 1, 'stock_entry', 1, 'entry', 9); INSERT INTO inventory_movements (id, product_id, sale_id, sale_line_id, movement_type, quantity_delta, reason) VALUES (43, 1, 10, 20, 'sale', -1, NULL), (44, 1, 10, 20, 'return', 1, NULL), (46, 1, 10, 20, 'cancellation', 1, 'reversed'); INSERT INTO inventory_movements (id, product_id, movement_type, quantity_delta, reason, request_id, counted_quantity, resulting_quantity) VALUES (45, 1, 'adjustment', -1, 'counted', 'adjustment', 7, 7); INSERT INTO products (id, category_id, sku, name, active, minimum_unit_price_centavos) VALUES (2, 1, 'OTHER', 'Other', 1, 1);").unwrap();
    for invalid in ["INSERT INTO inventory_movements (product_id, movement_type, quantity_delta, request_id, resulting_quantity) VALUES (1, 'stock_entry', -1, 'bad-sign', 1)", "INSERT INTO inventory_movements (product_id, movement_type, quantity_delta) VALUES (1, 'sale', -1)", "INSERT INTO inventory_movements (product_id, movement_type, quantity_delta, reason, request_id, counted_quantity, resulting_quantity) VALUES (1, 'adjustment', 1, ' ', 'bad-reason', 1, 1)", "INSERT INTO inventory_movements (product_id, sale_id, sale_line_id, movement_type, quantity_delta) VALUES (1, 10, 20, 'cancellation', 1)", "INSERT INTO inventory_movements (product_id, sale_id, sale_line_id, movement_type, quantity_delta) VALUES (2, 10, 20, 'return', 1)"] {
        assert!(connection.execute(invalid, []).is_err());
    }
    assert!(connection
        .execute(
            "UPDATE inventory_movements SET quantity_delta = -2 WHERE id = 40",
            [],
        )
        .is_err());
    assert!(connection
        .execute("DELETE FROM inventory_movements WHERE id = 40", [])
        .is_err());
    drop(connection);
    assert_eq!(user_version(&path), 10);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejects_invalid_v5_preflight_without_schema_advancement_or_rewrite() {
    let directory = temporary_directory("migration-v5-preflight");
    let path = create_version_five_database(&directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch("DROP TRIGGER inventory_movements_immutable_update; DROP TRIGGER inventory_movements_immutable_delete; PRAGMA ignore_check_constraints = ON; UPDATE inventory_movements SET sale_id = NULL WHERE id = 40;").unwrap();
    drop(connection);
    let before = legacy_facts(&path);
    assert!(open_database(&production_database_config(&directory)).is_err());
    assert_eq!(user_version(&path), 5);
    assert_eq!(legacy_facts(&path), before);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn migrates_v8_history_index_preserving_facts_and_reopens_with_normalized_uniqueness() {
    let directory = temporary_directory("migration-v7");
    let path = create_version_eight_database(&directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch("INSERT INTO attribute_definitions (id, category_id, label, field_type, required) VALUES (1, 1, 'retained', 'text', 1); INSERT INTO product_attribute_values VALUES (1, 1, 'kept', NULL, NULL, 'kept');").unwrap();
    drop(connection);
    let before = legacy_facts(&path);
    let config = production_database_config(&directory);
    let connection = open_database(&config).unwrap();
    assert_eq!(user_version(&path), 10);
    assert_eq!(connection.query_row("SELECT sql FROM sqlite_master WHERE name = 'sales_confirmed_history_idx'", [], |row| row.get::<_, String>(0)).unwrap(), "CREATE INDEX sales_confirmed_history_idx ON sales (confirmed_at DESC, id DESC) WHERE status = 'confirmed'");
    assert_eq!(legacy_facts(&path), before);
    assert_eq!(
        connection
            .query_row(
                "SELECT text_value FROM product_attribute_values WHERE product_id = 1",
                [],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
        "kept"
    );
    assert!(connection.execute("INSERT INTO products (category_id, sku, name, active, minimum_unit_price_centavos) SELECT category_id, 'NEW-001', lower(trim(name)), 1, 1 FROM products WHERE id = 1", []).is_err());
    drop(connection);
    drop(open_database(&config).unwrap());
    std::fs::remove_dir_all(directory).unwrap();
}
#[test]
fn creates_schema_v10_post_sale_fact_tables_and_immutability_triggers() {
    let directory = temporary_directory("migration-v10-foundation");
    let connection = open_database(&production_database_config(&directory)).unwrap();

    assert_eq!(
        connection
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        10
    );
    for table in [
        "post_sale_requests",
        "sale_returns",
        "sale_return_lines",
        "sale_cancellations",
        "sale_cancellation_lines",
    ] {
        assert!(connection
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
                [table],
                |row| row.get::<_, bool>(0),
            )
            .unwrap());
        for action in ["update", "delete"] {
            assert!(connection
                .query_row(
                    "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'trigger' AND name = ?1)",
                    [format!("{table}_immutable_{action}")],
                    |row| row.get::<_, bool>(0),
                )
                .unwrap());
        }
    }

    drop(connection);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn schema_v10_rejects_mismatched_or_mutable_correction_facts() {
    let directory = temporary_directory("migration-v10-constraints");
    create_version_eight_database(&directory);
    let connection = open_database(&production_database_config(&directory)).unwrap();

    connection.execute_batch("INSERT INTO post_sale_requests (id, request_id, operation_kind, sale_id, payload_version, canonical_payload, payload_sha256) VALUES (100, 'return-request', 'return', 10, 1, x'01', 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'); INSERT INTO sale_returns (id, sale_id) VALUES (100, 10); INSERT INTO inventory_movements (id, product_id, sale_id, sale_line_id, movement_type, quantity_delta) VALUES (100, 1, 10, 20, 'return', 1); INSERT INTO sale_return_lines VALUES (100, 10, 20, 1, 1, 100);").unwrap();
    assert!(connection
        .execute(
            "UPDATE post_sale_requests SET request_id = 'changed' WHERE id = 100",
            []
        )
        .is_err());
    assert!(connection.execute_batch("INSERT INTO post_sale_requests (id, request_id, operation_kind, sale_id, payload_version, canonical_payload, payload_sha256) VALUES (101, 'cancel-request', 'cancellation', 10, 1, x'02', 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'); INSERT INTO sale_cancellations (id, sale_id, reason) VALUES (101, 10, 'inventory correction'); INSERT INTO sale_cancellation_lines VALUES (101, 10, 20, 1, 0, 100);").is_err());

    drop(connection);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejects_duplicate_normalized_v7_names_before_schema_advancement() {
    let directory = temporary_directory("migration-v7-duplicates");
    let path = create_version_seven_database(&directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute("INSERT INTO products (category_id, sku, name, active, minimum_unit_price_centavos) SELECT category_id, 'NEW-001', lower(trim(name)), 1, 1 FROM products WHERE id = 1", []).unwrap();
    let before = legacy_facts(&path);
    assert_eq!(
        migration_compatibility(&connection).unwrap(),
        Some(MigrationCompatibility::DuplicateNormalizedProductName)
    );
    drop(connection);
    assert!(open_database(&production_database_config(&directory)).is_err());
    assert_eq!(user_version(&path), 7);
    assert_eq!(legacy_facts(&path), before);
    std::fs::remove_dir_all(directory).unwrap();
}
