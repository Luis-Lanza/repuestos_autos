use time::{format_description::well_known::Rfc3339, OffsetDateTime, UtcOffset};

const SQLITE_TIMESTAMP_FORMAT: &[time::format_description::FormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
const GROSS_PROFIT_UTC_FORMAT: &[time::format_description::FormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReportingError {
    InvalidRange,
    PersistedDataInvalid,
    Persistence,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardRange {
    from: String,
    to_exclusive: String,
}

impl DashboardRange {
    pub fn parse(from_utc: &str, to_exclusive_utc: &str) -> Result<Self, ReportingError> {
        Self::parse_with(from_utc, to_exclusive_utc, parse_utc)
    }

    pub fn parse_gross_profit(from_utc: &str, to_exclusive_utc: &str) -> Result<Self, ReportingError> {
        Self::parse_with(from_utc, to_exclusive_utc, parse_canonical_gross_profit_utc)
    }

    fn parse_with(
        from_utc: &str,
        to_exclusive_utc: &str,
        parse_bound: fn(&str) -> Result<OffsetDateTime, ReportingError>,
    ) -> Result<Self, ReportingError> {
        let from = parse_bound(from_utc)?;
        let to = parse_bound(to_exclusive_utc)?;
        if from >= to {
            return Err(ReportingError::InvalidRange);
        }
        Ok(Self {
            from: from.format(SQLITE_TIMESTAMP_FORMAT).map_err(|_| ReportingError::InvalidRange)?,
            to_exclusive: to.format(SQLITE_TIMESTAMP_FORMAT).map_err(|_| ReportingError::InvalidRange)?,
        })
    }

    pub(crate) fn bounds(&self) -> (&str, &str) {
        (&self.from, &self.to_exclusive)
    }
}

fn parse_utc(value: &str) -> Result<OffsetDateTime, ReportingError> {
    OffsetDateTime::parse(value, &Rfc3339)
        .map(|value| value.to_offset(UtcOffset::UTC))
        .map_err(|_| ReportingError::InvalidRange)
}

fn parse_canonical_gross_profit_utc(value: &str) -> Result<OffsetDateTime, ReportingError> {
    if !value.ends_with(".000Z") {
        return Err(ReportingError::InvalidRange);
    }
    let parsed = parse_utc(value)?;
    let canonical = parsed
        .format(GROSS_PROFIT_UTC_FORMAT)
        .map_err(|_| ReportingError::InvalidRange)?;
    if canonical != value {
        return Err(ReportingError::InvalidRange);
    }
    Ok(parsed)
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct RealizedGrossProfit {
    pub amount_centavos: i64,
    pub missing_cost_line_count: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct DashboardMetrics {
    pub effective_sale_count: i64,
    pub effective_total_centavos: i64,
    pub net_units_out: i64,
    pub cancelled_sale_count: i64,
    pub realized_gross_profit: RealizedGrossProfit,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct DashboardPeriod {
    pub metrics: DashboardMetrics,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct DashboardProduct {
    pub product_id: i64,
    pub sku: String,
    pub product_name: String,
    pub net_units_out: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct DashboardPayment {
    pub method: String,
    pub amount_applied_centavos: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct DashboardRecentSale {
    pub sale_id: i64,
    pub confirmed_at: String,
    pub status: String,
    pub total_centavos: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct DashboardStockAlert {
    pub product_id: i64,
    pub sku: String,
    pub product_name: String,
    pub quantity: i64,
    pub classification: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct DashboardReport {
    pub today: DashboardPeriod,
    pub month: DashboardPeriod,
    pub top_products: Vec<DashboardProduct>,
    pub payment_distribution: Vec<DashboardPayment>,
    pub recent_sales: Vec<DashboardRecentSale>,
    pub stock_alerts: Vec<DashboardStockAlert>,
}

pub trait DashboardReader {
    fn read(&self, today: &DashboardRange, month: &DashboardRange) -> Result<DashboardReport, ReportingError>;
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct GrossProfitReport {
    pub amount_centavos: i64,
    pub missing_cost_line_count: i64,
    pub activity_count: i64,
}

pub trait GrossProfitReader {
    fn read_gross_profit(&self, range: &DashboardRange) -> Result<GrossProfitReport, ReportingError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum GrossProfitOperationKind {
    #[serde(rename = "venta")]
    Sale,
    #[serde(rename = "devolución")]
    Return,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GrossProfitCostState {
    Known,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct GrossProfitOperationRow {
    pub occurred_at: String,
    pub operation_kind: GrossProfitOperationKind,
    pub sale_id: i64,
    pub return_id: Option<i64>,
    pub product_name: String,
    pub sku: String,
    pub signed_quantity: i64,
    pub negotiated_unit_price_centavos: i64,
    pub unit_cost_snapshot_centavos: Option<i64>,
    pub cost_state: GrossProfitCostState,
    pub signed_gross_profit_centavos: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct GrossProfitOperationsPage {
    pub rows: Vec<GrossProfitOperationRow>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
    pub total_pages: i64,
}

pub const GROSS_PROFIT_OPERATIONS_MAX_PAGE_SIZE: i64 = 100;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrossProfitOperationsPagination {
    pub page: i64,
    pub page_size: i64,
}

impl GrossProfitOperationsPagination {
    pub fn validate(page: i64, page_size: i64) -> Result<Self, ReportingError> {
        if page <= 0 || !(1..=GROSS_PROFIT_OPERATIONS_MAX_PAGE_SIZE).contains(&page_size) {
            return Err(ReportingError::InvalidRange);
        }
        page.checked_sub(1).and_then(|value| value.checked_mul(page_size))
            .ok_or(ReportingError::InvalidRange)?;
        Ok(Self { page, page_size })
    }

    pub(crate) fn offset(&self) -> Result<i64, ReportingError> {
        self.page.checked_sub(1).and_then(|value| value.checked_mul(self.page_size))
            .ok_or(ReportingError::InvalidRange)
    }
}

pub trait GrossProfitOperationsReader {
    fn read_gross_profit_operations(
        &self,
        range: &DashboardRange,
        pagination: &GrossProfitOperationsPagination,
    ) -> Result<GrossProfitOperationsPage, ReportingError>;
}

pub fn load_gross_profit_operations<R: GrossProfitOperationsReader>(
    reader: &R,
    range: DashboardRange,
    pagination: GrossProfitOperationsPagination,
) -> Result<GrossProfitOperationsPage, ReportingError> {
    reader.read_gross_profit_operations(&range, &pagination)
}

pub fn load_gross_profit<R: GrossProfitReader>(reader: &R, range: DashboardRange) -> Result<GrossProfitReport, ReportingError> {
    reader.read_gross_profit(&range)
}

pub fn load_dashboard<R: DashboardReader>(reader: &R, today: DashboardRange, month: DashboardRange) -> Result<DashboardReport, ReportingError> {
    reader.read(&today, &month)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_utc_ranges_and_normalizes_offsets_to_half_open_sqlite_bounds() {
        let range = DashboardRange::parse("2024-03-01T00:00:00-05:00", "2024-03-02T00:00:00-05:00").unwrap();
        assert_eq!(range.bounds(), ("2024-03-01 05:00:00", "2024-03-02 05:00:00"));
    }

    #[test]
    fn gross_profit_bounds_require_canonical_millisecond_utc_timestamps() {
        assert!(DashboardRange::parse_gross_profit("2024-03-10T05:00:00.000Z", "2024-03-11T04:00:00.000Z").is_ok());
        for invalid in [
            "2024-03-10T05:00:00Z",
            "2024-03-10T05:00:00.001Z",
            "2024-03-10T05:00:00.000000Z",
            "2024-03-10T05:00:00.000+00:00",
        ] {
            assert_eq!(DashboardRange::parse_gross_profit(invalid, "2024-03-11T04:00:00.000Z"), Err(ReportingError::InvalidRange));
        }
    }

    #[test]
    fn rejects_empty_or_reversed_ranges() {
        assert_eq!(DashboardRange::parse("2024-03-02T00:00:00Z", "2024-03-02T00:00:00Z"), Err(ReportingError::InvalidRange));
        assert_eq!(DashboardRange::parse("not-a-date", "2024-03-02T00:00:00Z"), Err(ReportingError::InvalidRange));
    }
}
