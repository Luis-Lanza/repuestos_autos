use crate::domain::{MoneyCentavos, Quantity};
use time::{format_description::well_known::Rfc3339, OffsetDateTime, UtcOffset};

const SALES_HISTORY_LIMIT: usize = 100;
const SALES_HISTORY_FETCH_LIMIT: usize = SALES_HISTORY_LIMIT + 1;
const SQLITE_TIMESTAMP_FORMAT: &[time::format_description::FormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HistoryError {
    InvalidRange,
    PersistedDataInvalid,
    Persistence,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryRange {
    from_sqlite: String,
    to_exclusive_sqlite: String,
}
impl HistoryRange {
    pub fn parse(from_utc: &str, to_exclusive_utc: &str) -> Result<Self, HistoryError> {
        let from = parse_utc(from_utc)?;
        let to = parse_utc(to_exclusive_utc)?;
        if from >= to {
            return Err(HistoryError::InvalidRange);
        }
        Ok(Self {
            from_sqlite: format_sqlite(from)?,
            to_exclusive_sqlite: format_sqlite(to)?,
        })
    }
    pub(crate) fn bounds(&self) -> (&str, &str) {
        (&self.from_sqlite, &self.to_exclusive_sqlite)
    }
}
fn parse_utc(value: &str) -> Result<OffsetDateTime, HistoryError> {
    OffsetDateTime::parse(value, &Rfc3339)
        .map(|value| value.to_offset(UtcOffset::UTC))
        .map_err(|_| HistoryError::InvalidRange)
}
fn format_sqlite(value: OffsetDateTime) -> Result<String, HistoryError> {
    value
        .format(SQLITE_TIMESTAMP_FORMAT)
        .map_err(|_| HistoryError::InvalidRange)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    Cash,
    Qr,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct SaleHistorySummary {
    pub sale_id: i64,
    pub confirmed_at: String,
    pub status: String,
    pub total_centavos: MoneyCentavos,
    pub line_count: u32,
    pub payment_count: u32,
    pub payment_methods: Vec<PaymentMethod>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaleHistoryPage {
    sales: Vec<SaleHistorySummary>,
    has_more: bool,
}
impl SaleHistoryPage {
    pub(crate) fn from_overfetch(mut sales: Vec<SaleHistorySummary>) -> Result<Self, HistoryError> {
        if sales.len() > SALES_HISTORY_FETCH_LIMIT {
            return Err(HistoryError::PersistedDataInvalid);
        }
        let has_more = sales.len() == SALES_HISTORY_FETCH_LIMIT;
        if has_more {
            sales.pop();
        }
        Ok(Self { sales, has_more })
    }
    pub fn sales(&self) -> &[SaleHistorySummary] {
        &self.sales
    }
    pub fn has_more(&self) -> bool {
        self.has_more
    }
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct HistoricalLine {
    pub product_id: i64,
    pub sku: Option<String>,
    pub product_name: Option<String>,
    pub quantity: Quantity,
    pub unit_price_centavos: MoneyCentavos,
    pub line_total_centavos: MoneyCentavos,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum HistoricalPayment {
    Cash {
        amount_applied_centavos: MoneyCentavos,
        amount_tendered_centavos: MoneyCentavos,
        change_given_centavos: MoneyCentavos,
    },
    Qr {
        amount_applied_centavos: MoneyCentavos,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct SaleHistoryDetail {
    pub sale_id: i64,
    pub confirmed_at: String,
    pub status: String,
    pub total_centavos: MoneyCentavos,
    pub lines: Vec<HistoricalLine>,
    pub payments: Vec<HistoricalPayment>,
}
pub trait SaleHistorySummaryReader {
    fn list(&self, range: &HistoryRange) -> Result<SaleHistoryPage, HistoryError>;
}
pub trait SaleHistoryDetailReader {
    fn detail(&self, sale_id: i64) -> Result<Option<SaleHistoryDetail>, HistoryError>;
}
pub(crate) const fn history_fetch_limit() -> usize {
    SALES_HISTORY_FETCH_LIMIT
}
