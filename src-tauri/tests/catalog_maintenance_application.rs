use repuestos_autos::application::catalog::repository::{
    CatalogMetadataRepository, ProductMetadata,
};
use repuestos_autos::application::catalog::{
    EditCatalogInput, EditCatalogUseCase, MaintainCatalogError, MaintainCatalogInput,
    MaintainCatalogUseCase,
};
use repuestos_autos::domain::catalog::{
    AttributeDefinition, CatalogActivity, CatalogIntent, CatalogSnapshot, CatalogTarget, FieldType,
    ValidatedAttributeValue,
};
use repuestos_autos::infrastructure::sqlite::{open_seeded_catalog, SqliteCatalogRepository};

struct MetadataRepository {
    duplicate: bool,
    write_stale: bool,
}

fn snapshot(target: CatalogTarget, revision: i64) -> CatalogSnapshot {
    CatalogSnapshot {
        target,
        activity: CatalogActivity::Active,
        category_activity: CatalogActivity::Active,
        active_products: 0,
        values_valid: true,
        revision,
    }
}

impl CatalogMetadataRepository for MetadataRepository {
    fn load(
        &self,
        _: &rusqlite::Transaction<'_>,
        target: CatalogTarget,
        _: i64,
    ) -> rusqlite::Result<Option<CatalogSnapshot>> {
        Ok(Some(snapshot(target, 0)))
    }
    fn category_name_exists(
        &self,
        _: &rusqlite::Transaction<'_>,
        _: i64,
        _: &str,
    ) -> rusqlite::Result<bool> {
        Ok(self.duplicate)
    }
    fn product_metadata_for_normalized_patch(
        &self,
        _: &rusqlite::Transaction<'_>,
        _: i64,
        _: &str,
        _: &str,
    ) -> rusqlite::Result<Option<ProductMetadata>> {
        Ok(Some(ProductMetadata {
            definitions: vec![AttributeDefinition {
                id: 1,
                field_type: FieldType::Number,
                required: true,
                options: vec![],
            }],
            duplicate_normalized_identity: self.duplicate,
        }))
    }
    fn edit_category(
        &self,
        _: &rusqlite::Transaction<'_>,
        _: i64,
        revision: i64,
        _: &str,
    ) -> rusqlite::Result<CatalogSnapshot> {
        Ok(snapshot(CatalogTarget::Category, revision + 1))
    }
    fn edit_product(
        &self,
        _: &rusqlite::Transaction<'_>,
        _: i64,
        revision: i64,
        _: &str,
        _: &str,
        _: i64,
        _: &[ValidatedAttributeValue],
    ) -> rusqlite::Result<CatalogSnapshot> {
        if self.write_stale {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(snapshot(CatalogTarget::Product, revision + 1))
    }
}

fn edit(
    input: EditCatalogInput,
    duplicate: bool,
    write_stale: bool,
) -> Result<i64, MaintainCatalogError> {
    let mut connection = rusqlite::Connection::open_in_memory().unwrap();
    EditCatalogUseCase::new(
        &mut connection,
        MetadataRepository {
            duplicate,
            write_stale,
        },
    )
    .execute(input)
    .map(|snapshot| snapshot.revision)
}

fn product(revision: i64) -> EditCatalogInput {
    EditCatalogInput::product(
        1,
        revision,
        "FLT-002",
        "  PREMIUM FILTER ",
        3_000,
        vec![repuestos_autos::application::catalog::AttributeValueInput {
            definition_id: 1,
            value: "2".into(),
        }],
    )
}

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

#[test]
fn metadata_edits_require_the_current_revision_and_normalized_unique_values() {
    assert_eq!(
        edit(EditCatalogInput::category(1, 1, "Filters"), false, false),
        Err(MaintainCatalogError::StaleCatalogRecord)
    );
    assert_eq!(
        edit(EditCatalogInput::category(1, 0, "  filters "), true, false),
        Err(MaintainCatalogError::MissingCatalogRecord)
    );
}

#[test]
fn product_patches_advance_or_reject_normalized_names_and_guarded_stale_writes() {
    let cases = [
        (false, 0, false, Ok(1)),
        (
            true,
            0,
            false,
            Err(MaintainCatalogError::MissingCatalogRecord),
        ),
        (
            false,
            1,
            false,
            Err(MaintainCatalogError::StaleCatalogRecord),
        ),
        (
            false,
            0,
            true,
            Err(MaintainCatalogError::StaleCatalogRecord),
        ),
    ];
    for (duplicate, revision, write_stale, expected) in cases {
        assert_eq!(edit(product(revision), duplicate, write_stale), expected);
    }
}
