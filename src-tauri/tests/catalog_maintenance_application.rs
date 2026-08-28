use repuestos_autos::application::catalog::{
    MaintainCatalogError, MaintainCatalogInput, MaintainCatalogUseCase,
};
use repuestos_autos::domain::catalog::{CatalogIntent, CatalogTarget};
use repuestos_autos::infrastructure::sqlite::{open_seeded_catalog, SqliteCatalogRepository};

#[test]
fn use_case_returns_stable_lifecycle_and_stale_revision_outcomes_before_writes() {
    let mut connection = open_seeded_catalog().unwrap();
    let blocked = MaintainCatalogUseCase::new(&mut connection, SqliteCatalogRepository).execute(
        MaintainCatalogInput::new(CatalogTarget::Category, 1, CatalogIntent::Archive, 0),
    );
    assert_eq!(blocked, Err(MaintainCatalogError::LifecycleBlocked));

    let stale = MaintainCatalogUseCase::new(&mut connection, SqliteCatalogRepository).execute(
        MaintainCatalogInput::new(CatalogTarget::Product, 1, CatalogIntent::Archive, 1),
    );
    assert_eq!(stale, Err(MaintainCatalogError::StaleCatalogRecord));
}
