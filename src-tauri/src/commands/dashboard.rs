use serde::{Deserialize, Serialize};

use crate::{
    application::reporting::{load_dashboard, DashboardRange, ReportingError},
    infrastructure::sqlite::dashboard_repository::SqliteDashboardReader,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardRequest {
    pub today_from_utc: String,
    pub today_to_exclusive_utc: String,
    pub month_from_utc: String,
    pub month_to_exclusive_utc: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DashboardResponse {
    Success { report: crate::application::reporting::DashboardReport },
    Error(DashboardError),
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct DashboardError {
    pub code: &'static str,
    pub message: &'static str,
}

pub fn dashboard(connection: &rusqlite::Connection, request: DashboardRequest) -> DashboardResponse {
    let today = match DashboardRange::parse(&request.today_from_utc, &request.today_to_exclusive_utc) {
        Ok(range) => range,
        Err(error) => return DashboardResponse::Error(map_error(error)),
    };
    let month = match DashboardRange::parse(&request.month_from_utc, &request.month_to_exclusive_utc) {
        Ok(range) => range,
        Err(error) => return DashboardResponse::Error(map_error(error)),
    };
    let reader = SqliteDashboardReader::new(connection);
    match load_dashboard(&reader, today, month) {
        Ok(report) => DashboardResponse::Success { report },
        Err(error) => DashboardResponse::Error(map_error(error)),
    }
}

pub fn persistence_failure() -> DashboardError {
    DashboardError { code: "persistence_failure", message: "The dashboard could not be loaded." }
}

fn map_error(error: ReportingError) -> DashboardError {
    match error {
        ReportingError::InvalidRange => DashboardError { code: "invalid_range", message: "The dashboard date range is invalid." },
        ReportingError::PersistedDataInvalid | ReportingError::Persistence => persistence_failure(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::sqlite::open_seeded_catalog;

    fn request() -> DashboardRequest {
        DashboardRequest {
            today_from_utc: "2024-03-10T05:00:00Z".into(),
            today_to_exclusive_utc: "2024-03-11T04:00:00Z".into(),
            month_from_utc: "2024-03-01T05:00:00Z".into(),
            month_to_exclusive_utc: "2024-04-01T04:00:00Z".into(),
        }
    }

    #[test]
    fn returns_one_read_only_success_envelope() {
        let connection = open_seeded_catalog().unwrap();
        let response = dashboard(&connection, request());
        assert!(matches!(response, DashboardResponse::Success { .. }));
    }

    #[test]
    fn maps_invalid_bounds_without_querying_storage() {
        let connection = open_seeded_catalog().unwrap();
        let mut input = request();
        input.today_to_exclusive_utc = input.today_from_utc.clone();
        assert_eq!(dashboard(&connection, input), DashboardResponse::Error(DashboardError { code: "invalid_range", message: "The dashboard date range is invalid." }));
    }
}
