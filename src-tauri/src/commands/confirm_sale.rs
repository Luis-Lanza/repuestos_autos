use serde::{Deserialize, Serialize};

use crate::application::sales::{
    ApplicationConfirmSaleRequest, ApplicationRequestedLine, ConfirmSaleError, ConfirmSaleUseCase,
};
use crate::domain::sales::{Payment, PaymentInput};
use crate::domain::{MoneyCentavos, Quantity, RequestId};
use crate::infrastructure::sqlite::sale_repository::SqliteSaleRepository;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfirmSaleRequest {
    pub request_id: String,
    pub lines: Vec<RequestedLine>,
    pub payment: PaymentInputRequest,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestedLine {
    pub product_id: i64,
    pub quantity: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaymentInputRequest {
    pub amount_tendered_centavos: Option<i64>,
    pub qr_applied_centavos: Option<i64>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConfirmSaleResponse {
    Success(PersistedSaleSummary),
    Error(CommandError),
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct CommandError {
    pub code: &'static str,
    pub message: &'static str,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct PersistedSaleSummary {
    pub sale_id: i64,
    pub request_id: String,
    pub status: String,
    pub confirmed_at: String,
    pub outcome: &'static str,
    pub lines: Vec<PersistedLine>,
    pub payments: Vec<Payment>,
    pub total_centavos: i64,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct PersistedLine {
    pub product_id: i64,
    pub sku: String,
    pub product_name: String,
    pub quantity: i64,
    pub unit_price_centavos: i64,
    pub line_total_centavos: i64,
}

pub fn confirm_sale(
    connection: &mut rusqlite::Connection,
    request: ConfirmSaleRequest,
) -> Result<ConfirmSaleResponse, String> {
    let request = match parse_request(request) {
        Ok(request) => request,
        Err(error) => return Ok(ConfirmSaleResponse::Error(error)),
    };
    let repository = SqliteSaleRepository;
    Ok(
        match ConfirmSaleUseCase::new(connection, &repository).confirm(request) {
            Ok(summary) => ConfirmSaleResponse::Success(map_summary(summary)),
            Err(error) => ConfirmSaleResponse::Error(map_error(error)),
        },
    )
}

fn parse_request(
    request: ConfirmSaleRequest,
) -> Result<ApplicationConfirmSaleRequest, CommandError> {
    let request_id = RequestId::parse(&request.request_id).map_err(|_| invalid_request())?;
    let lines = request
        .lines
        .into_iter()
        .map(|line| {
            Ok(ApplicationRequestedLine {
                product_id: line.product_id,
                quantity: Quantity::new(line.quantity).map_err(|_| invalid_quantity())?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ApplicationConfirmSaleRequest {
        request_id,
        lines,
        payment: PaymentInput {
            amount_tendered: parse_money(request.payment.amount_tendered_centavos)?,
            qr_applied: parse_money(request.payment.qr_applied_centavos)?,
        },
    })
}

fn parse_money(value: Option<i64>) -> Result<Option<MoneyCentavos>, CommandError> {
    value
        .map(|centavos| MoneyCentavos::new(centavos).map_err(|_| invalid_payment()))
        .transpose()
}

fn map_summary(summary: crate::application::sales::PersistedSaleSummary) -> PersistedSaleSummary {
    PersistedSaleSummary {
        sale_id: summary.sale_id,
        request_id: summary.request_id.as_uuid().to_string(),
        status: summary.status,
        confirmed_at: summary.confirmed_at,
        outcome: "confirmed",
        lines: summary
            .lines
            .into_iter()
            .map(|line| PersistedLine {
                product_id: line.product_id,
                sku: line.sku,
                product_name: line.product_name,
                quantity: line.quantity.value(),
                unit_price_centavos: line.negotiated_unit_price.value(),
                line_total_centavos: line.line_total.value(),
            })
            .collect(),
        payments: summary.payments,
        total_centavos: summary.total.value(),
    }
}

fn invalid_request() -> CommandError {
    CommandError {
        code: "invalid_request",
        message: "The request shape is invalid.",
    }
}

fn invalid_quantity() -> CommandError {
    CommandError {
        code: "invalid_quantity",
        message: "Quantity must be a positive whole number.",
    }
}

fn invalid_payment() -> CommandError {
    CommandError {
        code: "invalid_payment",
        message: "Payment values are invalid.",
    }
}

fn map_error(error: ConfirmSaleError) -> CommandError {
    let (code, message) = match error {
        ConfirmSaleError::ProductInactive => ("inactive_product", "The product is inactive."),
        ConfirmSaleError::ProductMissing => ("missing_product", "The product was not found."),
        ConfirmSaleError::InvalidQuantity => (
            "invalid_quantity",
            "Quantity must be a positive whole number.",
        ),
        ConfirmSaleError::QrExceedsTotal
        | ConfirmSaleError::CashTenderRequired
        | ConfirmSaleError::InsufficientCashTender
        | ConfirmSaleError::UnexpectedCashTender => {
            ("invalid_payment", "Payment values are invalid.")
        }
        ConfirmSaleError::InsufficientStock => {
            ("insufficient_stock", "Insufficient stock is available.")
        }
        ConfirmSaleError::DuplicateProduct => ("invalid_request", "The request shape is invalid."),
        ConfirmSaleError::MoneyOverflow
        | ConfirmSaleError::PersistedDataInvalid
        | ConfirmSaleError::Persistence => {
            ("persistence_failure", "The sale could not be persisted.")
        }
    };
    CommandError { code, message }
}
