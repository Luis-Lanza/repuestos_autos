use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    infrastructure::{
        filesystem::{BackupStore, StorageError},
        sqlite::{create_snapshot, stage_and_validate},
    },
    DatabaseState,
};

pub const ALLOWED_COMMANDS: [&str; 5] = [
    "choose_backup_destination_command",
    "choose_restore_source_command",
    "create_backup_command",
    "prepare_restore_command",
    "confirm_restore_command",
];

const RESTORE_TOKEN_TTL_SECONDS: u64 = 900;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateBackupRequest {
    pub destination: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrepareRestoreRequest {
    pub source: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfirmRestoreRequest {
    pub token: String,
    pub confirmed: bool,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PathSelection {
    Selected { path: PathBuf },
    Cancelled,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BackupResponse {
    Created {
        path: PathBuf,
        created_at_unix_seconds: u64,
        size_bytes: u64,
        schema_version: i64,
    },
    Prepared {
        token: String,
        size_bytes: u64,
        schema_version: i64,
    },
    Restored,
    Error {
        code: &'static str,
        message: &'static str,
    },
}

impl BackupResponse {
    pub fn error(code: &'static str) -> Self {
        let message = match code {
            "database_unavailable" => "The database is unavailable.",
            "destination_exists" => "A backup already exists at that destination.",
            "invalid_backup" => "The selected backup is invalid.",
            "unsupported_schema" => "The selected backup schema is unsupported.",
            "confirmation_required" => "Restore confirmation is required.",
            "token_invalid" => "The restore confirmation is invalid.",
            "token_expired" => "The restore confirmation has expired.",
            "restore_failed" => "The restore could not be completed.",
            _ => "Backup storage is unavailable.",
        };
        Self::Error { code, message }
    }

    fn from_internal_code(code: &str) -> Self {
        match code {
            "database_unavailable" => Self::error("database_unavailable"),
            _ => Self::error("storage_unavailable"),
        }
    }
}

struct PendingRestore {
    stage: PathBuf,
    sha256: String,
    expires_at: u64,
}

pub struct BackupCommandState {
    root: PathBuf,
    pending: HashMap<String, PendingRestore>,
}

impl BackupCommandState {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            pending: HashMap::new(),
        }
    }
}

pub fn select_path(path: Option<PathBuf>) -> PathSelection {
    path.map_or(PathSelection::Cancelled, |path| PathSelection::Selected {
        path,
    })
}

#[cfg(feature = "desktop")]
pub async fn select_callback_path(
    open_picker: impl FnOnce(Box<dyn FnOnce(Option<PathBuf>) + Send>),
) -> PathSelection {
    let (sender, mut receiver) = tauri::async_runtime::channel(1);
    open_picker(Box::new(move |path| {
        let _ = sender.try_send(select_path(path));
    }));
    receiver.recv().await.unwrap_or(PathSelection::Cancelled)
}

pub fn create_backup(
    state: &DatabaseState,
    commands: &BackupCommandState,
    request: CreateBackupRequest,
) -> BackupResponse {
    let created_at_unix_seconds = now_seconds();
    let snapshot = commands.root.join("backup-restore/snapshots").join(format!(
        "{created_at_unix_seconds}-{}.sqlite3",
        uuid::Uuid::new_v4()
    ));
    let metadata = match state.with_read(|connection| {
        create_snapshot(connection, &snapshot).map_err(|_| "storage_unavailable".into())
    }) {
        Ok(metadata) => metadata,
        Err(code) => return BackupResponse::from_internal_code(&code),
    };
    let file_name = format!(
        "backup-{created_at_unix_seconds}-{}.sqlite3",
        uuid::Uuid::new_v4()
    );
    let published = BackupStore::new(&commands.root).publish_snapshot(
        &snapshot,
        &request.destination.join("backup-restore"),
        &file_name,
    );
    let _ = fs::remove_file(snapshot);
    match published {
        Ok(published) => BackupResponse::Created {
            path: published.path,
            created_at_unix_seconds,
            size_bytes: published.size_bytes,
            schema_version: metadata.schema_version,
        },
        Err(error) => BackupResponse::error(storage_error_code(error)),
    }
}

pub fn prepare_restore(
    state: &DatabaseState,
    commands: &mut BackupCommandState,
    request: PrepareRestoreRequest,
) -> BackupResponse {
    if let Err(code) = state.with_read(|_| Ok(())) {
        return BackupResponse::from_internal_code(&code);
    }
    let token = uuid::Uuid::new_v4().to_string();
    let stage = commands
        .root
        .join("backup-restore/staging")
        .join(format!("{token}.sqlite3"));
    let metadata = match stage_and_validate(&request.source, &stage) {
        Ok(metadata) => metadata,
        Err(crate::infrastructure::sqlite::BackupValidationError::InvalidBackup) => {
            return BackupResponse::error("invalid_backup")
        }
        Err(crate::infrastructure::sqlite::BackupValidationError::UnsupportedSchema) => {
            return BackupResponse::error("unsupported_schema")
        }
    };
    let Ok(size_bytes) = fs::metadata(&stage).map(|metadata| metadata.len()) else {
        return BackupResponse::error("storage_unavailable");
    };
    let Ok(sha256) = checksum(&stage) else {
        return BackupResponse::error("storage_unavailable");
    };
    commands.pending.insert(
        token.clone(),
        PendingRestore {
            stage,
            sha256,
            expires_at: now_seconds().saturating_add(RESTORE_TOKEN_TTL_SECONDS),
        },
    );
    BackupResponse::Prepared {
        token,
        size_bytes,
        schema_version: metadata.schema_version,
    }
}

pub fn confirm_restore(
    state: &DatabaseState,
    commands: &mut BackupCommandState,
    request: ConfirmRestoreRequest,
) -> BackupResponse {
    let Some(pending) = commands.pending.get(&request.token) else {
        return BackupResponse::error("token_invalid");
    };
    if now_seconds() >= pending.expires_at {
        commands.pending.remove(&request.token);
        return BackupResponse::error("token_expired");
    }
    if !request.confirmed {
        return BackupResponse::error("confirmation_required");
    }
    let Some(pending) = commands.pending.remove(&request.token) else {
        return BackupResponse::error("token_invalid");
    };
    if !matches!(checksum(&pending.stage), Ok(checksum) if checksum == pending.sha256) {
        return BackupResponse::error("invalid_backup");
    }
    match state.install_validated_stage(&pending.stage, &BackupStore::new(&commands.root)) {
        Ok(()) => BackupResponse::Restored,
        Err(code) if code == "database_unavailable" => {
            BackupResponse::error("database_unavailable")
        }
        Err(_) => BackupResponse::error("restore_failed"),
    }
}

fn storage_error_code(error: StorageError) -> &'static str {
    match error {
        StorageError::DestinationExists => "destination_exists",
        StorageError::SelectionCancelled | StorageError::StorageUnavailable => {
            "storage_unavailable"
        }
    }
}

fn checksum(path: &Path) -> std::io::Result<String> {
    use std::io::Read;

    let mut file = fs::File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            return Ok(format!("{:x}", digest.finalize()));
        }
        digest.update(&buffer[..count]);
    }
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}
