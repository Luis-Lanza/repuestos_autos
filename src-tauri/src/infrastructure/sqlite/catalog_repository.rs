use rusqlite::{params, OptionalExtension, Result, Transaction};

use crate::application::catalog::repository::CreateProductRepository;
use crate::application::catalog::CreateProductInput;
use crate::domain::catalog::{AttributeDefinition, FieldType, ValidatedAttributeValue};

pub struct SqliteCatalogRepository;

impl CreateProductRepository for SqliteCatalogRepository {
    fn category_name(
        &self,
        transaction: &Transaction<'_>,
        category_id: i64,
    ) -> Result<Option<String>> {
        transaction
            .query_row(
                "SELECT name FROM categories WHERE id = ?1",
                [category_id],
                |row| row.get(0),
            )
            .optional()
    }

    fn attribute_definitions(
        &self,
        transaction: &Transaction<'_>,
        category_id: i64,
    ) -> Result<Vec<AttributeDefinition>> {
        let mut statement = transaction.prepare(
            "SELECT d.id, d.field_type, d.required, o.value FROM attribute_definitions d LEFT JOIN attribute_options o ON o.definition_id = d.id WHERE d.category_id = ?1 ORDER BY d.id, o.rowid",
        )?;
        let rows = statement
            .query_map([category_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, bool>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            })?
            .collect::<Result<Vec<_>>>()?;
        let mut definitions = Vec::new();
        for (id, field_type, required, option) in rows {
            if definitions
                .last()
                .map(|definition: &AttributeDefinition| definition.id)
                != Some(id)
            {
                definitions.push(AttributeDefinition {
                    id,
                    field_type: FieldType::parse(&field_type)
                        .map_err(|_| rusqlite::Error::InvalidQuery)?,
                    required,
                    options: Vec::new(),
                });
            }
            if let Some(option) = option {
                if let Some(definition) = definitions.last_mut() {
                    definition.options.push(option);
                }
            }
        }
        Ok(definitions)
    }

    fn sku_exists(&self, transaction: &Transaction<'_>, sku: &str) -> Result<bool> {
        transaction
            .query_row(
                "SELECT 1 FROM products WHERE lower(sku) = lower(?1)",
                [sku],
                |_| Ok(()),
            )
            .optional()
            .map(|value| value.is_some())
    }

    fn persist_product(
        &self,
        transaction: &Transaction<'_>,
        input: &CreateProductInput,
        values: &[ValidatedAttributeValue],
        category_name: &str,
    ) -> Result<i64> {
        transaction.execute("INSERT INTO products (category_id, sku, name, active, minimum_unit_price_centavos) VALUES (?1, ?2, ?3, 1, ?4)", params![input.category_id, input.sku.trim(), input.name.trim(), input.catalog_unit_price_centavos])?;
        let product_id = transaction.last_insert_rowid();
        for value in values {
            match value {
                ValidatedAttributeValue::Text { definition_id, value } => transaction.execute("INSERT INTO product_attribute_values (product_id, definition_id, text_value, searchable_value) VALUES (?1, ?2, ?3, ?3)", params![product_id, definition_id, value])?,
                ValidatedAttributeValue::Number { definition_id, value, searchable } => transaction.execute("INSERT INTO product_attribute_values (product_id, definition_id, number_value, searchable_value) VALUES (?1, ?2, ?3, ?4)", params![product_id, definition_id, value, searchable])?,
                ValidatedAttributeValue::Option { definition_id, value } => transaction.execute("INSERT INTO product_attribute_values (product_id, definition_id, option_value, searchable_value) VALUES (?1, ?2, ?3, ?3)", params![product_id, definition_id, value])?,
            };
        }
        transaction.execute(
            "INSERT INTO stock_balances (product_id, quantity) VALUES (?1, ?2)",
            params![product_id, input.opening_quantity],
        )?;
        transaction.execute("INSERT INTO inventory_movements (product_id, movement_type, quantity_delta) VALUES (?1, 'opening_stock', ?2)", params![product_id, input.opening_quantity])?;
        transaction.execute("INSERT INTO catalog_product_search (rowid, product_id, content) VALUES (?1, ?1, lower(?2 || ' ' || ?3 || ' ' || ?4 || ' ' || COALESCE((SELECT group_concat(searchable_value, ' ') FROM product_attribute_values WHERE product_id = ?1), '')))", params![product_id, input.sku.trim(), input.name.trim(), category_name])?;
        Ok(product_id)
    }
}
