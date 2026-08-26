use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};

use crate::domain::catalog::{
    validate_category, validate_product, AttributeValueDraft, CatalogValidationError,
    CategoryFieldDraft, FieldType,
};
use crate::infrastructure::sqlite::catalog_repository::SqliteCatalogRepository;

pub mod repository;

use repository::CreateProductRepository;

#[derive(Debug, PartialEq, Serialize)]
pub struct ProductSearchResult {
    pub product_id: i64,
    pub sku: String,
    pub name: String,
    pub category_name: String,
    pub available_quantity: i64,
    pub catalog_unit_price_centavos: i64,
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

#[derive(Clone, Debug, Deserialize)]
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
    pub catalog_unit_price_centavos: i64,
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
    pub catalog_unit_price_centavos: i64,
    pub available_quantity: i64,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CreateProductError {
    InvalidProduct,
    MissingCategory,
    DuplicateSku,
    InvalidCatalogPrice,
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
            input.catalog_unit_price_centavos,
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
            catalog_unit_price_centavos: input.catalog_unit_price_centavos,
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
        transaction
            .execute(
                "INSERT INTO attribute_definitions (category_id, label, field_type, required) VALUES (?1, ?2, ?3, ?4)",
                params![category_id, field.label.trim(), field.field_type.as_str(), field.required],
            )
            .map_err(|_| CreateCategoryError::Persistence)?;
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

fn load_category_fields(connection: &Connection, category_id: i64) -> Result<Vec<CategoryField>> {
    let mut statement = connection.prepare(
        "SELECT id, label, field_type, required FROM attribute_definitions WHERE category_id = ?1 ORDER BY id",
    )?;
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
        CatalogValidationError::InvalidCatalogPrice => CreateProductError::InvalidCatalogPrice,
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
        "SELECT p.id, p.sku, p.name, c.name, s.quantity, p.minimum_unit_price_centavos
         FROM catalog_product_search search
         JOIN products p ON p.id = search.product_id
         JOIN categories c ON c.id = p.category_id
         JOIN stock_balances s ON s.product_id = p.id
         WHERE search.content MATCH ?1 AND p.active = 1
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
