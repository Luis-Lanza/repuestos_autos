use rusqlite::{params, Connection, OptionalExtension, Result, TransactionBehavior};
use serde::{Deserialize, Serialize};

use crate::domain::catalog::{
    plan_transition, validate_category, validate_product, AttributeValueDraft, CatalogIntent,
    CatalogSnapshot, CatalogTarget, CatalogValidationError, CategoryFieldDraft, FieldType,
    MaintenanceError,
};
use crate::infrastructure::sqlite::catalog_repository::SqliteCatalogRepository;

pub mod repository;
pub(crate) mod bootstrap_demo;

pub use bootstrap_demo::{
    bootstrap_demo_catalog, BootstrapDemoError, BootstrapDemoOutcome, BootstrapDemoSummary,
};

use repository::{
    CatalogMaintenanceRepository, CatalogMetadataRepository, CreateProductRepository,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaintainCatalogInput {
    pub target: CatalogTarget,
    pub entity_id: i64,
    pub intent: CatalogIntent,
    pub expected_revision: i64,
}

impl MaintainCatalogInput {
    pub fn new(
        target: CatalogTarget,
        entity_id: i64,
        intent: CatalogIntent,
        expected_revision: i64,
    ) -> Self {
        Self {
            target,
            entity_id,
            intent,
            expected_revision,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaintainCatalogError {
    MissingCatalogRecord,
    LifecycleBlocked,
    StaleCatalogRecord,
    PersistenceFailure,
}

#[derive(Clone, Debug)]
pub enum EditCatalogInput {
    Category {
        entity_id: i64,
        expected_revision: i64,
        name: String,
    },
    Product {
        entity_id: i64,
        expected_revision: i64,
        sku: String,
        name: String,
        list_price_centavos: i64,
        minimum_sale_price_centavos: i64,
        attribute_values: Vec<AttributeValueInput>,
    },
}
impl EditCatalogInput {
    pub fn category(entity_id: i64, expected_revision: i64, name: impl Into<String>) -> Self {
        Self::Category {
            entity_id,
            expected_revision,
            name: name.into(),
        }
    }
    pub fn product(
        entity_id: i64,
        expected_revision: i64,
        sku: impl Into<String>,
        name: impl Into<String>,
        list_price_centavos: i64,
        minimum_sale_price_centavos: i64,
        attribute_values: Vec<AttributeValueInput>,
    ) -> Self {
        Self::Product {
            entity_id,
            expected_revision,
            sku: sku.into(),
            name: name.into(),
            list_price_centavos,
            minimum_sale_price_centavos,
            attribute_values,
        }
    }
}

pub struct EditCatalogUseCase<'connection, Repository> {
    connection: &'connection mut Connection,
    repository: Repository,
}
impl<'connection, Repository> EditCatalogUseCase<'connection, Repository>
where
    Repository: CatalogMetadataRepository,
{
    pub fn new(connection: &'connection mut Connection, repository: Repository) -> Self {
        Self {
            connection,
            repository,
        }
    }
    pub fn execute(
        self,
        input: EditCatalogInput,
    ) -> std::result::Result<CatalogSnapshot, MaintainCatalogError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| MaintainCatalogError::PersistenceFailure)?;
        let (target, id, revision) = match &input {
            EditCatalogInput::Category {
                entity_id,
                expected_revision,
                ..
            } => (CatalogTarget::Category, *entity_id, *expected_revision),
            EditCatalogInput::Product {
                entity_id,
                expected_revision,
                ..
            } => (CatalogTarget::Product, *entity_id, *expected_revision),
        };
        let snapshot = self
            .repository
            .load(&transaction, target, id)
            .map_err(|_| MaintainCatalogError::PersistenceFailure)?
            .ok_or(MaintainCatalogError::MissingCatalogRecord)?;
        if snapshot.revision != revision {
            return Err(MaintainCatalogError::StaleCatalogRecord);
        }
        let persisted = match input {
            EditCatalogInput::Category { name, .. } => {
                crate::domain::catalog::validate_maintenance_category(&name)
                    .map_err(|_| MaintainCatalogError::MissingCatalogRecord)?;
                if self
                    .repository
                    .category_name_exists(&transaction, id, name.trim())
                    .map_err(|_| MaintainCatalogError::PersistenceFailure)?
                {
                    return Err(MaintainCatalogError::MissingCatalogRecord);
                }
                self.repository
                    .edit_category(&transaction, id, revision, name.trim())
            }
            EditCatalogInput::Product {
                sku,
                name,
                list_price_centavos,
                minimum_sale_price_centavos,
                attribute_values,
                ..
            } => {
                let metadata = self
                    .repository
                    .product_metadata_for_normalized_patch(
                        &transaction,
                        id,
                        sku.trim(),
                        name.trim(),
                    )
                    .map_err(|_| MaintainCatalogError::PersistenceFailure)?
                    .ok_or(MaintainCatalogError::MissingCatalogRecord)?;
                let values = attribute_values
                    .iter()
                    .map(|value| AttributeValueDraft {
                        definition_id: value.definition_id,
                        value: value.value.clone(),
                    })
                    .collect::<Vec<_>>();
                let validated = crate::domain::catalog::validate_maintenance_product(
                    &sku,
                    &name,
                    list_price_centavos,
                    minimum_sale_price_centavos,
                    &metadata.definitions,
                    &values,
                )
                .map_err(|_| MaintainCatalogError::MissingCatalogRecord)?;
                if metadata.duplicate_normalized_identity {
                    return Err(MaintainCatalogError::MissingCatalogRecord);
                }
                self.repository.edit_product(
                    &transaction,
                    id,
                    revision,
                    sku.trim(),
                    name.trim(),
                    list_price_centavos,
                    minimum_sale_price_centavos,
                    &validated,
                )
            }
        }
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => MaintainCatalogError::StaleCatalogRecord,
            _ => MaintainCatalogError::PersistenceFailure,
        })?;
        transaction
            .commit()
            .map_err(|_| MaintainCatalogError::PersistenceFailure)?;
        Ok(persisted)
    }
}

