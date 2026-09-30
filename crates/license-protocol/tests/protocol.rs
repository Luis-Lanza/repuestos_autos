use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use license_protocol::{
    canonical_payload, key_id, sign_payload, validate_issued_at, validate_machine_hash,
    LicenseEnvelope, LicensePayload, ProtocolError, SIGNED_MESSAGE_PREFIX, PRODUCT_ID,
};

const MACHINE: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn fixture() -> (LicensePayload, SigningKey) {
    let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
    (
        LicensePayload {
            version: 1,
            license_id: "license-01".into(),
            key_id: key_id(&signing_key.verifying_key()),
            product_id: PRODUCT_ID.into(),
            machine_hash: MACHINE.into(),
            issued_at: "2026-09-29T12:34:56Z".into(),
        },
        signing_key,
    )
}

fn verify(bytes: &[u8], hash: &str, key: &VerifyingKey) -> Result<license_protocol::VerifiedLicense, ProtocolError> {
    license_protocol::verify_envelope(bytes, hash, &[key.clone()])
}

#[test]
fn canonical_payload_has_fixed_ascii_bytes_and_field_order() {
    let (payload, _) = fixture();
    assert_eq!(
        canonical_payload(&payload).unwrap(),
        format!("{{\"version\":1,\"license_id\":\"license-01\",\"key_id\":\"{}\",\"product_id\":\"{}\",\"machine_hash\":\"{}\",\"issued_at\":\"2026-09-29T12:34:56Z\"}}", payload.key_id, PRODUCT_ID, MACHINE).as_bytes()
    );
}

#[test]
fn deterministic_signed_envelope_verifies() {
    let key = SigningKey::from_bytes(&[0x42_u8; 32]);
    let payload = LicensePayload {
        version: 1,
        license_id: "license-01".into(),
        key_id: key_id(&key.verifying_key()),
        product_id: PRODUCT_ID.into(),
        machine_hash: MACHINE.into(),
        issued_at: "2026-09-29T12:34:56Z".into(),
    };
    const EXPECTED_KEY_ID: &str = "3097e2dee2cb4a34b53840cdb705aed71067c36f68db0e0f559c3f3fa043315f";
    const EXPECTED_CANONICAL: &str = "{\"version\":1,\"license_id\":\"license-01\",\"key_id\":\"3097e2dee2cb4a34b53840cdb705aed71067c36f68db0e0f559c3f3fa043315f\",\"product_id\":\"com.repuestosautos.app\",\"machine_hash\":\"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\",\"issued_at\":\"2026-09-29T12:34:56Z\"}";
    const EXPECTED_SIGNATURE: &str = "DE2OAn-0ig6EcL4tCNhWK_tgbQogfhB38Q7wg3EYdTokZ_gKxOOwoAa1R8u8u7br4BT2e-hwz5b6FElBaPMMBg";
    const EXPECTED_ENVELOPE: &str = "{\"format\":\"repuestos-autos-license\",\"payload\":\"eyJ2ZXJzaW9uIjoxLCJsaWNlbnNlX2lkIjoibGljZW5zZS0wMSIsImtleV9pZCI6IjMwOTdlMmRlZTJjYjRhMzRiNTM4NDBjZGI3MDVhZWQ3MTA2N2MzNmY2OGRiMGUwZjU1OWMzZjNmYTA0MzMxNWYiLCJwcm9kdWN0X2lkIjoiY29tLnJlcHVlc3Rvc2F1dG9zLmFwcCIsIm1hY2hpbmVfaGFzaCI6IjAxMjM0NTY3ODlhYmNkZWYwMTIzNDU2Nzg5YWJjZGVmMDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWYiLCJpc3N1ZWRfYXQiOiIyMDI2LTA5LTI5VDEyOjM0OjU2WiJ9\",\"signature\":\"DE2OAn-0ig6EcL4tCNhWK_tgbQogfhB38Q7wg3EYdTokZ_gKxOOwoAa1R8u8u7br4BT2e-hwz5b6FElBaPMMBg\"}";

    assert_eq!(key_id(&key.verifying_key()), EXPECTED_KEY_ID);
    assert_eq!(canonical_payload(&payload).unwrap(), EXPECTED_CANONICAL.as_bytes());
    let envelope = sign_payload(payload, &key).unwrap();
    let decoded: LicenseEnvelope = serde_json::from_slice(&envelope).unwrap();
    assert_eq!(decoded.signature, EXPECTED_SIGNATURE);
    assert_eq!(envelope, EXPECTED_ENVELOPE.as_bytes());
    let verified = verify(&envelope, MACHINE, &key.verifying_key()).unwrap();
    assert_eq!(verified.payload.license_id, "license-01");
}

