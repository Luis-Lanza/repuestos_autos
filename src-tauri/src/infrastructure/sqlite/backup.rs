use std::fs;
use std::path::Path;
use std::time::Duration;

use rusqlite::{backup::Backup, Connection, OpenFlags};

use super::{
    migrate_if_needed, validate_foreign_keys, CURRENT_SCHEMA_VERSION,
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
        if let Err(error) = fs::create_dir_all(parent) {
            report_io_failure("snapshot_directory_create", &error);
            return Err(BackupValidationError::InvalidBackup);
        }
    }
    let mut destination = match Connection::open(snapshot) {
        Ok(destination) => destination,
        Err(error) => {
            report_sqlite_failure("snapshot_open", &error);
            return Err(BackupValidationError::InvalidBackup);
        }
    };
    if let Err(error) = Backup::new(source, &mut destination)
        .and_then(|backup| backup.run_to_completion(128, Duration::from_millis(1), None))
    {
        report_sqlite_failure("snapshot_copy", &error);
        return Err(BackupValidationError::InvalidBackup);
    }
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
    validate_restored_database(&destination)?;
    metadata(&destination)
}

pub fn validate_restored_database(connection: &Connection) -> Result<(), BackupValidationError> {
    let version = metadata(connection)?.schema_version;
    if version != CURRENT_SCHEMA_VERSION {
        return Err(BackupValidationError::InvalidBackup);
    }
    super::validate_version_twenty_three_schema(connection)
        .map_err(|_| BackupValidationError::InvalidBackup)?;
    validate_foreign_keys(connection).map_err(|_| BackupValidationError::InvalidBackup)
}

fn metadata(connection: &Connection) -> Result<DatabaseMetadata, BackupValidationError> {
    let integrity: String = match connection.query_row("PRAGMA integrity_check", [], |row| row.get(0)) {
        Ok(integrity) => integrity,
        Err(error) => {
            report_sqlite_failure("snapshot_integrity_metadata", &error);
            return Err(BackupValidationError::InvalidBackup);
        }
    };
    if integrity != "ok" {
        report_unclassified_failure("snapshot_integrity_metadata");
        return Err(BackupValidationError::InvalidBackup);
    }
    match connection.query_row("PRAGMA user_version", [], |row| row.get(0)) {
        Ok(schema_version) => Ok(DatabaseMetadata { schema_version }),
        Err(error) => {
            report_sqlite_failure("snapshot_schema_metadata", &error);
            Err(BackupValidationError::InvalidBackup)
        }
    }
}

#[cfg(all(windows, debug_assertions))]
fn report_io_failure(operation: &str, error: &std::io::Error) {
    let (os_code, kind) = diagnostic_os_fields(error);
    eprintln!(
        "backup_diagnostic operation={operation} os_code={} kind={kind:?}",
        os_code.map_or_else(|| "none".to_string(), |code| code.to_string()),
    );
}

#[cfg(not(all(windows, debug_assertions)))]
fn report_io_failure(_operation: &str, _error: &std::io::Error) {}

#[cfg(all(windows, debug_assertions))]
fn report_sqlite_failure(operation: &str, _error: &rusqlite::Error) {
    // rusqlite does not expose the originating Windows system error separately.
    eprintln!("backup_diagnostic operation={operation} os_code=none kind=Other");
}

#[cfg(not(all(windows, debug_assertions)))]
fn report_sqlite_failure(_operation: &str, _error: &rusqlite::Error) {}

#[cfg(all(windows, debug_assertions))]
fn report_unclassified_failure(operation: &str) {
    eprintln!("backup_diagnostic operation={operation} os_code=none kind=Other");
}

#[cfg(not(all(windows, debug_assertions)))]
fn report_unclassified_failure(_operation: &str) {}

#[cfg(any(test, all(windows, debug_assertions)))]
fn diagnostic_os_fields(error: &std::io::Error) -> (Option<i32>, std::io::ErrorKind) {
    (error.raw_os_error(), error.kind())
}

#[cfg(test)]
mod diagnostic_tests {
    #[test]
    fn diagnostic_classification_keeps_only_os_code_and_error_kind() {
        let error = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        assert_eq!(super::diagnostic_os_fields(&error), (None, std::io::ErrorKind::PermissionDenied));
        #[cfg(windows)]
        {
            let error = std::io::Error::from_raw_os_error(5);
            assert_eq!(super::diagnostic_os_fields(&error), (Some(5), std::io::ErrorKind::PermissionDenied));
        }
    }
}
