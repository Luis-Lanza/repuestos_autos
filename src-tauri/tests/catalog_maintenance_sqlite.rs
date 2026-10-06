use repuestos_autos::application::catalog::{
    AttributeValueInput, EditCatalogInput, EditCatalogUseCase, EditCategorySchemaError,
    EditCategorySchemaInput, EditCategorySchemaUseCase, MaintainCatalogError, MaintainCatalogInput,
    MaintainCatalogUseCase,
};
use repuestos_autos::domain::catalog::{
    CatalogActivity, CatalogIntent, CatalogTarget, CategorySchemaField, FieldType,
};
use repuestos_autos::infrastructure::sqlite::{
    open_database, open_seeded_catalog, production_database_config, SqliteCatalogRepository,
    CURRENT_SCHEMA_VERSION,
};

const VERSION_ONE_FIXTURE: &str = include_str!("fixtures/version1_fixed_price_legacy.sql");
const MIGRATIONS_THROUGH_VERSION_EIGHTEEN: [&str; 17] = [
    include_str!("../src/infrastructure/sqlite/migrations/0002_fixed_price_checkout.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0003_sale_line_product_snapshots.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0004_product_onboarding.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0005_catalog_onboarding_hardening.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0006_operational_inventory_control.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0007_catalog_maintenance.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0008_catalog_metadata_name_uniqueness.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0009_sales_history_index.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0010_post_sale_lifecycle.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0011_sale_idempotency_conflicts.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0012_inventory_idempotency_conflicts.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0013_catalog_dual_pricing.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0014_sale_list_price_snapshot.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0015_catalog_price_cap.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0016_product_images.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0017_product_image_thumbnails.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0018_global_product_purchase_price.sql"),
];

fn maintain(
    connection: &mut rusqlite::Connection,
    target: CatalogTarget,
    id: i64,
    intent: CatalogIntent,
    revision: i64,
) -> Result<repuestos_autos::domain::catalog::CatalogSnapshot, MaintainCatalogError> {
    MaintainCatalogUseCase::new(connection, SqliteCatalogRepository)
        .execute(MaintainCatalogInput::new(target, id, intent, revision))
}

fn count(connection: &rusqlite::Connection, table: &str) -> i64 {
    connection
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap()
}

#[test]
fn product_names_may_repeat_on_edit() {
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute_batch("INSERT INTO products (id, sku, name, category_id, list_price_centavos, minimum_unit_price_centavos, active) VALUES (91, 'OTHER', 'Shared name', 2, 2500, 2500, 1);").unwrap();
    let result = EditCatalogUseCase::new(&mut connection, SqliteCatalogRepository)
        .execute(EditCatalogInput::product(1, 0, "FLT-001", "Shared name", 2000, 2500, 2500, 0, vec![]));
    assert_eq!(result.unwrap().revision, 1);
    assert_eq!(count(&connection, "catalog_audit"), 1);
}

#[test]
fn category_metadata_reports_authoritative_active_product_counts() {
    let connection = open_seeded_catalog().unwrap();
    let categories = repuestos_autos::application::catalog::list_category_metadata(
        &connection,
        SqliteCatalogRepository,
    )
    .unwrap();
    assert!(categories
        .iter()
        .any(|category| category.active_product_count == 0));
    let category = categories
        .iter()
        .find(|category| category.active_product_count > 0)
        .unwrap();
    assert_eq!(category.active_product_count, 1);
    let category_id = category.category_id;

    connection
        .execute(
            "UPDATE products SET active = 0 WHERE category_id = ?1",
            [category_id],
        )
        .unwrap();
    let categories = repuestos_autos::application::catalog::list_category_metadata(
        &connection,
        SqliteCatalogRepository,
    )
    .unwrap();
    assert_eq!(
        categories
            .iter()
            .find(|category| category.category_id == category_id)
            .unwrap()
            .active_product_count,
        0
    );
}

