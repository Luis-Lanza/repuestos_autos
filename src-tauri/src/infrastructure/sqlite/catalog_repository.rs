use rusqlite::{params, OptionalExtension, Result, Transaction};

use crate::application::catalog::repository::{
    CatalogMaintenanceRepository, CreateProductRepository,
};
use crate::application::catalog::CreateProductInput;
use crate::domain::catalog::{
    AttributeDefinition, CatalogActivity, CatalogSnapshot, CatalogTarget, FieldType,
    TransitionPlan, ValidatedAttributeValue,
};

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

impl CatalogMaintenanceRepository for SqliteCatalogRepository {
    fn load(
        &self,
        transaction: &Transaction<'_>,
        target: CatalogTarget,
        entity_id: i64,
    ) -> Result<Option<CatalogSnapshot>> {
        let row = match target {
            CatalogTarget::Category => transaction
                .query_row(
                    "SELECT active, active, (SELECT COUNT(*) FROM products WHERE category_id = categories.id AND active = 1), 1, revision FROM categories WHERE id = ?1",
                    [entity_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
                )
                .optional()?,
            CatalogTarget::Product => transaction
                .query_row(
                    "SELECT p.active, c.active, 0, NOT EXISTS (SELECT 1 FROM attribute_definitions d LEFT JOIN product_attribute_values v ON v.product_id = p.id AND v.definition_id = d.id WHERE d.category_id = p.category_id AND ((d.required = 1 AND v.definition_id IS NULL) OR (v.definition_id IS NOT NULL AND ((d.field_type = 'text' AND v.text_value IS NULL) OR (d.field_type = 'number' AND v.number_value IS NULL) OR (d.field_type = 'option' AND (v.option_value IS NULL OR NOT EXISTS (SELECT 1 FROM attribute_options o WHERE o.definition_id = d.id AND o.value = v.option_value))))))), p.revision FROM products p JOIN categories c ON c.id = p.category_id WHERE p.id = ?1",
                    [entity_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
                )
                .optional()?,
        };
        row.map(
            |(activity, category_activity, active_products, values_valid, revision)| {
                Ok(CatalogSnapshot {
                    target,
                    activity: activity_from_sql(activity)?,
                    category_activity: activity_from_sql(category_activity)?,
                    active_products,
                    values_valid,
                    revision,
                })
            },
        )
        .transpose()
    }

    fn apply(
        &self,
        transaction: &Transaction<'_>,
        target: CatalogTarget,
        entity_id: i64,
        plan: TransitionPlan,
    ) -> Result<CatalogSnapshot> {
        let before = self
            .load(transaction, target, entity_id)?
            .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        let active = matches!(plan.activity, CatalogActivity::Active) as i64;
        let updated = match target {
            CatalogTarget::Category => transaction.execute(
                "UPDATE categories SET active = ?1, revision = revision + 1 WHERE id = ?2 AND revision = ?3 AND (?1 = 1 OR NOT EXISTS (SELECT 1 FROM products WHERE category_id = ?2 AND active = 1))",
                params![active, entity_id, plan.expected_revision],
            ),
            CatalogTarget::Product => transaction.execute(
                "UPDATE products SET active = ?1, revision = revision + 1 WHERE id = ?2 AND revision = ?3 AND (?1 = 0 OR EXISTS (SELECT 1 FROM categories c WHERE c.id = products.category_id AND c.active = 1))",
                params![active, entity_id, plan.expected_revision],
            ),
        }
        ?;
        if updated != 1 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        if target == CatalogTarget::Product {
            refresh_product_search(transaction, entity_id)?;
        }
        let after = self
            .load(transaction, target, entity_id)?
            .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        transaction
            .execute(
                "INSERT INTO catalog_audit (entity_type, entity_id, operation, before_json, after_json, revision) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![target_name(target), entity_id, operation_name(plan.activity), snapshot_json(before), snapshot_json(after), after.revision],
            )
            ?;
        Ok(after)
    }
}

fn activity_from_sql(value: i64) -> Result<CatalogActivity> {
    match value {
        1 => Ok(CatalogActivity::Active),
        0 => Ok(CatalogActivity::Archived),
        _ => Err(rusqlite::Error::InvalidQuery),
    }
}

fn refresh_product_search(transaction: &Transaction<'_>, product_id: i64) -> Result<()> {
    transaction.execute(
        "DELETE FROM catalog_product_search WHERE rowid = ?1",
        [product_id],
    )?;
    transaction.execute(
        "INSERT INTO catalog_product_search (rowid, product_id, content) SELECT p.id, p.id, lower(p.sku || ' ' || p.name || ' ' || c.name || ' ' || COALESCE((SELECT group_concat(searchable_value, ' ') FROM product_attribute_values WHERE product_id = p.id), '')) FROM products p JOIN categories c ON c.id = p.category_id WHERE p.id = ?1",
        [product_id],
    )?;
    Ok(())
}

fn target_name(target: CatalogTarget) -> &'static str {
    match target {
        CatalogTarget::Category => "category",
        CatalogTarget::Product => "product",
    }
}

fn operation_name(activity: CatalogActivity) -> &'static str {
    match activity {
        CatalogActivity::Active => "reactivate",
        CatalogActivity::Archived => "archive",
    }
}

fn snapshot_json(snapshot: CatalogSnapshot) -> String {
    format!(
        "{{\"activity\":\"{}\",\"revision\":{}}}",
        activity_name(snapshot.activity),
        snapshot.revision
    )
}

fn activity_name(activity: CatalogActivity) -> &'static str {
    match activity {
        CatalogActivity::Active => "active",
        CatalogActivity::Archived => "archived",
    }
}
