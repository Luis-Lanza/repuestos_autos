use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};

use crate::domain::catalog::{
    validate_category, CatalogValidationError, CategoryFieldDraft, FieldType,
};

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

pub fn search_active_products(
    connection: &Connection,
    query: &str,
) -> Result<Vec<ProductSearchResult>> {
    let pattern = format!("%{}%", query.trim().to_lowercase());
    let mut statement = connection.prepare(
        "SELECT DISTINCT p.id, p.sku, p.name, c.name, s.quantity, p.minimum_unit_price_centavos
         FROM products p JOIN categories c ON c.id = p.category_id
         JOIN stock_balances s ON s.product_id = p.id
         LEFT JOIN product_searchable_values v ON v.product_id = p.id
         WHERE p.active = 1 AND (lower(p.sku) LIKE ?1 OR lower(p.name) LIKE ?1
           OR lower(c.name) LIKE ?1 OR lower(v.value) LIKE ?1) ORDER BY p.name",
    )?;
    let results = statement
        .query_map([pattern], |row| {
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
