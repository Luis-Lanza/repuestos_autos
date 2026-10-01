use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    infrastructure::{
        filesystem::{BackupStore, PublishedBackup, StorageError},
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

const RESTORE_TOKEN_TTL: Duration = Duration::from_secs(900);
const PICKER_TOKEN_TTL: Duration = Duration::from_secs(300);
const MAX_PENDING_RESTORES: usize = 8;
const MAX_STAGE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_SNAPSHOTS: usize = 8;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateBackupRequest {
    pub destination_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrepareRestoreRequest {
    pub source_token: String,
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
pub enum RestoreSourceSelection {
    Selected { token: String },
    Cancelled,
    Error { code: &'static str, message: &'static str },
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BackupDestinationSelection {
    Selected { token: String },
    Cancelled,
    Error { code: &'static str, message: &'static str },
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BackupResponse {
    Created {
        file_name: String,
        created_at_unix_seconds: u64,
        size_bytes: u64,
        schema_version: i64,
        durability_warning: bool,
        cleanup_warning: bool,
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
            "license_required" => "A valid license is required to restore a backup.",
            "database_unavailable" => "The database is unavailable.",
            "destination_exists" => "A backup already exists at that destination.",
            "unsupported_destination" => "Choose a fixed NTFS drive for backups.",
            "destination_token_invalid" => "The backup destination selection is invalid. Choose it again.",
            "destination_token_expired" => "The backup destination selection expired. Choose it again.",
            "source_token_invalid" => "The restore source selection is invalid. Choose it again.",
            "source_token_expired" => "The restore source selection expired. Choose it again.",
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

    fn message(&self) -> &'static str {
        match self {
            Self::Error { message, .. } => message,
            _ => "Backup storage is unavailable.",
        }
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
    expires_at: Instant,
}

struct PendingDestination {
    path: PathBuf,
    expires_at: Instant,
}

struct PendingSource {
    path: PathBuf,
    expires_at: Instant,
}

pub struct BackupCommandState {
    root: PathBuf,
    pending: HashMap<String, PendingRestore>,
    destinations: HashMap<String, PendingDestination>,
    sources: HashMap<String, PendingSource>,
}

impl BackupCommandState {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            pending: HashMap::new(),
            destinations: HashMap::new(),
            sources: HashMap::new(),
        }
    }

    pub fn select_backup_destination(&mut self, path: PathBuf) -> BackupDestinationSelection {
        if !self.prune_expired() { return BackupDestinationSelection::Error { code: "storage_unavailable", message: BackupResponse::error("storage_unavailable").message() }; }
        if BackupStore::validate_destination(&path).is_err() {
            return BackupDestinationSelection::Error {
                code: "unsupported_destination",
                message: BackupResponse::error("unsupported_destination").message(),
            };
        }
        let token = uuid::Uuid::new_v4().to_string();
        self.destinations.insert(token.clone(), PendingDestination {
            path,
            expires_at: Instant::now() + PICKER_TOKEN_TTL,
        });
        BackupDestinationSelection::Selected { token }
    }

    fn consume_destination(&mut self, token: &str) -> Result<PathBuf, &'static str> {
        let Some(pending) = self.destinations.remove(token) else {
            return Err("destination_token_invalid");
        };
        if Instant::now() >= pending.expires_at {
            return Err("destination_token_expired");
        }
        Ok(pending.path)
    }

    pub fn select_restore_source(&mut self, path: PathBuf) -> RestoreSourceSelection {
        if !self.prune_expired() { return RestoreSourceSelection::Error { code: "storage_unavailable", message: BackupResponse::error("storage_unavailable").message() }; }
        let token = uuid::Uuid::new_v4().to_string();
        self.sources.insert(token.clone(), PendingSource {
            path,
            expires_at: Instant::now() + PICKER_TOKEN_TTL,
        });
        RestoreSourceSelection::Selected { token }
    }

    fn consume_source(&mut self, token: &str) -> Result<PathBuf, &'static str> {
        let Some(pending) = self.sources.remove(token) else {
            return Err("source_token_invalid");
        };
        if Instant::now() >= pending.expires_at {
            return Err("source_token_expired");
        }
        Ok(pending.path)
    }

    fn prune_expired(&mut self) -> bool {
        self.prune_expired_using(&mut RealStageCleanup)
    }

    fn prune_expired_using(&mut self, cleanup: &mut impl StageCleanup) -> bool {
        let now = Instant::now();
        let expired: Vec<_> = self.pending.iter().filter_map(|(token, pending)| (pending.expires_at <= now).then_some(token.clone())).collect();
        let mut success = true;
        for token in expired {
            let removable = self.pending.get(&token).is_some_and(|pending| remove_stage_with_evidence_using(&pending.stage, cleanup));
            if removable { self.pending.remove(&token); } else { success = false; }
        }
        self.destinations.retain(|_, pending| pending.expires_at > now);
        self.sources.retain(|_, pending| pending.expires_at > now);
        success
    }

    #[cfg(test)]
    fn issue_test_destination(&mut self, path: PathBuf, expires_at: Instant) -> String {
        let token = uuid::Uuid::new_v4().to_string();
        self.destinations.insert(token.clone(), PendingDestination { path, expires_at });
        token
    }

    #[cfg(test)]
    fn issue_test_source(&mut self, path: PathBuf, expires_at: Instant) -> String {
        let token = uuid::Uuid::new_v4().to_string();
        self.sources.insert(token.clone(), PendingSource { path, expires_at });
        token
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
    commands: &mut BackupCommandState,
    request: CreateBackupRequest,
) -> BackupResponse {
    create_backup_using(state, commands, request, &mut RealStageCleanup)
}

fn create_backup_using(
    state: &DatabaseState,
    commands: &mut BackupCommandState,
    request: CreateBackupRequest,
    cleanup: &mut impl StageCleanup,
) -> BackupResponse {
    let backup_root = commands.root.clone();
    create_backup_with_publisher(state, commands, request, cleanup, |snapshot, destination, file_name| {
        BackupStore::new(&backup_root).publish_snapshot(snapshot, destination, file_name)
    })
}

fn create_backup_with_publisher(
    state: &DatabaseState,
    commands: &mut BackupCommandState,
    request: CreateBackupRequest,
    cleanup: &mut impl StageCleanup,
    mut publish: impl FnMut(&Path, &Path, &str) -> Result<PublishedBackup, StorageError>,
) -> BackupResponse {
    if !commands.prune_expired_using(cleanup) {
        report_create_backup_gate("expired_prune", "failed", "storage_unavailable");
        return BackupResponse::error("storage_unavailable");
    }
    let destination = match commands.consume_destination(&request.destination_token) {
        Ok(destination) => destination,
        Err(code) => {
            report_create_backup_gate("destination_token", "failed", destination_token_class(code));
            return BackupResponse::error(code);
        }
    };
    let created_at_unix_seconds = now_seconds();
    let snapshot_directory = commands.root.join("backup-restore/snapshots");
    if !reconcile_cleanup_evidence_using(&snapshot_directory, cleanup)
        || !prune_artifacts_using(&snapshot_directory, true, MAX_SNAPSHOTS, Duration::from_secs(7 * 24 * 60 * 60), &[], cleanup)
    {
        report_create_backup_gate("snapshot_prune", "failed", "storage_unavailable");
        return BackupResponse::error("storage_unavailable");
    }
    let snapshot = snapshot_directory.join(format!(
        "{created_at_unix_seconds}-{}.sqlite3",
        uuid::Uuid::new_v4()
    ));
    let metadata = match state.with_read(|connection| {
        let pages: u64 = connection.query_row("PRAGMA page_count", [], |row| row.get(0)).map_err(|error| {
            report_backup_sqlite_failure("snapshot_page_count", &error);
            "storage_unavailable".to_string()
        })?;
        let page_size: u64 = connection.query_row("PRAGMA page_size", [], |row| row.get(0)).map_err(|error| {
            report_backup_sqlite_failure("snapshot_page_size", &error);
            "storage_unavailable".to_string()
        })?;
        if pages.checked_mul(page_size).is_none_or(|size| size > MAX_STAGE_BYTES) {
            report_create_backup_gate("snapshot_size", "rejected", "limit_exceeded");
            return Err("storage_unavailable".into());
        }
        create_snapshot(connection, &snapshot).map_err(|_| {
            report_create_backup_gate("snapshot_result", "failed", "storage_unavailable");
            "storage_unavailable".into()
        })
    }) {
        Ok(metadata) => metadata,
        Err(code) => {
            report_create_backup_gate("state_read_snapshot", "failed", internal_backup_result_class(&code));
            if !remove_stage_with_evidence_using(&snapshot, cleanup) {
                report_create_backup_gate("final_cleanup", "failed", "unaccounted_failure");
                return BackupResponse::error("storage_unavailable");
            }
            report_create_backup_gate("final_cleanup", "completed", "cleaned");
            return BackupResponse::from_internal_code(&code);
        }
    };
    let file_name = format!(
        "backup-{created_at_unix_seconds}-{}.sqlite3",
        uuid::Uuid::new_v4()
    );
    let published = publish(&snapshot, &destination.join("backup-restore"), &file_name);
    if let Err(error) = &published {
        report_create_backup_gate("publication", "failed", storage_error_class(error));
    }
    let snapshot_cleanup = remove_stage_with_evidence_detailed_using(&snapshot, cleanup);
    report_create_backup_gate("final_cleanup", cleanup_result_outcome(snapshot_cleanup), cleanup_result_class(snapshot_cleanup));
    match published {
        Ok(published) => match snapshot_cleanup {
            CleanupResult::Cleaned => BackupResponse::Created {
                file_name,
                created_at_unix_seconds,
                size_bytes: published.size_bytes,
                schema_version: metadata.schema_version,
                durability_warning: published.durability_warning,
                cleanup_warning: false,
            },
            CleanupResult::DurablyEvidencedFailure => BackupResponse::Created {
                file_name,
                created_at_unix_seconds,
                size_bytes: published.size_bytes,
                schema_version: metadata.schema_version,
                durability_warning: published.durability_warning,
                cleanup_warning: true,
            },
            CleanupResult::UnaccountedFailure => BackupResponse::error("storage_unavailable"),
        },
        Err(_) if snapshot_cleanup != CleanupResult::Cleaned => BackupResponse::error("storage_unavailable"),
        Err(error) => BackupResponse::error(storage_error_code(error)),
    }
}

pub fn prepare_restore(
    state: &DatabaseState,
    commands: &mut BackupCommandState,
    request: PrepareRestoreRequest,
) -> BackupResponse {
    prepare_restore_using(state, commands, request, &mut RealStageCleanup)
}

fn prepare_restore_using(
    state: &DatabaseState,
    commands: &mut BackupCommandState,
    request: PrepareRestoreRequest,
    cleanup: &mut impl StageCleanup,
) -> BackupResponse {
    prepare_restore_with_inspection(state, commands, request, cleanup, &mut RealStageInspection)
}

trait StageInspection {
    fn metadata(&mut self, path: &Path) -> std::io::Result<(bool, u64)>;
    fn checksum(&mut self, path: &Path) -> std::io::Result<String>;
}

struct RealStageInspection;

impl StageInspection for RealStageInspection {
    fn metadata(&mut self, path: &Path) -> std::io::Result<(bool, u64)> {
        let metadata = fs::symlink_metadata(path)?;
        Ok((metadata.file_type().is_file(), metadata.len()))
    }

    fn checksum(&mut self, path: &Path) -> std::io::Result<String> {
        checksum(path)
    }
}

fn prepare_restore_with_inspection(
    state: &DatabaseState,
    commands: &mut BackupCommandState,
    request: PrepareRestoreRequest,
    cleanup: &mut impl StageCleanup,
    inspection: &mut impl StageInspection,
) -> BackupResponse {
    if !commands.prune_expired_using(cleanup) { return BackupResponse::error("storage_unavailable"); }
    if commands.pending.len() >= MAX_PENDING_RESTORES {
        return BackupResponse::error("storage_unavailable");
    }
    let staging = commands.root.join("backup-restore/staging");
    if !prune_artifacts_using(&staging, false, MAX_PENDING_RESTORES, Duration::from_secs(7 * 24 * 60 * 60), &commands.pending.values().map(|pending| pending.stage.clone()).collect::<Vec<_>>(), cleanup) {
        return BackupResponse::error("storage_unavailable");
    }
    if let Err(code) = state.with_read(|_| Ok(())) {
        return BackupResponse::from_internal_code(&code);
    }
    let token = uuid::Uuid::new_v4().to_string();
    let stage = commands
        .root
        .join("backup-restore/staging")
        .join(format!("{token}.sqlite3"));
    let source = match commands.consume_source(&request.source_token) {
        Ok(source) => source,
        Err(code) => return BackupResponse::error(code),
    };
    if !matches!(fs::symlink_metadata(&source), Ok(metadata) if metadata.file_type().is_file() && metadata.len() <= MAX_STAGE_BYTES) {
        return BackupResponse::error("invalid_backup");
    }
    let metadata = match stage_and_validate(&source, &stage) {
        Ok(metadata) => metadata,
        Err(crate::infrastructure::sqlite::BackupValidationError::InvalidBackup) => {
            return if remove_stage_with_evidence_using(&stage, cleanup) { BackupResponse::error("invalid_backup") } else { BackupResponse::error("storage_unavailable") };
        }
        Err(crate::infrastructure::sqlite::BackupValidationError::UnsupportedSchema) => {
            return if remove_stage_with_evidence_using(&stage, cleanup) { BackupResponse::error("unsupported_schema") } else { BackupResponse::error("storage_unavailable") };
        }
    };
    let Ok((is_file, size_bytes)) = inspection.metadata(&stage) else {
        if !remove_stage_with_evidence_using(&stage, cleanup) { return BackupResponse::error("storage_unavailable"); }
        return BackupResponse::error("storage_unavailable");
    };
    if !is_file || size_bytes > MAX_STAGE_BYTES {
        return if remove_stage_with_evidence_using(&stage, cleanup) { BackupResponse::error("invalid_backup") } else { BackupResponse::error("storage_unavailable") };
    }
    let Ok(sha256) = inspection.checksum(&stage) else {
        if !remove_stage_with_evidence_using(&stage, cleanup) { return BackupResponse::error("storage_unavailable"); }
        return BackupResponse::error("storage_unavailable");
    };
    commands.pending.insert(
        token.clone(),
        PendingRestore {
            stage,
            sha256,
            expires_at: Instant::now() + RESTORE_TOKEN_TTL,
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
    confirm_restore_using(state, commands, request, &mut RealStageCleanup)
}

fn confirm_restore_using(
    state: &DatabaseState,
    commands: &mut BackupCommandState,
    request: ConfirmRestoreRequest,
    cleanup: &mut impl StageCleanup,
) -> BackupResponse {
    let Some(pending) = commands.pending.remove(&request.token) else {
        if !commands.prune_expired_using(cleanup) { return BackupResponse::error("storage_unavailable"); }
        return BackupResponse::error("token_invalid");
    };
    if !commands.prune_expired_using(cleanup) {
        commands.pending.insert(request.token, pending);
        return BackupResponse::error("storage_unavailable");
    }
    if Instant::now() >= pending.expires_at {
        return if remove_stage_with_evidence_using(&pending.stage, cleanup) { BackupResponse::error("token_expired") } else { BackupResponse::error("storage_unavailable") };
    }
    if !request.confirmed {
        return if remove_stage_with_evidence_using(&pending.stage, cleanup) { BackupResponse::error("confirmation_required") } else { BackupResponse::error("storage_unavailable") };
    }
    if !matches!(checksum(&pending.stage), Ok(checksum) if checksum == pending.sha256) {
        return if remove_stage_with_evidence_using(&pending.stage, cleanup) { BackupResponse::error("invalid_backup") } else { BackupResponse::error("storage_unavailable") };
    }
    match state.install_validated_stage(&pending.stage, &BackupStore::new(&commands.root)) {
        Ok(()) => {
            if remove_stage_with_evidence_using(&pending.stage, cleanup) { BackupResponse::Restored } else { BackupResponse::error("storage_unavailable") }
        }
        Err(code) => {
            // Once a durable marker exists, the recovery protocol owns the stage/evidence.
            if matches!(BackupStore::new(&commands.root).read_restore_state(), Ok(None)) {
                if !remove_stage_with_evidence_using(&pending.stage, cleanup) { return BackupResponse::error("storage_unavailable"); }
            }
            if code == "database_unavailable" {
                BackupResponse::error("database_unavailable")
            } else {
                BackupResponse::error("restore_failed")
            }
        }
    }
}

#[cfg(all(windows, debug_assertions))]
fn report_create_backup_gate(operation: &'static str, outcome: &'static str, class: &'static str) {
    eprintln!("backup_diagnostic operation=create_backup_{operation} outcome={outcome} class={class}");
}

#[cfg(not(all(windows, debug_assertions)))]
fn report_create_backup_gate(_operation: &'static str, _outcome: &'static str, _class: &'static str) {}

fn destination_token_class(code: &str) -> &'static str {
    match code {
        "destination_token_invalid" => "destination_token_invalid",
        "destination_token_expired" => "destination_token_expired",
        _ => "other",
    }
}

fn internal_backup_result_class(code: &str) -> &'static str {
    match code {
        "database_unavailable" => "database_unavailable",
        "storage_unavailable" => "storage_unavailable",
        _ => "other",
    }
}

fn storage_error_class(error: &StorageError) -> &'static str {
    match error {
        StorageError::DestinationExists => "destination_exists",
        StorageError::UnsupportedDestination => "unsupported_destination",
        StorageError::SelectionCancelled | StorageError::StorageUnavailable => "storage_unavailable",
    }
}

fn cleanup_result_outcome(result: CleanupResult) -> &'static str {
    match result {
        CleanupResult::Cleaned => "completed",
        CleanupResult::DurablyEvidencedFailure | CleanupResult::UnaccountedFailure => "failed",
    }
}

fn cleanup_result_class(result: CleanupResult) -> &'static str {
    match result {
        CleanupResult::Cleaned => "cleaned",
        CleanupResult::DurablyEvidencedFailure => "durably_evidenced_failure",
        CleanupResult::UnaccountedFailure => "unaccounted_failure",
    }
}

fn storage_error_code(error: StorageError) -> &'static str {
    match error {
        StorageError::DestinationExists => "destination_exists",
        StorageError::UnsupportedDestination => "unsupported_destination",
        StorageError::SelectionCancelled | StorageError::StorageUnavailable => "storage_unavailable",
    }
}

