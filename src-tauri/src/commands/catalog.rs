use serde::{Deserialize, Serialize};

use crate::application::catalog;
use crate::domain::catalog::{CatalogActivity, CatalogIntent, CatalogTarget};
use crate::infrastructure::sqlite::SqliteCatalogRepository;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchProductsRequest {
    pub query: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaintainCatalogRequest {
    pub target: String,
    pub entity_id: i64,
    pub intent: String,
    pub expected_revision: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogMetadataDetailRequest {
    pub target: String,
    pub entity_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditAttributeValueRequest {
    pub definition_id: i64,
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "target", rename_all = "snake_case", deny_unknown_fields)]
pub enum EditCatalogRequest {
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
        catalog_unit_price_centavos: i64,
        attribute_values: Vec<EditAttributeValueRequest>,
    },
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CatalogMaintenanceResponse {
    Success(CatalogMaintenanceRecord),
    Error(CatalogMaintenanceError),
}
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CatalogMaintenanceListResponse {
    Success {
        records: Vec<CatalogMaintenanceRecord>,
    },
    Error(CatalogMaintenanceError),
}
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CatalogMetadataDetailResponse {
    Success(catalog::CatalogMetadataDetail),
    Error(CatalogMaintenanceError),
}
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct CatalogMaintenanceRecord {
    pub entity_id: i64,
    pub target: &'static str,
    pub label: String,
    pub activity: &'static str,
    pub revision: i64,
}
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct CatalogMaintenanceError {
    pub code: &'static str,
    pub message: &'static str,
}

pub use catalog::ProductSearchResult;

pub fn search_products(
    connection: &rusqlite::Connection,
    request: SearchProductsRequest,
) -> Result<Vec<ProductSearchResult>, String> {
    catalog::search_active_products(connection, &request.query)
        .map_err(|_| "persistence_failure".into())
}

pub fn list_catalog_maintenance(
    connection: &rusqlite::Connection,
) -> Result<CatalogMaintenanceListResponse, String> {
    let categories = connection
        .prepare("SELECT id, name, active, revision FROM categories ORDER BY name")
        .and_then(|mut statement| {
            statement
                .query_map([], |row| {
                    Ok(CatalogMaintenanceRecord {
                        entity_id: row.get(0)?,
                        target: "category",
                        label: row.get(1)?,
                        activity: activity_name(row.get(2)?),
                        revision: row.get(3)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()
        });
    let products = connection
        .prepare("SELECT id, sku || ' — ' || name, active, revision FROM products ORDER BY name")
        .and_then(|mut statement| {
            statement
                .query_map([], |row| {
                    Ok(CatalogMaintenanceRecord {
                        entity_id: row.get(0)?,
                        target: "product",
                        label: row.get(1)?,
                        activity: activity_name(row.get(2)?),
                        revision: row.get(3)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()
        });
    match categories.and_then(|mut categories: Vec<_>| {
        categories.extend(products?);
        Ok(categories)
    }) {
        Ok(records) => Ok(CatalogMaintenanceListResponse::Success { records }),
        Err(_) => Ok(CatalogMaintenanceListResponse::Error(persistence_error())),
    }
}

pub fn maintain_catalog(
    connection: &mut rusqlite::Connection,
    request: MaintainCatalogRequest,
) -> Result<CatalogMaintenanceResponse, String> {
    let (target, intent) = match (target(&request.target), intent(&request.intent)) {
        (Some(target), Some(intent)) if request.entity_id > 0 && request.expected_revision >= 0 => {
            (target, intent)
        }
        _ => return Ok(CatalogMaintenanceResponse::Error(validation_error())),
    };
    Ok(
        match catalog::MaintainCatalogUseCase::new(connection, SqliteCatalogRepository).execute(
            catalog::MaintainCatalogInput::new(
                target,
                request.entity_id,
                intent,
                request.expected_revision,
            ),
        ) {
            Ok(snapshot) => CatalogMaintenanceResponse::Success(CatalogMaintenanceRecord {
                entity_id: request.entity_id,
                target: target_name(target),
                label: String::new(),
                activity: activity(snapshot.activity),
                revision: snapshot.revision,
            }),
            Err(error) => CatalogMaintenanceResponse::Error(match error {
                catalog::MaintainCatalogError::LifecycleBlocked => CatalogMaintenanceError {
                    code: "lifecycle_blocked",
                    message: "This lifecycle change is not allowed.",
                },
                catalog::MaintainCatalogError::StaleCatalogRecord => CatalogMaintenanceError {
                    code: "stale_catalog_record",
                    message: "This catalog record changed. Reload and try again.",
                },
                catalog::MaintainCatalogError::MissingCatalogRecord => validation_error(),
                catalog::MaintainCatalogError::PersistenceFailure => persistence_error(),
            }),
        },
    )
}

pub fn edit_catalog(
    connection: &mut rusqlite::Connection,
    request: EditCatalogRequest,
) -> Result<CatalogMaintenanceResponse, String> {
    let (entity_id, target, input) = match request {
        EditCatalogRequest::Category {
            entity_id,
            expected_revision,
            name,
        } if entity_id > 0 && expected_revision >= 0 => (
            entity_id,
            "category",
            catalog::EditCatalogInput::category(entity_id, expected_revision, name),
        ),
        EditCatalogRequest::Product {
            entity_id,
            expected_revision,
            sku,
            name,
            catalog_unit_price_centavos,
            attribute_values,
        } if entity_id > 0 && expected_revision >= 0 => (
            entity_id,
            "product",
            catalog::EditCatalogInput::product(
                entity_id,
                expected_revision,
                sku,
                name,
                catalog_unit_price_centavos,
                attribute_values
                    .into_iter()
                    .map(|value| catalog::AttributeValueInput {
                        definition_id: value.definition_id,
                        value: value.value,
                    })
                    .collect(),
            ),
        ),
        _ => return Ok(CatalogMaintenanceResponse::Error(validation_error())),
    };
    Ok(
        match catalog::EditCatalogUseCase::new(connection, SqliteCatalogRepository).execute(input) {
            Ok(snapshot) => CatalogMaintenanceResponse::Success(CatalogMaintenanceRecord {
                entity_id,
                target,
                label: String::new(),
                activity: activity(snapshot.activity),
                revision: snapshot.revision,
            }),
            Err(error) => CatalogMaintenanceResponse::Error(map_maintenance_error(error)),
        },
    )
}

pub fn catalog_metadata_detail(
    connection: &rusqlite::Connection,
    request: CatalogMetadataDetailRequest,
) -> Result<CatalogMetadataDetailResponse, String> {
    let Some(target) = target(&request.target).filter(|_| request.entity_id > 0) else {
        return Ok(CatalogMetadataDetailResponse::Error(validation_error()));
    };
    Ok(
        match catalog::read_catalog_metadata_detail(connection, target, request.entity_id) {
            Ok(Some(detail)) => CatalogMetadataDetailResponse::Success(detail),
            Ok(None) => CatalogMetadataDetailResponse::Error(unavailable_error()),
            Err(_) => CatalogMetadataDetailResponse::Error(persistence_error()),
        },
    )
}

fn target(value: &str) -> Option<CatalogTarget> {
    match value {
        "category" => Some(CatalogTarget::Category),
        "product" => Some(CatalogTarget::Product),
        _ => None,
    }
}
fn intent(value: &str) -> Option<CatalogIntent> {
    match value {
        "archive" => Some(CatalogIntent::Archive),
        "reactivate" => Some(CatalogIntent::Reactivate),
        _ => None,
    }
}
fn target_name(target: CatalogTarget) -> &'static str {
    match target {
        CatalogTarget::Category => "category",
        CatalogTarget::Product => "product",
    }
}
fn activity_name(value: i64) -> &'static str {
    if value == 1 {
        "active"
    } else {
        "archived"
    }
}
fn activity(value: CatalogActivity) -> &'static str {
    match value {
        CatalogActivity::Active => "active",
        CatalogActivity::Archived => "archived",
    }
}
fn validation_error() -> CatalogMaintenanceError {
    CatalogMaintenanceError {
        code: "validation_error",
        message: "Review the catalog values and try again.",
    }
}
fn persistence_error() -> CatalogMaintenanceError {
    CatalogMaintenanceError {
        code: "persistence_failure",
        message: "The catalog could not be completed.",
    }
}
fn unavailable_error() -> CatalogMaintenanceError {
    CatalogMaintenanceError {
        code: "catalog_unavailable",
        message: "This catalog record is unavailable.",
    }
}
pub fn map_command_state_error(error: &str) -> CatalogMaintenanceError {
    if error == "database_unavailable" {
        unavailable_error()
    } else {
        persistence_error()
    }
}
fn map_maintenance_error(error: catalog::MaintainCatalogError) -> CatalogMaintenanceError {
    match error {
        catalog::MaintainCatalogError::LifecycleBlocked => CatalogMaintenanceError {
            code: "lifecycle_blocked",
            message: "This lifecycle change is not allowed.",
        },
        catalog::MaintainCatalogError::StaleCatalogRecord => CatalogMaintenanceError {
            code: "stale_catalog_record",
            message: "This catalog record changed. Reload and try again.",
        },
        catalog::MaintainCatalogError::MissingCatalogRecord => validation_error(),
        catalog::MaintainCatalogError::PersistenceFailure => persistence_error(),
    }
}
