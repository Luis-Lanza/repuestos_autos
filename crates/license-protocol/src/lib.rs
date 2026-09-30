#![deny(missing_docs)]

//! Canonical, bounded vendor license signing and verification protocol.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Maximum serialized license envelope size.
pub const MAX_ENVELOPE_BYTES: usize = 4096;
/// Maximum decoded canonical payload size.
pub const MAX_PAYLOAD_BYTES: usize = 1024;
/// Product identity protected by protocol v1.
pub const PRODUCT_ID: &str = "com.repuestosautos.app";
/// Domain-separation prefix for signatures.
pub const SIGNED_MESSAGE_PREFIX: &[u8] = b"repuestos-autos-license\0v1\0";
const SIGNATURE_BYTES: usize = 64;
const LICENSE_ID_MAX: usize = 64;
const ENVELOPE_FORMAT: &str = "repuestos-autos-license";

/// Stable, non-secret protocol failure codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ProtocolError {
    /// Input exceeds a protocol size bound.
    #[error("input_too_large")]
    InputTooLarge,
    /// JSON is malformed, non-canonical, reordered, duplicated, or has unknown fields.
    #[error("malformed_envelope")]
    MalformedEnvelope,
    /// Encoded payload or signature is not canonical unpadded base64url.
    #[error("invalid_base64")]
    InvalidBase64,
    /// Payload does not match the v1 field contract.
    #[error("invalid_payload")]
    InvalidPayload,
    /// Protocol version is not supported.
    #[error("unsupported_version")]
    UnsupportedVersion,
    /// License belongs to a different product.
    #[error("wrong_product")]
    WrongProduct,
    /// Signature has an invalid byte length.
    #[error("invalid_signature_size")]
    InvalidSignatureSize,
    /// No trusted verification key matches the envelope key identifier.
    #[error("unknown_key")]
    UnknownKey,
    /// Signature verification failed.
    #[error("invalid_signature")]
    InvalidSignature,
    /// License is bound to another machine digest.
    #[error("machine_mismatch")]
    MachineMismatch,
}

/// Canonical v1 license payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LicensePayload {
    /// Protocol version, always `1`.
    pub version: u8,
    /// Vendor-assigned license identifier.
    pub license_id: String,
    /// Identifier of the vendor verification key.
    pub key_id: String,
    /// Product identity, always [`PRODUCT_ID`].
    pub product_id: String,
    /// Lowercase hexadecimal SHA-256 installation digest.
    pub machine_hash: String,
    /// Issuance instant in UTC RFC3339 seconds form.
    pub issued_at: String,
}

/// Signed license envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LicenseEnvelope {
    /// Envelope format discriminator.
    pub format: String,
    /// Unpadded base64url canonical payload bytes.
    pub payload: String,
    /// Unpadded base64url Ed25519 signature bytes.
    pub signature: String,
}

/// Successful machine-bound license verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedLicense {
    /// Validated license payload.
    pub payload: LicensePayload,
}

/// Return the stable key identifier for an Ed25519 public key.
pub fn key_id(public_key: &VerifyingKey) -> String {
    hex_lower(&Sha256::digest(public_key.as_bytes()))
}

/// Validate a license identifier or key identifier.
pub fn validate_identifier(value: &str) -> Result<(), ProtocolError> {
    if value.is_empty()
        || value.len() > LICENSE_ID_MAX
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(ProtocolError::InvalidPayload);
    }
    Ok(())
}

/// Validate a lowercase hexadecimal SHA-256 installation digest.
pub fn validate_machine_hash(value: &str) -> Result<(), ProtocolError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ProtocolError::InvalidPayload);
    }
    Ok(())
}

/// Validate UTC RFC3339 timestamp with second precision and `Z` timezone.
pub fn validate_issued_at(value: &str) -> Result<(), ProtocolError> {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
        || bytes.iter().enumerate().any(|(i, byte)| {
            !matches!(i, 4 | 7 | 10 | 13 | 16 | 19) && !byte.is_ascii_digit()
        })
    {
        return Err(ProtocolError::InvalidPayload);
    }
    let year = number(&bytes[0..4]);
    let month = number(&bytes[5..7]);
    let day = number(&bytes[8..10]);
    let hour = number(&bytes[11..13]);
    let minute = number(&bytes[14..16]);
    let second = number(&bytes[17..19]);
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    if year == 0 || day == 0 || day > days || hour > 23 || minute > 59 || second > 59 {
        return Err(ProtocolError::InvalidPayload);
    }
    Ok(())
}

/// Serialize a valid payload into exact canonical compact ASCII bytes.
pub fn canonical_payload(payload: &LicensePayload) -> Result<Vec<u8>, ProtocolError> {
    validate_payload(payload)?;
    serde_json::to_vec(payload).map_err(|_| ProtocolError::InvalidPayload)
}

