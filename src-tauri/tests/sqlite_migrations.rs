use repuestos_autos::application::sales::{
    ApplicationConfirmSaleRequest, ApplicationRequestedLine, ConfirmSaleError, ConfirmSaleUseCase,
};
use repuestos_autos::domain::sales::PaymentInput;
use repuestos_autos::domain::{MoneyCentavos, Quantity, RequestId};
use repuestos_autos::infrastructure::sqlite::sale_repository::SqliteSaleRepository;
use repuestos_autos::infrastructure::sqlite::{
    migration_compatibility, open_database, open_seeded_catalog, production_database_config,
    MigrationCompatibility, CURRENT_SCHEMA_VERSION,
};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
const LEGACY_FIXTURE: &str = include_str!("fixtures/version1_fixed_price_legacy.sql");
const VERSION_ONE_MIGRATION: &str =
    include_str!("../src/infrastructure/sqlite/migrations/0001_confirm_sale.sql");
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
const VERSION_NINE_MIGRATION: &str = include_str!(
    "../src/infrastructure/sqlite/migrations/0009_sales_history_index.sql"
);
const VERSION_TEN_MIGRATION: &str = include_str!(
    "../src/infrastructure/sqlite/migrations/0010_post_sale_lifecycle.sql"
);
const VERSION_ELEVEN_MIGRATION: &str = include_str!(
    "../src/infrastructure/sqlite/migrations/0011_sale_idempotency_conflicts.sql"
);
const VERSION_TWELVE_MIGRATION: &str = include_str!(
    "../src/infrastructure/sqlite/migrations/0012_inventory_idempotency_conflicts.sql"
);
const VERSION_THIRTEEN_MIGRATION: &str = include_str!(
    "../src/infrastructure/sqlite/migrations/0013_catalog_dual_pricing.sql"
);
const VERSION_FOURTEEN_MIGRATION: &str = include_str!(
    "../src/infrastructure/sqlite/migrations/0014_sale_list_price_snapshot.sql"
);
const VERSION_FIFTEEN_MIGRATION: &str = include_str!(
    "../src/infrastructure/sqlite/migrations/0015_catalog_price_cap.sql"
);
const VERSION_SIXTEEN_MIGRATION: &str =
    include_str!("../src/infrastructure/sqlite/migrations/0016_product_images.sql");
const VERSION_SEVENTEEN_MIGRATION: &str = include_str!(
    "../src/infrastructure/sqlite/migrations/0017_product_image_thumbnails.sql"
);
const VERSION_EIGHTEEN_MIGRATION: &str = include_str!(
    "../src/infrastructure/sqlite/migrations/0018_global_product_purchase_price.sql"
);
const VERSION_NINETEEN_MIGRATION: &str = include_str!(
    "../src/infrastructure/sqlite/migrations/0019_category_field_lifecycle.sql"
);
const VERSION_TWENTY_MIGRATION: &str =
    include_str!("../src/infrastructure/sqlite/migrations/0020_product_locations.sql");
