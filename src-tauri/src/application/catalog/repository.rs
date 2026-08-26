use rusqlite::{Result, Transaction};

use crate::domain::catalog::{AttributeDefinition, ValidatedAttributeValue};

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
