use std::sync::Mutex;

use argon2::{Algorithm, Argon2, Params, Version};
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::{OsRng, RngCore}};

use crate::infrastructure::filesystem::catalog_access::{CatalogAccessConfig, CatalogAccessStorageError, CatalogAccessStore};

const MIN_PASSWORD_BYTES: usize = 8;
const MAX_SECRET_BYTES: usize = 1024;
const RECOVERY_CODE_BYTES: usize = 24;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogAccessError { Invalid, Unauthorized, AlreadyConfigured, NotConfigured, Storage, Corrupt, Pending }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogAccessStatus { SetupRequired, Locked, Unlocked, Unavailable }

struct PendingSetup { config: CatalogAccessConfig }
struct SessionInner {
    config: Option<CatalogAccessConfig>,
    pending: Option<PendingSetup>,
    unlocked: bool,
    unavailable: bool,
}

pub struct CatalogAccessSession {
    store: CatalogAccessStore,
    inner: Mutex<SessionInner>,
}

impl CatalogAccessSession {
    pub fn open(store: CatalogAccessStore) -> Self {
        let loaded = store.load();
        let (config, unavailable) = match loaded {
            Ok(config) => (config, false),
            Err(_) => (None, true),
        };
        Self { store, inner: Mutex::new(SessionInner { config, pending: None, unlocked: false, unavailable }) }
    }

    pub fn status(&self) -> CatalogAccessStatus {
        let Ok(inner) = self.inner.lock() else { return CatalogAccessStatus::Unavailable };
        if inner.unavailable { CatalogAccessStatus::Unavailable }
        else if inner.unlocked { CatalogAccessStatus::Unlocked }
        else if inner.config.is_some() { CatalogAccessStatus::Locked }
        else { CatalogAccessStatus::SetupRequired }
    }

    pub fn is_authorized(&self) -> bool { self.status() == CatalogAccessStatus::Unlocked }

    #[cfg(test)]
    pub fn set_test_authorized(&self, authorized: bool) {
        if let Ok(mut inner) = self.inner.lock() { inner.unlocked = authorized; }
    }

    pub fn begin_setup(&self, password: &str) -> Result<String, CatalogAccessError> {
        let mut inner = self.inner.lock().map_err(|_| CatalogAccessError::Storage)?;
        if inner.unavailable { return Err(CatalogAccessError::Storage); }
        if inner.config.is_some() { return Err(CatalogAccessError::AlreadyConfigured); }
        if inner.pending.is_some() { return Err(CatalogAccessError::Pending); }
        let password_hash = hash_secret(password)?;
        let recovery_code = new_recovery_code();
        let recovery_hash = hash_secret(&recovery_code)?;
        inner.pending = Some(PendingSetup { config: CatalogAccessConfig { version: 1, password_hash, recovery_hash } });
        Ok(recovery_code)
    }

    pub fn finish_setup(&self, confirmed: bool) -> Result<(), CatalogAccessError> {
        if !confirmed { return Err(CatalogAccessError::Invalid); }
        let mut inner = self.inner.lock().map_err(|_| CatalogAccessError::Storage)?;
        let pending = inner.pending.take().ok_or(CatalogAccessError::Pending)?;
        self.store.save(&pending.config).map_err(|_| CatalogAccessError::Storage)?;
        inner.config = Some(pending.config);
        inner.unlocked = true;
        Ok(())
    }

    pub fn unlock(&self, password: &str) -> Result<(), CatalogAccessError> {
        let mut inner = self.inner.lock().map_err(|_| CatalogAccessError::Storage)?;
        if inner.unavailable { return Err(CatalogAccessError::Storage); }
        let config = inner.config.as_ref().ok_or(CatalogAccessError::NotConfigured)?;
        if !verify_secret(password, &config.password_hash) { return Err(CatalogAccessError::Unauthorized); }
        inner.unlocked = true;
        Ok(())
    }

    pub fn change_password(&self, current: &str, replacement: &str) -> Result<(), CatalogAccessError> {
        let mut inner = self.inner.lock().map_err(|_| CatalogAccessError::Storage)?;
        if inner.unavailable { return Err(CatalogAccessError::Storage); }
        let config = inner.config.as_ref().ok_or(CatalogAccessError::NotConfigured)?;
        if !verify_secret(current, &config.password_hash) { return Err(CatalogAccessError::Unauthorized); }
        let replacement_hash = hash_secret(replacement)?;
        let updated = CatalogAccessConfig { version: 1, password_hash: replacement_hash, recovery_hash: config.recovery_hash.clone() };
        self.store.save(&updated).map_err(|_| CatalogAccessError::Storage)?;
        inner.config = Some(updated);
        inner.unlocked = true;
        Ok(())
    }

    pub fn begin_recovery(&self, recovery_code: &str, replacement: &str) -> Result<String, CatalogAccessError> {
        let mut inner = self.inner.lock().map_err(|_| CatalogAccessError::Storage)?;
        if inner.unavailable { return Err(CatalogAccessError::Storage); }
        if inner.pending.is_some() { return Err(CatalogAccessError::Pending); }
        let config = inner.config.as_ref().ok_or(CatalogAccessError::NotConfigured)?;
        if !verify_secret(recovery_code, &config.recovery_hash) { return Err(CatalogAccessError::Unauthorized); }
        let password_hash = hash_secret(replacement)?;
        let recovery_code = new_recovery_code();
        let recovery_hash = hash_secret(&recovery_code)?;
        inner.pending = Some(PendingSetup { config: CatalogAccessConfig { version: 1, password_hash, recovery_hash } });
        inner.unlocked = false;
        Ok(recovery_code)
    }

    pub fn finish_recovery(&self, confirmed: bool) -> Result<(), CatalogAccessError> {
        self.finish_setup(confirmed)
    }
}

fn hasher() -> Result<Argon2<'static>, CatalogAccessError> {
    let params = Params::new(19_456, 2, 1, None).map_err(|_| CatalogAccessError::Storage)?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

fn hash_secret(secret: &str) -> Result<String, CatalogAccessError> {
    if secret.len() < MIN_PASSWORD_BYTES || secret.len() > MAX_SECRET_BYTES { return Err(CatalogAccessError::Invalid); }
    let salt = SaltString::generate(&mut OsRng);
    hasher()?.hash_password(secret.as_bytes(), &salt).map(|hash| hash.to_string()).map_err(|_| CatalogAccessError::Storage)
}

fn verify_secret(secret: &str, encoded: &str) -> bool {
    if secret.len() < MIN_PASSWORD_BYTES || secret.len() > MAX_SECRET_BYTES { return false; }
    let Ok(hash) = PasswordHash::new(encoded) else { return false };
    hasher().is_ok_and(|argon| argon.verify_password(secret.as_bytes(), &hash).is_ok())
}

fn new_recovery_code() -> String {
    let mut bytes = [0_u8; RECOVERY_CODE_BYTES];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}

impl From<CatalogAccessStorageError> for CatalogAccessError {
    fn from(_: CatalogAccessStorageError) -> Self { Self::Storage }
}
