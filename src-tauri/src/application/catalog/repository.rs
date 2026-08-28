use rusqlite::{Result, Transaction};

use crate::domain::catalog::{
    AttributeDefinition, CatalogSnapshot, CatalogTarget, TransitionPlan, ValidatedAttributeValue,
};

use super::CreateProductInput;

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