#[test]
fn lifecycle_writes_are_guarded_audited_and_keep_category_and_product_independent() {
    let mut connection = open_seeded_catalog().unwrap();
    assert_eq!(
        maintain(
            &mut connection,
            CatalogTarget::Product,
            1,
            CatalogIntent::Archive,
            0
        )
        .unwrap()
        .activity,
        CatalogActivity::Archived
    );
    assert!(
        repuestos_autos::catalog::search_active_products(&connection, "filtro")
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        maintain(
            &mut connection,
            CatalogTarget::Category,
            1,
            CatalogIntent::Archive,
            0
        )
        .unwrap()
        .activity,
        CatalogActivity::Archived
    );
    assert_eq!(
        maintain(
            &mut connection,
            CatalogTarget::Category,
            1,
            CatalogIntent::Reactivate,
            1
        )
        .unwrap()
        .activity,
        CatalogActivity::Active
    );
    assert_eq!(
        connection
            .query_row("SELECT active FROM products WHERE id = 1", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(count(&connection, "catalog_audit"), 3);
    assert!(connection.execute("DELETE FROM catalog_audit", []).is_err());
}

#[test]
fn audit_failure_rolls_back_the_fact_search_and_audit_record() {
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute_batch("CREATE TRIGGER reject_catalog_audit BEFORE INSERT ON catalog_audit BEGIN SELECT RAISE(ABORT, 'forced'); END;").unwrap();
    assert_eq!(
        maintain(
            &mut connection,
            CatalogTarget::Product,
            1,
            CatalogIntent::Archive,
            0
        ),
        Err(MaintainCatalogError::PersistenceFailure)
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT active, revision FROM products WHERE id = 1",
                [],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            )
            .unwrap(),
        (1, 0)
    );
    assert_eq!(
        repuestos_autos::catalog::search_active_products(&connection, "filtro")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(count(&connection, "catalog_audit"), 0);
}

#[test]
fn search_requires_an_active_category_and_product() {
    let connection = open_seeded_catalog().unwrap();
    connection
        .execute("UPDATE categories SET active = 0 WHERE id = 1", [])
        .unwrap();
    assert!(
        repuestos_autos::catalog::search_active_products(&connection, "filtro")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn metadata_edits_guard_revisions_replace_values_refresh_search_and_audit_together() {
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute_batch("INSERT INTO attribute_definitions (id, category_id, label, field_type, required) VALUES (1, 1, 'old', 'text', 0), (2, 1, 'text', 'text', 1), (3, 1, 'number', 'number', 1), (4, 1, 'option', 'option', 1); INSERT INTO attribute_options VALUES (4, 'Toyota'); INSERT INTO product_attribute_values VALUES (1, 1, 'obsolete', NULL, NULL, 'obsolete');").unwrap();
    connection.execute("INSERT INTO sales (request_id, status, total_centavos, confirmed_at) VALUES ('metadata-price-history', 'confirmed', 2500, CURRENT_TIMESTAMP)", []).unwrap();
    let sale_id = connection.last_insert_rowid();
    connection.execute("INSERT INTO sale_lines (sale_id, product_id, quantity, negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos, line_total_centavos) VALUES (?1, 1, 1, 2500, 2500, 2500)", [sale_id]).unwrap();

    let edited = EditCatalogUseCase::new(&mut connection, SqliteCatalogRepository)
        .execute(EditCatalogInput::product(
            1,
            0,
            "NUE-001",
            "Nuevo filtro",
            2_000,
            4_000,
            3_000,
            0,
            vec![
                AttributeValueInput {
                    definition_id: 2,
                    value: "paper".into(),
                },
                AttributeValueInput {
                    definition_id: 3,
                    value: "2.5".into(),
                },
                AttributeValueInput {
                    definition_id: 4,
                    value: "Toyota".into(),
                },
            ],
        ))
        .unwrap();

    assert_eq!(edited.revision, 1);
    assert_eq!(
        connection
            .query_row(
                "SELECT sku, name, purchase_price_centavos, list_price_centavos, minimum_unit_price_centavos FROM products WHERE id = 1",
                [],
                |row| Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?
                ))
            )
            .unwrap(),
        ("NUE-001".into(), "Nuevo filtro".into(), 2_000, 4_000, 3_000)
    );
    assert_eq!(connection.query_row("SELECT COUNT(*) FROM product_attribute_values WHERE product_id = 1 AND ((definition_id = 2 AND text_value = 'paper') OR (definition_id = 3 AND number_value = 2.5) OR (definition_id = 4 AND option_value = 'Toyota'))", [], |row| row.get::<_, i64>(0)).unwrap(), 3);
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM product_attribute_values WHERE product_id = 1",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        3
    );
    assert!(
        repuestos_autos::catalog::search_active_products(&connection, "nue")
            .unwrap()
            .iter()
            .any(|product| product.sku == "NUE-001")
    );
    assert!(
        repuestos_autos::catalog::search_active_products(&connection, "flt-001")
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT negotiated_unit_price_centavos FROM sale_lines WHERE sale_id = ?1",
                [sale_id],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        2_500
    );
    EditCatalogUseCase::new(&mut connection, SqliteCatalogRepository)
        .execute(EditCatalogInput::category(1, 0, "Nuevo rubro"))
        .unwrap();
    assert!(
        repuestos_autos::catalog::search_active_products(&connection, "rubro")
            .unwrap()
            .len()
            == 1
    );
    assert!(
        repuestos_autos::catalog::search_active_products(&connection, "filtros")
            .unwrap()
            .is_empty()
    );
    assert_eq!(count(&connection, "catalog_audit"), 2);
}

#[test]
fn failed_metadata_audit_rolls_back_the_guarded_write_and_fts_document() {
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute_batch("CREATE TRIGGER reject_metadata_audit BEFORE INSERT ON catalog_audit BEGIN SELECT RAISE(ABORT, 'forced'); END;").unwrap();
    assert_eq!(
        EditCatalogUseCase::new(&mut connection, SqliteCatalogRepository).execute(
            EditCatalogInput::product(1, 0, "NUE-002", "Other", 2_000, 4_000, 3_000, 0, vec![])
        ),
        Err(MaintainCatalogError::PersistenceFailure)
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT sku, revision FROM products WHERE id = 1",
                [],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            )
            .unwrap(),
        ("FLT-001".into(), 0)
    );
    assert!(
        repuestos_autos::catalog::search_active_products(&connection, "nue")
            .unwrap()
            .is_empty()
    );
    assert_eq!(count(&connection, "catalog_audit"), 0);
}