const VERSION_TWENTY_ONE_MIGRATION: &str = include_str!(
    "../src/infrastructure/sqlite/migrations/0021_sale_line_cost_snapshot.sql"
);
const VERSION_TWENTY_TWO_MIGRATION: &str = include_str!(
    "../src/infrastructure/sqlite/migrations/0022_product_low_stock_threshold.sql"
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

fn create_v22_demo_database(directory: &Path) -> (PathBuf, repuestos_autos::infrastructure::sqlite::DatabaseConfig) {
    std::fs::create_dir_all(directory).unwrap();
    let config = production_database_config(directory);
    let path = config.path().to_path_buf();
    let connection = Connection::open(&path).unwrap();
    for (version, migration) in [
        (1, VERSION_ONE_MIGRATION),
        (2, VERSION_TWO_MIGRATION),
        (3, VERSION_THREE_MIGRATION),
        (4, VERSION_FOUR_MIGRATION),
        (5, VERSION_FIVE_MIGRATION),
        (6, VERSION_SIX_MIGRATION),
        (7, VERSION_SEVEN_MIGRATION),
        (8, VERSION_EIGHT_MIGRATION),
        (9, VERSION_NINE_MIGRATION),
        (10, VERSION_TEN_MIGRATION),
        (11, VERSION_ELEVEN_MIGRATION),
        (12, VERSION_TWELVE_MIGRATION),
        (13, VERSION_THIRTEEN_MIGRATION),
        (14, VERSION_FOURTEEN_MIGRATION),
        (15, VERSION_FIFTEEN_MIGRATION),
        (16, VERSION_SIXTEEN_MIGRATION),
        (17, VERSION_SEVENTEEN_MIGRATION),
        (18, VERSION_EIGHTEEN_MIGRATION),
        (19, VERSION_NINETEEN_MIGRATION),
        (20, VERSION_TWENTY_MIGRATION),
        (21, VERSION_TWENTY_ONE_MIGRATION),
        (22, VERSION_TWENTY_TWO_MIGRATION),
    ] {
        if version == 19 {
            connection.pragma_update(None, "foreign_keys", false).unwrap();
        }
        connection.execute_batch(migration).unwrap();
        connection.pragma_update(None, "user_version", version).unwrap();
        if version == 19 {
            connection.pragma_update(None, "foreign_keys", true).unwrap();
        }
    }
    assert_eq!(
        connection.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0)).unwrap(),
        22,
        "demo fixture must stop before the schema-v23 cleanup",
    );
    for (table, column) in [
        ("products", "low_stock_threshold"),
        ("products", "primary_location_id"),
        ("sale_lines", "unit_cost_snapshot_centavos"),
    ] {
        assert!(connection.query_row(
            "SELECT EXISTS (SELECT 1 FROM pragma_table_info(?1) WHERE name = ?2)",
            rusqlite::params![table, column],
            |row| row.get::<_, bool>(0),
        ).unwrap(), "schema-v22 fixture is missing {table}.{column}");
    }
    for (sku, stock, search) in [
        ("FLT-001", 8_i64, "flt-001 filtro de aceite filtros toyota "),
        ("BUJ-001", 4_i64, "buj-001 bujia archivada bujias  "),
    ] {
        assert_eq!(connection.query_row(
            "SELECT b.quantity FROM products p JOIN stock_balances b ON b.product_id = p.id WHERE p.sku = ?1",
            [sku], |row| row.get::<_, i64>(0),
        ).unwrap(), stock);
        assert_eq!(connection.query_row(
            "SELECT content FROM catalog_product_search f JOIN products p ON p.id = f.product_id WHERE p.sku = ?1",
            [sku], |row| row.get::<_, String>(0),
        ).unwrap(), search);
    }
    drop(connection);
    (path, config)
}