/// Sign a payload using Ed25519 and return its canonical envelope JSON.
pub fn sign_payload(
    mut payload: LicensePayload,
    signing_key: &SigningKey,
) -> Result<Vec<u8>, ProtocolError> {
    let public_key = signing_key.verifying_key();
    payload.key_id = key_id(&public_key);
    let payload_bytes = canonical_payload(&payload)?;
    if payload_bytes.len() > MAX_PAYLOAD_BYTES {
        return Err(ProtocolError::InputTooLarge);
    }
    let mut message = Vec::with_capacity(SIGNED_MESSAGE_PREFIX.len() + payload_bytes.len());
    message.extend_from_slice(SIGNED_MESSAGE_PREFIX);
    message.extend_from_slice(&payload_bytes);
    let signature = signing_key.sign(&message);
    let envelope = LicenseEnvelope {
        format: ENVELOPE_FORMAT.to_owned(),
        payload: URL_SAFE_NO_PAD.encode(payload_bytes),
        signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
    };
    serde_json::to_vec(&envelope).map_err(|_| ProtocolError::MalformedEnvelope)
}

/// Decode, validate, verify, and machine-bind an envelope.
pub fn verify_envelope(
    bytes: &[u8],
    expected_machine_hash: &str,
    trusted_keys: &[VerifyingKey],
) -> Result<VerifiedLicense, ProtocolError> {
    if bytes.len() > MAX_ENVELOPE_BYTES {
        return Err(ProtocolError::InputTooLarge);
    }
    validate_machine_hash(expected_machine_hash)?;
    let envelope: LicenseEnvelope = serde_json::from_slice(bytes).map_err(|_| ProtocolError::MalformedEnvelope)?;
    if envelope.format != ENVELOPE_FORMAT {
        return Err(ProtocolError::MalformedEnvelope);
    }
    let canonical_envelope = serde_json::to_vec(&envelope).map_err(|_| ProtocolError::MalformedEnvelope)?;
    if canonical_envelope != bytes {
        return Err(ProtocolError::MalformedEnvelope);
    }
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(envelope.payload.as_bytes())
        .map_err(|_| ProtocolError::InvalidBase64)?;
    if URL_SAFE_NO_PAD.encode(&payload_bytes) != envelope.payload {
        return Err(ProtocolError::InvalidBase64);
    }
    if payload_bytes.len() > MAX_PAYLOAD_BYTES {
        return Err(ProtocolError::InputTooLarge);
    }
    let payload: LicensePayload = serde_json::from_slice(&payload_bytes).map_err(|_| ProtocolError::MalformedEnvelope)?;
    let canonical = canonical_payload(&payload)?;
    if canonical != payload_bytes {
        return Err(ProtocolError::MalformedEnvelope);
    }
    if payload.version != 1 {
        return Err(ProtocolError::UnsupportedVersion);
    }
    if payload.product_id != PRODUCT_ID {
        return Err(ProtocolError::WrongProduct);
    }
    if payload.machine_hash != expected_machine_hash {
        return Err(ProtocolError::MachineMismatch);
    }
    let signature_bytes = URL_SAFE_NO_PAD
        .decode(envelope.signature.as_bytes())
        .map_err(|_| ProtocolError::InvalidBase64)?;
    if URL_SAFE_NO_PAD.encode(&signature_bytes) != envelope.signature {
        return Err(ProtocolError::InvalidBase64);
    }
    if signature_bytes.len() != SIGNATURE_BYTES {
        return Err(ProtocolError::InvalidSignatureSize);
    }
    let signature = Signature::from_slice(&signature_bytes).map_err(|_| ProtocolError::InvalidSignatureSize)?;
    let mut message = Vec::with_capacity(SIGNED_MESSAGE_PREFIX.len() + payload_bytes.len());
    message.extend_from_slice(SIGNED_MESSAGE_PREFIX);
    message.extend_from_slice(&payload_bytes);
    let key = trusted_keys
        .iter()
        .find(|candidate| key_id(candidate) == payload.key_id)
        .ok_or(ProtocolError::UnknownKey)?;
    key.verify_strict(&message, &signature).map_err(|_| ProtocolError::InvalidSignature)?;
    Ok(VerifiedLicense { payload })
}

fn validate_payload(payload: &LicensePayload) -> Result<(), ProtocolError> {
    if payload.version != 1 {
        return Err(ProtocolError::UnsupportedVersion);
    }
    validate_identifier(&payload.license_id)?;
    validate_identifier(&payload.key_id)?;
    if payload.product_id != PRODUCT_ID {
        return Err(ProtocolError::WrongProduct);
    }
    validate_machine_hash(&payload.machine_hash)?;
    validate_issued_at(&payload.issued_at)
}

fn number(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0, |value, byte| value * 10 + u32::from(byte - b'0'))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
