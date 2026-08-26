use serde::Serialize;

use crate::application::catalog::{
    self, Category, CreateCategoryError, CreateCategoryInput, CreateProductError,
    CreateProductInput, CreatedProduct,
};

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CreateCategoryResponse {
    Success(Category),
    Error(OnboardingError),
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CreateProductResponse {
    Success(CreatedProduct),
    Error(OnboardingError),
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ListCategoriesResponse {
    Success { categories: Vec<Category> },
    Error(OnboardingError),
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct OnboardingError {
    pub code: &'static str,
    pub message: &'static str,
}

pub fn list_categories(
    connection: &rusqlite::Connection,
) -> Result<ListCategoriesResponse, String> {
    Ok(match catalog::list_categories(connection) {
        Ok(categories) => ListCategoriesResponse::Success { categories },
        Err(_) => ListCategoriesResponse::Error(OnboardingError {
            code: "persistence_failure",
            message: "Categories could not be loaded.",
        }),
    })
}

pub fn create_category(
    connection: &mut rusqlite::Connection,
    request: CreateCategoryInput,
) -> Result<CreateCategoryResponse, String> {
    Ok(match catalog::create_category(connection, request) {
        Ok(category) => CreateCategoryResponse::Success(category),
        Err(error) => CreateCategoryResponse::Error(map_category_error(error)),
    })
}

pub fn create_product(
    connection: &mut rusqlite::Connection,
    request: CreateProductInput,
) -> Result<CreateProductResponse, String> {
    Ok(match catalog::create_product(connection, request) {
        Ok(product) => CreateProductResponse::Success(product),
        Err(error) => CreateProductResponse::Error(map_product_error(error)),
    })
}

fn map_category_error(error: CreateCategoryError) -> OnboardingError {
    let (code, message) = match error {
        CreateCategoryError::InvalidCategory => ("invalid_category", "Category name is required."),
        CreateCategoryError::InvalidFieldDefinition => (
            "invalid_field_definition",
            "Category field definitions are invalid.",
        ),
        CreateCategoryError::DuplicateCategory => {
            ("duplicate_category", "Category name already exists.")
        }
        CreateCategoryError::Persistence => (
            "persistence_failure",
            "The category could not be persisted.",
        ),
    };
    OnboardingError { code, message }
}

fn map_product_error(error: CreateProductError) -> OnboardingError {
    let (code, message) = match error {
        CreateProductError::InvalidProduct => {
            ("invalid_product", "SKU and product name are required.")
        }
        CreateProductError::MissingCategory => {
            ("missing_category", "The selected category was not found.")
        }
        CreateProductError::DuplicateSku => ("duplicate_sku", "SKU already exists."),
        CreateProductError::InvalidCatalogPrice => (
            "invalid_catalog_price",
            "Catalog price must be a positive whole number of centavos.",
        ),
        CreateProductError::InvalidOpeningQuantity => (
            "invalid_opening_quantity",
            "Opening stock must be a positive whole number.",
        ),
        CreateProductError::MissingRequiredField => (
            "missing_required_field",
            "A required category field is missing.",
        ),
        CreateProductError::InvalidAttributeValue => (
            "invalid_attribute_value",
            "A category field value is invalid.",
        ),
        CreateProductError::Persistence => {
            ("persistence_failure", "The product could not be persisted.")
        }
    };
    OnboardingError { code, message }
}
