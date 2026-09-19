use time::{format_description::well_known::Rfc3339, OffsetDateTime, UtcOffset};

const SQLITE_TIMESTAMP_FORMAT: &[time::format_description::FormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

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
        let from = parse_utc(from_utc)?;
        let to = parse_utc(to_exclusive_utc)?;
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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct DashboardMetrics {
    pub effective_sale_count: i64,
    pub effective_total_centavos: i64,
    pub net_units_out: i64,
    pub cancelled_sale_count: i64,
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
    fn rejects_empty_or_reversed_ranges() {
        assert_eq!(DashboardRange::parse("2024-03-02T00:00:00Z", "2024-03-02T00:00:00Z"), Err(ReportingError::InvalidRange));
        assert_eq!(DashboardRange::parse("not-a-date", "2024-03-02T00:00:00Z"), Err(ReportingError::InvalidRange));
    }
}
