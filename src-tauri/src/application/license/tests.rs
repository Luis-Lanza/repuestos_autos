use super::*;
use ed25519_dalek::{Signer, SigningKey};
use license_protocol::{sign_payload, LicensePayload, SIGNED_MESSAGE_PREFIX};
use serde::Serialize;
use std::{sync::{Arc, Mutex}, fs};

const GUID: &str = "550E8400-E29B-41D4-A716-446655440000";
struct Identity(Result<String, ()>);
impl MachineIdentityProvider for Identity { fn machine_guid(&self) -> Result<String, ()> { self.0.clone() } }
#[derive(Clone, Default)]
struct MemoryStorage(Arc<Mutex<Option<Vec<u8>>>>);
impl LicenseStorage for MemoryStorage {
    fn read(&self) -> Result<Option<Vec<u8>>, ()> { Ok(self.0.lock().unwrap().clone()) }
    fn replace_atomically(&self, bytes: &[u8]) -> Result<(), ()> { *self.0.lock().unwrap() = Some(bytes.to_vec()); Ok(()) }
}
fn keys(seed: u8) -> SigningKey { SigningKey::from_bytes(&[seed; 32]) }
fn license(machine_hash: &str, key: &SigningKey) -> Vec<u8> {
    sign_payload(LicensePayload { version: 1, license_id: "test-license".into(), key_id: String::new(), product_id: PRODUCT_ID.into(), machine_hash: machine_hash.into(), issued_at: "2026-09-29T00:00:00Z".into() }, key).unwrap()
}
fn service(storage: MemoryStorage, key: &SigningKey) -> LicenseService<Identity, MemoryStorage> {
    LicenseService::with_test_keys(Identity(Ok(GUID.into())), storage, vec![key.verifying_key()])
}

#[test]
fn embedded_production_public_key_matches_its_key_id() {
    let public_key = production_key();
    assert_eq!(license_protocol::key_id(&public_key), PRODUCTION_KEY_ID);
}

#[test]
fn normalizes_guid_and_matches_domain_separated_hash_vector() {
    let canonical = "550e8400-e29b-41d4-a716-446655440000";
    assert_eq!(normalize_machine_guid(GUID).as_deref(), Some(canonical));
    assert_eq!(derive_machine_hash(canonical), "bea70f837f88f40ba27fb9174da639aaac1576743bc153cfd656679b2f6093ef");
    for invalid in ["550e8400e29b41d4a716446655440000", "{550e8400-e29b-41d4-a716-446655440000}", "550e8400-e29b-41d4-a716-44665544000Z"] { assert!(normalize_machine_guid(invalid).is_none()); }
}

#[test]
fn verifies_valid_and_reports_missing_and_tampered_files() {
    let key = keys(7);
    let storage = MemoryStorage::default();
    let service = service(storage.clone(), &key);
    assert_eq!(service.status(), LicenseState::LicenseMissing);
    let code = service.installation_code().unwrap();
    let bytes = license(&code, &key);
    assert_eq!(service.import(&bytes), LicenseState::Active);
    assert_eq!(service.status(), LicenseState::Active);
    let mut tampered = bytes.clone();
    tampered[10] ^= 1;
    assert_eq!(service.import(&tampered), LicenseState::LicenseFileInvalid);
    assert_eq!(service.status(), LicenseState::Active);
}

#[test]
fn distinguishes_machine_mismatch_and_unknown_key() {
    let key = keys(7);
    let storage = MemoryStorage::default();
    let service = service(storage, &key);
    assert_eq!(service.import(&license(&"0".repeat(64), &key)), LicenseState::LicenseMachineMismatch);
    let other = keys(8);
    assert_eq!(service.import(&license(&service.installation_code().unwrap(), &other)), LicenseState::LicenseUnknownKey);
}

#[derive(Serialize)]
struct Envelope<'a> { format: &'static str, payload: String, signature: String, #[serde(skip)] _life: std::marker::PhantomData<&'a ()> }
fn signed_arbitrary(payload: &[u8], key: &SigningKey) -> Vec<u8> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    let mut message = SIGNED_MESSAGE_PREFIX.to_vec(); message.extend_from_slice(payload);
    serde_json::to_vec(&Envelope { format: "repuestos-autos-license", payload: URL_SAFE_NO_PAD.encode(payload), signature: URL_SAFE_NO_PAD.encode(key.sign(&message).to_bytes()), _life: std::marker::PhantomData }).unwrap()
}

#[test]
fn rejects_wrong_product_and_preserves_existing_license_on_failed_import() {
    let key = keys(7);
    let storage = MemoryStorage::default();
    let service = service(storage.clone(), &key);
    let valid = license(&service.installation_code().unwrap(), &key);
    assert_eq!(service.import(&valid), LicenseState::Active);
    let prior = storage.read().unwrap().unwrap();
    let payload = LicensePayload { version: 1, license_id: "wrong-product".into(), key_id: license_protocol::key_id(&key.verifying_key()), product_id: "other.product".into(), machine_hash: service.installation_code().unwrap(), issued_at: "2026-09-29T00:00:00Z".into() };
    let wrong_product = signed_arbitrary(&serde_json::to_vec(&payload).unwrap(), &key);
    assert_eq!(service.import(&wrong_product), LicenseState::LicenseWrongProduct);
    assert_eq!(service.import(b"not a license"), LicenseState::LicenseFileInvalid);
    assert_eq!(storage.read().unwrap().unwrap(), prior);
}

#[test]
fn file_storage_writes_the_license_filename_atomically() {
    let root = std::env::temp_dir().join(format!("license-storage-{}-{}", std::process::id(), uuid::Uuid::new_v4()));
    let storage = FileLicenseStorage::new(&root);
    storage.replace_atomically(b"first").unwrap();
    storage.replace_atomically(b"second").unwrap();
    assert_eq!(fs::read(root.join("license.lic")).unwrap(), b"second");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}
