use serde::{Deserialize, Serialize};

use crate::{
    application::sales::{
        HistoryError, HistoryRange, SaleHistoryDetail, SaleHistoryDetailReader, SaleHistorySummary,
        SaleHistorySummaryReader,
    },
    infrastructure::sqlite::sale_history_repository::SqliteSaleHistoryReader,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListSalesHistoryRequest {
    pub from_utc: String,
    pub to_exclusive_utc: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SalesHistoryListResponse {
    Success {
        sales: Vec<SaleHistorySummary>,
        has_more: bool,
    },
    Error(SalesHistoryCommandError),
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SalesHistoryDetailResponse {
    Success { detail: SaleHistoryDetail },
    Error(SalesHistoryCommandError),
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct SalesHistoryCommandError {
    pub code: &'static str,
    pub message: &'static str,
}

pub fn list_sales_history(
    connection: &rusqlite::Connection,
    request: ListSalesHistoryRequest,
) -> SalesHistoryListResponse {
    let range = match HistoryRange::parse(&request.from_utc, &request.to_exclusive_utc) {
        Ok(range) => range,
        Err(error) => return SalesHistoryListResponse::Error(map_error(error)),
    };
    let reader = SqliteSaleHistoryReader::new(connection);
    match reader.list(&range) {
        Ok(page) => SalesHistoryListResponse::Success {
            sales: page.sales().to_vec(),
            has_more: page.has_more(),
        },
        Err(error) => SalesHistoryListResponse::Error(map_error(error)),
    }
}

pub fn sale_history_detail(
    connection: &rusqlite::Connection,
    sale_id: i64,
) -> SalesHistoryDetailResponse {
    let reader = SqliteSaleHistoryReader::new(connection);
    match reader.detail(sale_id) {
        Ok(Some(detail)) => SalesHistoryDetailResponse::Success { detail },
        Ok(None) => SalesHistoryDetailResponse::Error(SalesHistoryCommandError {
            code: "sale_not_found",
            message: "The sale was not found.",
        }),
        Err(error) => SalesHistoryDetailResponse::Error(map_error(error)),
    }
}

pub fn persistence_failure() -> SalesHistoryCommandError {
    SalesHistoryCommandError {
        code: "persistence_failure",
        message: "Sales history could not be loaded.",
    }
}

fn map_error(error: HistoryError) -> SalesHistoryCommandError {
    match error {
        HistoryError::InvalidRange => SalesHistoryCommandError {
            code: "invalid_range",
            message: "The history date range is invalid.",
        },
        HistoryError::PersistedDataInvalid | HistoryError::Persistence => persistence_failure(),
    }
}
