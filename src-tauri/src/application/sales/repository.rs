use rusqlite::Transaction;

use super::{ApplicationRequestedLine, PersistedSaleSummary};
use crate::domain::sales::{PaymentError, Sale, SaleLine};
use crate::domain::RequestId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfirmSaleError {
    DuplicateProduct,
    ProductMissing,
    ProductInactive,
    InvalidQuantity,
    MoneyOverflow,
    QrExceedsTotal,
    CashTenderRequired,
    InsufficientCashTender,
    UnexpectedCashTender,
    InsufficientStock,
    PersistedDataInvalid,
    Persistence,
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
    ExistingConfirmed(PersistedSaleSummary),
    ExistingIncomplete,
    ExistingCorrupt,
}

pub trait ConfirmSaleRepository {
    fn reserve_or_load(
        &self,
        transaction: &Transaction<'_>,
        request_id: &RequestId,
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
