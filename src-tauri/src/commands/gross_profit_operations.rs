use serde::{Deserialize, Serialize};

use crate::{
    application::reporting::{
        load_gross_profit_operations, DashboardRange, GrossProfitOperationsPage,
        GrossProfitOperationsPagination, ReportingError,
    },
    infrastructure::sqlite::dashboard_repository::SqliteDashboardReader,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrossProfitOperationsRequest {
    pub from_utc: String,
    pub to_exclusive_utc: String,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GrossProfitOperationsResponse {
    Success { report: GrossProfitOperationsPage },
    Error(GrossProfitOperationsError),
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct GrossProfitOperationsError {
    pub code: &'static str,
    pub message: &'static str,
}

pub fn gross_profit_operations(
    connection: &rusqlite::Connection,
    request: GrossProfitOperationsRequest,
) -> GrossProfitOperationsResponse {
    let range = match DashboardRange::parse_gross_profit(&request.from_utc, &request.to_exclusive_utc) {
        Ok(range) => range,
        Err(_) => return invalid_request(),
    };
    let pagination = match GrossProfitOperationsPagination::validate(request.page, request.page_size) {
        Ok(pagination) => pagination,
        Err(_) => return invalid_request(),
    };
    match load_gross_profit_operations(&SqliteDashboardReader::new(connection), range, pagination) {
        Ok(report) => GrossProfitOperationsResponse::Success { report },
        Err(ReportingError::InvalidRange) => invalid_request(),
        Err(ReportingError::PersistedDataInvalid | ReportingError::Persistence) => {
            GrossProfitOperationsResponse::Error(GrossProfitOperationsError {
                code: "persistence_failure",
                message: "The gross-profit operations could not be loaded.",
            })
        }
    }
}

fn invalid_request() -> GrossProfitOperationsResponse {
    GrossProfitOperationsResponse::Error(GrossProfitOperationsError {
        code: "invalid_request",
        message: "The gross-profit operations request is invalid.",
    })
}