#[test]
fn altered_payload_and_signature_are_rejected() {
    let (payload, key) = fixture();
    let envelope = sign_payload(payload, &key).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&envelope).unwrap();
    let payload_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        value["payload"].as_str().unwrap(),
    )
    .unwrap();
    let changed_payload = String::from_utf8(payload_bytes)
        .unwrap()
        .replacen("license-01", "license-02", 1);
    value["payload"] = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        changed_payload,
    )
    .into();
    assert!(verify(&serde_json::to_vec(&value).unwrap(), MACHINE, &key.verifying_key()).is_err());

    let mut signature = base64::Engine::decode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        value["signature"].as_str().unwrap_or_default(),
    )
    .unwrap_or_else(|_| {
        let valid: serde_json::Value = serde_json::from_slice(&envelope).unwrap();
        base64::Engine::decode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            valid["signature"].as_str().unwrap(),
        )
        .unwrap()
    });
    signature[0] ^= 1;
    let mut valid: serde_json::Value = serde_json::from_slice(&envelope).unwrap();
    valid["signature"] = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        signature,
    )
    .into();
    assert_eq!(
        verify(&serde_json::to_vec(&valid).unwrap(), MACHINE, &key.verifying_key()).unwrap_err(),
        ProtocolError::InvalidSignature
    );
}

#[test]
fn rejects_noncanonical_ed25519_scalar_with_strict_verification() {
    let (payload, key) = fixture();
    let envelope = sign_payload(payload, &key).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&envelope).unwrap();
    let encoded_signature = value["signature"].as_str().unwrap();
    let mut signature = base64::Engine::decode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        encoded_signature,
    )
    .unwrap();

    // S + group_order is a non-canonical scalar representation of the same value.
    const GROUP_ORDER: [u8; 32] = [
        0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9,
        0xde, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x10,
    ];
    let mut carry = 0_u16;
    for (scalar_byte, order_byte) in signature[32..].iter_mut().zip(GROUP_ORDER) {
        let sum = u16::from(*scalar_byte) + u16::from(order_byte) + carry;
        *scalar_byte = sum as u8;
        carry = sum >> 8;
    }
    assert_eq!(carry, 0);

    let payload_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        value["payload"].as_str().unwrap(),
    )
    .unwrap();
    let mut message = SIGNED_MESSAGE_PREFIX.to_vec();
    message.extend_from_slice(&payload_bytes);
    let noncanonical = Signature::from_slice(&signature).unwrap();
    assert!(key
        .verifying_key()
        .verify_strict(&message, &noncanonical)
        .is_err());

    value["signature"] = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        signature,
    )
    .into();
    assert_eq!(
        verify(&serde_json::to_vec(&value).unwrap(), MACHINE, &key.verifying_key()).unwrap_err(),
        ProtocolError::InvalidSignature
    );
}

#[test]
fn wrong_product_and_machine_are_rejected() {
    let (mut payload, key) = fixture();
    payload.product_id = "other.product".into();
    let payload_bytes = serde_json::to_vec(&payload).unwrap();
    let mut message = SIGNED_MESSAGE_PREFIX.to_vec();
    message.extend_from_slice(&payload_bytes);
    let signature = key.sign(&message);
    let envelope = LicenseEnvelope {
        format: "repuestos-autos-license".into(),
        payload: base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, payload_bytes),
        signature: base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, signature.to_bytes()),
    };
    assert_eq!(
        verify(&serde_json::to_vec(&envelope).unwrap(), MACHINE, &key.verifying_key()).unwrap_err(),
        ProtocolError::WrongProduct
    );

    let (payload, key) = fixture();
    let envelope = sign_payload(payload, &key).unwrap();
    let another_machine = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    assert_eq!(verify(&envelope, another_machine, &key.verifying_key()).unwrap_err(), ProtocolError::MachineMismatch);
}

