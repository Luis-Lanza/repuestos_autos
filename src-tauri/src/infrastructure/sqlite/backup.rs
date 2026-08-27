use std::fs;
use std::path::Path;
use std::time::Duration;

use rusqlite::{backup::Backup, Connection, OpenFlags};

use super::{
    migrate_if_needed, validate_foreign_keys, validate_version_five_schema, CURRENT_SCHEMA_VERSION,
};

#[derive(Debug, PartialEq, Eq)]
pub enum BackupValidationError {
    InvalidBackup,
    UnsupportedSchema,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DatabaseMetadata {
    pub schema_version: i64,
}

pub fn create_snapshot(
    source: &Connection,
    snapshot: &Path,
) -> Result<DatabaseMetadata, BackupValidationError> {
    if let Some(parent) = snapshot.parent() {
        fs::create_dir_all(parent).map_err(|_| BackupValidationError::InvalidBackup)?;
    }
    let mut destination =
        Connection::open(snapshot).map_err(|_| BackupValidationError::InvalidBackup)?;
    Backup::new(source, &mut destination)
        .and_then(|backup| backup.run_to_completion(128, Duration::from_millis(1), None))
        .map_err(|_| BackupValidationError::InvalidBackup)?;
    metadata(&destination)
}

pub fn stage_and_validate(
    source: &Path,
    stage: &Path,
) -> Result<DatabaseMetadata, BackupValidationError> {
    let source = Connection::open_with_flags(source, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| BackupValidationError::InvalidBackup)?;
    if let Some(parent) = stage.parent() {
        fs::create_dir_all(parent).map_err(|_| BackupValidationError::InvalidBackup)?;
    }
    let mut destination =
        Connection::open(stage).map_err(|_| BackupValidationError::InvalidBackup)?;
    Backup::new(&source, &mut destination)
        .and_then(|backup| backup.run_to_completion(128, Duration::from_millis(1), None))
        .map_err(|_| BackupValidationError::InvalidBackup)?;
    let version = metadata(&destination)?.schema_version;
    if !(1..=CURRENT_SCHEMA_VERSION).contains(&version) {
        return Err(BackupValidationError::UnsupportedSchema);
    }
    migrate_if_needed(&mut destination).map_err(|_| BackupValidationError::InvalidBackup)?;
    validate_version_five_schema(&destination).map_err(|_| BackupValidationError::InvalidBackup)?;
    validate_foreign_keys(&destination).map_err(|_| BackupValidationError::InvalidBackup)?;
    metadata(&destination)
}

pub fn validate_restored_database(connection: &Connection) -> Result<(), BackupValidationError> {
    metadata(connection)?;
    validate_version_five_schema(connection).map_err(|_| BackupValidationError::InvalidBackup)?;
    validate_foreign_keys(connection).map_err(|_| BackupValidationError::InvalidBackup)
}

fn metadata(connection: &Connection) -> Result<DatabaseMetadata, BackupValidationError> {
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|_| BackupValidationError::InvalidBackup)?;
    if integrity != "ok" {
        return Err(BackupValidationError::InvalidBackup);
    }
    connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map(|schema_version| DatabaseMetadata { schema_version })
        .map_err(|_| BackupValidationError::InvalidBackup)
}
