use sha2::{Digest, Sha256};

use crate::domain::RequestId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryError(&'static str);

impl InventoryError {
    pub const INVALID_REQUEST: Self = Self("invalid_request");
    pub const INVALID_QUANTITY: Self = Self("invalid_quantity");
    pub const INVALID_PRICE: Self = Self("invalid_price");
    pub const INVALID_COUNT: Self = Self("invalid_count");
    pub const REASON_REQUIRED: Self = Self("reason_required");
    pub const UNCHANGED_COUNT: Self = Self("unchanged_count");
    pub const QUANTITY_OVERFLOW: Self = Self("quantity_overflow");
    pub const MISSING_PRODUCT: Self = Self("missing_product");
    pub const INACTIVE_PRODUCT: Self = Self("inactive_product");
    pub const PERSISTED_DATA_INVALID: Self = Self("persisted_data_invalid");
    pub const PERSISTENCE_FAILURE: Self = Self("persistence_failure");
    pub const REQUEST_CONFLICT: Self = Self("request_conflict");
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
pub struct UnitPurchasePrice(i64);
impl UnitPurchasePrice {
    pub fn new(value: i64) -> Result<Self, InventoryError> {
        (value > 0 && value <= 9_007_199_254_740_991)
            .then_some(Self(value))
            .ok_or(InventoryError::INVALID_PRICE)
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

impl OperationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StockEntry => "stock_entry",
            Self::PhysicalCount => "physical_count",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryIdentity {
    operation_kind: String,
    payload_version: i64,
    canonical_payload: Vec<u8>,
    payload_sha256: String,
}

impl InventoryIdentity {
    pub const PAYLOAD_VERSION: i64 = 2;

    fn new(operation_kind: OperationKind, payload: Vec<u8>) -> Self {
        let payload_version = if payload.starts_with(b"12:inventory/v2") { 2 } else { 1 };
        Self {
            operation_kind: operation_kind.as_str().into(),
            payload_version,
            payload_sha256: format!("{:x}", Sha256::digest(&payload)),
            canonical_payload: payload,
        }
    }

    pub fn operation_kind(&self) -> &str {
        &self.operation_kind
    }

    pub fn payload_version(&self) -> i64 {
        self.payload_version
    }

    pub fn canonical_payload(&self) -> &[u8] {
        &self.canonical_payload
    }

    pub fn payload_sha256(&self) -> &str {
        &self.payload_sha256
    }

    pub fn from_persisted(
        operation_kind: Option<&str>,
        payload_version: Option<i64>,
        canonical_payload: Option<&[u8]>,
        payload_sha256: Option<&str>,
    ) -> Result<Option<Self>, InventoryError> {
        if operation_kind.is_none()
            && payload_version.is_none()
            && canonical_payload.is_none()
            && payload_sha256.is_none()
        {
            return Ok(None);
        }
        let (
            Some(operation_kind),
            Some(payload_version),
            Some(canonical_payload),
            Some(payload_sha256),
        ) = (
            operation_kind,
            payload_version,
            canonical_payload,
            payload_sha256,
        )
        else {
            return Err(InventoryError::PERSISTENCE_FAILURE);
        };
        let kind = match operation_kind {
            "stock_entry" => OperationKind::StockEntry,
            "physical_count" => OperationKind::PhysicalCount,
            _ => return Err(InventoryError::PERSISTENCE_FAILURE),
        };
        let Some(reencoded_payload) = canonical_payload_from_persisted(canonical_payload, kind)
        else {
            return Err(InventoryError::PERSISTENCE_FAILURE);
        };
        let version_marker_matches = match payload_version {
            1 => canonical_payload.starts_with(b"12:inventory/v1"),
            2 => canonical_payload.starts_with(b"12:inventory/v2"),
            _ => false,
        };
        if !version_marker_matches
            || payload_sha256.len() != 64
            || !payload_sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || canonical_payload != reencoded_payload.as_slice()
            || payload_sha256 != format!("{:x}", Sha256::digest(canonical_payload))
        {
            return Err(InventoryError::PERSISTENCE_FAILURE);
        }
        Ok(Some(Self {
            operation_kind: operation_kind.into(),
            payload_version,
            canonical_payload: canonical_payload.to_vec(),
            payload_sha256: payload_sha256.into(),
        }))
    }
}

fn canonical_payload_from_persisted(payload: &[u8], kind: OperationKind) -> Option<Vec<u8>> {
    let mut cursor = 0;
    let marker = read_field(payload, &mut cursor)?;
    let stored_kind = read_field(payload, &mut cursor)?;
    let product_id = read_field(payload, &mut cursor)?;
    let requested_value = read_field(payload, &mut cursor)?;
    let product_id = std::str::from_utf8(product_id).ok()?.parse::<i64>().ok()?;
    let requested_value = std::str::from_utf8(requested_value)
        .ok()?
        .parse::<i64>()
        .ok()?;
    if (marker != b"inventory/v1" && marker != b"inventory/v2") || stored_kind != kind.as_str().as_bytes() {
        return None;
    }

    let decoded = match kind {
        OperationKind::StockEntry => {
            if requested_value <= 0 {
                return None;
            }
            let (purchase, sale, minimum) = if marker == b"inventory/v2" {
                let purchase = read_number(payload, &mut cursor)?;
                if purchase <= 0 || purchase > 9_007_199_254_740_991 { return None; }
                let sale = read_optional_number(payload, &mut cursor)?;
                let minimum = read_optional_number(payload, &mut cursor)?;
                if sale.into_iter().chain(minimum).any(|price| price <= 0 || price > 9_007_199_254_740_991)
                    || matches!((sale, minimum), (Some(sale), Some(minimum)) if minimum > sale) { return None; }
                (Some(purchase), sale, minimum)
            } else { (None, None, None) };
            let note_state = read_field(payload, &mut cursor)?;
            let note = match note_state {
                b"null" => None,
                b"value" => Some(std::str::from_utf8(read_field(payload, &mut cursor)?).ok()?),
                _ => return None,
            };
            if cursor != payload.len() {
                return None;
            }
            InventoryPayload::StockEntry {
                product_id,
                quantity: requested_value,
                purchase,
                sale,
                minimum,
                note,
            }
        }
        OperationKind::PhysicalCount => {
            if requested_value < 0 {
                return None;
            }
            let reason = std::str::from_utf8(read_field(payload, &mut cursor)?).ok()?;
            if reason.is_empty() || reason != reason.trim() || cursor != payload.len() {
                return None;
            }
            if marker != b"inventory/v1" { return None; }
            InventoryPayload::PhysicalCount {
                product_id,
                count: requested_value,
                reason,
            }
        }
    };
    Some(encode_payload(decoded).1)
}

enum InventoryPayload<'a> {
    StockEntry {
        product_id: i64,
        quantity: i64,
        purchase: Option<i64>,
        sale: Option<i64>,
        minimum: Option<i64>,
        note: Option<&'a str>,
    },
    PhysicalCount {
        product_id: i64,
        count: i64,
        reason: &'a str,
    },
}

fn encode_payload(payload: InventoryPayload<'_>) -> (OperationKind, Vec<u8>) {
    let mut encoded = Vec::new();
    match payload {
        InventoryPayload::StockEntry {
            product_id,
            quantity,
            purchase,
            sale,
            minimum,
            note,
        } => {
            let version = if purchase.is_some() { 2 } else { 1 };
            append_field(&mut encoded, if version == 2 { b"inventory/v2" } else { b"inventory/v1" });
            append_field(&mut encoded, OperationKind::StockEntry.as_str().as_bytes());
            append_number(&mut encoded, product_id);
            append_number(&mut encoded, quantity);
            if let Some(purchase) = purchase {
                append_number(&mut encoded, purchase);
                append_optional_number(&mut encoded, sale);
                append_optional_number(&mut encoded, minimum);
            }
            append_nullable_text(&mut encoded, note);
            (OperationKind::StockEntry, encoded)
        }
        InventoryPayload::PhysicalCount {
            product_id,
            count,
            reason,
        } => {
            append_field(&mut encoded, b"inventory/v1");
            append_field(
                &mut encoded,
                OperationKind::PhysicalCount.as_str().as_bytes(),
            );
            append_number(&mut encoded, product_id);
            append_number(&mut encoded, count);
            append_field(&mut encoded, reason.as_bytes());
            (OperationKind::PhysicalCount, encoded)
        }
    }
}

fn read_number(payload: &[u8], cursor: &mut usize) -> Option<i64> {
    std::str::from_utf8(read_field(payload, cursor)?).ok()?.parse().ok()
}
fn read_optional_number(payload: &[u8], cursor: &mut usize) -> Option<Option<i64>> {
    match read_field(payload, cursor)? {
        b"null" => Some(None),
        b"value" => Some(Some(read_number(payload, cursor)?)),
        _ => None,
    }
}
fn append_optional_number(payload: &mut Vec<u8>, value: Option<i64>) {
    match value {
        Some(value) => { append_field(payload, b"value"); append_number(payload, value); }
        None => append_field(payload, b"null"),
    }
}

fn read_field<'a>(payload: &'a [u8], cursor: &mut usize) -> Option<&'a [u8]> {
    let start = *cursor;
    while *cursor < payload.len() && payload[*cursor].is_ascii_digit() {
        *cursor += 1;
    }
    if *cursor == start || *cursor >= payload.len() || payload[*cursor] != b':' {
        return None;
    }
    let length = std::str::from_utf8(&payload[start..*cursor])
        .ok()?
        .parse::<usize>()
        .ok()?;
    *cursor += 1;
    let end = (*cursor).checked_add(length)?;
    if end > payload.len() {
        return None;
    }
    let value = &payload[*cursor..end];
    *cursor = end;
    Some(value)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InventoryOperation {
    StockEntry {
        product_id: i64,
        request_id: RequestId,
        quantity: StockEntryQuantity,
        unit_purchase_price: UnitPurchasePrice,
        sale_price_centavos: Option<i64>,
        minimum_sale_price_centavos: Option<i64>,
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
    pub fn identity(&self) -> InventoryIdentity {
        let (kind, payload) = match self {
            Self::StockEntry {
                product_id,
                quantity,
                unit_purchase_price,
                sale_price_centavos,
                minimum_sale_price_centavos,
                note,
                ..
            } => encode_payload(InventoryPayload::StockEntry {
                product_id: *product_id,
                quantity: quantity.value(),
                purchase: Some(unit_purchase_price.value()),
                sale: *sale_price_centavos,
                minimum: *minimum_sale_price_centavos,
                note: note.as_deref(),
            }),
            Self::PhysicalCount {
                product_id,
                count,
                reason,
                ..
            } => encode_payload(InventoryPayload::PhysicalCount {
                product_id: *product_id,
                count: count.value(),
                reason: reason.as_str(),
            }),
        };
        InventoryIdentity::new(kind, payload)
    }

    pub fn stock_entry_with_prices(
        product_id: i64,
        request_id: RequestId,
        quantity: i64,
        unit_purchase_price_centavos: i64,
        sale_price_centavos: Option<i64>,
        minimum_sale_price_centavos: Option<i64>,
        note: Option<String>,
    ) -> Result<Self, InventoryError> {
        for price in [sale_price_centavos, minimum_sale_price_centavos].into_iter().flatten() {
            if price <= 0 || price > 9_007_199_254_740_991 { return Err(InventoryError::INVALID_PRICE); }
        }
        if matches!((sale_price_centavos, minimum_sale_price_centavos), (Some(sale), Some(minimum)) if minimum > sale) {
            return Err(InventoryError::INVALID_PRICE);
        }
        Ok(Self::StockEntry {
            product_id,
            request_id,
            quantity: StockEntryQuantity::new(quantity)?,
            unit_purchase_price: UnitPurchasePrice::new(unit_purchase_price_centavos)?,
            sale_price_centavos,
            minimum_sale_price_centavos,
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

fn append_field(payload: &mut Vec<u8>, value: &[u8]) {
    payload.extend_from_slice(value.len().to_string().as_bytes());
    payload.push(b':');
    payload.extend_from_slice(value);
}

fn append_number(payload: &mut Vec<u8>, value: i64) {
    append_field(payload, value.to_string().as_bytes());
}

fn append_nullable_text(payload: &mut Vec<u8>, value: Option<&str>) {
    match value {
        Some(value) => {
            append_field(payload, b"value");
            append_field(payload, value.as_bytes());
        }
        None => append_field(payload, b"null"),
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
    pub note: Option<String>,
}

impl PersistedInventoryOperation {
    #[expect(
        clippy::too_many_arguments,
        reason = "the constructor validates one complete persisted inventory operation"
    )]
    pub fn new(
        kind: OperationKind,
        request_id: RequestId,
        product_id: i64,
        previous_quantity: i64,
        quantity_delta: i64,
        resulting_quantity: i64,
        occurred_at: &str,
        note: Option<String>,
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
            note,
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
