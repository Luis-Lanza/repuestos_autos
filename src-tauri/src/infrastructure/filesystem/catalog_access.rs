use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "catalog-access.json";
const MAX_CONFIG_BYTES: u64 = 16 * 1024;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogAccessConfig {
    pub version: u8,
    pub password_hash: String,
    pub recovery_hash: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogAccessStorageError {
    Unavailable,
    Invalid,
}

#[derive(Clone)]
pub struct CatalogAccessStore {
    path: PathBuf,
}

#[cfg(windows)]
fn replace_file(source: &Path, destination: &Path) -> Result<(), CatalogAccessStorageError> {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
    use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH};
    let source: Vec<u16> = OsStr::new(source).encode_wide().chain(Some(0)).collect();
    let destination: Vec<u16> = OsStr::new(destination).encode_wide().chain(Some(0)).collect();
    let replaced = unsafe { MoveFileExW(source.as_ptr(), destination.as_ptr(), MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH) };
    if replaced == 0 { Err(CatalogAccessStorageError::Unavailable) } else { Ok(()) }
}

#[cfg(not(windows))]
fn replace_file(source: &Path, destination: &Path) -> Result<(), CatalogAccessStorageError> {
    fs::rename(source, destination).map_err(|_| CatalogAccessStorageError::Unavailable)
}

#[cfg(unix)]
fn sync_parent_directory(directory: &Path) -> Result<(), CatalogAccessStorageError> {
    fs::File::open(directory)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| CatalogAccessStorageError::Unavailable)
}

// MoveFileExW's MOVEFILE_WRITE_THROUGH requests durable completion on Windows; opening
// directory handles for FlushFileBuffers is not supported consistently across Windows versions.
#[cfg(windows)]
fn sync_parent_directory(_directory: &Path) -> Result<(), CatalogAccessStorageError> {
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn sync_parent_directory(_directory: &Path) -> Result<(), CatalogAccessStorageError> {
    Err(CatalogAccessStorageError::Unavailable)
}

impl CatalogAccessStore {
    pub fn new(app_data: &Path) -> Self {
        Self { path: app_data.join(FILE_NAME) }
    }

    pub fn load(&self) -> Result<Option<CatalogAccessConfig>, CatalogAccessStorageError> {
        let metadata = match fs::symlink_metadata(&self.path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(CatalogAccessStorageError::Unavailable),
        };
        if !metadata.file_type().is_file() || metadata.len() > MAX_CONFIG_BYTES {
            return Err(CatalogAccessStorageError::Invalid);
        }
        let bytes = fs::read(&self.path).map_err(|_| CatalogAccessStorageError::Unavailable)?;
        let config: CatalogAccessConfig = serde_json::from_slice(&bytes).map_err(|_| CatalogAccessStorageError::Invalid)?;
        if config.version != 1 || config.password_hash.len() > 2048 || config.recovery_hash.len() > 2048 {
            return Err(CatalogAccessStorageError::Invalid);
        }
        Ok(Some(config))
    }

    pub fn save(&self, config: &CatalogAccessConfig) -> Result<(), CatalogAccessStorageError> {
        let root = self.path.parent().ok_or(CatalogAccessStorageError::Unavailable)?;
        fs::create_dir_all(root).map_err(|_| CatalogAccessStorageError::Unavailable)?;
        let temporary = root.join(format!(".catalog-access-{}.tmp", uuid::Uuid::new_v4()));
        let bytes = serde_json::to_vec(config).map_err(|_| CatalogAccessStorageError::Unavailable)?;
        let result = (|| {
            let mut file = OpenOptions::new().write(true).create_new(true).open(&temporary)
                .map_err(|_| CatalogAccessStorageError::Unavailable)?;
            file.write_all(&bytes).map_err(|_| CatalogAccessStorageError::Unavailable)?;
            file.sync_all().map_err(|_| CatalogAccessStorageError::Unavailable)?;
            drop(file);
            replace_file(&temporary, &self.path)?;
            sync_parent_directory(root)?;
            Ok(())
        })();
        if result.is_err() { let _ = fs::remove_file(&temporary); }
        result
    }
}
