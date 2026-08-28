use repuestos_autos::application::catalog::{
    MaintainCatalogError, MaintainCatalogInput, MaintainCatalogUseCase,
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
