use serde::{Deserialize, Serialize};

use crate::application::inventory::{
    confirm_physical_count, confirm_stock_entry, list_inventory_alerts,
};
use crate::commands::confirm_sale::CommandError;
use crate::domain::inventory::{AlertClassification, InventoryError};
use crate::domain::RequestId;
use crate::infrastructure::sqlite::SqliteInventoryRepository;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StockEntryRequest {
    pub request_id: String,
    pub product_id: i64,
    pub quantity: i64,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalCountRequest {
    pub request_id: String,
    pub product_id: i64,
    pub count: i64,
    pub reason: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InventoryCommandResponse {
    Success(PersistedInventoryOperation),
    Alerts(InventoryAlerts),
    Error(CommandError),
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct PersistedInventoryOperation {
    pub request_id: String,
    pub product_id: i64,
    pub previous_quantity: i64,
    pub quantity_delta: i64,
    pub resulting_quantity: i64,
    pub occurred_at: String,
    pub note: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct InventoryAlerts {
    pub alerts: Vec<InventoryAlert>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct InventoryAlert {
    pub product_id: i64,
    pub product_name: String,
    pub quantity: i64,
    pub classification: &'static str,
}

pub fn confirm_stock_entry_command(
    connection: &mut rusqlite::Connection,
    request: StockEntryRequest,
) -> Result<InventoryCommandResponse, String> {
    let request_id = match RequestId::parse(&request.request_id) {
        Ok(value) => value,
        Err(_) => return Ok(InventoryCommandResponse::Error(invalid_request())),
    };
    let mut repository = SqliteInventoryRepository::new(connection);
    Ok(
        match confirm_stock_entry(
            &mut repository,
            request.product_id,
            request_id,
            request.quantity,
            request.note,
        ) {
            Ok(result) => InventoryCommandResponse::Success(map_result(result)),
            Err(error) => InventoryCommandResponse::Error(map_error(error)),
        },
    )
}

pub fn confirm_physical_count_command(
    connection: &mut rusqlite::Connection,
    request: PhysicalCountRequest,
) -> Result<InventoryCommandResponse, String> {
    let request_id = match RequestId::parse(&request.request_id) {
        Ok(value) => value,
        Err(_) => return Ok(InventoryCommandResponse::Error(invalid_request())),
    };
    let mut repository = SqliteInventoryRepository::new(connection);
    Ok(
        match confirm_physical_count(
            &mut repository,
            request.product_id,
            request_id,
            request.count,
            &request.reason,
        ) {
            Ok(result) => InventoryCommandResponse::Success(map_result(result)),
            Err(error) => InventoryCommandResponse::Error(map_error(error)),
        },
    )
}

pub fn list_inventory_alerts_command(
    connection: &mut rusqlite::Connection,
) -> Result<InventoryCommandResponse, String> {
    let repository = SqliteInventoryRepository::new(connection);
    Ok(match list_inventory_alerts(&repository) {
        Ok(alerts) => InventoryCommandResponse::Alerts(InventoryAlerts {
            alerts: alerts
                .into_iter()
                .map(|alert| InventoryAlert {
                    product_id: alert.product_id,
                    product_name: alert.product_name,
                    quantity: alert.quantity,
                    classification: match alert.classification {
                        AlertClassification::OutOfStock => "out_of_stock",
                        AlertClassification::LowStock => "low_stock",
                    },
                })
                .collect(),
        }),
        Err(error) => InventoryCommandResponse::Error(map_error(error)),
    })
}

fn map_result(
    result: crate::domain::inventory::PersistedInventoryOperation,
) -> PersistedInventoryOperation {
    PersistedInventoryOperation {
        request_id: result.request_id.as_uuid().to_string(),
        product_id: result.product_id,
        previous_quantity: result.previous_quantity,
        quantity_delta: result.quantity_delta,
        resulting_quantity: result.resulting_quantity,
        occurred_at: result.occurred_at,
        note: result.note,
    }
}

fn invalid_request() -> CommandError {
    CommandError {
        code: "invalid_request",
        message: "The request shape is invalid.",
    }
}
fn map_error(error: InventoryError) -> CommandError {
    let message = match error.code() {
        "invalid_quantity" => "Quantity must be a positive whole number.",
        "invalid_count" => "Count must be a non-negative whole number.",
        "reason_required" => "A reason is required.",
        "missing_product" => "The product was not found.",
        "inactive_product" => "The product is inactive.",
        "unchanged_count" => "The count matches the current stock.",
        _ => "The inventory operation could not be completed.",
    };
    CommandError {
        code: error.code(),
        message,
    }
}