pub(crate) fn remove_stage_with_evidence(path: &Path) -> bool {
    remove_stage_with_evidence_using(path, &mut RealStageCleanup)
}

pub(crate) trait StageCleanup {
    fn remove(&mut self, path: &Path) -> std::io::Result<()>;
    fn sync_parent(&mut self, path: &Path) -> std::io::Result<()>;
    fn create_evidence(&mut self, path: &Path) -> std::io::Result<()>;
    fn write_evidence(&mut self, path: &Path) -> std::io::Result<()>;
    fn sync_evidence(&mut self, path: &Path) -> std::io::Result<()>;
}

pub(crate) struct RealStageCleanup;

impl StageCleanup for RealStageCleanup {
    fn remove(&mut self, path: &Path) -> std::io::Result<()> {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_file() => fs::remove_file(path),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
            _ => Err(std::io::ErrorKind::InvalidData.into()),
        }
    }

    fn sync_parent(&mut self, path: &Path) -> std::io::Result<()> {
        sync_parent_directory(path)
    }

    fn create_evidence(&mut self, path: &Path) -> std::io::Result<()> {
        OpenOptions::new().write(true).create_new(true).open(path).map(drop)
    }

    fn write_evidence(&mut self, path: &Path) -> std::io::Result<()> {
        use std::io::Write;
        OpenOptions::new().write(true).open(path)?.write_all(b"restore-stage-cleanup-required\n")
    }

    fn sync_evidence(&mut self, path: &Path) -> std::io::Result<()> {
        OpenOptions::new().write(true).open(path)?.sync_all()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CleanupResult {
    Cleaned,
    DurablyEvidencedFailure,
    UnaccountedFailure,
}

pub(crate) fn remove_stage_with_evidence_using(path: &Path, cleanup: &mut impl StageCleanup) -> bool {
    remove_stage_with_evidence_detailed_using(path, cleanup) == CleanupResult::Cleaned
}

fn remove_stage_with_evidence_detailed_using(path: &Path, cleanup: &mut impl StageCleanup) -> CleanupResult {
    let Some(parent) = path.parent() else { return CleanupResult::UnaccountedFailure };
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else { return CleanupResult::UnaccountedFailure };
    let record = parent.join(format!(".{file_name}.cleanup-needed"));

    // Persist the bounded recovery record before deleting anything. If any later operation
    // fails, startup can safely retry without depending on the original stage still existing.
    if let Err(error) = cleanup.create_evidence(&record) {
        report_backup_io_failure("cleanup_evidence_create", &error);
        return CleanupResult::UnaccountedFailure;
    }
    if let Err(error) = cleanup.write_evidence(&record) {
        report_backup_io_failure("cleanup_evidence_write", &error);
        return CleanupResult::UnaccountedFailure;
    }
    if let Err(error) = cleanup.sync_evidence(&record) {
        report_backup_io_failure("cleanup_evidence_sync", &error);
        return CleanupResult::UnaccountedFailure;
    }
    if let Err(error) = cleanup.sync_parent(&record) {
        report_backup_io_failure("cleanup_evidence_directory_sync", &error);
        return CleanupResult::UnaccountedFailure;
    }
    if let Err(error) = cleanup.remove(path) {
        report_backup_io_failure("cleanup_artifact_remove", &error);
        return CleanupResult::DurablyEvidencedFailure;
    }
    if let Err(error) = cleanup.sync_parent(path) {
        report_backup_io_failure("cleanup_artifact_directory_sync", &error);
        return CleanupResult::DurablyEvidencedFailure;
    }
    if let Err(error) = cleanup.remove(&record) {
        report_backup_io_failure("cleanup_evidence_remove", &error);
        return recreate_cleanup_evidence(&record, cleanup);
    }
    if let Err(error) = cleanup.sync_parent(&record) {
        report_backup_io_failure("cleanup_evidence_directory_sync", &error);
        return recreate_cleanup_evidence(&record, cleanup);
    }
    CleanupResult::Cleaned
}

fn recreate_cleanup_evidence(record: &Path, cleanup: &mut impl StageCleanup) -> CleanupResult {
    if let Err(error) = cleanup.create_evidence(record) {
        report_backup_io_failure("cleanup_evidence_recreate", &error);
        return CleanupResult::UnaccountedFailure;
    }
    if let Err(error) = cleanup.write_evidence(record) {
        report_backup_io_failure("cleanup_evidence_rewrite", &error);
        return CleanupResult::UnaccountedFailure;
    }
    if let Err(error) = cleanup.sync_evidence(record) {
        report_backup_io_failure("cleanup_evidence_resync", &error);
        return CleanupResult::UnaccountedFailure;
    }
    if let Err(error) = cleanup.sync_parent(record) {
        report_backup_io_failure("cleanup_evidence_directory_resync", &error);
        return CleanupResult::UnaccountedFailure;
    }
    CleanupResult::DurablyEvidencedFailure
}

#[cfg(all(windows, debug_assertions))]
fn report_backup_sqlite_failure(operation: &str, _error: &rusqlite::Error) {
    // rusqlite does not expose a separate originating Windows system error.
    eprintln!("backup_diagnostic operation={operation} os_code=none kind=Other");
}

#[cfg(not(all(windows, debug_assertions)))]
fn report_backup_sqlite_failure(_operation: &str, _error: &rusqlite::Error) {}

#[cfg(all(windows, debug_assertions))]
fn report_backup_io_failure(operation: &str, error: &std::io::Error) {
    eprintln!(
        "backup_diagnostic operation={operation} os_code={} kind={:?}",
        error.raw_os_error().map_or_else(|| "none".to_string(), |code| code.to_string()),
        error.kind(),
    );
}

#[cfg(not(all(windows, debug_assertions)))]
fn report_backup_io_failure(_operation: &str, _error: &std::io::Error) {}

/// Reconcile durable cleanup records after restart. The artifact is removed and its directory
/// synced before its evidence is unlinked; failure at either point keeps startup unavailable.
pub(crate) fn reconcile_cleanup_evidence(directory: &Path) -> bool {
    reconcile_cleanup_evidence_using(directory, &mut RealStageCleanup)
}

pub(crate) fn reconcile_cleanup_evidence_using(directory: &Path, cleanup: &mut impl StageCleanup) -> bool {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return true,
        Err(error) => {
            report_backup_io_failure("cleanup_reconcile_directory_read", &error);
            return false;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                report_backup_io_failure("cleanup_reconcile_entry_read", &error);
                return false;
            }
        };
        let name = entry.file_name();
        let Some(name) = name.to_str() else { return false };
        let Some(artifact_name) = name.strip_prefix('.').and_then(|name| name.strip_suffix(".cleanup-needed")) else { continue };
        if artifact_name.is_empty() { return false; }
        let artifact = directory.join(artifact_name);
        let record = entry.path();
        if let Err(error) = cleanup.remove(&artifact) {
            report_backup_io_failure("cleanup_reconcile_artifact_remove", &error);
            return false;
        }
        if let Err(error) = cleanup.sync_parent(&artifact) {
            report_backup_io_failure("cleanup_reconcile_artifact_directory_sync", &error);
            return false;
        }
        match fs::symlink_metadata(&record) {
            Ok(metadata) if metadata.file_type().is_file() => {}
            Err(error) => {
                report_backup_io_failure("cleanup_reconcile_evidence_metadata", &error);
                recreate_cleanup_evidence(&record, cleanup);
                return false;
            }
            _ => {
                recreate_cleanup_evidence(&record, cleanup);
                return false;
            }
        }
        if let Err(error) = cleanup.remove(&record) {
            report_backup_io_failure("cleanup_reconcile_evidence_remove", &error);
            recreate_cleanup_evidence(&record, cleanup);
            return false;
        }
        if let Err(error) = cleanup.sync_parent(&record) {
            report_backup_io_failure("cleanup_reconcile_evidence_directory_sync", &error);
            recreate_cleanup_evidence(&record, cleanup);
            return false;
        }
    }
    true
}

