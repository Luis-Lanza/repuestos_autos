use rusqlite::{params, OptionalExtension, Result, Transaction};

use crate::application::catalog::bootstrap_demo::{
    BootstrapDemoEligibility, BootstrapDemoPlan, BootstrapDemoRepository,
};
use crate::application::catalog::repository::{
    CatalogMaintenanceRepository, CatalogMetadataRepository, CreateProductRepository,
    ProductMetadata,
};
use crate::application::catalog::CreateProductInput;
use crate::domain::catalog::{
    AttributeDefinition, CatalogActivity, CatalogSnapshot, CatalogTarget, FieldType,
    TransitionPlan, ValidatedAttributeValue,
};

const BOOTSTRAP_DEMO_OCCURRED_AT: &str = "2025-01-01T00:00:00Z";

pub struct SqliteCatalogRepository;

impl BootstrapDemoRepository for SqliteCatalogRepository {
    fn inspect_bootstrap(
        &self,
        transaction: &Transaction<'_>,
        plan: &BootstrapDemoPlan,
    ) -> Result<BootstrapDemoEligibility> {
        if is_complete_demo_state(transaction, plan)? {
            return Ok(BootstrapDemoEligibility::Complete);
        }
        if is_pristine_bootstrap_state(transaction)? {
            return Ok(BootstrapDemoEligibility::Pristine);
        }
        Ok(BootstrapDemoEligibility::Refused)
    }

    fn persist_bootstrap(
        &self,
        transaction: &Transaction<'_>,
        plan: &BootstrapDemoPlan,
    ) -> Result<()> {
        let mut category_ids = Vec::with_capacity(plan.categories.len());
        for category in &plan.categories {
            transaction.execute(
                "INSERT INTO categories (name) VALUES (?1)",
                [&category.name],
            )?;
            category_ids.push(transaction.last_insert_rowid());
        }

        for product in &plan.products {
            let category_id = *category_ids
                .get(product.category_index)
                .ok_or(rusqlite::Error::InvalidQuery)?;
            let input = CreateProductInput {
                sku: product.sku.clone(),
                name: product.name.clone(),
                category_id,
                list_price_centavos: product.list_price_centavos,
                minimum_sale_price_centavos: product.minimum_sale_price_centavos,
                opening_quantity: product.opening_quantity,
                attribute_values: Vec::new(),
            };
            self.persist_product_with_opening_timestamp(
                transaction,
                &input,
                &[],
                &plan.categories[product.category_index].name,
                Some(BOOTSTRAP_DEMO_OCCURRED_AT),
            )?;
        }

        let previous_revision: i64 = transaction.query_row(
            "SELECT revision FROM products WHERE id = 2 AND sku = 'BUJ-001' AND active = 0",
            [],
            |row| row.get(0),
        )?;
        if transaction.execute(
            "UPDATE products SET active = 1, revision = revision + 1 WHERE id = 2 AND sku = 'BUJ-001' AND active = 0 AND revision = ?1",
            [previous_revision],
        )? != 1 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        refresh_product_search(transaction, 2)?;
        transaction.execute(
            "INSERT INTO catalog_audit (entity_type, entity_id, operation, before_json, after_json, revision, occurred_at) VALUES ('product', 2, 'reactivate', ?1, ?2, ?3, ?4)",
            params![
                format!("{{\"activity\":\"archived\",\"revision\":{previous_revision}}}"),
                format!("{{\"activity\":\"active\",\"revision\":{}}}", previous_revision + 1),
                previous_revision + 1,
                BOOTSTRAP_DEMO_OCCURRED_AT,
            ],
        )?;
        Ok(())
    }
}

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
        self.persist_product_with_opening_timestamp(transaction, input, values, category_name, None)
    }
}

