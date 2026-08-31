use serde::{Deserialize, Serialize};

use crate::{
    application::sales::{
        CancelSaleRequest as ApplicationCancelSaleRequest, CancellationResult, CreateReturnRequest,
        PostSaleError, PostSaleLifecycleUseCase, PostSaleUseCase, ReturnResult,
        SaleLifecycleStatus,
    },
    domain::{sales::RequestedReturnLine as ApplicationReturnLine, RequestId},
    infrastructure::sqlite::{SqlitePostSaleRepository, SqlitePostSaleTransactionFactory},
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateSaleReturnRequest {
    pub request_id: String,
    pub sale_id: i64,
    pub lines: Vec<RequestedReturnLine>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestedReturnLine {
    pub sale_line_id: i64,
    pub quantity: i64,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CancelSaleRequest {
    pub request_id: String,
    pub sale_id: i64,
    pub reason: String,
}
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CommandResponse<Result> {
    Success { result: Result },
    Error(crate::commands::confirm_sale::CommandError),
}

pub type PostSaleCommandResponse = CommandResponse<ReturnCommandResult>;
pub type CancelSaleCommandResponse = CommandResponse<CancelSaleCommandResult>;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct ReturnCommandResult {
    pub request_id: String,
    pub return_id: i64,
    pub sale_id: i64,
    pub status: &'static str,
    pub occurred_at: String,
    pub lines: Vec<ReturnCommandLine>,
}
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct ReturnCommandLine {
    pub sale_line_id: i64,
    pub product_id: i64,
    pub quantity: i64,
}
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct CancelSaleCommandResult {
    pub request_id: String,
    pub cancellation_id: i64,
    pub sale_id: i64,
    pub status: &'static str,
    pub occurred_at: String,
    pub reason: String,
    pub lines: Vec<CancelSaleCommandLine>,
}
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct CancelSaleCommandLine {
    pub sale_line_id: i64,
    pub product_id: i64,
    pub restored_quantity: i64,
}

pub fn create_sale_return(
    connection: &mut rusqlite::Connection,
    request: CreateSaleReturnRequest,
) -> PostSaleCommandResponse {
    let request = match parse_return(request) {
        Ok(request) => request,
        Err(error) => return PostSaleCommandResponse::Error(error),
    };
    let repository = SqlitePostSaleRepository;
    let mut transactions = SqlitePostSaleTransactionFactory::new(connection);
    match PostSaleUseCase::new(&mut transactions, &repository).create_return(request) {
        Ok(result) => PostSaleCommandResponse::Success {
            result: map_return(result),
        },
        Err(error) => PostSaleCommandResponse::Error(map_error(error)),
    }
}

pub fn cancel_sale(
    connection: &mut rusqlite::Connection,
    request: CancelSaleRequest,
) -> CancelSaleCommandResponse {
    let request = match parse_cancellation(request) {
        Ok(request) => request,
        Err(error) => return CancelSaleCommandResponse::Error(error),
    };
    let repository = SqlitePostSaleRepository;
    let mut transactions = SqlitePostSaleTransactionFactory::new(connection);
    match PostSaleUseCase::new(&mut transactions, &repository).cancel_sale(request) {
        Ok(result) => CancelSaleCommandResponse::Success {
            result: map_cancellation(result),
        },
        Err(error) => CancelSaleCommandResponse::Error(map_error(error)),
    }
}

fn parse_return(
    request: CreateSaleReturnRequest,
) -> Result<CreateReturnRequest, crate::commands::confirm_sale::CommandError> {
    let request_id = RequestId::parse(&request.request_id).map_err(|_| invalid_request())?;
    CreateReturnRequest::new(
        request_id,
        request.sale_id,
        request
            .lines
            .into_iter()
            .map(|line| ApplicationReturnLine {
                sale_line_id: line.sale_line_id,
                quantity: line.quantity,
            })
            .collect(),
    )
    .map_err(map_error)
}

fn parse_cancellation(
    request: CancelSaleRequest,
) -> Result<ApplicationCancelSaleRequest, crate::commands::confirm_sale::CommandError> {
    let request_id = RequestId::parse(&request.request_id).map_err(|_| invalid_request())?;
    ApplicationCancelSaleRequest::new(request_id, request.sale_id, request.reason)
        .map_err(map_error)
}

fn map_return(result: ReturnResult) -> ReturnCommandResult {
    ReturnCommandResult {
        request_id: result.request_id,
        return_id: result.return_id,
        sale_id: result.sale_id,
        status: status(result.status),
        occurred_at: result.occurred_at,
        lines: result
            .lines
            .into_iter()
            .map(|line| ReturnCommandLine {
                sale_line_id: line.sale_line_id,
                product_id: line.product_id,
                quantity: line.quantity,
            })
            .collect(),
    }
}

fn map_cancellation(result: CancellationResult) -> CancelSaleCommandResult {
    CancelSaleCommandResult {
        request_id: result.request_id,
        cancellation_id: result.cancellation_id,
        sale_id: result.sale_id,
        status: status(result.status),
        occurred_at: result.occurred_at,
        reason: result.reason,
        lines: result
            .lines
            .into_iter()
            .map(|line| CancelSaleCommandLine {
                sale_line_id: line.sale_line_id,
                product_id: line.product_id,
                restored_quantity: line.restored_quantity,
            })
            .collect(),
    }
}

fn status(status: SaleLifecycleStatus) -> &'static str {
    match status {
        SaleLifecycleStatus::Confirmed => "confirmed",
        SaleLifecycleStatus::Cancelled => "cancelled",
    }
}

fn invalid_request() -> crate::commands::confirm_sale::CommandError {
    crate::commands::confirm_sale::CommandError {
        code: "invalid_request",
        message: "The inventory correction request is invalid.",
    }
}

fn map_error(error: PostSaleError) -> crate::commands::confirm_sale::CommandError {
    use crate::domain::sales::PostSaleDomainError::*;
    let code = match error {
        PostSaleError::InvalidRequest | PostSaleError::Domain(EmptyReturn) => "invalid_request",
        PostSaleError::Domain(InvalidQuantity) => "invalid_quantity",
        PostSaleError::Domain(DuplicateSaleLine) => "duplicate_sale_line",
        PostSaleError::Domain(SaleNotFound) => "sale_not_found",
        PostSaleError::Domain(SaleNotConfirmed) => "sale_not_confirmed",
        PostSaleError::Domain(SaleCancelled) => "sale_cancelled",
        PostSaleError::Domain(SaleLineNotFound) => "sale_line_not_found",
        PostSaleError::Domain(QuantityExceedsRemaining) => "quantity_exceeds_remaining",
        PostSaleError::Domain(CancellationReasonRequired) => "cancellation_reason_required",
        PostSaleError::Domain(CancellationAlreadyRecorded) => "cancellation_already_recorded",
        PostSaleError::RequestConflict => "request_conflict",
        PostSaleError::Domain(InvalidOriginalQuantity) | PostSaleError::PersistenceFailure => {
            "persistence_failure"
        }
    };
    crate::commands::confirm_sale::CommandError {
        code,
        message: "The inventory correction could not be completed.",
    }
}

pub fn persistence_failure() -> crate::commands::confirm_sale::CommandError {
    crate::commands::confirm_sale::CommandError {
        code: "persistence_failure",
        message: "The inventory correction could not be completed.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::sales::PostSaleDomainError::*;

    fn assert_safe_transport_text(text: &str) {
        const FORBIDDEN: &[&str] = &[
            "sql",
            "schema",
            "driver",
            "select",
            "sqlite",
            "table",
            "constraint",
            "rusqlite",
            "refund",
            "reimbursement",
            "reversal",
            "credit",
            "settlement",
            "payment",
        ];
        assert!(["sql", "schema", "driver"]
            .iter()
            .all(|word| FORBIDDEN.contains(word)));
        assert!(FORBIDDEN
            .iter()
            .all(|word| !text.to_lowercase().contains(word)));
    }

    #[test]
    fn every_application_error_has_a_stable_inventory_only_transport_error() {
        let mappings = [
            (PostSaleError::InvalidRequest, "invalid_request"),
            (PostSaleError::Domain(EmptyReturn), "invalid_request"),
            (PostSaleError::Domain(InvalidQuantity), "invalid_quantity"),
            (
                PostSaleError::Domain(DuplicateSaleLine),
                "duplicate_sale_line",
            ),
            (PostSaleError::Domain(SaleNotFound), "sale_not_found"),
            (
                PostSaleError::Domain(SaleNotConfirmed),
                "sale_not_confirmed",
            ),
            (PostSaleError::Domain(SaleCancelled), "sale_cancelled"),
            (
                PostSaleError::Domain(SaleLineNotFound),
                "sale_line_not_found",
            ),
            (
                PostSaleError::Domain(QuantityExceedsRemaining),
                "quantity_exceeds_remaining",
            ),
            (
                PostSaleError::Domain(CancellationReasonRequired),
                "cancellation_reason_required",
            ),
            (
                PostSaleError::Domain(CancellationAlreadyRecorded),
                "cancellation_already_recorded",
            ),
            (PostSaleError::RequestConflict, "request_conflict"),
            (
                PostSaleError::Domain(InvalidOriginalQuantity),
                "persistence_failure",
            ),
            (PostSaleError::PersistenceFailure, "persistence_failure"),
        ];
        for (input, code) in mappings {
            let error = map_error(input);
            assert_eq!(error.code, code);
            assert_eq!(
                error.message,
                "The inventory correction could not be completed."
            );
            assert_safe_transport_text(error.message);
        }
        for error in [invalid_request(), persistence_failure()] {
            assert!(error.message.contains("inventory correction"));
            assert_safe_transport_text(error.message);
        }
    }
}
