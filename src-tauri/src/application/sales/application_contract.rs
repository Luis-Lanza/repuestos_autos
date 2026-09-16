use std::collections::HashSet;

use rusqlite::{Connection, Transaction};
use sha2::{Digest, Sha256};

use crate::domain::sales::{PaymentBreakdown, PaymentInput, Sale};
use crate::domain::{MoneyCentavos, Quantity, RequestId};

use super::{ConfirmSaleError, ConfirmSaleRepository, PersistedSaleSummary, Reservation};

pub struct ApplicationRequestedLine {
    pub product_id: i64,
    pub quantity: Quantity,
    pub captured_unit_price: MoneyCentavos,
    pub captured_revision: i64,
    /// Explicit negotiated-price route; `None` preserves the legacy behavior.
    pub final_unit_price: Option<MoneyCentavos>,
    pub acknowledged_price: Option<MoneyCentavos>,
    pub acknowledged_revision: Option<i64>,
}

pub struct ApplicationConfirmSaleRequest {
    pub request_id: RequestId,
    pub lines: Vec<ApplicationRequestedLine>,
    pub payment: PaymentInput,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaleIdentity {
    operation_kind: String,
    payload_version: i64,
    canonical_payload: Vec<u8>,
    payload_sha256: String,
}

impl SaleIdentity {
    pub fn from_request(request: &ApplicationConfirmSaleRequest) -> Self {
        let mut payload = Vec::new();
        let explicit_final_price = request.lines.iter().any(|line| line.final_unit_price.is_some());
            append_field(&mut payload, if explicit_final_price { b"confirm_sale/v2" } else { b"confirm_sale/v1" });
        append_field(&mut payload, request.lines.len().to_string().as_bytes());
        for line in &request.lines {
            append_number(&mut payload, line.product_id);
            append_number(&mut payload, line.quantity.value());
            append_number(&mut payload, line.captured_unit_price.value());
            append_number(&mut payload, line.captured_revision);
            if explicit_final_price {
                append_nullable_number(&mut payload, line.final_unit_price.map(MoneyCentavos::value));
            }
            append_nullable_number(
                &mut payload,
                line.acknowledged_price.map(MoneyCentavos::value),
            );
            append_nullable_number(&mut payload, line.acknowledged_revision);
        }
        append_nullable_number(
            &mut payload,
            request.payment.amount_tendered.map(MoneyCentavos::value),
        );
        append_nullable_number(
            &mut payload,
            request.payment.qr_applied.map(MoneyCentavos::value),
        );
        let digest = format!("{:x}", Sha256::digest(&payload));
        Self {
            operation_kind: "confirm_sale".into(),
            payload_version: if explicit_final_price { 2 } else { 1 },
            canonical_payload: payload,
            payload_sha256: digest,
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
}

fn append_field(payload: &mut Vec<u8>, value: &[u8]) {
    payload.extend_from_slice(value.len().to_string().as_bytes());
    payload.push(b':');
    payload.extend_from_slice(value);
}

fn append_number(payload: &mut Vec<u8>, value: i64) {
    append_field(payload, value.to_string().as_bytes());
}

fn append_nullable_number(payload: &mut Vec<u8>, value: Option<i64>) {
    match value {
        Some(value) => {
            append_field(payload, b"value");
            append_number(payload, value);
        }
        None => append_field(payload, b"null"),
    }
}

pub struct ConfirmSaleUseCase<'connection, 'repository, R> {
    connection: &'connection mut Connection,
    repository: &'repository R,
}

impl<'connection, 'repository, R: ConfirmSaleRepository>
    ConfirmSaleUseCase<'connection, 'repository, R>
{
    pub fn new(connection: &'connection mut Connection, repository: &'repository R) -> Self {
        Self {
            connection,
            repository,
        }
    }

    pub fn confirm(
        self,
        request: ApplicationConfirmSaleRequest,
    ) -> Result<PersistedSaleSummary, ConfirmSaleError> {
        let repository = self.repository;
        let identity = SaleIdentity::from_request(&request);
        let transaction = self
            .connection
            .transaction()
            .map_err(|_| ConfirmSaleError::Persistence)?;
        match Self::confirm_in_transaction(repository, &transaction, request, &identity) {
            Ok(summary) => transaction
                .commit()
                .map(|_| summary)
                .map_err(|_| ConfirmSaleError::Persistence),
            Err(error) => transaction
                .rollback()
                .map(|_| Err(error))
                .map_err(|_| ConfirmSaleError::Persistence)?,
        }
    }

    fn confirm_in_transaction(
        repository: &R,
        transaction: &Transaction<'_>,
        request: ApplicationConfirmSaleRequest,
        identity: &SaleIdentity,
    ) -> Result<PersistedSaleSummary, ConfirmSaleError> {
        match repository.reserve_or_load(
            transaction,
            &request.request_id,
            identity.operation_kind(),
            identity.payload_version(),
            identity.canonical_payload(),
            identity.payload_sha256(),
        )? {
            Reservation::ExistingConfirmed {
                summary,
                operation_kind,
                payload_version,
                canonical_payload,
                payload_sha256,
            } => {
                validate_existing_identity(
                    identity,
                    &operation_kind,
                    payload_version,
                    canonical_payload.as_deref(),
                    payload_sha256.as_deref(),
                )?;
                return summary.ok_or(ConfirmSaleError::PersistedDataInvalid);
            }
            Reservation::ExistingIncomplete {
                operation_kind,
                payload_version,
                canonical_payload,
                payload_sha256,
            }
            | Reservation::ExistingCorrupt {
                operation_kind,
                payload_version,
                canonical_payload,
                payload_sha256,
            } => {
                validate_existing_identity(
                    identity,
                    &operation_kind,
                    payload_version,
                    canonical_payload.as_deref(),
                    payload_sha256.as_deref(),
                )?;
                return Err(ConfirmSaleError::Persistence);
            }
            Reservation::Reserved => (),
        }
        reject_duplicate_products(&request.lines)?;
        let lines = repository.resolve_lines(transaction, &request.lines)?;
        let total = lines.iter().try_fold(
            MoneyCentavos::new(0).map_err(|_| ConfirmSaleError::PersistedDataInvalid)?,
            |sum, line| {
                sum.checked_add(line.total())
                    .map_err(|_| ConfirmSaleError::MoneyOverflow)
            },
        )?;
        let payments = PaymentBreakdown::derive(total, request.payment)?;
        let sale = Sale::new(lines, payments.payments().to_vec())
            .map_err(|_| ConfirmSaleError::PersistedDataInvalid)?;
        repository.persist_confirmed(transaction, &request.request_id, &sale)
    }
}

fn validate_existing_identity(
    expected: &SaleIdentity,
    operation_kind: &Option<String>,
    payload_version: Option<i64>,
    canonical_payload: Option<&[u8]>,
    payload_sha256: Option<&str>,
) -> Result<(), ConfirmSaleError> {
    if operation_kind.is_none()
        && payload_version.is_none()
        && canonical_payload.is_none()
        && payload_sha256.is_none()
    {
        return Err(ConfirmSaleError::RequestConflict);
    }

    let (
        Some(operation_kind),
        Some(payload_version),
        Some(canonical_payload),
        Some(payload_sha256),
    ) = (
        operation_kind.as_deref(),
        payload_version,
        canonical_payload,
        payload_sha256,
    )
    else {
        return Err(ConfirmSaleError::PersistedDataInvalid);
    };

    if operation_kind != expected.operation_kind()
        || payload_version != expected.payload_version()
        || payload_sha256.len() != 64
        || payload_sha256 != format!("{:x}", Sha256::digest(canonical_payload))
        || !valid_confirm_sale_payload(canonical_payload)
    {
        return Err(ConfirmSaleError::PersistedDataInvalid);
    }
    if canonical_payload != expected.canonical_payload()
        || payload_sha256 != expected.payload_sha256()
    {
        return Err(ConfirmSaleError::RequestConflict);
    }
    Ok(())
}

fn valid_confirm_sale_payload(payload: &[u8]) -> bool {
    let mut offset = 0;
    let version = match next_field(payload, &mut offset) {
        Some(b"confirm_sale/v1") => 1,
        Some(b"confirm_sale/v2") => 2,
        _ => return false,
    };
    let Some(line_count) = next_field(payload, &mut offset).and_then(parse_usize) else {
        return false;
    };
    if line_count == 0 {
        return false;
    }
    for _ in 0..line_count {
        for _ in 0..4 {
            if next_field(payload, &mut offset)
                .and_then(parse_i64)
                .is_none()
            {
                return false;
            }
        }
        if version == 2 && !next_positive_nullable_number(payload, &mut offset) {
            return false;
        }
        if !next_nullable_number(payload, &mut offset)
            || !next_nullable_number(payload, &mut offset)
        {
            return false;
        }
    }
    next_nullable_number(payload, &mut offset)
        && next_nullable_number(payload, &mut offset)
        && offset == payload.len()
}

fn next_positive_nullable_number(payload: &[u8], offset: &mut usize) -> bool {
    match next_field(payload, offset) {
        Some(b"null") => true,
        Some(b"value") => next_field(payload, offset)
            .and_then(parse_i64)
            .is_some_and(|value| value > 0),
        _ => false,
    }
}

fn next_nullable_number(payload: &[u8], offset: &mut usize) -> bool {
    match next_field(payload, offset) {
        Some(b"null") => true,
        Some(b"value") => next_field(payload, offset).and_then(parse_i64).is_some(),
        _ => false,
    }
}

fn next_field<'payload>(payload: &'payload [u8], offset: &mut usize) -> Option<&'payload [u8]> {
    let delimiter = payload
        .get(*offset..)?
        .iter()
        .position(|byte| *byte == b':')?;
    let length_end = offset.checked_add(delimiter)?;
    let length_bytes = payload.get(*offset..length_end)?;
    let length = std::str::from_utf8(length_bytes)
        .ok()?
        .parse::<usize>()
        .ok()?;
    if length.to_string().as_bytes() != length_bytes {
        return None;
    }
    let value_start = length_end.checked_add(1)?;
    let value_end = value_start.checked_add(length)?;
    let value = payload.get(value_start..value_end)?;
    *offset = value_end;
    Some(value)
}

fn parse_i64(value: &[u8]) -> Option<i64> {
    let parsed = std::str::from_utf8(value).ok()?.parse::<i64>().ok()?;
    (parsed.to_string().as_bytes() == value).then_some(parsed)
}

fn parse_usize(value: &[u8]) -> Option<usize> {
    let parsed = parse_i64(value)?;
    usize::try_from(parsed).ok()
}

fn reject_duplicate_products(lines: &[ApplicationRequestedLine]) -> Result<(), ConfirmSaleError> {
    let mut product_ids = HashSet::with_capacity(lines.len());
    if lines
        .iter()
        .any(|line| !product_ids.insert(line.product_id))
    {
        return Err(ConfirmSaleError::DuplicateProduct);
    }
    Ok(())
}