#[test]
fn unsupported_version_and_untrusted_key_are_rejected() {
    let (mut payload, key) = fixture();
    payload.version = 2;
    assert_eq!(sign_payload(payload, &key).unwrap_err(), ProtocolError::UnsupportedVersion);
    let (payload, key) = fixture();
    let envelope = sign_payload(payload, &key).unwrap();
    let other = SigningKey::from_bytes(&[8_u8; 32]);
    assert_eq!(verify(&envelope, MACHINE, &other.verifying_key()).unwrap_err(), ProtocolError::UnknownKey);
}

#[test]
fn duplicate_unknown_reordered_and_whitespace_payloads_are_rejected() {
    let (payload, key) = fixture();
    let envelope = sign_payload(payload, &key).unwrap();
    let encoded: serde_json::Value = serde_json::from_slice(&envelope).unwrap();
    let payload_b64 = encoded["payload"].as_str().unwrap();
    let signature = encoded["signature"].as_str().unwrap();
    let decoded = base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, payload_b64).unwrap();
    let json = String::from_utf8(decoded).unwrap();
    for bad in [
        json.replacen("\"version\":1,", "\"version\":1,\"version\":1,", 1),
        json.replacen("\"version\":1,", "\"unknown\":0,\"version\":1,", 1),
        json.replacen("\"version\":1,\"license_id\":\"license-01\"", "\"license_id\":\"license-01\",\"version\":1", 1),
        json.replacen(",\"license_id\":", ", \"license_id\":", 1),
    ] {
        let bad_payload = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, bad.as_bytes());
        let tampered = format!("{{\"format\":\"repuestos-autos-license\",\"payload\":\"{bad_payload}\",\"signature\":\"{signature}\"}}");
        assert!(verify(tampered.as_bytes(), MACHINE, &key.verifying_key()).is_err());
    }

    let duplicate_envelope = String::from_utf8(envelope.clone()).unwrap().replacen("{\"format\":", "{\"format\":\"repuestos-autos-license\",\"format\":", 1);
    assert_eq!(verify(duplicate_envelope.as_bytes(), MACHINE, &key.verifying_key()).unwrap_err(), ProtocolError::MalformedEnvelope);
    let unknown_envelope = String::from_utf8(envelope.clone()).unwrap().replacen("{\"format\":", "{\"extra\":0,\"format\":", 1);
    assert_eq!(verify(unknown_envelope.as_bytes(), MACHINE, &key.verifying_key()).unwrap_err(), ProtocolError::MalformedEnvelope);
    let original: serde_json::Value = serde_json::from_slice(&envelope).unwrap();
    let reordered_envelope = format!(
        "{{\"payload\":\"{}\",\"format\":\"repuestos-autos-license\",\"signature\":\"{}\"}}",
        original["payload"].as_str().unwrap(),
        original["signature"].as_str().unwrap()
    );
    assert_eq!(verify(reordered_envelope.as_bytes(), MACHINE, &key.verifying_key()).unwrap_err(), ProtocolError::MalformedEnvelope);
}

#[test]
fn malformed_base64_and_signature_lengths_are_rejected() {
    let (payload, key) = fixture();
    let envelope = sign_payload(payload, &key).unwrap();
    let text = String::from_utf8(envelope).unwrap();
    let bad_payload = text.replacen("\"payload\":\"", "\"payload\":\"!", 1);
    assert_eq!(verify(bad_payload.as_bytes(), MACHINE, &key.verifying_key()).unwrap_err(), ProtocolError::InvalidBase64);
    let short_sig = text.replacen("\"signature\":\"", "\"signature\":\"AA", 1);
    assert_eq!(verify(short_sig.as_bytes(), MACHINE, &key.verifying_key()).unwrap_err(), ProtocolError::InvalidSignatureSize);
    assert_eq!(verify(text.as_bytes(), "ABC", &key.verifying_key()).unwrap_err(), ProtocolError::InvalidPayload);
}

#[test]
fn installation_digest_and_issued_at_are_strictly_validated() {
    assert!(validate_machine_hash(MACHINE).is_ok());
    assert!(validate_machine_hash(&MACHINE.to_ascii_uppercase()).is_err());
    assert!(validate_machine_hash("0").is_err());
    assert!(validate_issued_at("2024-02-29T23:59:59Z").is_ok());
    assert!(validate_issued_at("2025-02-29T23:59:59Z").is_err());
    assert!(validate_issued_at("2024-01-01T00:00:00+00:00").is_err());
}

#[test]
fn signed_message_prefix_is_exact() {
    assert_eq!(SIGNED_MESSAGE_PREFIX, b"repuestos-autos-license\0v1\0");
}
