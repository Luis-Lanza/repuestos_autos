use time::{format_description::well_known::Rfc3339, OffsetDateTime, UtcOffset};

const MAX_PAGE: u32 = 10_000;
pub const DEFAULT_PAGE_SIZE: u32 = 50;
pub const MAX_PAGE_SIZE: u32 = 100;
pub const MAX_EXPORT_ROWS: usize = 2_000;
pub const MAX_PRODUCT_OPTIONS_PAGE_SIZE: u32 = 20;
const MAX_PRODUCT_SEARCH_QUERY_LENGTH: usize = 100;
const SQLITE_TIMESTAMP_FORMAT: &[time::format_description::FormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovementType {
    OpeningStock,
    StockEntry,
    Sale,
    Return,
    Adjustment,
    Cancellation,
}

impl MovementType {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "opening_stock" => Some(Self::OpeningStock),
            "stock_entry" => Some(Self::StockEntry),
            "sale" => Some(Self::Sale),
            "return" => Some(Self::Return),
            "adjustment" => Some(Self::Adjustment),
            "cancellation" => Some(Self::Cancellation),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpeningStock => "opening_stock",
            Self::StockEntry => "stock_entry",
            Self::Sale => "sale",
            Self::Return => "return",
            Self::Adjustment => "adjustment",
            Self::Cancellation => "cancellation",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MovementLedgerError {
    InvalidRange,
    InvalidFilter,
    InvalidPage,
    PersistedDataInvalid,
    Persistence,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovementLedgerQuery {
    from_sqlite: String,
    to_exclusive_sqlite: String,
    product_id: Option<i64>,
    movement_type: Option<MovementType>,
    page: u32,
    page_size: u32,
}

impl MovementLedgerQuery {
    pub fn new(
        from_utc: &str,
        to_exclusive_utc: &str,
        product_id: Option<i64>,
        movement_type: Option<MovementType>,
        page: u32,
        page_size: u32,
    ) -> Result<Self, MovementLedgerError> {
        let from = parse_utc(from_utc)?;
        let to = parse_utc(to_exclusive_utc)?;
        if from >= to {
            return Err(MovementLedgerError::InvalidRange);
        }
        if product_id.is_some_and(|id| id <= 0) {
            return Err(MovementLedgerError::InvalidFilter);
        }
        if page == 0 || page > MAX_PAGE || page_size == 0 || page_size > MAX_PAGE_SIZE {
            return Err(MovementLedgerError::InvalidPage);
        }
        (page - 1)
            .checked_mul(page_size)
            .filter(|offset| *offset <= MAX_PAGE * MAX_PAGE_SIZE)
            .ok_or(MovementLedgerError::InvalidPage)?;
        Ok(Self {
            from_sqlite: format_sqlite(from)?,
            to_exclusive_sqlite: format_sqlite(to)?,
            product_id,
            movement_type,
            page,
            page_size,
        })
    }

    pub(crate) fn sql_parameters(&self) -> (&str, &str, Option<i64>, Option<&str>, i64, i64) {
        (
            &self.from_sqlite,
            &self.to_exclusive_sqlite,
            self.product_id,
            self.movement_type.map(MovementType::as_str),
            i64::from(self.page_size) + 1,
            i64::from((self.page - 1) * self.page_size),
        )
    }

    pub fn page(&self) -> u32 {
        self.page
    }

    pub fn page_size(&self) -> u32 {
        self.page_size
    }
}

fn parse_utc(value: &str) -> Result<OffsetDateTime, MovementLedgerError> {
    let value = OffsetDateTime::parse(value, &Rfc3339)
        .map(|value| value.to_offset(UtcOffset::UTC))
        .map_err(|_| MovementLedgerError::InvalidRange)?;
    if value.nanosecond() != 0 {
        return Err(MovementLedgerError::InvalidRange);
    }
    Ok(value)
}

fn format_sqlite(value: OffsetDateTime) -> Result<String, MovementLedgerError> {
    value
        .format(SQLITE_TIMESTAMP_FORMAT)
        .map_err(|_| MovementLedgerError::InvalidRange)
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct MovementLedgerRow {
    pub movement_id: i64,
    pub occurred_at: String,
    pub product_id: i64,
    pub product_name: String,
    pub product_sku: String,
    pub movement_type: String,
    pub quantity_delta: i64,
    pub resulting_quantity: Option<i64>,
    pub reason: Option<String>,
    pub note: Option<String>,
    pub sale_id: Option<i64>,
    pub sale_line_id: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovementLedgerPage {
    pub rows: Vec<MovementLedgerRow>,
    pub page: u32,
    pub page_size: u32,
    pub has_more: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct MovementLedgerProductOption {
    pub product_id: i64,
    pub product_name: String,
    pub product_sku: String,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovementLedgerProductOptionsQuery {
    query: String,
    page: u32,
    page_size: u32,
}

impl MovementLedgerProductOptionsQuery {
    pub fn new(query: &str, page: u32, page_size: u32) -> Result<Self, MovementLedgerError> {
        let query = query.trim();
        if query.chars().count() > MAX_PRODUCT_SEARCH_QUERY_LENGTH {
            return Err(MovementLedgerError::InvalidFilter);
        }
        if page == 0 || page > MAX_PAGE || page_size == 0 || page_size > MAX_PRODUCT_OPTIONS_PAGE_SIZE {
            return Err(MovementLedgerError::InvalidPage);
        }
        (page - 1)
            .checked_mul(page_size)
            .filter(|offset| *offset <= MAX_PAGE * MAX_PRODUCT_OPTIONS_PAGE_SIZE)
            .ok_or(MovementLedgerError::InvalidPage)?;
        Ok(Self { query: query.to_owned(), page, page_size })
    }

    pub(crate) fn sql_parameters(&self) -> (String, i64, i64, u32, u32) {
        let escaped = self.query.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
        let pattern = format!("%{escaped}%");
        (pattern, i64::from((self.page - 1) * self.page_size), i64::from(self.page_size) + 1, self.page, self.page_size)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovementLedgerProductOptionPage {
    pub products: Vec<MovementLedgerProductOption>,
    pub page: u32,
    pub page_size: u32,
    pub has_more: bool,
}

pub trait MovementLedgerReader {
    fn list(&self, query: &MovementLedgerQuery) -> Result<MovementLedgerPage, MovementLedgerError>;
    fn product_options(&self, query: &MovementLedgerProductOptionsQuery) -> Result<MovementLedgerProductOptionPage, MovementLedgerError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_the_six_persisted_movement_types() {
        for value in ["opening_stock", "stock_entry", "sale", "return", "adjustment", "cancellation"] {
            assert!(MovementType::parse(value).is_some(), "{value}");
        }
        assert_eq!(MovementType::parse("unknown"), None);
    }

    #[test]
    fn query_rejects_invalid_bounds_filters_and_pages() {
        assert_eq!(MovementLedgerQuery::new("bad", "2025-01-02T00:00:00Z", None, None, 1, DEFAULT_PAGE_SIZE), Err(MovementLedgerError::InvalidRange));
        assert_eq!(MovementLedgerQuery::new("2025-01-02T00:00:00Z", "2025-01-01T00:00:00Z", None, None, 1, DEFAULT_PAGE_SIZE), Err(MovementLedgerError::InvalidRange));
        assert_eq!(MovementLedgerQuery::new("2025-01-01T00:00:00Z", "2025-01-02T00:00:00Z", Some(0), None, 1, DEFAULT_PAGE_SIZE), Err(MovementLedgerError::InvalidFilter));
        assert_eq!(MovementLedgerQuery::new("2025-01-01T00:00:00Z", "2025-01-02T00:00:00Z", None, None, 0, DEFAULT_PAGE_SIZE), Err(MovementLedgerError::InvalidPage));
        assert_eq!(MovementLedgerQuery::new("2025-01-01T00:00:00Z", "2025-01-02T00:00:00Z", None, None, 1, MAX_PAGE_SIZE + 1), Err(MovementLedgerError::InvalidPage));
    }
}
