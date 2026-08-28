use repuestos_autos::application::catalog::{
    AttributeValueInput, EditCatalogInput, EditCatalogUseCase, MaintainCatalogError,
    MaintainCatalogInput, MaintainCatalogUseCase,
};
use repuestos_autos::domain::catalog::{CatalogActivity, CatalogIntent, CatalogTarget};
use repuestos_autos::infrastructure::sqlite::{open_seeded_catalog, SqliteCatalogRepository};

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
            3_000,
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
                "SELECT sku, name, minimum_unit_price_centavos FROM products WHERE id = 1",
                [],
                |row| Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?
                ))
            )
            .unwrap(),
        ("NUE-001".into(), "Nuevo filtro".into(), 3_000)
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
            EditCatalogInput::product(1, 0, "NUE-002", "Other", 3_000, vec![])
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