fn sync_parent_directory(path: &Path) -> std::io::Result<()> {
    let parent = path.parent().ok_or(std::io::ErrorKind::InvalidInput)?;
    sync_directory(parent)
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> std::io::Result<()> {
    fs::File::open(path)?.sync_all()
}

#[cfg(windows)]
fn sync_directory(path: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{CloseHandle, GENERIC_WRITE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FlushFileBuffers, FILE_FLAG_BACKUP_SEMANTICS,
        FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_DELETE, FILE_SHARE_READ,
        FILE_SHARE_WRITE, OPEN_EXISTING,
    };

    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    if wide.contains(&0) {
        let error = std::io::Error::from(std::io::ErrorKind::InvalidInput);
        report_backup_io_failure("cleanup_directory_sync_path", &error);
        return Err(error);
    }
    wide.push(0);
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
            std::ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        let error = std::io::Error::last_os_error();
        report_backup_io_failure("cleanup_directory_sync_open", &error);
        return Err(error);
    }
    let flush_result = unsafe { FlushFileBuffers(handle) };
    let flush_error = (flush_result == 0).then(std::io::Error::last_os_error);
    if let Some(error) = &flush_error {
        report_backup_io_failure("cleanup_directory_sync_flush", error);
    }
    let close_result = unsafe { CloseHandle(handle) };
    let close_error = (close_result == 0).then(std::io::Error::last_os_error);
    if let Some(error) = &close_error {
        report_backup_io_failure("cleanup_directory_sync_close", error);
    }
    match (flush_error, close_error) {
        (Some(error), _) => Err(error),
        (None, Some(error)) => Err(error),
        (None, None) => Ok(()),
    }
}