#[test]
fn category_schema_edits_add_retire_audit_and_preserve_definition_ids_and_historical_values() {
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute_batch(
        "INSERT INTO attribute_definitions (id, category_id, label, field_type, required) VALUES (71, 1, 'Material', 'text', 0);
         INSERT INTO product_attribute_values (product_id, definition_id, text_value, searchable_value) VALUES (1, 71, 'Vintage steel', 'Vintage steel');",
    ).unwrap();
    let retained = CategorySchemaField {
        definition_id: Some(71),
        label: "Material".into(),
        field_type: FieldType::Text,
        required: false,
        options: vec![],
    };
    let added = CategorySchemaField {
        definition_id: None,
        label: "Length".into(),
        field_type: FieldType::Number,
        required: true,
        options: vec![],
    };
    let updated = EditCategorySchemaUseCase::new(&mut connection, SqliteCatalogRepository)
        .execute(EditCategorySchemaInput {
            category_id: 1,
            expected_revision: 0,
            fields: vec![retained, added],
        })
        .unwrap();
    assert_eq!(updated.revision, 1);
    let added_id: i64 = connection
        .query_row(
            "SELECT id FROM attribute_definitions WHERE category_id = 1 AND label = 'Length'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(added_id > 71);
    assert_eq!(
        connection
            .query_row(
                "SELECT active FROM attribute_definitions WHERE id = 71",
                [],
                |row| row.get::<_, bool>(0),
            )
            .unwrap(),
        true
    );

    let retired = EditCategorySchemaUseCase::new(&mut connection, SqliteCatalogRepository)
        .execute(EditCategorySchemaInput {
            category_id: 1,
            expected_revision: 1,
            fields: vec![],
        })
        .unwrap();
    assert_eq!(retired.revision, 2);
    assert_eq!(
        connection
            .query_row(
                "SELECT active FROM attribute_definitions WHERE id = 71",
                [],
                |row| row.get::<_, bool>(0),
            )
            .unwrap(),
        false
    );
    assert_eq!(connection.query_row(
        "SELECT text_value FROM product_attribute_values WHERE product_id = 1 AND definition_id = 71", [], |row| row.get::<_, String>(0),
    ).unwrap(), "Vintage steel");
    assert_eq!(connection.query_row(
        "SELECT COUNT(*) FROM catalog_audit WHERE entity_type = 'category' AND operation = 'edit_metadata' AND revision IN (1, 2)", [], |row| row.get::<_, i64>(0),
    ).unwrap(), 2);
    let details = repuestos_autos::application::catalog::read_catalog_metadata_detail(
        &connection,
        CatalogTarget::Category,
        1,
    )
    .unwrap()
    .unwrap();
    let repuestos_autos::application::catalog::CatalogMetadataDetail::Category {
        attribute_definitions,
        ..
    } = details
    else {
        panic!("expected category details")
    };
    assert!(attribute_definitions
        .iter()
        .any(|field| field.definition_id == 71 && !field.active));

    let replacement = EditCategorySchemaUseCase::new(&mut connection, SqliteCatalogRepository)
        .execute(EditCategorySchemaInput {
            category_id: 1,
            expected_revision: 2,
            fields: vec![CategorySchemaField {
                definition_id: None,
                label: "Material".into(),
                field_type: FieldType::Option,
                required: true,
                options: vec!["Steel".into()],
            }],
        })
        .unwrap();
    assert_eq!(replacement.revision, 3);
    let replacement_id: i64 = connection.query_row(
        "SELECT id FROM attribute_definitions WHERE category_id = 1 AND label = 'Material' AND active = 1",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_ne!(replacement_id, 71);
    assert_eq!(connection.query_row(
        "SELECT COUNT(*) FROM attribute_definitions WHERE category_id = 1 AND label = 'Material'",
        [],
        |row| row.get::<_, i64>(0),
    ).unwrap(), 2);
    assert_eq!(connection.query_row(
        "SELECT text_value FROM product_attribute_values WHERE product_id = 1 AND definition_id = 71",
        [],
        |row| row.get::<_, String>(0),
    ).unwrap(), "Vintage steel");
    let foreign_key_violations: i64 = connection
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(foreign_key_violations, 0);
}

#[test]
fn category_schema_edit_rejects_stale_revision_and_rolls_back_when_audit_fails() {
    let mut connection = open_seeded_catalog().unwrap();
    let mut input = EditCategorySchemaInput {
        category_id: 2,
        expected_revision: 1,
        fields: vec![CategorySchemaField {
            definition_id: None,
            label: "Length".into(),
            field_type: FieldType::Number,
            required: false,
            options: vec![],
        }],
    };
    assert_eq!(
        EditCategorySchemaUseCase::new(&mut connection, SqliteCatalogRepository)
            .execute(input.clone()),
        Err(EditCategorySchemaError::StaleCategory)
    );
    connection.execute_batch("CREATE TRIGGER reject_category_schema_audit BEFORE INSERT ON catalog_audit WHEN new.entity_type = 'category' BEGIN SELECT RAISE(ABORT, 'forced'); END;").unwrap();
    input.expected_revision = 0;
    assert_eq!(
        EditCategorySchemaUseCase::new(&mut connection, SqliteCatalogRepository).execute(input),
        Err(EditCategorySchemaError::PersistenceFailure)
    );
    assert_eq!(
        connection
            .query_row("SELECT revision FROM categories WHERE id = 2", [], |row| {
                row.get::<_, i64>(0)
            })
            .unwrap(),
        0
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM attribute_definitions WHERE category_id = 2",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    assert_eq!(count(&connection, "catalog_audit"), 0);
}

#[test]
fn product_edit_preserves_retired_values_and_detail_exposes_them() {
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute_batch(
        "INSERT INTO attribute_definitions (id, category_id, label, field_type, required, active)
         VALUES (71, 1, 'Legacy material', 'text', 0, 0), (72, 1, 'Current length', 'number', 1, 1);
         INSERT INTO product_attribute_values (product_id, definition_id, text_value, searchable_value)
         VALUES (1, 71, 'Vintage steel', 'Vintage steel');
         INSERT INTO product_attribute_values (product_id, definition_id, number_value, searchable_value)
         VALUES (1, 72, 12.5, '12.5');",
    ).unwrap();

    EditCatalogUseCase::new(&mut connection, SqliteCatalogRepository)
        .execute(EditCatalogInput::product(
            1,
            0,
            "FLT-001",
            "Oil filter",
            2_000,
            2_500,
            2_500,
            0,
            vec![AttributeValueInput {
                definition_id: 72,
                value: "15.5".into(),
            }],
        ))
        .unwrap();

    assert_eq!(connection.query_row(
        "SELECT text_value FROM product_attribute_values WHERE product_id = 1 AND definition_id = 71",
        [],
        |row| row.get::<_, String>(0),
    ).unwrap(), "Vintage steel");
    let details = repuestos_autos::application::catalog::read_catalog_metadata_detail(
        &connection,
        CatalogTarget::Product,
        1,
    )
    .unwrap()
    .unwrap();
    let repuestos_autos::application::catalog::CatalogMetadataDetail::Product {
        attribute_definitions,
        attribute_values,
        ..
    } = details
    else {
        panic!("expected product details")
    };
    assert!(attribute_definitions
        .iter()
        .any(|field| field.definition_id == 71 && !field.active));
    assert!(attribute_values
        .iter()
        .any(|value| value.definition_id == 71 && value.value == "Vintage steel"));
}

#[test]
fn current_schema_migrates_and_validates_field_lifecycle_metadata() {
    let connection = open_seeded_catalog().unwrap();
    assert_eq!(
        connection
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        CURRENT_SCHEMA_VERSION
    );
    let columns = connection
        .prepare("PRAGMA table_info(attribute_definitions)")
        .unwrap()
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, bool>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    assert!(columns
        .iter()
        .any(|(name, data_type, not_null, default_value)| {
            name == "active"
                && data_type == "INTEGER"
                && *not_null
                && default_value.as_deref() == Some("1")
        }));
    let table_sql: String = connection
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'attribute_definitions'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(table_sql
        .to_ascii_lowercase()
        .contains("check (active in (0, 1))"));

    let index_columns = connection
        .prepare("PRAGMA index_info(attribute_definitions_category_active_idx)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(2))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    assert_eq!(index_columns, ["category_id", "active", "id"]);
    let directory = std::env::temp_dir().join(format!(
        "category-field-schema-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let config = production_database_config(&directory);
    drop(open_database(&config).unwrap());
    let malformed = rusqlite::Connection::open(directory.join("repuestos-autos.sqlite3")).unwrap();
    malformed
        .execute_batch(
            "DROP INDEX attribute_definitions_category_active_idx;
         CREATE INDEX attribute_definitions_category_active_idx
         ON attribute_definitions (active, category_id, id);",
        )
        .unwrap();
    drop(malformed);
    assert!(open_database(&config).is_err());

    let malformed = rusqlite::Connection::open(directory.join("repuestos-autos.sqlite3")).unwrap();
    malformed.execute_batch(
        "DROP INDEX attribute_definitions_category_active_idx;
         CREATE INDEX attribute_definitions_category_active_idx
         ON attribute_definitions (category_id, active, id);
         DROP INDEX attribute_definitions_active_category_label_idx;
         CREATE INDEX attribute_definitions_active_category_label_idx
         ON attribute_definitions (category_id, label) WHERE active = 1;",
    ).unwrap();
    drop(malformed);
    assert!(open_database(&config).is_err());

    let malformed = rusqlite::Connection::open(directory.join("repuestos-autos.sqlite3")).unwrap();
    malformed.execute_batch(
        "DROP INDEX attribute_definitions_category_active_idx;
         CREATE INDEX attribute_definitions_category_active_idx
         ON attribute_definitions (category_id, active, id);
         DROP INDEX attribute_definitions_active_category_label_idx;
         CREATE UNIQUE INDEX attribute_definitions_active_category_label_idx
         ON attribute_definitions (category_id, label) WHERE active = 1 OR active = 0;",
    ).unwrap();
    drop(malformed);
    assert!(open_database(&config).is_err());

    let malformed = rusqlite::Connection::open(directory.join("repuestos-autos.sqlite3")).unwrap();
    malformed
        .execute_batch(
            "DROP INDEX attribute_definitions_category_active_idx;
             CREATE INDEX attribute_definitions_category_active_idx
             ON attribute_definitions (category_id, active, id);
             DROP INDEX attribute_definitions_active_category_label_idx;
             CREATE UNIQUE INDEX attribute_definitions_active_category_label_idx
             ON attribute_definitions (category_id, label) WHERE active = 1;
             CREATE UNIQUE INDEX differently_named_global_category_label
             ON attribute_definitions (category_id, label);",
        )
        .unwrap();
    drop(malformed);
    assert!(open_database(&config).is_err());
}

#[test]
fn version_nineteen_migration_preserves_definition_options_values_and_foreign_keys() {
    let directory = std::env::temp_dir().join(format!(
        "category-field-migration-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let config = production_database_config(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    let legacy = rusqlite::Connection::open(config.path()).unwrap();
    legacy.execute_batch(VERSION_ONE_FIXTURE).unwrap();
    legacy.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    for (index, migration) in MIGRATIONS_THROUGH_VERSION_EIGHTEEN.iter().enumerate() {
        legacy.execute_batch(migration).unwrap();
        legacy
            .pragma_update(None, "user_version", (index + 2) as i64)
            .unwrap();
    }
    assert_eq!(legacy.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0)).unwrap(), 18);
    legacy
        .execute_batch(
            "INSERT INTO attribute_definitions (id, category_id, label, field_type, required)
         VALUES (81, 1, 'Legacy migration field', 'option', 1);
         INSERT INTO attribute_options (definition_id, value) VALUES (81, 'Original option');
         INSERT INTO product_attribute_values
             (product_id, definition_id, option_value, searchable_value)
         VALUES (1, 81, 'Original option', 'Original option');
         CREATE UNIQUE INDEX legacy_attribute_definition_category_label
         ON attribute_definitions (category_id, label);
         PRAGMA user_version = 18;",
        )
        .unwrap();
    drop(legacy);

    let migrated = open_database(&config).unwrap();
    assert_eq!(
        migrated
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        CURRENT_SCHEMA_VERSION
    );
    assert_eq!(migrated.query_row(
        "SELECT id FROM attribute_definitions WHERE category_id = 1 AND label = 'Legacy migration field'",
        [],
        |row| row.get::<_, i64>(0),
    ).unwrap(), 81);
    assert_eq!(
        migrated
            .query_row(
                "SELECT value FROM attribute_options WHERE definition_id = 81",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
        "Original option"
    );
    assert_eq!(migrated.query_row(
        "SELECT option_value FROM product_attribute_values WHERE product_id = 1 AND definition_id = 81",
        [],
        |row| row.get::<_, String>(0),
    ).unwrap(), "Original option");
    assert_eq!(
        migrated
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get::<_, i64>(0)
            },)
            .unwrap(),
        0
    );
}

#[test]
fn product_images_decode_png_jpeg_and_webp_and_reject_invalid_or_mismatched_content() {
    use repuestos_autos::application::catalog::{ProductImage, ProductImageValidationError};

    for (mime, bytes) in [
        ("image/png", generated_png(2, 2)),
        ("image/jpeg", generated_jpeg(2, 2)),
        ("image/webp", generated_webp(2, 2)),
    ] {
        let image = ProductImage::new(mime, bytes.clone()).unwrap();
        assert_eq!(image.mime_type(), mime);
        assert_eq!(image.bytes(), bytes);
    }
    assert_eq!(
        ProductImage::new("image/png", generated_jpeg(1, 1)).unwrap_err(),
        ProductImageValidationError::InvalidImage
    );
    assert_eq!(
        ProductImage::new("image/gif", generated_png(1, 1)).unwrap_err(),
        ProductImageValidationError::UnsupportedImage
    );
    assert_eq!(
        ProductImage::new("image/png", b"not an image".to_vec()).unwrap_err(),
        ProductImageValidationError::InvalidImage
    );
}

#[test]
fn product_image_validation_enforces_byte_and_dimension_boundaries() {
    use repuestos_autos::application::catalog::{
        ProductImage, ProductImageValidationError, MAX_PRODUCT_IMAGE_BYTES,
        MAX_PRODUCT_IMAGE_DIMENSION,
    };

    let mut exact_limit = generated_png(1, 1);
    exact_limit.resize(MAX_PRODUCT_IMAGE_BYTES, 0);
    assert!(ProductImage::new("image/png", exact_limit).is_ok());
    assert_eq!(
        ProductImage::new("image/png", vec![0; MAX_PRODUCT_IMAGE_BYTES + 1]).unwrap_err(),
        ProductImageValidationError::ImageTooLarge
    );
    assert!(ProductImage::new("image/png", generated_png(MAX_PRODUCT_IMAGE_DIMENSION, 1)).is_ok());
    assert_eq!(
        ProductImage::new(
            "image/png",
            generated_png(MAX_PRODUCT_IMAGE_DIMENSION + 1, 1)
        )
        .unwrap_err(),
        ProductImageValidationError::ImageTooLarge
    );
}

#[test]
fn product_image_repository_replaces_reads_removes_and_rejects_missing_products() {
    use image::GenericImageView;
    use repuestos_autos::application::catalog::{
        read_product_image, remove_product_image, replace_product_image, ProductImage,
        ProductImagePersistenceError,
    };

    let mut connection = open_seeded_catalog().unwrap();
    let first = ProductImage::new("image/png", generated_png(1, 1)).unwrap();
    let replacement = ProductImage::new("image/webp", generated_webp(1, 1)).unwrap();
    replace_product_image(&mut connection, 1, 0, &first).unwrap();
    assert!(connection
        .execute(
            "UPDATE product_images SET thumbnail_mime_type = 'image/jpeg', thumbnail_bytes = NULL
         WHERE product_id = 1",
            [],
        )
        .is_err());
    let stored_first = read_product_image(&connection, 1).unwrap().unwrap();
    assert_eq!(stored_first, first);
    let first_thumbnail = stored_first.thumbnail().unwrap();
    assert_eq!(first_thumbnail.mime_type(), "image/jpeg");
    assert_eq!(
        image::load_from_memory(first_thumbnail.bytes())
            .unwrap()
            .dimensions(),
        (1, 1)
    );

    replace_product_image(&mut connection, 1, 1, &replacement).unwrap();
    let stored_replacement = read_product_image(&connection, 1).unwrap().unwrap();
    assert_eq!(stored_replacement.mime_type(), "image/webp");
    assert_eq!(stored_replacement.bytes(), replacement.bytes());
    let thumbnail = stored_replacement.thumbnail().unwrap();
    assert_eq!(thumbnail.mime_type(), "image/jpeg");
    assert_eq!(
        image::load_from_memory(thumbnail.bytes())
            .unwrap()
            .dimensions(),
        (1, 1)
    );
    let large = ProductImage::new("image/png", generated_png(600, 300)).unwrap();
    let large_thumbnail = large.thumbnail().unwrap();
    let dimensions = image::load_from_memory(large_thumbnail.bytes())
        .unwrap()
        .dimensions();
    assert!(dimensions.0 <= 256 && dimensions.1 <= 256);
    assert_eq!(count(&connection, "product_images"), 1);
    assert_eq!(count(&connection, "product_images"), 1);
    assert_eq!(remove_product_image(&mut connection, 1, 2).unwrap(), 3);
    assert_eq!(read_product_image(&connection, 1).unwrap(), None);
    assert_eq!(remove_product_image(&mut connection, 1, 3).unwrap(), 4);
    assert_eq!(
        replace_product_image(&mut connection, 999, 0, &first),
        Err(ProductImagePersistenceError::MissingProduct)
    );
    assert_eq!(count(&connection, "product_images"), 0);
}

#[test]
fn persisted_jpeg_thumbnail_rejects_png_or_webp_bytes() {
    use repuestos_autos::application::catalog::{
        read_product_image, replace_product_image, ProductImage, ProductImagePersistenceError,
    };

    for mislabeled_bytes in [generated_png(2, 2), generated_webp(2, 2)] {
        let mut connection = open_seeded_catalog().unwrap();
        let original = ProductImage::new("image/png", generated_png(3, 3)).unwrap();
        replace_product_image(&mut connection, 1, 0, &original).unwrap();
        connection
            .execute(
                "UPDATE product_images SET thumbnail_mime_type = 'image/jpeg', thumbnail_bytes = ?1
             WHERE product_id = 1",
                [mislabeled_bytes],
            )
            .unwrap();

        assert_eq!(
            read_product_image(&connection, 1),
            Err(ProductImagePersistenceError::PersistenceFailure)
        );
    }
}

#[test]
fn legacy_product_image_without_thumbnail_is_readable_and_replacement_derives_one() {
    use repuestos_autos::application::catalog::{
        read_product_image, replace_product_image, ProductImage,
    };

    let mut connection = open_seeded_catalog().unwrap();
    let old_bytes = generated_png(3, 2);
    connection
        .execute(
            "INSERT INTO product_images (product_id, mime_type, image_bytes)
         VALUES (?1, ?2, ?3)",
            rusqlite::params![1, "image/png", old_bytes],
        )
        .unwrap();
    let old = read_product_image(&connection, 1).unwrap().unwrap();
    assert!(old.thumbnail().is_none());

    let replacement = ProductImage::new("image/jpeg", generated_jpeg(2, 1)).unwrap();
    replace_product_image(&mut connection, 1, 0, &replacement).unwrap();
    let stored = read_product_image(&connection, 1).unwrap().unwrap();
    assert_eq!(stored.bytes(), replacement.bytes());
    assert!(stored.thumbnail().is_some());
}

#[test]
fn failed_product_image_replacement_keeps_the_previous_record() {
    use repuestos_autos::application::catalog::{
        read_product_image, replace_product_image, ProductImage, ProductImagePersistenceError,
    };

    let mut connection = open_seeded_catalog().unwrap();
    let original = ProductImage::new("image/png", generated_png(1, 1)).unwrap();
    let replacement = ProductImage::new("image/jpeg", generated_jpeg(1, 1)).unwrap();
    replace_product_image(&mut connection, 1, 0, &original).unwrap();
    connection.execute_batch("CREATE TRIGGER reject_product_image_update BEFORE UPDATE ON product_images BEGIN SELECT RAISE(ABORT, 'forced'); END;").unwrap();
    assert_eq!(
        replace_product_image(&mut connection, 1, 1, &replacement),
        Err(ProductImagePersistenceError::PersistenceFailure)
    );
    assert_eq!(read_product_image(&connection, 1).unwrap(), Some(original));
}

#[test]
fn product_image_replace_and_remove_advance_revision_and_reject_stale_writes() {
    use repuestos_autos::application::catalog::{
        read_product_image, remove_product_image, replace_product_image, ProductImage,
        ProductImagePersistenceError,
    };

    let mut connection = open_seeded_catalog().unwrap();
    let image = ProductImage::new("image/png", generated_png(2, 2)).unwrap();
    assert_eq!(
        replace_product_image(&mut connection, 1, 0, &image).unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row("SELECT revision FROM products WHERE id = 1", [], |row| row
                .get::<_, i64>(
                0
            ))
            .unwrap(),
        1
    );
    assert_eq!(
        replace_product_image(&mut connection, 1, 0, &image),
        Err(ProductImagePersistenceError::StaleCatalogRecord)
    );
    assert_eq!(read_product_image(&connection, 1).unwrap(), Some(image));
    assert_eq!(remove_product_image(&mut connection, 1, 1).unwrap(), 2);
    assert_eq!(read_product_image(&connection, 1).unwrap(), None);
    assert_eq!(
        remove_product_image(&mut connection, 1, 1),
        Err(ProductImagePersistenceError::StaleCatalogRecord)
    );
}

#[test]
fn product_thumbnail_read_rejects_non_jpeg_persisted_bytes() {
    use repuestos_autos::application::catalog::{
        read_product_image_thumbnail, replace_product_image, ProductImage,
        ProductImagePersistenceError,
    };

    let mut connection = open_seeded_catalog().unwrap();
    let image = ProductImage::new("image/png", generated_png(2, 2)).unwrap();
    replace_product_image(&mut connection, 1, 0, &image).unwrap();
    connection
        .execute(
            "UPDATE product_images SET thumbnail_bytes = ?1 WHERE product_id = 1",
            [generated_png(2, 2)],
        )
        .unwrap();
    assert_eq!(
        read_product_image_thumbnail(&connection, 1),
        Err(ProductImagePersistenceError::PersistenceFailure)
    );
}

#[test]
fn product_thumbnail_read_returns_only_bounded_jpeg_with_revision_identity() {
    use repuestos_autos::application::catalog::{
        read_product_image_thumbnail, replace_product_image, ProductImage,
    };

    let mut connection = open_seeded_catalog().unwrap();
    let image = ProductImage::new("image/png", generated_png(600, 300)).unwrap();
    replace_product_image(&mut connection, 1, 0, &image).unwrap();
    let thumbnail = read_product_image_thumbnail(&connection, 1)
        .unwrap()
        .unwrap();
    assert_eq!(thumbnail.product_id, 1);
    assert_eq!(thumbnail.revision, 1);
    assert_eq!(thumbnail.mime_type, "image/jpeg");
    assert!(thumbnail.bytes.len() < image.bytes().len());
    assert!(thumbnail.bytes.starts_with(&[0xff, 0xd8]));
}

fn generated_png(width: u32, height: u32) -> Vec<u8> {
    use image::ImageEncoder;
    let pixels = vec![255; (width * height * 4) as usize];
    let mut bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut bytes)
        .write_image(&pixels, width, height, image::ExtendedColorType::Rgba8)
        .unwrap();
    bytes
}

fn generated_jpeg(width: u32, height: u32) -> Vec<u8> {
    use image::ImageEncoder;
    let pixels = vec![127; (width * height * 3) as usize];
    let mut bytes = Vec::new();
    image::codecs::jpeg::JpegEncoder::new(&mut bytes)
        .write_image(&pixels, width, height, image::ExtendedColorType::Rgb8)
        .unwrap();
    bytes
}

fn generated_webp(width: u32, height: u32) -> Vec<u8> {
    use image::ImageEncoder;
    let pixels = vec![127; (width * height * 3) as usize];
    let mut bytes = Vec::new();
    image::codecs::webp::WebPEncoder::new_lossless(&mut bytes)
        .write_image(&pixels, width, height, image::ExtendedColorType::Rgb8)
        .unwrap();
    bytes
}