fn assert_v22_seed_is_preserved(name: &str, setup_sql: &str) {
    let directory = temporary_directory(name);
    let (path, config) = create_v22_demo_database(&directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch(setup_sql).unwrap();
    drop(connection);

    let connection = open_database(&config).unwrap();
    assert_eq!(connection.query_row("SELECT COUNT(*) FROM products WHERE sku = 'FLT-001'", [], |row| row.get::<_, i64>(0)).unwrap(), 1, "{name}");
    assert_eq!(connection.query_row("SELECT COUNT(*) FROM categories WHERE id = 1", [], |row| row.get::<_, i64>(0)).unwrap(), 1, "{name}");
    assert!(connection.prepare("PRAGMA foreign_key_check").unwrap().query([]).unwrap().next().unwrap().is_none());
    drop(connection);
    drop(open_database(&config).unwrap());
    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION);
    std::fs::remove_dir_all(directory).unwrap();
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
fn create_version_twelve_database(directory: &Path) -> PathBuf {
    let path = create_version_eight_database(directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch(VERSION_NINE_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 9).unwrap();
    connection.execute_batch(VERSION_TEN_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 10).unwrap();
    connection.execute_batch(VERSION_ELEVEN_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 11).unwrap();
    connection.execute_batch(VERSION_TWELVE_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 12).unwrap();
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
fn fresh_v24_accepts_repeated_names_but_keeps_nonempty_name_policy() {
    use repuestos_autos::application::catalog::{CreateProductError, CreateProductInput, CreateProductUseCase};
    use repuestos_autos::infrastructure::sqlite::SqliteCatalogRepository;
    let directory = temporary_directory("fresh-v24-name-policy");
    let config = production_database_config(&directory);
    let mut connection = open_database(&config).unwrap();
    assert_eq!(user_version(config.path()), 24);
    connection.execute("INSERT INTO categories (id, name) VALUES (1, 'Customer category')", []).unwrap();
    for (sku, name, valid) in [("A", "Bujía", true), ("B", "Bujía", true), ("C", "   ", false), ("D", "", false)] {
        let result = CreateProductUseCase::new(&mut connection, SqliteCatalogRepository).execute(CreateProductInput {
            sku: sku.into(), name: name.into(), category_id: 1, purchase_price_centavos: 100,
            sale_price_centavos: 200, minimum_sale_price_centavos: 150, low_stock_threshold: None,
            opening_quantity: 1, attribute_values: vec![],
        });
        if valid { assert!(result.is_ok(), "{result:?}"); }
        else { assert_eq!(result, Err(CreateProductError::InvalidProduct)); }
    }
    drop(connection);
    assert_eq!(open_database(&config).unwrap().query_row("SELECT COUNT(*) FROM products", [], |row| row.get::<_, i64>(0)).unwrap(), 2);
}

#[test]
fn schema_v24_allows_repeated_names_and_preserves_v23_facts_and_objects() {
    let directory = temporary_directory("migration-v24-repeated-names");
    let (path, config) = create_v22_demo_database(&directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch(
        "INSERT INTO catalog_audit (entity_type, entity_id, operation, before_json, after_json, revision) VALUES ('product', 1, 'edit', '{}', '{}', 1), ('product', 2, 'edit', '{}', '{}', 1);",
    ).unwrap();
    connection.execute_batch(include_str!("../src/infrastructure/sqlite/migrations/0023_remove_untouched_demo_catalog.sql")).unwrap();
    connection.pragma_update(None, "user_version", 23).unwrap();
    let objects = |connection: &Connection| {
        connection.prepare("SELECT type, name, tbl_name, COALESCE(sql, '') FROM sqlite_schema WHERE name <> 'products_normalized_name_idx' ORDER BY type, name")
            .unwrap().query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?)))
            .unwrap().collect::<rusqlite::Result<Vec<_>>>().unwrap()
    };
    let before_objects = objects(&connection);
    let before_facts = legacy_facts(&path);
    drop(connection);
    let connection = open_database(&config).unwrap();
    assert_eq!(user_version(&path), 24);
    assert_eq!(objects(&connection), before_objects);
    assert_eq!(legacy_facts(&path), before_facts);
    assert_eq!(connection.query_row("SELECT after_json, revision FROM catalog_audit", [], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))).unwrap(), ("{}".into(), 1));
    assert_eq!(connection.query_row("SELECT content FROM catalog_product_search WHERE product_id = 1", [], |row| row.get::<_, String>(0)).unwrap(), "flt-001 filtro de aceite filtros toyota ");
    for (sku, name) in [("NEW-1", "Filtro de aceite"), ("NEW-2", " FILTRO DE ACEITE ")] {
        connection.execute("INSERT INTO products (category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (2, ?1, ?2, 1, 2500, 2500)", rusqlite::params![sku, name]).unwrap();
    }
    // SKU uniqueness is global, including archived products and other categories.
    for sku in ["flt-001", " FLT-001 ", "buj-001", " BUJ-001 "] {
        assert!(connection.execute("INSERT INTO products (category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (2, ?1, 'Filtro de aceite', 1, 2500, 2500)", [sku]).is_err());
    }
    assert!(connection.execute("INSERT INTO categories (name) VALUES (' filtros ')", []).is_err());
    assert!(connection.execute("INSERT INTO products (category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (999, 'BAD-FK', 'Filtro de aceite', 1, 2500, 2500)", []).is_err());
    drop(connection);
    let reopened = open_database(&config).unwrap();
    assert_eq!(reopened.query_row("SELECT COUNT(*) FROM products", [], |row| row.get::<_, i64>(0)).unwrap(), 4);
    assert_eq!(objects(&reopened), before_objects);
    drop(reopened);
}