#[cfg(not(any(unix, windows)))]
fn sync_directory(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

fn prune_artifacts_using(directory: &Path, snapshots: bool, maximum: usize, maximum_age: Duration, protected: &[PathBuf], cleanup: &mut impl StageCleanup) -> bool {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return true,
        Err(error) => {
            report_backup_io_failure("artifact_prune_directory_read", &error);
            return false;
        }
    };
    let now = SystemTime::now();
    let mut count = 0usize;
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                report_backup_io_failure("artifact_prune_entry_read", &error);
                return false;
            }
        };
        let path = entry.path();
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_file() => metadata,
            Err(error) => {
                report_backup_io_failure("artifact_prune_metadata", &error);
                return false;
            }
            _ => return false,
        };
        let name = path.file_name().and_then(|value| value.to_str()).unwrap_or_default();
        let recognized = if snapshots {
            name.strip_suffix(".sqlite3").and_then(|stem| stem.split_once('-')).is_some_and(|(timestamp, id)| timestamp.parse::<u64>().is_ok() && uuid::Uuid::parse_str(id).is_ok())
        } else {
            name.strip_suffix(".sqlite3").is_some_and(|id| uuid::Uuid::parse_str(id).is_ok())
        };
        if !recognized { return false; }
        let aged = metadata.modified().ok().and_then(|modified| now.duration_since(modified).ok()).is_some_and(|age| age > maximum_age);
        if aged && !protected.contains(&path) {
            let removed = remove_stage_with_evidence_using(&path, cleanup);
            if !removed { return false; }
        } else {
            count += 1;
        }
    }
    count < maximum
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

#[cfg(test)]
mod destination_token_tests {
    use super::*;

    #[test]
    fn unknown_tokens_cannot_bypass_native_selection_and_tokens_are_single_use() {
        let mut state = BackupCommandState::new("/app");
        assert_eq!(state.consume_destination("webview-path-token"), Err("destination_token_invalid"));
        assert_eq!(state.consume_source("webview-path-token"), Err("source_token_invalid"));

        let token = state.issue_test_destination(PathBuf::from("/trusted/selected"), Instant::now() + Duration::from_secs(60));
        assert_eq!(state.consume_destination(&token), Ok(PathBuf::from("/trusted/selected")));
        assert_eq!(state.consume_destination(&token), Err("destination_token_invalid"));
    }

    #[test]
    fn expired_destination_tokens_are_consumed_and_rejected() {
        let mut state = BackupCommandState::new("/app");
        let token = state.issue_test_destination(PathBuf::from("/trusted/selected"), Instant::now() - Duration::from_secs(1));
        assert_eq!(state.consume_destination(&token), Err("destination_token_expired"));
        assert_eq!(state.consume_destination(&token), Err("destination_token_invalid"));
    }

    #[test]
    fn picker_cancellation_is_not_a_destination_error() {
        assert_eq!(select_path(None), PathSelection::Cancelled);
    }

    #[test]
    fn restore_source_tokens_are_native_held_short_lived_and_single_use() {
        let mut state = BackupCommandState::new("/app");
        let RestoreSourceSelection::Selected { token } = state.select_restore_source(PathBuf::from("/trusted/backup.sqlite3")) else { panic!("picker selection should issue an opaque token"); };
        assert_eq!(state.consume_source(&token), Ok(PathBuf::from("/trusted/backup.sqlite3")));
        assert_eq!(state.consume_source(&token), Err("source_token_invalid"));

        let expired = state.issue_test_source(PathBuf::from("/trusted/expired.sqlite3"), Instant::now() - Duration::from_secs(1));
        assert_eq!(state.consume_source(&expired), Err("source_token_expired"));
        assert_eq!(state.consume_source(&expired), Err("source_token_invalid"));
    }

    #[derive(Default)]
    struct RecordingCleanup {
        calls: Vec<&'static str>,
        fail: Option<&'static str>,
        fail_at: Option<usize>,
    }

    impl RecordingCleanup {
        fn record(&mut self, operation: &'static str) -> std::io::Result<()> {
            self.calls.push(operation);
            if self.fail == Some(operation) || self.fail_at == Some(self.calls.len()) {
                return Err(std::io::ErrorKind::Other.into());
            }
            Ok(())
        }
    }

    impl StageCleanup for RecordingCleanup {
        fn remove(&mut self, path: &Path) -> std::io::Result<()> {
            self.record("remove")?;
            match fs::symlink_metadata(path) {
                Ok(metadata) if metadata.file_type().is_file() => fs::remove_file(path),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                _ => Err(std::io::ErrorKind::InvalidData.into()),
            }
        }
        fn sync_parent(&mut self, path: &Path) -> std::io::Result<()> {
            self.record("sync_parent")?;
            sync_parent_directory(path)
        }
        fn create_evidence(&mut self, path: &Path) -> std::io::Result<()> {
            self.record("create_evidence")?;
            OpenOptions::new().write(true).create_new(true).open(path).map(drop)
        }
        fn write_evidence(&mut self, path: &Path) -> std::io::Result<()> {
            self.record("write_evidence")?;
            use std::io::Write;
            OpenOptions::new().write(true).open(path)?.write_all(b"restore-stage-cleanup-required\n")
        }
        fn sync_evidence(&mut self, path: &Path) -> std::io::Result<()> {
            self.record("sync_evidence")?;
            OpenOptions::new().write(true).open(path)?.sync_all()
        }
    }

