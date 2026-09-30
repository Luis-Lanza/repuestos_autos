use serde::Serialize;
use std::{path::PathBuf, sync::Arc};

use crate::application::license::{FileLicenseStorage, LicenseService, LicenseState};

pub type ProductionLicenseService = LicenseService<crate::infrastructure::windows_machine_identity::SystemMachineIdentity, FileLicenseStorage>;

#[derive(Clone)]
pub struct LicenseCommandState {
    pub service: Arc<ProductionLicenseService>,
    pub selected_file: Arc<std::sync::Mutex<Option<PathBuf>>>,
    #[cfg(test)]
    test_authorized: Arc<std::sync::atomic::AtomicBool>,
}

impl LicenseCommandState {
    pub fn new(service: ProductionLicenseService) -> Self {
        Self {
            service: Arc::new(service),
            selected_file: Arc::new(std::sync::Mutex::new(None)),
            #[cfg(test)]
            test_authorized: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn authorize_business_operation(&self) -> Result<(), &'static str> {
        #[cfg(test)]
        if self.test_authorized.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(());
        }
        if self.service.status() == LicenseState::Active {
            Ok(())
        } else {
            Err("license_required")
        }
    }

    #[cfg(test)]
    pub fn set_test_authorized(&self, authorized: bool) {
        self.test_authorized.store(authorized, std::sync::atomic::Ordering::Relaxed);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LicenseStatusResponse {
    Status { code: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InstallationCodeResponse {
    Code { code: String },
    Error { code: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LicenseFileSelection {
    Selected,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LicenseImportResponse {
    Imported { status: &'static str },
    Cancelled,
    Error { code: &'static str },
}

pub fn status(service: &ProductionLicenseService) -> LicenseStatusResponse {
    LicenseStatusResponse::Status { code: service.status().code() }
}

pub fn installation_code(service: &ProductionLicenseService) -> InstallationCodeResponse {
    match service.installation_code() {
        Ok(code) => InstallationCodeResponse::Code { code },
        Err(_) => InstallationCodeResponse::Error { code: "machine_identity_unavailable" },
    }
}

pub fn import_selected(service: &ProductionLicenseService, selected: Option<PathBuf>) -> LicenseImportResponse {
    let Some(path) = selected else { return LicenseImportResponse::Cancelled; };
    if !path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("lic")) {
        return LicenseImportResponse::Error { code: "license_file_invalid" };
    }
    use std::io::Read;
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return LicenseImportResponse::Error { code: "license_file_invalid" },
    };
    let mut bytes = Vec::new();
    if file.take((license_protocol::MAX_ENVELOPE_BYTES + 1) as u64).read_to_end(&mut bytes).is_err()
        || bytes.len() > license_protocol::MAX_ENVELOPE_BYTES
    { return LicenseImportResponse::Error { code: "license_file_invalid" }; }
    match service.import(&bytes) {
        LicenseState::Active => LicenseImportResponse::Imported { status: service.status().code() },
        state => LicenseImportResponse::Error { code: state.code() },
    }
}

#[cfg(test)]
#[path = "license_tests.rs"]
mod tests;