impl SqliteCatalogRepository {
    fn persist_product_with_opening_timestamp(
        &self,
        transaction: &Transaction<'_>,
        input: &CreateProductInput,
        values: &[ValidatedAttributeValue],
        category_name: &str,
        occurred_at: Option<&str>,
    ) -> Result<i64> {
        transaction.execute("INSERT INTO products (category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (?1, ?2, ?3, 1, ?4, ?5)", params![input.category_id, input.sku.trim(), input.name.trim(), input.list_price_centavos, input.minimum_sale_price_centavos])?;
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
        match occurred_at {
            Some(occurred_at) => transaction.execute("INSERT INTO inventory_movements (product_id, movement_type, quantity_delta, occurred_at) VALUES (?1, 'opening_stock', ?2, ?3)", params![product_id, input.opening_quantity, occurred_at])?,
            None => transaction.execute("INSERT INTO inventory_movements (product_id, movement_type, quantity_delta) VALUES (?1, 'opening_stock', ?2)", params![product_id, input.opening_quantity])?,
        };
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
        let before = CatalogMaintenanceRepository::load(self, transaction, target, entity_id)?
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
        let after = CatalogMaintenanceRepository::load(self, transaction, target, entity_id)?
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

impl CatalogMetadataRepository for SqliteCatalogRepository {
    fn load(
        &self,
        transaction: &Transaction<'_>,
        target: CatalogTarget,
        entity_id: i64,
    ) -> Result<Option<CatalogSnapshot>> {
        CatalogMaintenanceRepository::load(self, transaction, target, entity_id)
    }

    fn category_name_exists(
        &self,
        transaction: &Transaction<'_>,
        id: i64,
        name: &str,
    ) -> Result<bool> {
        transaction.query_row("SELECT EXISTS(SELECT 1 FROM categories WHERE id <> ?1 AND lower(trim(name)) = lower(trim(?2)))", params![id, name], |row| row.get(0))
    }

    fn product_metadata_for_normalized_patch(
        &self,
        transaction: &Transaction<'_>,
        id: i64,
        sku: &str,
        name: &str,
    ) -> Result<Option<ProductMetadata>> {
        let Some(category_id) = transaction
            .query_row(
                "SELECT category_id FROM products WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .optional()?
        else {
            return Ok(None);
        };
        let definitions =
            CreateProductRepository::attribute_definitions(self, transaction, category_id)?;
        let duplicate_normalized_identity = transaction.query_row("SELECT EXISTS(SELECT 1 FROM products WHERE id <> ?1 AND (lower(trim(sku)) = lower(trim(?2)) OR lower(trim(name)) = lower(trim(?3))))", params![id, sku, name], |row| row.get(0))?;
        Ok(Some(ProductMetadata {
            definitions,
            duplicate_normalized_identity,
        }))
    }

    fn edit_category(
        &self,
        transaction: &Transaction<'_>,
        id: i64,
        revision: i64,
        name: &str,
    ) -> Result<CatalogSnapshot> {
        let target = CatalogTarget::Category;
        let before = category_metadata_json(transaction, id)?;
        if transaction.execute("UPDATE OR IGNORE categories SET name = ?1, revision = revision + 1 WHERE id = ?2 AND revision = ?3", params![name, id, revision])? != 1 { return Err(rusqlite::Error::QueryReturnedNoRows) }
        refresh_category_search(transaction, id)?;
        let after = category_metadata_json(transaction, id)?;
        let snapshot = CatalogMaintenanceRepository::load(self, transaction, target, id)?
            .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        audit_metadata(transaction, target, id, before, after, snapshot.revision)?;
        Ok(snapshot)
    }

    fn edit_product(
        &self,
        transaction: &Transaction<'_>,
        id: i64,
        revision: i64,
        sku: &str,
        name: &str,
        list_price_centavos: i64,
        minimum_sale_price_centavos: i64,
        values: &[ValidatedAttributeValue],
    ) -> Result<CatalogSnapshot> {
        let target = CatalogTarget::Product;
        let before = product_metadata_json(transaction, id)?;
        if transaction.execute("UPDATE OR IGNORE products SET sku = ?1, name = ?2, list_price_centavos = ?3, minimum_unit_price_centavos = ?4, revision = revision + 1 WHERE id = ?5 AND revision = ?6", params![sku, name, list_price_centavos, minimum_sale_price_centavos, id, revision])? != 1 { return Err(rusqlite::Error::QueryReturnedNoRows) }
        transaction.execute(
            "DELETE FROM product_attribute_values WHERE product_id = ?1",
            [id],
        )?;
        for value in values {
            insert_attribute_value(transaction, id, value)?;
        }
        refresh_product_search(transaction, id)?;
        let after = product_metadata_json(transaction, id)?;
        let snapshot = CatalogMaintenanceRepository::load(self, transaction, target, id)?
            .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        audit_metadata(transaction, target, id, before, after, snapshot.revision)?;
        Ok(snapshot)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct MovementFingerprint {
    id: i64,
    product_id: i64,
    sale_id: Option<i64>,
    sale_line_id: Option<i64>,
    movement_type: String,
    quantity_delta: i64,
    occurred_at: String,
    reason: Option<String>,
    operator_id: Option<String>,
    source_reference: Option<String>,
    request_id: Option<String>,
    counted_quantity: Option<i64>,
    resulting_quantity: Option<i64>,
    operation_kind: Option<String>,
    payload_version: Option<i64>,
    canonical_payload: Option<Vec<u8>>,
    payload_sha256: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct AuditFingerprint {
    id: i64,
    entity_type: String,
    entity_id: i64,
    operation: String,
    before_json: String,
    after_json: String,
    revision: i64,
    occurred_at: String,
}

fn is_pristine_bootstrap_state(transaction: &Transaction<'_>) -> Result<bool> {
    Ok(load_categories(transaction)? == vec![
        (1, "Filtros".into(), 1, 0),
        (2, "Bujias".into(), 1, 0),
    ] && load_products(transaction)?
        == vec![
            (1, 1, "FLT-001".into(), "Filtro de aceite".into(), 1, 0, 2500, 2500),
            (2, 2, "BUJ-001".into(), "Bujia archivada".into(), 0, 0, 1800, 1800),
        ] && load_stock(transaction)? == vec![(1, 8), (2, 4)]
        && load_searchable_values(transaction)? == vec![(1, "vehicle".into(), "Toyota".into())]
        && load_fts(transaction)?
            == vec![
                (1, 1, fts_content("FLT-001", "Filtro de aceite", "Filtros", Some("Toyota"), true)),
                (2, 2, fts_content("BUJ-001", "Bujia archivada", "Bujias", None, true)),
            ]
        && load_movements(transaction)?.is_empty()
        && load_audit(transaction)?.is_empty()
        && count(transaction, "SELECT COUNT(*) FROM attribute_definitions")? == 0
        && count(transaction, "SELECT COUNT(*) FROM attribute_options")? == 0
        && count(transaction, "SELECT COUNT(*) FROM product_attribute_values")? == 0
        && empty_business_activity(transaction)?)
}

fn is_complete_demo_state(
    transaction: &Transaction<'_>,
    plan: &BootstrapDemoPlan,
) -> Result<bool> {
    let mut expected_categories = vec![
        (1, "Filtros".into(), 1, 0),
        (2, "Bujias".into(), 1, 0),
    ];
    expected_categories.extend(
        plan.categories
            .iter()
            .enumerate()
            .map(|(index, category)| (index as i64 + 3, category.name.clone(), 1, 0)),
    );

    let mut expected_products = vec![
        (1, 1, "FLT-001".into(), "Filtro de aceite".into(), 1, 0, 2500, 2500),
        (2, 2, "BUJ-001".into(), "Bujia archivada".into(), 1, 1, 1800, 1800),
    ];
    expected_products.extend(plan.products.iter().enumerate().map(|(index, product)| {
        (
            index as i64 + 3,
            product.category_index as i64 + 3,
            product.sku.clone(),
            product.name.clone(),
            1,
            0,
            product.list_price_centavos,
            product.minimum_sale_price_centavos,
        )
    }));

    let mut expected_stock = vec![(1, 8), (2, 4)];
    expected_stock.extend(
        plan.products
            .iter()
            .enumerate()
            .map(|(index, product)| (index as i64 + 3, product.opening_quantity)),
    );

    let mut expected_fts = vec![
        (1, 1, fts_content("FLT-001", "Filtro de aceite", "Filtros", Some("Toyota"), true)),
        (2, 2, fts_content("BUJ-001", "Bujia archivada", "Bujias", None, false)),
    ];
    expected_fts.extend(plan.products.iter().enumerate().map(|(index, product)| {
        (
            index as i64 + 3,
            index as i64 + 3,
            fts_content(
                &product.sku,
                &product.name,
                &plan.categories[product.category_index].name,
                None,
                false,
            ),
        )
    }));

    let expected_movements = plan
        .products
        .iter()
        .enumerate()
        .map(|(index, product)| MovementFingerprint {
            id: index as i64 + 1,
            product_id: index as i64 + 3,
            sale_id: None,
            sale_line_id: None,
            movement_type: "opening_stock".into(),
            quantity_delta: product.opening_quantity,
            occurred_at: BOOTSTRAP_DEMO_OCCURRED_AT.into(),
            reason: None,
            operator_id: None,
            source_reference: None,
            request_id: None,
            counted_quantity: None,
            resulting_quantity: None,
            operation_kind: None,
            payload_version: None,
            canonical_payload: None,
            payload_sha256: None,
        })
        .collect::<Vec<_>>();
    let expected_audit = vec![AuditFingerprint {
        id: 1,
        entity_type: "product".into(),
        entity_id: 2,
        operation: "reactivate".into(),
        before_json: "{\"activity\":\"archived\",\"revision\":0}".into(),
        after_json: "{\"activity\":\"active\",\"revision\":1}".into(),
        revision: 1,
        occurred_at: BOOTSTRAP_DEMO_OCCURRED_AT.into(),
    }];

    Ok(load_categories(transaction)? == expected_categories
        && load_products(transaction)? == expected_products
        && load_stock(transaction)? == expected_stock
        && load_searchable_values(transaction)? == vec![(1, "vehicle".into(), "Toyota".into())]
        && load_fts(transaction)? == expected_fts
        && load_movements(transaction)? == expected_movements
        && load_audit(transaction)? == expected_audit
        && count(transaction, "SELECT COUNT(*) FROM attribute_definitions")? == 0
        && count(transaction, "SELECT COUNT(*) FROM attribute_options")? == 0
        && count(transaction, "SELECT COUNT(*) FROM product_attribute_values")? == 0
        && empty_business_activity(transaction)?)
}

fn load_categories(transaction: &Transaction<'_>) -> Result<Vec<(i64, String, i64, i64)>> {
    transaction
        .prepare("SELECT id, name, active, revision FROM categories ORDER BY id")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?
        .collect()
}

fn load_products(
    transaction: &Transaction<'_>,
) -> Result<Vec<(i64, i64, String, String, i64, i64, i64, i64)>> {
    transaction
        .prepare("SELECT id, category_id, sku, name, active, revision, list_price_centavos, minimum_unit_price_centavos FROM products ORDER BY id")?
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            ))
        })?
        .collect()
}

fn load_stock(transaction: &Transaction<'_>) -> Result<Vec<(i64, i64)>> {
    transaction
        .prepare("SELECT product_id, quantity FROM stock_balances ORDER BY product_id")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect()
}

fn load_searchable_values(transaction: &Transaction<'_>) -> Result<Vec<(i64, String, String)>> {
    transaction
        .prepare("SELECT product_id, field_name, value FROM product_searchable_values ORDER BY product_id, field_name")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect()
}

fn load_fts(transaction: &Transaction<'_>) -> Result<Vec<(i64, i64, String)>> {
    transaction
        .prepare("SELECT rowid, product_id, content FROM catalog_product_search ORDER BY rowid")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect()
}

fn load_movements(transaction: &Transaction<'_>) -> Result<Vec<MovementFingerprint>> {
    transaction
        .prepare("SELECT id, product_id, sale_id, sale_line_id, movement_type, quantity_delta, occurred_at, reason, operator_id, source_reference, request_id, counted_quantity, resulting_quantity, operation_kind, payload_version, canonical_payload, payload_sha256 FROM inventory_movements ORDER BY id")?
        .query_map([], |row| {
            Ok(MovementFingerprint {
                id: row.get(0)?,
                product_id: row.get(1)?,
                sale_id: row.get(2)?,
                sale_line_id: row.get(3)?,
                movement_type: row.get(4)?,
                quantity_delta: row.get(5)?,
                occurred_at: row.get(6)?,
                reason: row.get(7)?,
                operator_id: row.get(8)?,
                source_reference: row.get(9)?,
                request_id: row.get(10)?,
                counted_quantity: row.get(11)?,
                resulting_quantity: row.get(12)?,
                operation_kind: row.get(13)?,
                payload_version: row.get(14)?,
                canonical_payload: row.get(15)?,
                payload_sha256: row.get(16)?,
            })
        })?
        .collect()
}

fn load_audit(transaction: &Transaction<'_>) -> Result<Vec<AuditFingerprint>> {
    transaction
        .prepare("SELECT id, entity_type, entity_id, operation, before_json, after_json, revision, occurred_at FROM catalog_audit ORDER BY id")?
        .query_map([], |row| {
            Ok(AuditFingerprint {
                id: row.get(0)?,
                entity_type: row.get(1)?,
                entity_id: row.get(2)?,
                operation: row.get(3)?,
                before_json: row.get(4)?,
                after_json: row.get(5)?,
                revision: row.get(6)?,
                occurred_at: row.get(7)?,
            })
        })?
        .collect()
}

fn fts_content(
    sku: &str,
    name: &str,
    category: &str,
    searchable_value: Option<&str>,
    includes_searchable_segment: bool,
) -> String {
    let content = if includes_searchable_segment {
        format!(
            "{sku} {name} {category} {} {}",
            searchable_value.unwrap_or_default(),
            ""
        )
    } else {
        format!("{sku} {name} {category} {}", searchable_value.unwrap_or_default())
    };
    content.to_lowercase()
}

fn empty_business_activity(transaction: &Transaction<'_>) -> Result<bool> {
    for sql in [
        "SELECT COUNT(*) FROM sales",
        "SELECT COUNT(*) FROM sale_lines",
        "SELECT COUNT(*) FROM sale_payments",
        "SELECT COUNT(*) FROM post_sale_requests",
        "SELECT COUNT(*) FROM sale_returns",
        "SELECT COUNT(*) FROM sale_return_lines",
        "SELECT COUNT(*) FROM sale_cancellations",
        "SELECT COUNT(*) FROM sale_cancellation_lines",
    ] {
        if count(transaction, sql)? != 0 {
            return Ok(false);
        }
    }
    Ok(true)
}

fn count(transaction: &Transaction<'_>, sql: &str) -> Result<i64> {
    transaction.query_row(sql, [], |row| row.get(0))
}

fn insert_attribute_value(
    transaction: &Transaction<'_>,
    product_id: i64,
    value: &ValidatedAttributeValue,
) -> Result<()> {
    let (definition_id, text, number, option, searchable) = match value {
        ValidatedAttributeValue::Text {
            definition_id,
            value,
        } => (*definition_id, Some(value), None, None, value),
        ValidatedAttributeValue::Number {
            definition_id,
            value,
            searchable,
        } => (*definition_id, None, Some(value), None, searchable),
        ValidatedAttributeValue::Option {
            definition_id,
            value,
        } => (*definition_id, None, None, Some(value), value),
    };
    transaction.execute("INSERT INTO product_attribute_values (product_id, definition_id, text_value, number_value, option_value, searchable_value) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", params![product_id, definition_id, text, number, option, searchable])?;
    Ok(())
}

fn category_metadata_json(transaction: &Transaction<'_>, id: i64) -> Result<String> {
    transaction.query_row(
        "SELECT json_object('name', name, 'revision', revision) FROM categories WHERE id = ?1",
        [id],
        |row| row.get(0),
    )
}

fn product_metadata_json(transaction: &Transaction<'_>, id: i64) -> Result<String> {
    transaction.query_row("SELECT json_object('sku', p.sku, 'name', p.name, 'list_price_centavos', p.list_price_centavos, 'minimum_sale_price_centavos', p.minimum_sale_price_centavos, 'revision', p.revision, 'attribute_values', json(COALESCE((SELECT json_group_array(json_object('definition_id', definition_id, 'text_value', text_value, 'number_value', number_value, 'option_value', option_value)) FROM product_attribute_values WHERE product_id = p.id), '[]'))) FROM products p WHERE p.id = ?1", [id], |row| row.get(0))
}

fn audit_metadata(
    transaction: &Transaction<'_>,
    target: CatalogTarget,
    id: i64,
    before: String,
    after: String,
    revision: i64,
) -> Result<()> {
    transaction.execute("INSERT INTO catalog_audit (entity_type, entity_id, operation, before_json, after_json, revision) VALUES (?1, ?2, 'edit_metadata', ?3, ?4, ?5)", params![target_name(target), id, before, after, revision])?;
    Ok(())
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

fn refresh_category_search(transaction: &Transaction<'_>, category_id: i64) -> Result<()> {
    let products = transaction
        .prepare("SELECT id FROM products WHERE category_id = ?1")?
        .query_map([category_id], |row| row.get(0))?
        .collect::<Result<Vec<_>>>()?;
    products
        .into_iter()
        .try_for_each(|id| refresh_product_search(transaction, id))
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
