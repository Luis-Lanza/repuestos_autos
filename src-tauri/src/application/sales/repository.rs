use rusqlite::Transaction;

use super::{ApplicationRequestedLine, PersistedSaleSummary};
use crate::domain::sales::{PaymentError, Sale, SaleLine};
use crate::domain::{MoneyCentavos, RequestId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfirmSaleError {
    DuplicateProduct,
    ProductMissing,
    ProductInactive,
    StaleCatalogPrice {
        product_id: i64,
        current_unit_price: MoneyCentavos,
        current_revision: i64,
    },
    InvalidQuantity,
    MoneyOverflow,
    QrExceedsTotal,
    CashTenderRequired,
    InsufficientCashTender,
    UnexpectedCashTender,
    InsufficientStock,
    PersistedDataInvalid,
    Persistence,
    RequestConflict,
}

impl From<PaymentError> for ConfirmSaleError {
    fn from(error: PaymentError) -> Self {
        match error {
            PaymentError::QrExceedsTotal => Self::QrExceedsTotal,
            PaymentError::CashTenderRequired => Self::CashTenderRequired,
            PaymentError::InsufficientCashTender => Self::InsufficientCashTender,
            PaymentError::UnexpectedCashTender => Self::UnexpectedCashTender,
            PaymentError::MoneyOverflow => Self::MoneyOverflow,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reservation {
    Reserved,
    ExistingConfirmed {
        summary: Option<PersistedSaleSummary>,
        operation_kind: Option<String>,
        payload_version: Option<i64>,
        canonical_payload: Option<Vec<u8>>,
        payload_sha256: Option<String>,
    },
    ExistingIncomplete {
        operation_kind: Option<String>,
        payload_version: Option<i64>,
        canonical_payload: Option<Vec<u8>>,
        payload_sha256: Option<String>,
    },
    ExistingCorrupt {
        operation_kind: Option<String>,
        payload_version: Option<i64>,
        canonical_payload: Option<Vec<u8>>,
        payload_sha256: Option<String>,
    },
}

pub trait ConfirmSaleRepository {
    fn reserve_or_load(
        &self,
        transaction: &Transaction<'_>,
        request_id: &RequestId,
        operation_kind: &str,
        payload_version: i64,
        canonical_payload: &[u8],
        payload_sha256: &str,
    ) -> Result<Reservation, ConfirmSaleError>;

    fn resolve_lines(
        &self,
        transaction: &Transaction<'_>,
        requested: &[ApplicationRequestedLine],
    ) -> Result<Vec<SaleLine>, ConfirmSaleError>;

    fn persist_confirmed(
        &self,
        transaction: &Transaction<'_>,
        request_id: &RequestId,
        sale: &Sale,
    ) -> Result<PersistedSaleSummary, ConfirmSaleError>;
}

pub trait SaleRepository {
    fn reserve_request_id(
        &self,
        transaction: &Transaction<'_>,
        request_id: &str,
    ) -> Result<bool, String>;

    fn current_line(
        &self,
        transaction: &Transaction<'_>,
        line: super::confirm_sale::RequestedLine,
    ) -> Result<SaleLine, String>;

    fn load_summary(
        &self,
        transaction: &Transaction<'_>,
        request_id: &str,
    ) -> Result<PersistedSaleSummary, String>;
}