pub struct MaintainCatalogUseCase<'connection, Repository> {
    connection: &'connection mut Connection,
    repository: Repository,
}

impl<'connection, Repository> MaintainCatalogUseCase<'connection, Repository>
where
    Repository: CatalogMaintenanceRepository,
{
    pub fn new(connection: &'connection mut Connection, repository: Repository) -> Self {
        Self {
            connection,
            repository,
        }
    }

    pub fn execute(
        self,
        input: MaintainCatalogInput,
    ) -> std::result::Result<CatalogSnapshot, MaintainCatalogError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| MaintainCatalogError::PersistenceFailure)?;
        let snapshot = self
            .repository
            .load(&transaction, input.target, input.entity_id)
            .map_err(|_| MaintainCatalogError::PersistenceFailure)?
            .ok_or(MaintainCatalogError::MissingCatalogRecord)?;
        if snapshot.revision != input.expected_revision {
            return Err(MaintainCatalogError::StaleCatalogRecord);
        }
        let plan = plan_transition(&snapshot, input.intent).map_err(map_maintenance_error)?;
        let persisted = self
            .repository
            .apply(&transaction, input.target, input.entity_id, plan)
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => MaintainCatalogError::StaleCatalogRecord,
                _ => MaintainCatalogError::PersistenceFailure,
            })?;
        transaction
            .commit()
            .map_err(|_| MaintainCatalogError::PersistenceFailure)?;
        Ok(persisted)
    }
}

