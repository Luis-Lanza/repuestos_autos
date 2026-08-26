use crate::domain::RequestId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryError(&'static str);

impl InventoryError {
    pub const INVALID_REQUEST: Self = Self("invalid_request");
    pub const INVALID_QUANTITY: Self = Self("invalid_quantity");
    pub const INVALID_COUNT: Self = Self("invalid_count");
    pub const REASON_REQUIRED: Self = Self("reason_required");
    pub const UNCHANGED_COUNT: Self = Self("unchanged_count");
    pub const QUANTITY_OVERFLOW: Self = Self("quantity_overflow");
    pub const MISSING_PRODUCT: Self = Self("missing_product");
    pub const INACTIVE_PRODUCT: Self = Self("inactive_product");
    pub const PERSISTED_DATA_INVALID: Self = Self("persisted_data_invalid");
    pub const PERSISTENCE_FAILURE: Self = Self("persistence_failure");
    pub fn code(&self) -> &'static str {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StockEntryQuantity(i64);
impl StockEntryQuantity {
    pub fn new(value: i64) -> Result<Self, InventoryError> {
        (value > 0)
            .then_some(Self(value))
            .ok_or(InventoryError::INVALID_QUANTITY)
    }
    pub fn value(self) -> i64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalCount(i64);
impl PhysicalCount {
    pub fn new(value: i64) -> Result<Self, InventoryError> {
        (value >= 0)
            .then_some(Self(value))
            .ok_or(InventoryError::INVALID_COUNT)
    }
    pub fn value(self) -> i64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdjustmentReason(String);
impl AdjustmentReason {
    pub fn new(value: &str) -> Result<Self, InventoryError> {
        let value = value.trim();
        (!value.is_empty())
            .then(|| Self(value.into()))
            .ok_or(InventoryError::REASON_REQUIRED)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationKind {
    StockEntry,
    PhysicalCount,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InventoryOperation {
    StockEntry {
        product_id: i64,
        request_id: RequestId,
        quantity: StockEntryQuantity,
        note: Option<String>,
    },
    PhysicalCount {
        product_id: i64,
        request_id: RequestId,
        count: PhysicalCount,
        reason: AdjustmentReason,
    },
}

impl InventoryOperation {
    pub fn stock_entry(
        product_id: i64,
        request_id: RequestId,
        quantity: i64,
        note: Option<String>,
    ) -> Result<Self, InventoryError> {
        Ok(Self::StockEntry {
            product_id,
            request_id,
            quantity: StockEntryQuantity::new(quantity)?,
            note,
        })
    }
    pub fn physical_count(
        product_id: i64,
        request_id: RequestId,
        count: i64,
        reason: &str,
    ) -> Result<Self, InventoryError> {
        Ok(Self::PhysicalCount {
            product_id,
            request_id,
            count: PhysicalCount::new(count)?,
            reason: AdjustmentReason::new(reason)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersistedInventoryOperation {
    pub kind: OperationKind,
    pub request_id: RequestId,
    pub product_id: i64,
    pub previous_quantity: i64,
    pub quantity_delta: i64,
    pub resulting_quantity: i64,
    pub occurred_at: String,
}

impl PersistedInventoryOperation {
    pub fn new(
        kind: OperationKind,
        request_id: RequestId,
        product_id: i64,
        previous_quantity: i64,
        quantity_delta: i64,
        resulting_quantity: i64,
        occurred_at: &str,
    ) -> Result<Self, InventoryError> {
        if previous_quantity < 0 || resulting_quantity < 0 {
            return Err(InventoryError::PERSISTED_DATA_INVALID);
        }
        if previous_quantity
            .checked_add(quantity_delta)
            .ok_or(InventoryError::QUANTITY_OVERFLOW)?
            != resulting_quantity
        {
            return Err(InventoryError::PERSISTED_DATA_INVALID);
        }
        if (kind == OperationKind::StockEntry && quantity_delta <= 0)
            || (kind == OperationKind::PhysicalCount && quantity_delta == 0)
        {
            return Err(if kind == OperationKind::PhysicalCount {
                InventoryError::UNCHANGED_COUNT
            } else {
                InventoryError::PERSISTED_DATA_INVALID
            });
        }
        Ok(Self {
            kind,
            request_id,
            product_id,
            previous_quantity,
            quantity_delta,
            resulting_quantity,
            occurred_at: occurred_at.into(),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertClassification {
    OutOfStock,
    LowStock,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryAlert {
    pub product_id: i64,
    pub product_name: String,
    pub quantity: i64,
    pub classification: AlertClassification,
}
impl InventoryAlert {
    pub fn for_product(
        product_id: i64,
        product_name: &str,
        active: bool,
        quantity: i64,
    ) -> Option<Self> {
        let classification = match (active, quantity) {
            (true, 0) => AlertClassification::OutOfStock,
            (true, 1) => AlertClassification::LowStock,
            _ => return None,
        };
        Some(Self {
            product_id,
            product_name: product_name.into(),
            quantity,
            classification,
        })
    }

    pub fn sort_key(&self) -> (AlertClassification, String, i64) {
        (
            self.classification,
            self.product_name.to_lowercase(),
            self.product_id,
        )
    }
}
