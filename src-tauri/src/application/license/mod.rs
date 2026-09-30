use license_protocol::{verify_envelope, ProtocolError, VerifiedLicense, MAX_ENVELOPE_BYTES, PRODUCT_ID};
use sha2::{Digest, Sha256};
use std::path::Path;

pub const PRODUCTION_KEY_ID: &str = "24cd3b399059804e5ea6cfc43ff785c364dc1593bf7783e676e8defd4daa995d";
const PUBLIC_KEY_HEX: &str = "e869bd20891d1bf5c56007619f4d6d49d5d678395742659f6c5f5d1c52703f02";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicenseState {
    Active,
    ActivationRequired,
    LicenseMissing,
    LicenseFileInvalid,
    LicenseUnsupportedVersion,
    LicenseUnknownKey,
    LicenseSignatureInvalid,
    LicenseWrongProduct,
    LicenseMachineMismatch,
    MachineIdentityUnavailable,
    LicenseStorageUnavailable,
}

impl LicenseState {
    pub fn code(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::ActivationRequired => "activation_required",
            Self::LicenseMissing => "license_missing",
            Self::LicenseFileInvalid => "license_file_invalid",
            Self::LicenseUnsupportedVersion => "license_unsupported_version",
            Self::LicenseUnknownKey => "license_unknown_key",
            Self::LicenseSignatureInvalid => "license_signature_invalid",
            Self::LicenseWrongProduct => "license_wrong_product",
            Self::LicenseMachineMismatch => "license_machine_mismatch",
            Self::MachineIdentityUnavailable => "machine_identity_unavailable",
            Self::LicenseStorageUnavailable => "license_storage_unavailable",
        }
    }
}

pub trait MachineIdentityProvider: Send + Sync {
    fn machine_guid(&self) -> Result<String, ()>;
}

pub trait LicenseStorage: Send + Sync {
    fn read(&self) -> Result<Option<Vec<u8>>, ()>;
    fn replace_atomically(&self, bytes: &[u8]) -> Result<(), ()>;
}

pub struct LicenseService<I, S> {
    identity: I,
    storage: S,
    trusted_keys: Vec<ed25519_dalek::VerifyingKey>,
}

impl<I: MachineIdentityProvider, S: LicenseStorage> LicenseService<I, S> {
    pub fn new(identity: I, storage: S) -> Self {
        Self { identity, storage, trusted_keys: vec![production_key()] }
    }

    #[cfg(test)]
    pub fn with_test_keys(identity: I, storage: S, trusted_keys: Vec<ed25519_dalek::VerifyingKey>) -> Self {
        Self { identity, storage, trusted_keys }
    }

    pub fn installation_code(&self) -> Result<String, LicenseState> {
        self.machine_hash().map_err(|_| LicenseState::MachineIdentityUnavailable)
    }

    pub fn status(&self) -> LicenseState {
        let machine_hash = match self.machine_hash() {
            Ok(hash) => hash,
            Err(()) => return LicenseState::MachineIdentityUnavailable,
        };
        let bytes = match self.storage.read() {
            Ok(Some(bytes)) => bytes,
            Ok(None) => return LicenseState::LicenseMissing,
            Err(()) => return LicenseState::LicenseStorageUnavailable,
        };
        match verify_with_keys(&bytes, &machine_hash, &self.trusted_keys) {
            Ok(_) => LicenseState::Active,
            Err(state) => state,
        }
    }

    pub fn import(&self, bytes: &[u8]) -> LicenseState {
        if bytes.len() > MAX_ENVELOPE_BYTES { return LicenseState::LicenseFileInvalid; }
        let machine_hash = match self.machine_hash() {
            Ok(hash) => hash,
            Err(()) => return LicenseState::MachineIdentityUnavailable,
        };
        if let Err(state) = verify_with_keys(bytes, &machine_hash, &self.trusted_keys) { return state; }
        if self.storage.replace_atomically(bytes).is_err() { return LicenseState::LicenseStorageUnavailable; }
        LicenseState::Active
    }

    fn machine_hash(&self) -> Result<String, ()> {
        let raw = self.identity.machine_guid()?;
        let canonical = normalize_machine_guid(&raw).ok_or(())?;
        Ok(derive_machine_hash(&canonical))
    }
}