fn map_maintenance_error(error: MaintenanceError) -> MaintainCatalogError {
    match error {
        MaintenanceError::LifecycleBlocked => MaintainCatalogError::LifecycleBlocked,
        _ => MaintainCatalogError::PersistenceFailure,
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct ProductSearchResult {
    pub product_id: i64,
    pub sku: String,
    pub name: String,
    pub category_name: String,
    pub available_quantity: i64,
    pub catalog_unit_price_centavos: i64,
    pub list_price_centavos: i64,
    pub minimum_sale_price_centavos: i64,
    pub revision: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductStockFilter {
    All,
    LowStock,
    OutOfStock,
    Available,
    Alerts,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductActivityFilter {
    Active,
    Archived,
    All,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrowseProductsInput {
    pub query: Option<String>,
    pub category_id: Option<i64>,
    pub stock_filter: ProductStockFilter,
    pub activity_filter: ProductActivityFilter,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct ProductBrowseResult {
    pub product_id: i64,
    pub category_id: i64,
    pub sku: String,
    pub name: String,
    pub category_name: String,
    pub available_quantity: i64,
    pub catalog_unit_price_centavos: i64,
    pub list_price_centavos: i64,
    pub minimum_sale_price_centavos: i64,
    pub revision: i64,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct ProductBrowseCategory {
    pub category_id: i64,
    pub name: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct ProductBrowsePage {
    pub products: Vec<ProductBrowseResult>,
    pub categories: Vec<ProductBrowseCategory>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
    pub total_pages: i64,
}

pub fn browse_active_products(
    connection: &Connection,
    input: &BrowseProductsInput,
) -> Result<ProductBrowsePage> {
    const MAX_PAGE_SIZE: i64 = 50;
    if input.page < 1 || input.page_size < 1 || input.page_size > MAX_PAGE_SIZE {
        return Err(rusqlite::Error::InvalidQuery);
    }
    let query = input.query.as_deref().and_then(normalized_search_query);
    let stock_clause = match input.stock_filter {
        ProductStockFilter::All => "1 = 1",
        ProductStockFilter::LowStock => "s.quantity BETWEEN 1 AND 1",
        ProductStockFilter::OutOfStock => "s.quantity = 0",
        ProductStockFilter::Available => "s.quantity > 1",
        ProductStockFilter::Alerts => "s.quantity <= 1",
    };
    let activity_clause = match input.activity_filter {
        ProductActivityFilter::Active => "p.active = 1 AND c.active = 1",
        ProductActivityFilter::Archived => "p.active = 0 AND c.active = 1",
        ProductActivityFilter::All => "c.active = 1",
    };
    let (search_clause, category_clause) = if query.is_some() {
        ("search.content MATCH ?1", if input.category_id.is_some() { "AND p.category_id = ?2" } else { "" })
    } else {
        ("1 = 1", if input.category_id.is_some() { "AND p.category_id = ?1" } else { "" })
    };
    let count_sql = format!(
        "SELECT COUNT(*) FROM catalog_product_search search
         JOIN products p ON p.id = search.product_id
         JOIN categories c ON c.id = p.category_id
         JOIN stock_balances s ON s.product_id = p.id
         WHERE {search_clause} {category_clause}
           AND {activity_clause} AND {stock_clause}"
    );
    let mut count_args: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some(ref query) = query { count_args.push(query); }
    if let Some(ref category_id) = input.category_id { count_args.push(category_id); }
    let total = connection.query_row(
        &count_sql,
        rusqlite::params_from_iter(count_args),
        |row| row.get::<_, i64>(0),
    )?;
    let total_pages = (total + input.page_size - 1) / input.page_size;
    let offset = (input.page - 1)
        .checked_mul(input.page_size)
        .ok_or(rusqlite::Error::InvalidQuery)?;
    let limit_index = 1 + query.is_some() as usize + input.category_id.is_some() as usize;
    let offset_index = limit_index + 1;
    let products_sql = format!(
        "SELECT p.id, p.category_id, p.sku, p.name, c.name, s.quantity,
                p.list_price_centavos, p.minimum_unit_price_centavos, p.revision
         FROM catalog_product_search search
         JOIN products p ON p.id = search.product_id
         JOIN categories c ON c.id = p.category_id
         JOIN stock_balances s ON s.product_id = p.id
         WHERE {search_clause} {category_clause}
           AND {activity_clause} AND {stock_clause}
         ORDER BY lower(p.name), p.id
         LIMIT ?{limit_index} OFFSET ?{offset_index}"
    );
    let mut product_args: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some(ref query) = query { product_args.push(query); }
    if let Some(ref category_id) = input.category_id { product_args.push(category_id); }
    product_args.push(&input.page_size);
    product_args.push(&offset);
    let products = connection
        .prepare(&products_sql)?
        .query_map(
            rusqlite::params_from_iter(product_args),
            |row| {
                Ok(ProductBrowseResult {
                    product_id: row.get(0)?,
                    category_id: row.get(1)?,
                    sku: row.get(2)?,
                    name: row.get(3)?,
                    category_name: row.get(4)?,
                    available_quantity: row.get(5)?,
                    catalog_unit_price_centavos: row.get(6)?,
                    list_price_centavos: row.get(6)?,
                    minimum_sale_price_centavos: row.get(7)?,
                    revision: row.get(8)?,
                })
            },
        )?
        .collect::<Result<Vec<_>>>()?;
    let mut categories = connection.prepare(
        "SELECT id, name FROM categories WHERE active = 1 ORDER BY lower(name), id",
    )?;
    let categories = categories
        .query_map([], |row| {
            Ok(ProductBrowseCategory {
                category_id: row.get(0)?,
                name: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(ProductBrowsePage {
        products,
        categories,
        page: input.page,
        page_size: input.page_size,
        total,
        total_pages,
    })
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CategoryFieldInput {
    pub label: String,
    pub field_type: String,
    pub required: bool,
    pub options: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateCategoryInput {
    pub name: String,
    pub fields: Vec<CategoryFieldInput>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CategoryField {
    pub definition_id: i64,
    pub label: String,
    pub field_type: String,
    pub required: bool,
    pub options: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Category {
    pub category_id: i64,
    pub name: String,
    pub fields: Vec<CategoryField>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CreateCategoryError {
    InvalidCategory,
    InvalidFieldDefinition,
    DuplicateCategory,
    Persistence,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AttributeValueInput {
    pub definition_id: i64,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateProductInput {
    pub sku: String,
    pub name: String,
    pub category_id: i64,
    pub list_price_centavos: i64,
    pub minimum_sale_price_centavos: i64,
    pub opening_quantity: i64,
    pub attribute_values: Vec<AttributeValueInput>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CreatedProduct {
    pub product_id: i64,
    pub sku: String,
    pub name: String,
    pub category_id: i64,
    pub category_name: String,
    pub list_price_centavos: i64,
    pub minimum_sale_price_centavos: i64,
    pub available_quantity: i64,
    pub active: bool,
}

#[derive(Debug, Serialize)]
#[serde(tag = "target", rename_all = "snake_case")]
pub enum CatalogMetadataDetail {
    Category {
        entity_id: i64,
        name: String,
        activity: &'static str,
        revision: i64,
        attribute_definitions: Vec<CategoryField>,
    },
    Product {
        entity_id: i64,
        category_id: i64,
        sku: String,
        name: String,
        list_price_centavos: i64,
        minimum_sale_price_centavos: i64,
        activity: &'static str,
        revision: i64,
        attribute_definitions: Vec<CategoryField>,
        attribute_values: Vec<AttributeValueInput>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CreateProductError {
    InvalidProduct,
    MissingCategory,
    DuplicateSku,
    InvalidListPrice,
    InvalidMinimumSalePrice,
    MinimumSalePriceExceedsListPrice,
    InvalidOpeningQuantity,
    MissingRequiredField,
    InvalidAttributeValue,
    Persistence,
}

pub struct CreateProductUseCase<'connection, Repository> {
    connection: &'connection mut Connection,
    repository: Repository,
}

impl<'connection, Repository> CreateProductUseCase<'connection, Repository>
where
    Repository: CreateProductRepository,
{
    pub fn new(connection: &'connection mut Connection, repository: Repository) -> Self {
        Self {
            connection,
            repository,
        }
    }

    pub fn execute(
        self,
        input: CreateProductInput,
    ) -> std::result::Result<CreatedProduct, CreateProductError> {
        let transaction = self
            .connection
            .transaction()
            .map_err(|_| CreateProductError::Persistence)?;
        let category_name = self
            .repository
            .category_name(&transaction, input.category_id)
            .map_err(|_| CreateProductError::Persistence)?
            .ok_or(CreateProductError::MissingCategory)?;
        let definitions = self
            .repository
            .attribute_definitions(&transaction, input.category_id)
            .map_err(|_| CreateProductError::Persistence)?;
        let values = input
            .attribute_values
            .iter()
            .map(|value| AttributeValueDraft {
                definition_id: value.definition_id,
                value: value.value.clone(),
            })
            .collect::<Vec<_>>();
        let validated = validate_product(
            &input.sku,
            &input.name,
            input.list_price_centavos,
            input.minimum_sale_price_centavos,
            input.opening_quantity,
            &definitions,
            &values,
        )
        .map_err(map_product_validation)?;
        if self
            .repository
            .sku_exists(&transaction, input.sku.trim())
            .map_err(|_| CreateProductError::Persistence)?
        {
            return Err(CreateProductError::DuplicateSku);
        }
        let product_id = self
            .repository
            .persist_product(&transaction, &input, &validated, &category_name)
            .map_err(|_| CreateProductError::Persistence)?;
        transaction
            .commit()
            .map_err(|_| CreateProductError::Persistence)?;
        Ok(CreatedProduct {
            product_id,
            sku: input.sku.trim().into(),
            name: input.name.trim().into(),
            category_id: input.category_id,
            category_name,
            list_price_centavos: input.list_price_centavos,
            minimum_sale_price_centavos: input.minimum_sale_price_centavos,
            available_quantity: input.opening_quantity,
            active: true,
        })
    }
}

pub fn list_categories(connection: &Connection) -> Result<Vec<Category>> {
    let mut statement = connection.prepare("SELECT id, name FROM categories ORDER BY name")?;
    let categories = statement
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>>>()?;
    categories
        .into_iter()
        .map(|(category_id, name)| {
            Ok(Category {
                category_id,
                name,
                fields: load_category_fields(connection, category_id)?,
            })
        })
        .collect()
}

pub fn create_category(
    connection: &mut Connection,
    input: CreateCategoryInput,
) -> std::result::Result<Category, CreateCategoryError> {
    let fields = input
        .fields
        .iter()
        .map(|field| {
            Ok(CategoryFieldDraft {
                label: field.label.clone(),
                field_type: FieldType::parse(&field.field_type).map_err(map_category_validation)?,
                required: field.required,
                options: field.options.clone(),
            })
        })
        .collect::<std::result::Result<Vec<_>, CreateCategoryError>>()?;
    validate_category(&input.name, &fields).map_err(map_category_validation)?;
    let duplicate = connection
        .query_row(
            "SELECT 1 FROM categories WHERE lower(name) = lower(?1)",
            [input.name.trim()],
            |_| Ok(()),
        )
        .optional()
        .map_err(|_| CreateCategoryError::Persistence)?
        .is_some();
    if duplicate {
        return Err(CreateCategoryError::DuplicateCategory);
    }

    let transaction = connection
        .transaction()
        .map_err(|_| CreateCategoryError::Persistence)?;
    transaction
        .execute(
            "INSERT INTO categories (name) VALUES (?1)",
            [input.name.trim()],
        )
        .map_err(|_| CreateCategoryError::Persistence)?;
    let category_id = transaction.last_insert_rowid();
    for field in fields {
        transaction.execute("INSERT INTO attribute_definitions (category_id, label, field_type, required) VALUES (?1, ?2, ?3, ?4)", params![category_id, field.label.trim(), field.field_type.as_str(), field.required]).map_err(|_| CreateCategoryError::Persistence)?;
        let definition_id = transaction.last_insert_rowid();
        for option in field.options {
            transaction
                .execute(
                    "INSERT INTO attribute_options (definition_id, value) VALUES (?1, ?2)",
                    params![definition_id, option.trim()],
                )
                .map_err(|_| CreateCategoryError::Persistence)?;
        }
    }
    transaction
        .commit()
        .map_err(|_| CreateCategoryError::Persistence)?;
    list_categories(connection)
        .map_err(|_| CreateCategoryError::Persistence)?
        .into_iter()
        .find(|category| category.category_id == category_id)
        .ok_or(CreateCategoryError::Persistence)
}

pub fn create_product(
    connection: &mut Connection,
    input: CreateProductInput,
) -> std::result::Result<CreatedProduct, CreateProductError> {
    CreateProductUseCase::new(connection, SqliteCatalogRepository).execute(input)
}

pub fn read_catalog_metadata_detail(
    connection: &Connection,
    target: CatalogTarget,
    entity_id: i64,
) -> Result<Option<CatalogMetadataDetail>> {
    match target {
        CatalogTarget::Category => connection
            .query_row(
                "SELECT name, active, revision FROM categories WHERE id = ?1",
                [entity_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?
            .map(|(name, active, revision)| {
                Ok(CatalogMetadataDetail::Category {
                    entity_id,
                    name,
                    activity: if active { "active" } else { "archived" },
                    revision,
                    attribute_definitions: load_category_fields(connection, entity_id)?,
                })
            })
            .transpose(),
        CatalogTarget::Product => connection
            .query_row(
                "SELECT category_id, sku, name, list_price_centavos, minimum_unit_price_centavos, active, revision FROM products WHERE id = ?1",
                [entity_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
            )
            .optional()?
            .map(|(category_id, sku, name, list_price_centavos, minimum_sale_price_centavos, active, revision)| {
                let mut statement = connection.prepare("SELECT definition_id, searchable_value FROM product_attribute_values WHERE product_id = ?1 ORDER BY definition_id")?;
                let attribute_values = statement.query_map([entity_id], |row| Ok(AttributeValueInput { definition_id: row.get(0)?, value: row.get(1)? }))?.collect::<Result<Vec<_>>>()?;
                Ok(CatalogMetadataDetail::Product {
                    entity_id,
                    category_id,
                    sku,
                    name,
                    list_price_centavos,
                    minimum_sale_price_centavos,
                    activity: if active { "active" } else { "archived" },
                    revision,
                    attribute_definitions: load_category_fields(connection, category_id)?,
                    attribute_values,
                })
            })
            .transpose(),
    }
}

fn load_category_fields(connection: &Connection, category_id: i64) -> Result<Vec<CategoryField>> {
    let mut statement = connection.prepare("SELECT id, label, field_type, required FROM attribute_definitions WHERE category_id = ?1 ORDER BY id")?;
    let fields = statement
        .query_map([category_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, bool>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>>>()?;
    fields
        .into_iter()
        .map(|(definition_id, label, field_type, required)| {
            let mut options = connection.prepare(
                "SELECT value FROM attribute_options WHERE definition_id = ?1 ORDER BY rowid",
            )?;
            let option_values = options
                .query_map([definition_id], |row| row.get(0))?
                .collect::<Result<Vec<_>>>()?;
            Ok(CategoryField {
                definition_id,
                label,
                field_type,
                required,
                options: option_values,
            })
        })
        .collect()
}

fn map_category_validation(error: CatalogValidationError) -> CreateCategoryError {
    match error {
        CatalogValidationError::InvalidCategory => CreateCategoryError::InvalidCategory,
        _ => CreateCategoryError::InvalidFieldDefinition,
    }
}

fn map_product_validation(error: CatalogValidationError) -> CreateProductError {
    match error {
        CatalogValidationError::InvalidProduct => CreateProductError::InvalidProduct,
        CatalogValidationError::InvalidListPrice => CreateProductError::InvalidListPrice,
        CatalogValidationError::InvalidMinimumSalePrice => {
            CreateProductError::InvalidMinimumSalePrice
        }
        CatalogValidationError::MinimumSalePriceExceedsListPrice => {
            CreateProductError::MinimumSalePriceExceedsListPrice
        }
        CatalogValidationError::InvalidOpeningQuantity => {
            CreateProductError::InvalidOpeningQuantity
        }
        CatalogValidationError::MissingRequiredField => CreateProductError::MissingRequiredField,
        CatalogValidationError::InvalidAttributeValue => CreateProductError::InvalidAttributeValue,
        _ => CreateProductError::Persistence,
    }
}

pub fn search_active_products(
    connection: &Connection,
    query: &str,
) -> Result<Vec<ProductSearchResult>> {
    let Some(query) = normalized_search_query(query) else {
        return Ok(Vec::new());
    };
    let mut statement = connection.prepare(
        "SELECT p.id, p.sku, p.name, c.name, s.quantity,
         p.list_price_centavos,
         p.minimum_unit_price_centavos, p.revision
         FROM catalog_product_search search
         JOIN products p ON p.id = search.product_id
         JOIN categories c ON c.id = p.category_id
         JOIN stock_balances s ON s.product_id = p.id
          WHERE search.content MATCH ?1 AND p.active = 1 AND c.active = 1
         ORDER BY p.name
         LIMIT 20",
    )?;
    let results = statement
        .query_map([query], |row| {
            Ok(ProductSearchResult {
                product_id: row.get(0)?,
                sku: row.get(1)?,
                name: row.get(2)?,
                category_name: row.get(3)?,
                available_quantity: row.get(4)?,
                catalog_unit_price_centavos: row.get(5)?,
                list_price_centavos: row.get(5)?,
                minimum_sale_price_centavos: row.get(6)?,
                revision: row.get(7)?,
            })
        })?
        .collect();
    results
}

fn normalized_search_query(query: &str) -> Option<String> {
    let terms = query
        .split(|character: char| !character.is_alphanumeric())
        .filter(|term| !term.is_empty())
        .map(|term| format!("\"{}\"*", term.to_lowercase()))
        .collect::<Vec<_>>();

    (!terms.is_empty()).then(|| terms.join(" "))
}
