use serde::{Deserialize, Serialize};

use crate::application::sales;
use crate::domain::sales::Payment;
use crate::domain::{MoneyCentavos, Quantity, RequestId};

#[derive(Debug, Deserialize)]
pub struct ConfirmSaleRequest {
    pub request_id: String,
    pub lines: Vec<RequestedLine>,
    pub payments: Vec<PaymentRequest>,
}

#[derive(Debug, Deserialize)]
pub struct RequestedLine {
    pub product_id: i64,
    pub quantity: i64,
    pub negotiated_unit_price_centavos: i64,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "method", rename_all = "lowercase")]
pub enum PaymentRequest {
    Cash {
        amount_applied_centavos: i64,
        amount_tendered_centavos: i64,
        change_given_centavos: i64,
    },
    Qr {
        amount_applied_centavos: i64,
    },
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
    pub negotiated_unit_price_centavos: i64,
    pub minimum_unit_price_snapshot_centavos: i64,
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
    Ok(match sales::confirm_sale(connection, request) {
        Ok(summary) => ConfirmSaleResponse::Success(map_summary(summary)),
        Err(error) => ConfirmSaleResponse::Error(map_error(&error)),
    })
}

fn parse_request(request: ConfirmSaleRequest) -> Result<sales::ConfirmSaleRequest, CommandError> {
    let request_id = RequestId::parse(&request.request_id).map_err(|_| invalid_request())?;
    let lines = request
        .lines
        .into_iter()
        .map(|line| {
            Ok(sales::RequestedLine {
                product_id: line.product_id,
                quantity: Quantity::new(line.quantity).map_err(|_| invalid_quantity())?,
                negotiated_unit_price: MoneyCentavos::new(line.negotiated_unit_price_centavos)
                    .map_err(|_| invalid_request())?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let payments = request
        .payments
        .into_iter()
        .map(|payment| match payment {
            PaymentRequest::Cash {
                amount_applied_centavos,
                amount_tendered_centavos,
                change_given_centavos,
            } => Payment::cash(
                MoneyCentavos::new(amount_applied_centavos).map_err(|_| invalid_payment())?,
                MoneyCentavos::new(amount_tendered_centavos).map_err(|_| invalid_payment())?,
                MoneyCentavos::new(change_given_centavos).map_err(|_| invalid_payment())?,
            )
            .map_err(|_| invalid_payment()),
            PaymentRequest::Qr {
                amount_applied_centavos,
            } => Ok(Payment::qr(
                MoneyCentavos::new(amount_applied_centavos).map_err(|_| invalid_payment())?,
            )),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(sales::ConfirmSaleRequest {
        request_id,
        lines,
        payments,
    })
}

fn map_summary(summary: sales::PersistedSaleSummary) -> PersistedSaleSummary {
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
                negotiated_unit_price_centavos: line.negotiated_unit_price.value(),
                minimum_unit_price_snapshot_centavos: line.minimum_unit_price_snapshot.value(),
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

fn map_error(error: &str) -> CommandError {
    let (code, message) = match error {
        "product is inactive" => ("inactive_product", "The product is inactive."),
        "product is missing" => ("missing_product", "The product was not found."),
        "negotiated price is below the current minimum" => (
            "price_below_minimum",
            "The negotiated price is below the current minimum.",
        ),
        "quantity must be positive" => (
            "invalid_quantity",
            "Quantity must be a positive whole number.",
        ),
        "applied payments must equal the sale total"
        | "cash tender and change are inconsistent" => {
            ("invalid_payment", "Payment values are invalid.")
        }
        "insufficient stock" => ("insufficient_stock", "Insufficient stock is available."),
        _ => ("persistence_failure", "The sale could not be persisted."),
    };
    CommandError { code, message }
}