#[test]
fn migration_v20_and_v21_upgrade_contracts_preserve_stock_and_legacy_costs() {
    let directory = temporary_directory("migration-v20-product-locations");
    let path = create_legacy_database(&directory);
    let before = legacy_facts(&path);
    let connection = open_database(&production_database_config(&directory)).unwrap();

    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION);
    assert_eq!(legacy_facts(&path), before);
    assert_eq!(connection.query_row("SELECT COUNT(*) FROM location_schema", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
    assert_eq!(connection.query_row("SELECT primary_location_id FROM products WHERE id = 1", [], |row| row.get::<_, Option<i64>>(0)).unwrap(), None);
    assert_eq!(connection.query_row("SELECT COUNT(*) FROM location_segments", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
    drop(connection);
    drop(open_database(&production_database_config(&directory)).unwrap());
    assert_eq!(legacy_facts(&path), before);
    std::fs::remove_dir_all(directory).unwrap();
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
        CURRENT_SCHEMA_VERSION
    );
    assert_eq!(connection.query_row("SELECT low_stock_threshold FROM products WHERE id = 1", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
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
            .query_row(
                "SELECT unit_cost_snapshot_centavos FROM sale_lines WHERE id = 20",
                [],
                |row| row.get::<_, Option<i64>>(0),
            )
            .unwrap(),
        None,
        "legacy sale costs must remain unknown after migration",
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
        CURRENT_SCHEMA_VERSION
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
fn pre_v11_confirmed_sale_fails_closed_after_current_schema_migration() {
    let directory = temporary_directory("migration-legacy-sale-idempotency");
    let path = create_legacy_database(&directory);
    assert_eq!(user_version(&path), 1);
    let before = legacy_facts(&path);
    let mut connection = open_database(&production_database_config(&directory)).unwrap();
    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION);

    let result = ConfirmSaleUseCase::new(&mut connection, &SqliteSaleRepository).confirm(
        ApplicationConfirmSaleRequest {
            request_id: RequestId::parse("550e8400-e29b-41d4-a716-446655440099").unwrap(),
            lines: vec![ApplicationRequestedLine {
                product_id: 1,
                quantity: Quantity::new(2).unwrap(),
                captured_unit_price: MoneyCentavos::new(2_500).unwrap(),
                captured_revision: 0,
                final_unit_price: None,
                acknowledged_price: None,
                acknowledged_revision: None,
            }],
            payment: PaymentInput {
                amount_tendered: None,
                qr_applied: Some(MoneyCentavos::new(5_000).unwrap()),
            },
        },
    );

    assert_eq!(result, Err(ConfirmSaleError::RequestConflict));
    assert_eq!(legacy_facts(&path), before);
    assert_eq!(
        connection
            .query_row(
                "SELECT operation_kind, payload_version, canonical_payload, payload_sha256 FROM sales WHERE id = 10",
                [],
                |row| Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, Option<Vec<u8>>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                )),
            )
            .unwrap(),
        (None, None, None, None)
    );
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
fn schema_v23_fresh_database_has_no_demo_catalog_and_reopens_idempotently() {
    let directory = temporary_directory("migration-v23-fresh-empty");
    let config = production_database_config(&directory);
    let connection = open_database(&config).unwrap();
    assert_eq!(user_version(&config.path()), CURRENT_SCHEMA_VERSION);
    for table in ["categories", "products", "stock_balances", "product_searchable_values", "catalog_product_search"] {
        assert_eq!(connection.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row.get::<_, i64>(0)).unwrap(), 0, "{table}");
    }
    assert!(connection.prepare("PRAGMA foreign_key_check").unwrap().query([]).unwrap().next().unwrap().is_none());
    drop(connection);
    let reopened = open_database(&config).unwrap();
    assert_eq!(reopened.query_row("SELECT COUNT(*) FROM products", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
    drop(reopened);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn schema_v23_removes_only_the_exact_untouched_v22_seed() {
    let directory = temporary_directory("migration-v23-clean-seed");
    let (path, config) = create_v22_demo_database(&directory);
    let connection = open_database(&config).unwrap();
    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION);
    for table in ["categories", "products", "stock_balances", "product_searchable_values", "catalog_product_search"] {
        assert_eq!(connection.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row.get::<_, i64>(0)).unwrap(), 0, "{table}");
    }
    assert!(connection.prepare("PRAGMA foreign_key_check").unwrap().query([]).unwrap().next().unwrap().is_none());
    drop(connection);
    let reopened = open_database(&config).unwrap();
    assert_eq!(reopened.query_row("SELECT COUNT(*) FROM products", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
    drop(reopened);
    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn schema_v23_preserves_seed_products_with_sales_movements_or_changed_balances() {
    assert_v22_seed_is_preserved(
        "migration-v23-seed-sale",
        "INSERT INTO sales (id, request_id, status, total_centavos, confirmed_at) VALUES (1, 'seed-sale', 'confirmed', 2500, '2025-01-01T00:00:00Z');
         INSERT INTO sale_lines (id, sale_id, product_id, quantity, negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos, line_total_centavos) VALUES (1, 1, 1, 1, 2500, 2500, 2500);",
    );
    assert_v22_seed_is_preserved(
        "migration-v23-seed-movement",
        "INSERT INTO inventory_movements (id, product_id, movement_type, quantity_delta) VALUES (1, 1, 'opening_stock', 1);",
    );
    assert_v22_seed_is_preserved(
        "migration-v23-changed-balance",
        "UPDATE stock_balances SET quantity = 7 WHERE product_id = 1;",
    );
    assert_v22_seed_is_preserved(
        "migration-v23-changed-search-state",
        "UPDATE product_searchable_values SET value = 'Honda' WHERE product_id = 1;",
    );
}

#[test]
fn schema_v23_preserves_seed_product_with_attribute_value() {
    let directory = temporary_directory("migration-v23-seed-attribute-value");
    let (path, config) = create_v22_demo_database(&directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch(
        "INSERT INTO attribute_definitions (id, category_id, label, field_type, required) VALUES (1, 1, 'Vehicle', 'text', 0);
         INSERT INTO product_attribute_values (product_id, definition_id, text_value, searchable_value) VALUES (1, 1, 'Toyota Corolla', 'Toyota Corolla');",
    ).unwrap();
    drop(connection);

    let connection = open_database(&config).unwrap();
    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION);
    assert_eq!(
        connection.query_row(
            "SELECT sku, name FROM products WHERE id = 1",
            [],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        ).unwrap(),
        ("FLT-001".to_string(), "Filtro de aceite".to_string()),
    );
    assert_eq!(
        connection.query_row(
            "SELECT text_value, searchable_value FROM product_attribute_values WHERE product_id = 1 AND definition_id = 1",
            [],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        ).unwrap(),
        ("Toyota Corolla".to_string(), "Toyota Corolla".to_string()),
        "the customer-owned attribute value must not be deleted",
    );
    assert!(connection.prepare("PRAGMA foreign_key_check").unwrap().query([]).unwrap().next().unwrap().is_none());
    drop(connection);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn schema_v23_preserves_seed_product_with_changed_core_field() {
    let directory = temporary_directory("migration-v23-seed-changed-name");
    let (path, config) = create_v22_demo_database(&directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute("UPDATE products SET name = 'Customer-renamed filter' WHERE id = 1", []).unwrap();
    drop(connection);

    let connection = open_database(&config).unwrap();
    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION);
    assert_eq!(
        connection.query_row(
            "SELECT sku, name, active, list_price_centavos, minimum_unit_price_centavos FROM products WHERE id = 1",
            [],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, bool>(2)?, row.get::<_, i64>(3)?, row.get::<_, i64>(4)?)),
        ).unwrap(),
        ("FLT-001".to_string(), "Customer-renamed filter".to_string(), true, 2_500, 2_500),
        "the product with a customer-modified core field must remain intact",
    );
    assert_eq!(connection.query_row("SELECT COUNT(*) FROM stock_balances WHERE product_id = 1", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
    assert!(connection.prepare("PRAGMA foreign_key_check").unwrap().query([]).unwrap().next().unwrap().is_none());
    drop(connection);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn schema_v23_preserves_products_with_images_audits_category_definitions_or_locations() {
    assert_v22_seed_is_preserved(
        "migration-v23-seed-image",
        "INSERT INTO product_images (product_id, mime_type, image_bytes) VALUES (1, 'image/png', x'01');",
    );
    assert_v22_seed_is_preserved(
        "migration-v23-seed-audit",
        "INSERT INTO catalog_audit (entity_type, entity_id, operation, before_json, after_json, revision) VALUES ('product', 1, 'edit', '{}', '{}', 1);",
    );
    assert_v22_seed_is_preserved(
        "migration-v23-category-audit",
        "INSERT INTO catalog_audit (entity_type, entity_id, operation, before_json, after_json, revision) VALUES ('category', 1, 'edit', '{}', '{}', 1);",
    );
    assert_v22_seed_is_preserved(
        "migration-v23-category-definition",
        "INSERT INTO attribute_definitions (id, category_id, label, field_type, required) VALUES (1, 1, 'Customer field', 'text', 0);",
    );
    assert_v22_seed_is_preserved(
        "migration-v23-product-location",
        "INSERT INTO product_locations (id, code) VALUES (1, 'A-1'); UPDATE products SET primary_location_id = 1 WHERE id = 1;",
    );
}

#[test]
fn migrates_a_new_version_zero_database_through_version_eleven() {
    let directory = temporary_directory("migration-version-zero");
    let connection = open_database(&production_database_config(&directory)).unwrap();
    assert_eq!(
        connection
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        CURRENT_SCHEMA_VERSION
    );
    drop(connection);
    std::fs::remove_dir_all(directory).unwrap();
}
#[test]
fn rejects_unknown_future_schema_versions_without_mutation() {
    let directory = temporary_directory("migration-future-version");
    let path = create_legacy_database(&directory);
    let connection = Connection::open(&path).unwrap();
    connection
        .pragma_update(None, "user_version", CURRENT_SCHEMA_VERSION + 1)
        .unwrap();
    drop(connection);
    let before = legacy_facts(&path);
    assert!(open_database(&production_database_config(&directory)).is_err());
    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION + 1);
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
        CURRENT_SCHEMA_VERSION
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
    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION);
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
fn migrates_valid_v5_history_verbatim_and_reopens_at_version_eleven() {
    let directory = temporary_directory("migration-version-five");
    let path = create_version_five_database(&directory);
    let before = legacy_facts(&path);
    let config = production_database_config(&directory);
    let connection = open_database(&config).unwrap();
    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION);
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
    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION);
    drop(open_database(&config).unwrap());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn migrates_version_six_additively_with_immutable_sale_prices_and_audits() {
    let directory = temporary_directory("migration-version-six");
    let path = create_version_five_database(&directory);
    let before = legacy_facts(&path);
    let connection = open_database(&production_database_config(&directory)).unwrap();

    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION);
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
    connection.execute_batch("INSERT INTO inventory_movements (id, product_id, movement_type, quantity_delta) VALUES (41, 1, 'opening_stock', 1); INSERT INTO inventory_movements (id, product_id, movement_type, quantity_delta, request_id, resulting_quantity) VALUES (42, 1, 'stock_entry', 1, 'entry', 9); INSERT INTO inventory_movements (id, product_id, sale_id, sale_line_id, movement_type, quantity_delta, reason) VALUES (43, 1, 10, 20, 'sale', -1, NULL), (44, 1, 10, 20, 'return', 1, NULL), (46, 1, 10, 20, 'cancellation', 1, 'reversed'); INSERT INTO inventory_movements (id, product_id, movement_type, quantity_delta, reason, request_id, counted_quantity, resulting_quantity) VALUES (45, 1, 'adjustment', -1, 'counted', 'adjustment', 7, 7); INSERT INTO products (id, category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (2, 1, 'OTHER', 'Other', 1, 1, 1);").unwrap();
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
    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION);
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
fn migrates_v8_history_index_preserving_facts_and_reopens_with_repeated_names() {
    let directory = temporary_directory("migration-v7");
    let path = create_version_eight_database(&directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch("INSERT INTO attribute_definitions (id, category_id, label, field_type, required) VALUES (1, 1, 'retained', 'text', 1); INSERT INTO product_attribute_values VALUES (1, 1, 'kept', NULL, NULL, 'kept');").unwrap();
    drop(connection);
    let before = legacy_facts(&path);
    let config = production_database_config(&directory);
    let connection = open_database(&config).unwrap();
    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION);
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
    assert!(connection.execute("INSERT INTO products (category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) SELECT category_id, 'NEW-001', lower(trim(name)), 1, 1, 1 FROM products WHERE id = 1", []).is_ok());
    drop(connection);
    drop(open_database(&config).unwrap());
    std::fs::remove_dir_all(directory).unwrap();
}
#[test]
fn migration_v14_adds_nullable_immutable_list_snapshots_without_fabricating_legacy_facts() {
    let directory = temporary_directory("migration-v14-list-snapshot");
    let path = create_version_twelve_database(&directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch(VERSION_THIRTEEN_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 13).unwrap();
    connection.execute_batch(VERSION_FOURTEEN_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 14).unwrap();
    drop(connection);

    let connection = open_database(&production_database_config(&directory)).unwrap();
    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION);
    assert!(connection
        .query_row(
            "SELECT list_price_snapshot_centavos IS NULL FROM sale_lines WHERE id = 20",
            [],
            |row| row.get::<_, bool>(0),
        )
        .unwrap());
    assert!(connection
        .execute(
            "UPDATE sale_lines SET list_price_snapshot_centavos = 3001 WHERE id = 20",
            [],
        )
        .is_err());
    drop(connection);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn upgrades_v15_with_one_to_one_product_images_and_cascade_delete() {
    let directory = temporary_directory("migration-v16-product-images");
    let path = create_version_twelve_database(&directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch(VERSION_THIRTEEN_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 13).unwrap();
    connection.execute_batch(VERSION_FOURTEEN_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 14).unwrap();
    connection.execute_batch(VERSION_FIFTEEN_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 15).unwrap();
    drop(connection);

    let connection = open_database(&production_database_config(&directory)).unwrap();

    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION);
    connection.execute(
        "INSERT INTO products (id, category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (2, 1, 'IMAGE-001', 'Image test', 1, 100, 100)",
        [],
    ).unwrap();
    let image = vec![0x89, 0x50, 0x4e, 0x47, 0x00, 0xff];
    connection.execute(
        "INSERT INTO product_images (product_id, mime_type, image_bytes) VALUES (?1, ?2, ?3)",
        rusqlite::params![2, "image/png", &image],
    ).unwrap();
    assert_eq!(
        connection.query_row("SELECT mime_type, image_bytes, thumbnail_mime_type, thumbnail_bytes FROM product_images WHERE product_id = 2", [], |row| Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?, row.get::<_, Option<String>>(2)?, row.get::<_, Option<Vec<u8>>>(3)?))).unwrap(),
        ("image/png".to_string(), image, None, None),
    );
    assert!(connection.execute(
        "INSERT INTO product_images (product_id, mime_type, image_bytes) VALUES (2, 'image/jpeg', x'01')", []
    ).is_err());
    assert!(connection.execute(
        "INSERT INTO product_images (product_id, mime_type, image_bytes) VALUES (999, 'image/png', x'01')", []
    ).is_err());
    assert!(connection.execute(
        "INSERT INTO product_images (product_id, mime_type, image_bytes) VALUES (1, 'text/plain', x'01')", []
    ).is_err());
    assert!(connection.execute(
        "INSERT INTO product_images (product_id, mime_type, image_bytes) VALUES (2, 'image/png', x'')", []
    ).is_err());
    assert!(connection.execute(
        "INSERT INTO product_images (product_id, mime_type, image_bytes) VALUES (2, 'image/png', zeroblob(2097153))", []
    ).is_err());
    assert!(connection.execute(
        "UPDATE product_images SET thumbnail_mime_type = 'image/jpeg' WHERE product_id = 2", []
    ).is_err());
    connection.execute("DELETE FROM products WHERE id = 2", []).unwrap();
    assert_eq!(connection.query_row("SELECT COUNT(*) FROM product_images", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
    drop(connection);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn migrates_schema_v12_to_explicit_positive_list_and_minimum_prices() {
    let directory = temporary_directory("migration-v13-prices");
    let path = create_version_twelve_database(&directory);
    let connection = open_database(&production_database_config(&directory)).unwrap();
    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION);
    assert_eq!(
        connection
            .query_row("SELECT list_price_centavos, minimum_unit_price_centavos FROM products WHERE id = 1", [], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))
            .unwrap(),
        (2_500, 2_500)
    );
    assert!(!connection.query_row("SELECT EXISTS (SELECT 1 FROM products WHERE list_price_centavos <= 0 OR minimum_unit_price_centavos <= 0 OR minimum_unit_price_centavos > list_price_centavos)", [], |row| row.get::<_, bool>(0)).unwrap());
    drop(connection);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejects_non_positive_legacy_minimum_before_schema_v13_advancement() {
    let directory = temporary_directory("migration-v13-invalid-minimum");
    let path = create_version_twelve_database(&directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute("UPDATE products SET minimum_unit_price_centavos = 0 WHERE id = 1", []).unwrap();
    drop(connection);
    assert!(open_database(&production_database_config(&directory)).is_err());
    assert_eq!(user_version(&path), 12);
    let connection = Connection::open(&path).unwrap();
    assert!(!connection.query_row("SELECT EXISTS (SELECT 1 FROM pragma_table_info('products') WHERE name = 'list_price_centavos')", [], |row| row.get::<_, bool>(0)).unwrap());
    drop(connection);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn v15_product_price_triggers_reject_invalid_values_but_accept_the_exact_cap() {
    const CAP: i64 = 9_007_199_254_740_991;
    let connection = open_seeded_catalog().unwrap();

    connection
        .execute(
            "INSERT INTO products (category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (1, 'CAP-001', 'At cap', 1, ?1, ?1)",
            [CAP],
        )
        .unwrap();
    for statement in [
        "INSERT INTO products (category_id, sku, name, active, minimum_unit_price_centavos) VALUES (1, 'MISSING-LIST', 'Missing list', 1, 1)",
        "INSERT INTO products (category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (1, 'MAX-LIST', 'Sentinel list', 1, 9223372036854775807, 1)",
    ] {
        assert!(connection.execute(statement, []).is_err(), "{statement}");
    }
    assert!(connection
        .execute(
            "INSERT INTO products (category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (1, 'OVER-MIN', 'Over minimum', 1, ?1, ?2)",
            rusqlite::params![CAP, CAP + 1],
        )
        .is_err());
    assert!(connection
        .execute(
            "UPDATE products SET list_price_centavos = ?1 WHERE sku = 'CAP-001'",
            [CAP + 1],
        )
        .is_err());
}

#[test]
fn rejects_v14_persisted_sentinel_price_before_advancing_to_v15() {
    let directory = temporary_directory("migration-v15-sentinel");
    let path = create_version_twelve_database(&directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch(VERSION_THIRTEEN_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 13).unwrap();
    connection.execute_batch(VERSION_FOURTEEN_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 14).unwrap();
    connection
        .execute(
            "UPDATE products SET list_price_centavos = 9223372036854775807 WHERE id = 1",
            [],
        )
        .unwrap();
    drop(connection);

    assert!(open_database(&production_database_config(&directory)).is_err());
    assert_eq!(user_version(&path), 14);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn low_stock_threshold_is_mandatory_positive_integer_with_default_one() {
    let connection = open_seeded_catalog().unwrap();
    assert_eq!(connection.query_row("SELECT low_stock_threshold FROM products WHERE id = 1", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
    assert!(connection.execute("UPDATE products SET low_stock_threshold = 0 WHERE id = 1", []).is_err());
    assert!(connection.execute("UPDATE products SET low_stock_threshold = 1.5 WHERE id = 1", []).is_err());
    connection.execute("INSERT INTO products (category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (1, 'DEFAULT-THRESHOLD', 'Default threshold', 1, 100, 100)", []).unwrap();
    assert_eq!(connection.query_row("SELECT low_stock_threshold FROM products WHERE sku = 'DEFAULT-THRESHOLD'", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
}

#[test]
fn creates_schema_v11_with_sale_idempotency_identity_columns() {
    let directory = temporary_directory("migration-v10-foundation");
    let connection = open_database(&production_database_config(&directory)).unwrap();

    assert_eq!(
        connection
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        CURRENT_SCHEMA_VERSION
    );
    for column in [
        "operation_kind",
        "payload_version",
        "canonical_payload",
        "payload_sha256",
    ] {
        assert!(connection
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM pragma_table_info('sales') WHERE name = ?1)",
                [column],
                |row| row.get::<_, bool>(0),
            )
            .unwrap());
    }
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
fn creates_schema_v12_with_inventory_idempotency_identity_columns() {
    let directory = temporary_directory("migration-v11-inventory-foundation");
    let connection = open_database(&production_database_config(&directory)).unwrap();

    assert_eq!(
        connection
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        CURRENT_SCHEMA_VERSION
    );
    for column in [
        "operation_kind",
        "payload_version",
        "canonical_payload",
        "payload_sha256",
    ] {
        assert!(connection
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM pragma_table_info('inventory_movements') WHERE name = ?1)",
                [column],
                |row| row.get::<_, bool>(0),
            )
            .unwrap());
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

#[test]
fn v18_adds_nullable_purchase_prices_and_preserves_unknown_history_on_v17_upgrade() {
    let directory = temporary_directory("migration-v18-purchase-prices");
    let path = create_version_twelve_database(&directory);
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch(VERSION_THIRTEEN_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 13).unwrap();
    connection.execute_batch(VERSION_FOURTEEN_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 14).unwrap();
    connection.execute_batch(VERSION_FIFTEEN_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 15).unwrap();
    connection.execute_batch(VERSION_SIXTEEN_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 16).unwrap();
    connection.execute_batch(VERSION_SEVENTEEN_MIGRATION).unwrap();
    connection.pragma_update(None, "user_version", 17).unwrap();
    drop(connection);

    let config = production_database_config(&directory);
    let connection = open_database(&config).unwrap();
    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION);
    assert_eq!(
        connection.query_row(
            "SELECT purchase_price_centavos FROM products WHERE id = 1",
            [],
            |row| row.get::<_, Option<i64>>(0),
        ).unwrap(),
        None,
    );
    assert_eq!(
        connection.query_row(
            "SELECT unit_purchase_price_centavos FROM inventory_movements WHERE id = 40",
            [],
            |row| row.get::<_, Option<i64>>(0),
        ).unwrap(),
        None,
    );
    assert!(connection.execute(
        "INSERT INTO products (category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos, purchase_price_centavos) VALUES (1, 'COST-001', 'Cost', 1, 100, 100, 0)",
        [],
    ).is_err());
    drop(connection);

    let reopened = open_database(&config).unwrap();
    assert_eq!(user_version(&path), CURRENT_SCHEMA_VERSION);
    assert_eq!(
        reopened.query_row(
            "SELECT purchase_price_centavos FROM products WHERE id = 1",
            [],
            |row| row.get::<_, Option<i64>>(0),
        ).unwrap(),
        None,
    );
    drop(reopened);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn fresh_v18_schema_has_nullable_price_columns_and_rejects_malformed_costs() {
    let directory = temporary_directory("migration-v18-fresh-schema");
    let connection = open_database(&production_database_config(&directory)).unwrap();
    assert_eq!(user_version(&directory.join("repuestos-autos.sqlite3")), CURRENT_SCHEMA_VERSION);
    connection.execute("INSERT INTO categories (name) VALUES ('Fresh category')", []).unwrap();
    connection.execute(
        "INSERT INTO products (category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (1, 'FRESH-001', 'Fresh product', 1, 100, 100)",
        [],
    ).unwrap();
    let product_id = connection.last_insert_rowid();
    assert_eq!(
        connection.query_row(
            "SELECT purchase_price_centavos FROM products WHERE id = ?1",
            [product_id],
            |row| row.get::<_, Option<i64>>(0),
        ).unwrap(),
        None,
    );
    assert!(connection.execute(
        "UPDATE products SET purchase_price_centavos = -1 WHERE id = ?1",
        [product_id],
    ).is_err());
    connection.execute(
        "INSERT INTO inventory_movements (product_id, movement_type, quantity_delta, unit_purchase_price_centavos) VALUES (?1, 'opening_stock', 1, NULL)",
        [product_id],
    ).unwrap();
    let movement_id = connection.last_insert_rowid();
    assert_eq!(
        connection.query_row(
            "SELECT unit_purchase_price_centavos FROM inventory_movements WHERE id = ?1",
            [movement_id],
            |row| row.get::<_, Option<i64>>(0),
        ).unwrap(),
        None,
    );
    assert!(connection.execute(
        "UPDATE inventory_movements SET unit_purchase_price_centavos = 100 WHERE id = ?1",
        [movement_id],
    ).is_err());
    assert!(connection.execute(
        "INSERT INTO inventory_movements (product_id, movement_type, quantity_delta, unit_purchase_price_centavos) VALUES (?1, 'opening_stock', 1, 0)",
        [product_id],
    ).is_err());
    drop(connection);
    std::fs::remove_dir_all(directory).unwrap();
}