    fn test_directory() -> PathBuf {
        let path = std::env::temp_dir().join(format!("backup-command-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(path.join("backup-restore/staging")).unwrap();
        path
    }

    #[test]
    fn invalid_source_does_not_leave_a_stage_file() {
        let root = test_directory();
        let invalid = root.join("invalid.sqlite3");
        fs::write(&invalid, b"not a database").unwrap();
        let mut commands = BackupCommandState::new(&root);
        let source_token = commands.issue_test_source(invalid, Instant::now() + Duration::from_secs(30));
        let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();

        assert!(matches!(prepare_restore(&state, &mut commands, PrepareRestoreRequest { source_token }), BackupResponse::Error { code: "invalid_backup", .. }));
        assert_eq!(fs::read_dir(root.join("backup-restore/staging")).unwrap().count(), 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn oversized_source_is_rejected_before_stage_creation() {
        let root = test_directory();
        let source = root.join("oversized.sqlite3");
        fs::File::create(&source).unwrap().set_len(MAX_STAGE_BYTES + 1).unwrap();
        let mut commands = BackupCommandState::new(&root);
        let source_token = commands.issue_test_source(source, Instant::now() + Duration::from_secs(30));
        let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();

        assert!(matches!(prepare_restore(&state, &mut commands, PrepareRestoreRequest { source_token }), BackupResponse::Error { code: "invalid_backup", .. }));
        assert_eq!(fs::read_dir(root.join("backup-restore/staging")).unwrap().count(), 0);
        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn successful_restore_removes_stage_after_durable_protocol_finishes() {
        let root = test_directory();
        let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();
        let source = root.join("source.sqlite3");
        state.with_read(|connection| crate::infrastructure::sqlite::create_snapshot(connection, &source).map(|_| ()).map_err(|_| "snapshot".into())).unwrap();
        let mut commands = BackupCommandState::new(&root);
        let source_token = commands.issue_test_source(source, Instant::now() + Duration::from_secs(30));
        let BackupResponse::Prepared { token, .. } = prepare_restore(&state, &mut commands, PrepareRestoreRequest { source_token }) else { panic!("valid source should be staged"); };
        let stage = commands.pending[&token].stage.clone();

        assert_eq!(confirm_restore(&state, &mut commands, ConfirmRestoreRequest { token, confirmed: true }), BackupResponse::Restored);
        assert!(!stage.exists());
        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn confirm_terminal_cleanup_failures_are_storage_errors_and_do_not_install() {
        let root = test_directory();
        let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();
        for (expired, confirmed, corrupt, expected_code) in [
            (true, true, false, "token_expired"),
            (false, false, false, "confirmation_required"),
            (false, true, true, "invalid_backup"),
        ] {
            let stage = root.join("backup-restore/staging/candidate.sqlite3");
            fs::write(&stage, if corrupt { &b"changed"[..] } else { &b"candidate"[..] }).unwrap();
            let token = uuid::Uuid::new_v4().to_string();
            let expires_at = if expired { Instant::now() - Duration::from_secs(1) } else { Instant::now() + Duration::from_secs(30) };
            let mut commands = BackupCommandState::new(&root);
            commands.pending.insert(token.clone(), PendingRestore {
                stage: stage.clone(),
                sha256: "does-not-match".into(),
                expires_at,
            });
            let mut cleanup = RecordingCleanup { fail: Some("create_evidence"), ..Default::default() };
            assert!(matches!(confirm_restore_using(&state, &mut commands, ConfirmRestoreRequest { token, confirmed }, &mut cleanup), BackupResponse::Error { code: "storage_unavailable", .. }), "expected terminal outcome {expected_code}");
            assert!(stage.exists());
            assert_eq!(cleanup.calls, vec!["create_evidence"]);
            assert!(!BackupStore::new(&root).read_restore_state().unwrap_or_default().is_some());
        }
        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[derive(Clone, Copy)]
    enum InspectionFailure { Metadata, NonFile, Oversized, Checksum }

    struct FailingInspection(InspectionFailure);

    impl StageInspection for FailingInspection {
        fn metadata(&mut self, path: &Path) -> std::io::Result<(bool, u64)> {
            match self.0 {
                InspectionFailure::Metadata => Err(std::io::ErrorKind::Other.into()),
                InspectionFailure::NonFile => Ok((false, 1)),
                InspectionFailure::Oversized => Ok((true, MAX_STAGE_BYTES + 1)),
                InspectionFailure::Checksum => RealStageInspection.metadata(path),
            }
        }

        fn checksum(&mut self, _path: &Path) -> std::io::Result<String> {
            Err(std::io::ErrorKind::Other.into())
        }
    }

    #[test]
    fn prepare_validation_and_post_stage_cleanup_failures_are_storage_unavailable() {
        for scenario in ["invalid", "unsupported", "metadata", "non_file", "oversized", "checksum"] {
            let root = test_directory();
            let source = root.join("source.sqlite3");
            match scenario {
                "invalid" => fs::write(&source, b"not sqlite").unwrap(),
                "unsupported" => {
                    let connection = rusqlite::Connection::open(&source).unwrap();
                    connection.pragma_update(None, "user_version", i64::MAX).unwrap();
                }
                _ => {
                    let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();
                    state.with_read(|connection| crate::infrastructure::sqlite::create_snapshot(connection, &source).map(|_| ()).map_err(|_| "snapshot".into())).unwrap();
                }
            }
            let mut commands = BackupCommandState::new(&root);
            let source_token = commands.issue_test_source(source, Instant::now() + Duration::from_secs(30));
            let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();
            let mut cleanup = RecordingCleanup { fail: Some("remove"), ..Default::default() };
            let response = match scenario {
                "metadata" | "non_file" | "oversized" | "checksum" => {
                    let failure = match scenario {
                        "metadata" => InspectionFailure::Metadata,
                        "non_file" => InspectionFailure::NonFile,
                        "oversized" => InspectionFailure::Oversized,
                        _ => InspectionFailure::Checksum,
                    };
                    prepare_restore_with_inspection(&state, &mut commands, PrepareRestoreRequest { source_token }, &mut cleanup, &mut FailingInspection(failure))
                }
                _ => prepare_restore_using(&state, &mut commands, PrepareRestoreRequest { source_token }, &mut cleanup),
            };
            assert!(matches!(response, BackupResponse::Error { code: "storage_unavailable", .. }), "{scenario}: {response:?}");
            assert!(commands.pending.is_empty(), "{scenario}: no token may be issued");
            let staging = root.join("backup-restore/staging");
            let artifacts: Vec<_> = fs::read_dir(&staging).unwrap().map(Result::unwrap).collect();
            assert!(!artifacts.is_empty(), "{scenario}: stage or cleanup evidence retained");
            assert!(artifacts.iter().any(|entry| entry.file_name().to_string_lossy().ends_with(".cleanup-needed")), "{scenario}: durable cleanup evidence retained");
            assert!(BackupStore::new(&root).read_restore_state().unwrap().is_none(), "{scenario}: cleanup failure must not claim marker ownership");
            drop(state);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn parent_sync_failures_preserve_evidence_after_each_cleanup_boundary() {
        for (fail_at, expected_parent_syncs) in [(4, 1), (6, 2), (8, 4)] {
            let root = test_directory();
            let stage = root.join("stage.sqlite3");
            let evidence = root.join(".stage.sqlite3.cleanup-needed");
            fs::write(&stage, b"stage").unwrap();
            let mut cleanup = RecordingCleanup { fail_at: Some(fail_at), ..Default::default() };
            assert!(!remove_stage_with_evidence_using(&stage, &mut cleanup), "call {fail_at}");
            assert!(evidence.exists(), "call {fail_at}: cleanup evidence remains or is recreated");
            assert_eq!(fs::read(&evidence).unwrap(), b"restore-stage-cleanup-required\n");
            assert_eq!(cleanup.calls.iter().filter(|call| **call == "sync_parent").count(), expected_parent_syncs);
            if fail_at == 6 || fail_at == 8 {
                assert!(!stage.exists(), "call {fail_at}: artifact removal already occurred");
            } else {
                assert!(stage.exists(), "call {fail_at}: artifact remains owned");
            }
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn confirm_terminal_cleanup_failures_retain_evidence_without_false_outcomes() {
        for scenario in ["expired", "unconfirmed", "checksum", "pre_marker", "success"] {
            let root = test_directory();
            let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();
            let stage = root.join(format!("backup-restore/staging/{scenario}.sqlite3"));
            let (contents, expected_checksum, confirmed, expires_at) = match scenario {
                "success" => {
                    state.with_read(|connection| crate::infrastructure::sqlite::create_snapshot(connection, &stage).map(|_| ()).map_err(|_| "snapshot".into())).unwrap();
                    (fs::read(&stage).unwrap(), None, true, Instant::now() + Duration::from_secs(30))
                }
                "pre_marker" => {
                    state.with_read(|connection| crate::infrastructure::sqlite::create_snapshot(connection, &stage).map(|_| ()).map_err(|_| "snapshot".into())).unwrap();
                    (fs::read(&stage).unwrap(), None, true, Instant::now() + Duration::from_secs(30))
                }
                "expired" => (b"expired".to_vec(), None, true, Instant::now() - Duration::from_secs(1)),
                "unconfirmed" => (b"unconfirmed".to_vec(), None, false, Instant::now() + Duration::from_secs(30)),
                _ => (b"tampered".to_vec(), Some("different checksum".to_string()), true, Instant::now() + Duration::from_secs(30)),
            };
            if scenario != "success" && scenario != "pre_marker" { fs::write(&stage, &contents).unwrap(); }
            if scenario == "pre_marker" { fs::create_dir(root.join("pre-restore.sqlite3")).unwrap(); }
            let token = uuid::Uuid::new_v4().to_string();
            let sha256 = expected_checksum.unwrap_or_else(|| checksum(&stage).unwrap());
            let token_expired = scenario == "expired";
            let mut commands = BackupCommandState::new(&root);
            commands.pending.insert(token.clone(), PendingRestore { stage: stage.clone(), sha256, expires_at });
            let mut cleanup = RecordingCleanup { fail: Some("remove"), ..Default::default() };
            let response = confirm_restore_using(&state, &mut commands, ConfirmRestoreRequest { token, confirmed }, &mut cleanup);
            assert!(matches!(response, BackupResponse::Error { code: "storage_unavailable", .. }), "{scenario}: {response:?}");
            let evidence = stage.with_file_name(format!(".{}.cleanup-needed", stage.file_name().unwrap().to_string_lossy()));
            assert!(evidence.exists(), "{scenario}: cleanup evidence retained");
            if scenario == "success" {
                assert!(!stage.exists(), "successful installation transfers the stage to the canonical database");
                assert!(crate::infrastructure::sqlite::production_database_config(&root).path().is_file());
            } else {
                assert!(stage.exists(), "{scenario}: injected artifact-removal failure retains the stage");
            }
            assert!(BackupStore::new(&root).read_restore_state().unwrap().is_none(), "{scenario}: no active marker may be falsely claimed");
            assert!(!matches!(response, BackupResponse::Restored));
            if token_expired || !confirmed { assert!(commands.pending.is_empty()); }
            drop(state);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn expired_and_aged_artifact_pruning_fail_closed_with_retained_evidence() {
        for artifact_pruning in [false, true] {
            let root = test_directory();
            let stage = if artifact_pruning {
                root.join(format!("backup-restore/staging/{}.sqlite3", uuid::Uuid::new_v4()))
            } else {
                root.join("backup-restore/staging/expired.sqlite3")
            };
            fs::write(&stage, b"abandoned").unwrap();
            let mut commands = BackupCommandState::new(&root);
            if !artifact_pruning {
                let token = uuid::Uuid::new_v4().to_string();
                commands.pending.insert(token, PendingRestore { stage: stage.clone(), sha256: String::new(), expires_at: Instant::now() - Duration::from_secs(1) });
            } else {
                let old = std::time::SystemTime::now() - Duration::from_secs(8 * 24 * 60 * 60);
                fs::File::options().write(true).open(&stage).unwrap().set_times(std::fs::FileTimes::new().set_modified(old)).unwrap();
            }
            let source = root.join("unused.sqlite3");
            fs::write(&source, b"invalid").unwrap();
            let source_token = commands.issue_test_source(source, Instant::now() + Duration::from_secs(30));
            let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();
            let mut cleanup = RecordingCleanup { fail: Some("remove"), ..Default::default() };
            let response = prepare_restore_using(&state, &mut commands, PrepareRestoreRequest { source_token }, &mut cleanup);
            assert!(matches!(response, BackupResponse::Error { code: "storage_unavailable", .. }), "artifact_pruning={artifact_pruning}: {response:?}");
            assert!(stage.exists());
            assert!(stage.with_file_name(format!(".{}.cleanup-needed", stage.file_name().unwrap().to_string_lossy())).exists());
            assert!(BackupStore::new(&root).read_restore_state().unwrap().is_none());
            drop(state);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn published_backup_with_snapshot_cleanup_failure_returns_created_warning_and_evidence() {
        let root = test_directory();
        let destination = root.join("selected");
        let mut commands = BackupCommandState::new(&root);
        let destination_token = commands.issue_test_destination(destination.clone(), Instant::now() + Duration::from_secs(30));
        let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();
        let mut cleanup = RecordingCleanup { fail: Some("remove"), ..Default::default() };
        let response = create_backup_with_publisher(
            &state,
            &mut commands,
            CreateBackupRequest { destination_token },
            &mut cleanup,
            |snapshot, published_to, file_name| {
                assert!(snapshot.is_file());
                assert_eq!(published_to, destination.join("backup-restore"));
                fs::create_dir_all(published_to).unwrap();
                fs::copy(snapshot, published_to.join(file_name)).unwrap();
                assert!(published_to.join(file_name).is_file());
                Ok(PublishedBackup { path: published_to.join(file_name), size_bytes: fs::metadata(snapshot).unwrap().len(), sha256: "verified".into(), durability_warning: false })
            },
        );
        let BackupResponse::Created { file_name, cleanup_warning, durability_warning, .. } = response else { panic!("successful publication remains Created when internal cleanup fails"); };
        assert!(cleanup_warning);
        assert!(!durability_warning);
        let snapshot = fs::read_dir(root.join("backup-restore/snapshots")).unwrap().map(Result::unwrap).find(|entry| entry.file_name().to_string_lossy().ends_with(".sqlite3")).unwrap().path();
        let evidence = snapshot.with_file_name(format!(".{}.cleanup-needed", snapshot.file_name().unwrap().to_string_lossy()));
        assert_eq!(fs::read(&evidence).unwrap(), b"restore-stage-cleanup-required\n");
        assert!(file_name.starts_with("backup-"));
        assert_eq!(cleanup.calls, vec!["create_evidence", "write_evidence", "sync_evidence", "sync_parent", "remove"]);
        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn published_backup_without_durable_cleanup_evidence_is_storage_unavailable() {
        for failed_operation in ["create_evidence", "write_evidence", "sync_evidence", "sync_parent"] {
            let root = test_directory();
            let destination = root.join("selected");
            let mut commands = BackupCommandState::new(&root);
            let destination_token = commands.issue_test_destination(destination.clone(), Instant::now() + Duration::from_secs(30));
            let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();
            let mut cleanup = RecordingCleanup { fail: Some(failed_operation), ..Default::default() };
            let response = create_backup_with_publisher(
                &state,
                &mut commands,
                CreateBackupRequest { destination_token },
                &mut cleanup,
                |snapshot, published_to, file_name| {
                    fs::create_dir_all(published_to).unwrap();
                    fs::copy(snapshot, published_to.join(file_name)).unwrap();
                    Ok(PublishedBackup { path: published_to.join(file_name), size_bytes: fs::metadata(snapshot).unwrap().len(), sha256: "verified".into(), durability_warning: false })
                },
            );
            assert!(matches!(response, BackupResponse::Error { code: "storage_unavailable", .. }), "{failed_operation}: {response:?}");
            assert!(destination.join("backup-restore").read_dir().unwrap().any(|entry| entry.unwrap().file_name().to_string_lossy().starts_with("backup-")));
            let snapshot_directory = root.join("backup-restore/snapshots");
            let snapshot = fs::read_dir(&snapshot_directory).unwrap().map(Result::unwrap).find(|entry| entry.file_name().to_string_lossy().ends_with(".sqlite3")).unwrap().path();
            let evidence = snapshot.with_file_name(format!(".{}.cleanup-needed", snapshot.file_name().unwrap().to_string_lossy()));
            if failed_operation == "create_evidence" {
                assert!(!evidence.exists());
            }
            if failed_operation == "write_evidence" {
                assert_ne!(fs::read(&evidence).unwrap_or_default(), b"restore-stage-cleanup-required\n");
            }
            drop(state);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn failed_publication_and_failed_snapshot_cleanup_are_storage_unavailable() {
        let root = test_directory();
        let mut commands = BackupCommandState::new(&root);
        let destination_token = commands.issue_test_destination(PathBuf::from("/selected"), Instant::now() + Duration::from_secs(30));
        let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();
        let mut cleanup = RecordingCleanup { fail: Some("remove"), ..Default::default() };
        let response = create_backup_with_publisher(
            &state,
            &mut commands,
            CreateBackupRequest { destination_token },
            &mut cleanup,
            |_, _, _| Err(StorageError::DestinationExists),
        );
        assert!(matches!(response, BackupResponse::Error { code: "storage_unavailable", .. }));
        let evidence = fs::read_dir(root.join("backup-restore/snapshots")).unwrap().map(Result::unwrap).find(|entry| entry.file_name().to_string_lossy().ends_with(".cleanup-needed")).unwrap().path();
        assert_eq!(fs::read(evidence).unwrap(), b"restore-stage-cleanup-required\n");
        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn create_backup_reconciles_stale_snapshot_evidence_before_pruning() {
        let root = test_directory();
        let snapshot_dir = root.join("backup-restore/snapshots");
        fs::create_dir_all(&snapshot_dir).unwrap();
        let stale = snapshot_dir.join(format!("1-{}.sqlite3", uuid::Uuid::new_v4()));
        let evidence = stale.with_file_name(format!(".{}.cleanup-needed", stale.file_name().unwrap().to_string_lossy()));
        fs::write(&stale, b"abandoned snapshot").unwrap();
        fs::write(&evidence, b"restore-stage-cleanup-required\n").unwrap();
        let mut commands = BackupCommandState::new(&root);
        let destination = root.join("selected");
        let destination_token = commands.issue_test_destination(destination.clone(), Instant::now() + Duration::from_secs(30));
        let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();
        let mut cleanup = RecordingCleanup::default();

        let response = create_backup_with_publisher(
            &state,
            &mut commands,
            CreateBackupRequest { destination_token },
            &mut cleanup,
            |snapshot, published_to, file_name| {
                assert!(!stale.exists());
                assert!(!evidence.exists());
                assert!(snapshot.is_file());
                fs::create_dir_all(published_to).unwrap();
                fs::copy(snapshot, published_to.join(file_name)).unwrap();
                Ok(PublishedBackup { path: published_to.join(file_name), size_bytes: fs::metadata(snapshot).unwrap().len(), sha256: "verified".into(), durability_warning: false })
            },
        );

        assert!(matches!(response, BackupResponse::Created { .. }));
        assert!(!stale.exists());
        assert!(!evidence.exists());
        assert!(cleanup.calls.starts_with(&["remove", "sync_parent", "remove", "sync_parent"]));
        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn create_backup_reconciliation_failure_retains_snapshot_evidence_and_blocks() {
        let root = test_directory();
        let snapshot_dir = root.join("backup-restore/snapshots");
        fs::create_dir_all(&snapshot_dir).unwrap();
        let stale = snapshot_dir.join(format!("1-{}.sqlite3", uuid::Uuid::new_v4()));
        let evidence = stale.with_file_name(format!(".{}.cleanup-needed", stale.file_name().unwrap().to_string_lossy()));
        fs::write(&stale, b"abandoned snapshot").unwrap();
        fs::write(&evidence, b"restore-stage-cleanup-required\n").unwrap();
        let mut commands = BackupCommandState::new(&root);
        let destination_token = commands.issue_test_destination(root.join("selected"), Instant::now() + Duration::from_secs(30));
        let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();
        let mut cleanup = RecordingCleanup { fail: Some("remove"), ..Default::default() };

        let response = create_backup_with_publisher(
            &state,
            &mut commands,
            CreateBackupRequest { destination_token },
            &mut cleanup,
            |_, _, _| panic!("publication must not run when cleanup evidence cannot be reconciled"),
        );

        assert!(matches!(response, BackupResponse::Error { code: "storage_unavailable", .. }));
        assert!(stale.exists());
        assert_eq!(fs::read(&evidence).unwrap(), b"restore-stage-cleanup-required\n");
        assert_eq!(cleanup.calls, vec!["remove"]);
        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn create_backup_accepts_an_ordinary_empty_snapshot_directory() {
        let root = test_directory();
        let snapshot_dir = root.join("backup-restore/snapshots");
        fs::create_dir_all(&snapshot_dir).unwrap();
        let mut commands = BackupCommandState::new(&root);
        let destination = root.join("selected");
        let destination_token = commands.issue_test_destination(destination.clone(), Instant::now() + Duration::from_secs(30));
        let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();
        let mut cleanup = RecordingCleanup::default();

        let response = create_backup_with_publisher(
            &state,
            &mut commands,
            CreateBackupRequest { destination_token },
            &mut cleanup,
            |snapshot, published_to, file_name| {
                assert!(snapshot.is_file());
                fs::create_dir_all(published_to).unwrap();
                fs::copy(snapshot, published_to.join(file_name)).unwrap();
                Ok(PublishedBackup { path: published_to.join(file_name), size_bytes: fs::metadata(snapshot).unwrap().len(), sha256: "verified".into(), durability_warning: false })
            },
        );

        assert!(matches!(response, BackupResponse::Created { .. }));
        assert_eq!(cleanup.calls.first(), Some(&"create_evidence"));
        assert_eq!(fs::read_dir(&snapshot_dir).unwrap().count(), 0);
        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn aged_snapshot_pruning_fails_closed_during_backup_creation() {
        let root = test_directory();
        let snapshot_dir = root.join("backup-restore/snapshots");
        fs::create_dir_all(&snapshot_dir).unwrap();
        let snapshot = snapshot_dir.join(format!("1-{}.sqlite3", uuid::Uuid::new_v4()));
        fs::write(&snapshot, b"abandoned snapshot").unwrap();
        let old = SystemTime::now() - Duration::from_secs(8 * 24 * 60 * 60);
        fs::File::options().write(true).open(&snapshot).unwrap().set_times(std::fs::FileTimes::new().set_modified(old)).unwrap();
        let mut commands = BackupCommandState::new(&root);
        let destination_token = commands.issue_test_destination(PathBuf::from("/selected"), Instant::now() + Duration::from_secs(30));
        let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();
        let mut cleanup = RecordingCleanup { fail: Some("remove"), ..Default::default() };

        let response = create_backup_with_publisher(
            &state,
            &mut commands,
            CreateBackupRequest { destination_token },
            &mut cleanup,
            |_, _, _| panic!("publication must not run while pruning fails"),
        );
        assert!(matches!(response, BackupResponse::Error { code: "storage_unavailable", .. }));
        assert!(snapshot.exists());
        assert_eq!(fs::read(snapshot.with_file_name(format!(".{}.cleanup-needed", snapshot.file_name().unwrap().to_string_lossy()))).unwrap(), b"restore-stage-cleanup-required\n");
        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn expired_stage_pruning_fails_closed_during_picker_selection() {
        use std::os::unix::fs::symlink;

        let root = test_directory();
        let stage = root.join("backup-restore/staging/expired.sqlite3");
        let target = root.join("target");
        fs::write(&target, b"retained stage target").unwrap();
        symlink(&target, &stage).unwrap();
        let mut commands = BackupCommandState::new(&root);
        let token = uuid::Uuid::new_v4().to_string();
        commands.pending.insert(token, PendingRestore { stage: stage.clone(), sha256: String::new(), expires_at: Instant::now() - Duration::from_secs(1) });

        assert!(matches!(commands.select_restore_source(PathBuf::from("/selected.sqlite3")), RestoreSourceSelection::Error { code: "storage_unavailable", .. }));
        assert!(fs::symlink_metadata(&stage).unwrap().file_type().is_symlink());
        assert_eq!(fs::read(&target).unwrap(), b"retained stage target");
        assert_eq!(fs::read(stage.with_file_name(".expired.sqlite3.cleanup-needed")).unwrap(), b"restore-stage-cleanup-required\n");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cleanup_failure_matrix_stops_at_failure_and_retains_bounded_evidence() {
        for (failure, expected) in [
            ("remove", vec!["create_evidence", "write_evidence", "sync_evidence", "sync_parent", "remove"]),
            ("sync_parent", vec!["create_evidence", "write_evidence", "sync_evidence", "sync_parent"]),
            ("create_evidence", vec!["create_evidence"]),
            ("write_evidence", vec!["create_evidence", "write_evidence"]),
            ("sync_evidence", vec!["create_evidence", "write_evidence", "sync_evidence"]),
        ] {
            let root = test_directory();
            let stage = root.join("stage.sqlite3");
            fs::write(&stage, b"stage").unwrap();
            let mut cleanup = RecordingCleanup { fail: Some(failure), ..Default::default() };
            assert!(!remove_stage_with_evidence_using(&stage, &mut cleanup), "{failure}");
            assert_eq!(cleanup.calls, expected, "{failure}");
            let evidence = root.join(".stage.sqlite3.cleanup-needed");
            assert!(stage.exists(), "failed before deletion: {failure}");
            if failure == "remove" || failure == "write_evidence" || failure == "sync_evidence" || failure == "sync_parent" { assert!(evidence.exists()); }
            if failure == "create_evidence" { assert!(!evidence.exists()); }
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[cfg(unix)]
    #[test]
    fn failed_stage_removal_retains_cleanup_evidence_and_reports_failure() {
        use std::os::unix::fs::symlink;

        let root = test_directory();
        let target = root.join("target");
        fs::write(&target, b"must not be followed or removed").unwrap();
        let stage = root.join("stage.sqlite3");
        symlink(&target, &stage).unwrap();

        assert!(!remove_stage_with_evidence(&stage));
        assert!(fs::symlink_metadata(&stage).unwrap().file_type().is_symlink());
        assert_eq!(fs::read(&target).unwrap(), b"must not be followed or removed");
        assert_eq!(
            fs::read(root.join(".stage.sqlite3.cleanup-needed")).unwrap(),
            b"restore-stage-cleanup-required\n",
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn startup_cleanup_reconciliation_retries_and_removes_evidence_only_after_cleanup() {
        let root = test_directory();
        let stage = root.join("retry.sqlite3");
        let evidence = root.join(".retry.sqlite3.cleanup-needed");
        fs::write(&stage, b"abandoned").unwrap();
        fs::write(&evidence, b"restore-stage-cleanup-required\n").unwrap();

        assert!(reconcile_cleanup_evidence(&root));
        assert!(!stage.exists());
        assert!(!evidence.exists());
        fs::remove_dir_all(root).unwrap();
    }

    struct RemoveEvidenceDuringArtifactSync {
        record: PathBuf,
        calls: Vec<&'static str>,
    }

    impl StageCleanup for RemoveEvidenceDuringArtifactSync {
        fn remove(&mut self, path: &Path) -> std::io::Result<()> {
            self.calls.push("remove");
            match fs::symlink_metadata(path) {
                Ok(metadata) if metadata.file_type().is_file() => fs::remove_file(path),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                _ => Err(std::io::ErrorKind::InvalidData.into()),
            }
        }

        fn sync_parent(&mut self, path: &Path) -> std::io::Result<()> {
            self.calls.push("sync_parent");
            if path != self.record {
                fs::remove_file(&self.record)?;
            }
            sync_parent_directory(path)
        }

        fn create_evidence(&mut self, path: &Path) -> std::io::Result<()> {
            self.calls.push("create_evidence");
            OpenOptions::new().write(true).create_new(true).open(path).map(drop)
        }

        fn write_evidence(&mut self, path: &Path) -> std::io::Result<()> {
            self.calls.push("write_evidence");
            use std::io::Write;
            OpenOptions::new().write(true).open(path)?.write_all(b"restore-stage-cleanup-required\n")
        }

        fn sync_evidence(&mut self, path: &Path) -> std::io::Result<()> {
            self.calls.push("sync_evidence");
            OpenOptions::new().write(true).open(path)?.sync_all()
        }
    }

    #[test]
    fn startup_reconciliation_recreates_evidence_after_metadata_inspection_failure() {
        let root = test_directory();
        let stage = root.join("retry.sqlite3");
        let evidence = root.join(".retry.sqlite3.cleanup-needed");
        fs::write(&stage, b"abandoned").unwrap();
        fs::write(&evidence, b"restore-stage-cleanup-required\n").unwrap();
        let mut cleanup = RemoveEvidenceDuringArtifactSync {
            record: evidence.clone(),
            calls: Vec::new(),
        };

        assert!(!reconcile_cleanup_evidence_using(&root, &mut cleanup));
        assert!(!stage.exists());
        assert_eq!(fs::read(&evidence).unwrap(), b"restore-stage-cleanup-required\n");
        assert_eq!(
            cleanup.calls,
            vec!["remove", "sync_parent", "create_evidence", "write_evidence", "sync_evidence", "sync_parent"],
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn startup_reconciliation_injected_failure_keeps_evidence_and_artifact() {
        let root = test_directory();
        let stage = root.join("retry.sqlite3");
        let evidence = root.join(".retry.sqlite3.cleanup-needed");
        fs::write(&stage, b"abandoned").unwrap();
        fs::write(&evidence, b"restore-stage-cleanup-required\n").unwrap();
        let mut cleanup = RecordingCleanup { fail: Some("remove"), ..Default::default() };

        assert!(!reconcile_cleanup_evidence_using(&root, &mut cleanup));
        assert!(stage.exists());
        assert!(evidence.exists());
        assert!(!BackupStore::new(&root).read_restore_state().unwrap_or_default().is_some());
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn startup_cleanup_failure_retains_evidence_and_fails_closed() {
        use std::os::unix::fs::symlink;

        let root = test_directory();
        let target = root.join("target");
        let stage = root.join("retry.sqlite3");
        let evidence = root.join(".retry.sqlite3.cleanup-needed");
        fs::write(&target, b"source evidence").unwrap();
        symlink(&target, &stage).unwrap();
        fs::write(&evidence, b"restore-stage-cleanup-required\n").unwrap();

        assert!(!reconcile_cleanup_evidence(&root));
        assert!(fs::symlink_metadata(&stage).unwrap().file_type().is_symlink());
        assert_eq!(fs::read(&target).unwrap(), b"source evidence");
        assert!(evidence.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn staging_count_limit_fails_closed_without_creating_another_stage() {
        let root = test_directory();
        let staging = root.join("backup-restore/staging");
        for _ in 0..MAX_PENDING_RESTORES {
            fs::write(staging.join(format!("{}.sqlite3", uuid::Uuid::new_v4())), b"bounded").unwrap();
        }
        let mut commands = BackupCommandState::new(&root);
        let invalid = root.join("invalid.sqlite3");
        fs::write(&invalid, b"not a database").unwrap();
        let source_token = commands.issue_test_source(invalid, Instant::now() + Duration::from_secs(30));
        let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();

        assert!(matches!(prepare_restore(&state, &mut commands, PrepareRestoreRequest { source_token }), BackupResponse::Error { code: "storage_unavailable", .. }));
        assert_eq!(fs::read_dir(staging).unwrap().count(), MAX_PENDING_RESTORES);
        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn invalid_checksum_consumes_token_and_removes_stage() {
        let root = test_directory();
        let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();
        let source = root.join("source.sqlite3");
        state.with_read(|connection| crate::infrastructure::sqlite::create_snapshot(connection, &source).map(|_| ()).map_err(|_| "snapshot".into())).unwrap();
        let mut commands = BackupCommandState::new(&root);
        let source_token = commands.issue_test_source(source, Instant::now() + Duration::from_secs(30));
        let BackupResponse::Prepared { token, .. } = prepare_restore(&state, &mut commands, PrepareRestoreRequest { source_token }) else { panic!("valid source should be staged"); };
        let stage = commands.pending[&token].stage.clone();
        fs::write(&stage, b"changed after validation").unwrap();

        assert!(matches!(confirm_restore(&state, &mut commands, ConfirmRestoreRequest { token: token.clone(), confirmed: true }), BackupResponse::Error { code: "invalid_backup", .. }));
        assert!(!stage.exists());
        assert!(matches!(confirm_restore(&state, &mut commands, ConfirmRestoreRequest { token, confirmed: true }), BackupResponse::Error { code: "token_invalid", .. }));
        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn expired_and_unconfirmed_restore_tokens_reclaim_their_stages_and_cannot_replay() {
        let root = test_directory();
        let expired_stage = root.join("backup-restore/staging/expired.sqlite3");
        let cancelled_stage = root.join("backup-restore/staging/cancelled.sqlite3");
        fs::write(&expired_stage, b"expired").unwrap();
        fs::write(&cancelled_stage, b"cancelled").unwrap();
        let mut commands = BackupCommandState::new(&root);
        let expired = uuid::Uuid::new_v4().to_string();
        commands.pending.insert(expired.clone(), PendingRestore { stage: expired_stage.clone(), sha256: String::new(), expires_at: Instant::now() - Duration::from_secs(1) });
        let cancelled = uuid::Uuid::new_v4().to_string();
        commands.pending.insert(cancelled.clone(), PendingRestore { stage: cancelled_stage.clone(), sha256: String::new(), expires_at: Instant::now() + Duration::from_secs(30) });
        let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();

        assert!(matches!(confirm_restore(&state, &mut commands, ConfirmRestoreRequest { token: expired.clone(), confirmed: true }), BackupResponse::Error { code: "token_expired", .. }));
        assert!(!expired_stage.exists());
        assert!(matches!(confirm_restore(&state, &mut commands, ConfirmRestoreRequest { token: cancelled.clone(), confirmed: false }), BackupResponse::Error { code: "confirmation_required", .. }));
        assert!(!cancelled_stage.exists());
        assert!(matches!(confirm_restore(&state, &mut commands, ConfirmRestoreRequest { token: expired, confirmed: true }), BackupResponse::Error { code: "token_invalid", .. }));
        assert!(matches!(confirm_restore(&state, &mut commands, ConfirmRestoreRequest { token: cancelled, confirmed: true }), BackupResponse::Error { code: "token_invalid", .. }));
        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(not(windows))]
    #[test]
    fn failed_installation_cleans_stage_when_durable_restore_did_not_take_ownership() {
        let root = test_directory();
        let stage = root.join("backup-restore/staging/stage.sqlite3");
        fs::write(&stage, b"bad stage").unwrap();
        let mut commands = BackupCommandState::new(&root);
        let token = uuid::Uuid::new_v4().to_string();
        commands.pending.insert(token.clone(), PendingRestore { stage: stage.clone(), sha256: checksum(&stage).unwrap(), expires_at: Instant::now() + Duration::from_secs(30) });
        let state = DatabaseState::open(crate::infrastructure::sqlite::production_database_config(&root)).unwrap();
        assert!(matches!(confirm_restore(&state, &mut commands, ConfirmRestoreRequest { token, confirmed: true }), BackupResponse::Error { code: "restore_failed", .. }));
        assert!(!stage.exists());
        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(not(windows))]
    #[test]
    fn unverified_host_destination_is_not_issued_a_token() {
        let directory = std::env::temp_dir();
        let mut state = BackupCommandState::new("/app");
        assert!(matches!(
            state.select_backup_destination(directory),
            BackupDestinationSelection::Error { code: "unsupported_destination", .. }
        ));
        assert!(state.destinations.is_empty());
    }
}