pub fn normalize_machine_guid(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    if bytes.len() != 36 || bytes.iter().enumerate().any(|(index, byte)| {
        if [8, 13, 18, 23].contains(&index) { *byte != b'-' }
        else { !byte.is_ascii_hexdigit() }
    }) { return None; }
    Some(value.to_ascii_lowercase())
}

pub fn derive_machine_hash(canonical_guid: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(b"repuestos-autos-machine-binding\0v1\0");
    digest.update(PRODUCT_ID.as_bytes());
    digest.update(canonical_guid.as_bytes());
    digest.finalize().iter().map(|byte| format!("{byte:02x}")).collect()
}

fn production_key() -> ed25519_dalek::VerifyingKey {
    let key_bytes = decode_hex(PUBLIC_KEY_HEX).expect("embedded public key is valid hex");
    let key: [u8; 32] = key_bytes.try_into().expect("embedded public key is 32 bytes");
    let public_key = ed25519_dalek::VerifyingKey::from_bytes(&key).expect("embedded public key is valid");
    assert_eq!(license_protocol::key_id(&public_key), PRODUCTION_KEY_ID);
    public_key
}

fn verify_with_keys(bytes: &[u8], machine_hash: &str, keys: &[ed25519_dalek::VerifyingKey]) -> Result<VerifiedLicense, LicenseState> {
    verify_envelope(bytes, machine_hash, keys).map_err(map_protocol)
}

fn map_protocol(error: ProtocolError) -> LicenseState {
    match error {
        ProtocolError::UnsupportedVersion => LicenseState::LicenseUnsupportedVersion,
        ProtocolError::UnknownKey => LicenseState::LicenseUnknownKey,
        ProtocolError::InvalidSignature => LicenseState::LicenseSignatureInvalid,
        ProtocolError::WrongProduct => LicenseState::LicenseWrongProduct,
        ProtocolError::MachineMismatch => LicenseState::LicenseMachineMismatch,
        _ => LicenseState::LicenseFileInvalid,
    }
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    value.as_bytes().chunks_exact(2).map(|pair| {
        Some((hex(pair[0])? << 4) | hex(pair[1])?)
    }).collect()
}
fn hex(value: u8) -> Option<u8> { match value { b'0'..=b'9' => Some(value-b'0'), b'a'..=b'f' => Some(value-b'a'+10), _ => None } }

pub struct FileLicenseStorage { path: std::path::PathBuf }
impl FileLicenseStorage {
    pub fn new(app_data: &Path) -> Self { Self { path: app_data.join("license.lic") } }
}
impl LicenseStorage for FileLicenseStorage {
    fn read(&self) -> Result<Option<Vec<u8>>, ()> {
        use std::io::Read;
        let file = match std::fs::File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(()),
        };
        let mut bytes = Vec::new();
        file.take((MAX_ENVELOPE_BYTES + 1) as u64).read_to_end(&mut bytes).map_err(|_| ())?;
        Ok(Some(bytes))
    }
    fn replace_atomically(&self, bytes: &[u8]) -> Result<(), ()> {
        use std::io::Write;
        std::fs::create_dir_all(self.path.parent().ok_or(())?).map_err(|_| ())?;
        let temp = self.path.with_file_name(format!("license.{}.tmp", uuid::Uuid::new_v4()));
        let result = (|| {
            let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&temp).map_err(|_| ())?;
            file.write_all(bytes).map_err(|_| ())?;
            file.sync_all().map_err(|_| ())?;
            replace_file(&temp, &self.path)?;
            #[cfg(not(windows))]
            if let Some(parent) = self.path.parent() { std::fs::File::open(parent).and_then(|directory| directory.sync_all()).map_err(|_| ())?; }
            Ok(())
        })();
        if result.is_err() { let _ = std::fs::remove_file(temp); }
        result
    }
}

#[cfg(not(windows))]
fn replace_file(from: &Path, to: &Path) -> Result<(), ()> { std::fs::rename(from, to).map_err(|_| ()) }
#[cfg(windows)]
fn replace_file(from: &Path, to: &Path) -> Result<(), ()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH};
    let from: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let to: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    let ok = unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH) };
    if ok == 0 { Err(()) } else { Ok(()) }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
