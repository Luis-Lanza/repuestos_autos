use serde::{Deserialize, Serialize};

use crate::{
    application::reporting::{load_gross_profit, DashboardRange, GrossProfitReport, ReportingError},
    infrastructure::sqlite::dashboard_repository::SqliteDashboardReader,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrossProfitRequest {
    pub from_utc: String,
    pub to_exclusive_utc: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GrossProfitResponse {
    Success { report: GrossProfitReport },
    Error(GrossProfitError),
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct GrossProfitError {
    pub code: &'static str,
    pub message: &'static str,
}

pub fn gross_profit(connection: &rusqlite::Connection, request: GrossProfitRequest) -> GrossProfitResponse {
    let range = match DashboardRange::parse_gross_profit(&request.from_utc, &request.to_exclusive_utc) {
        Ok(range) => range,
        Err(_) => return invalid_range(),
    };
    match load_gross_profit(&SqliteDashboardReader::new(connection), range) {
        Ok(report) => GrossProfitResponse::Success { report },
        Err(ReportingError::InvalidRange) => invalid_range(),
        Err(ReportingError::PersistedDataInvalid | ReportingError::Persistence) => GrossProfitResponse::Error(GrossProfitError {
            code: "persistence_failure",
            message: "The gross-profit report could not be loaded.",
        }),
    }
}

fn invalid_range() -> GrossProfitResponse {
    GrossProfitResponse::Error(GrossProfitError { code: "invalid_range", message: "The gross-profit date range is invalid." })
}
