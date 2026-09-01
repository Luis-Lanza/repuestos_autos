use rusqlite::{Result, Transaction};

use crate::domain::catalog::{
    AttributeDefinition, CatalogSnapshot, CatalogTarget, TransitionPlan, ValidatedAttributeValue,
};

use super::CreateProductInput;

pub trait CatalogMetadataRepository {
    fn load(
        &self,
        transaction: &Transaction<'_>,
        target: CatalogTarget,
        entity_id: i64,
    ) -> Result<Option<CatalogSnapshot>>;
    fn category_name_exists(
        &self,
        transaction: &Transaction<'_>,
        id: i64,
        name: &str,
    ) -> Result<bool>;
    fn product_metadata_for_normalized_patch(
        &self,
        transaction: &Transaction<'_>,
        id: i64,
        sku: &str,
        name: &str,
    ) -> Result<Option<ProductMetadata>>;
    fn edit_category(
        &self,
        transaction: &Transaction<'_>,
        id: i64,
        revision: i64,
        name: &str,
    ) -> Result<CatalogSnapshot>;
    #[expect(
        clippy::too_many_arguments,
        reason = "the repository seam keeps the validated product patch fields explicit"
    )]
    fn edit_product(
        &self,
        transaction: &Transaction<'_>,
        id: i64,
        revision: i64,
        sku: &str,
        name: &str,
        price: i64,
        values: &[ValidatedAttributeValue],
    ) -> Result<CatalogSnapshot>;
}

pub struct ProductMetadata {
    pub definitions: Vec<AttributeDefinition>,
    pub duplicate_normalized_identity: bool,
}

pub trait CreateProductRepository {
    fn category_name(
        &self,
        transaction: &Transaction<'_>,
        category_id: i64,
    ) -> Result<Option<String>>;
    fn attribute_definitions(
        &self,
        transaction: &Transaction<'_>,
        category_id: i64,
    ) -> Result<Vec<AttributeDefinition>>;
    fn sku_exists(&self, transaction: &Transaction<'_>, sku: &str) -> Result<bool>;
    fn persist_product(
        &self,
        transaction: &Transaction<'_>,
        input: &CreateProductInput,
        values: &[ValidatedAttributeValue],
        category_name: &str,
    ) -> Result<i64>;
}

pub trait CatalogMaintenanceRepository {
    fn load(
        &self,
        transaction: &Transaction<'_>,
        target: CatalogTarget,
        entity_id: i64,
    ) -> Result<Option<CatalogSnapshot>>;
    fn apply(
        &self,
        transaction: &Transaction<'_>,
        target: CatalogTarget,
        entity_id: i64,
        plan: TransitionPlan,
    ) -> Result<CatalogSnapshot>;
}
