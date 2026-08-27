use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use repuestos_autos::infrastructure::filesystem::{BackupStore, StorageError};
use repuestos_autos::infrastructure::sqlite::{
    create_snapshot, stage_and_validate, BackupValidationError, CURRENT_SCHEMA_VERSION,
};
use rusqlite::Connection;

const LEGACY: &str = include_str!("fixtures/version1_fixed_price_legacy.sql");
const MIGRATIONS: [&str; 5] = [
    include_str!("../src/infrastructure/sqlite/migrations/0002_fixed_price_checkout.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0003_sale_line_product_snapshots.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0004_product_onboarding.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0005_catalog_onboarding_hardening.sql"),
    include_str!("../src/infrastructure/sqlite/migrations/0006_operational_inventory_control.sql"),
];

fn temporary_directory(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("r-a-{name}-{}", std::process::id()))
}

fn versioned_database(path: &Path, version: i64) {
    let connection = Connection::open(path).unwrap();
    connection.execute_batch(LEGACY).unwrap();
    for (index, migration) in MIGRATIONS.iter().enumerate().take((version - 1) as usize) {
        connection.execute_batch(migration).unwrap();
        connection
            .pragma_update(None, "user_version", (index + 2) as i64)
            .unwrap();
    }
}

#[test]
fn publishes_a_synced_non_overwriting_backup_to_native_paths() {
    let directory = temporary_directory("publish");
    let snapshot = directory.join("snapshot.sqlite3");
    let destination = directory.join("USB á");
    fs::create_dir_all(&destination).unwrap();
    fs::write(&snapshot, b"consistent snapshot").unwrap();

    let store = BackupStore::new(&directory.join("app-data"));
    let published = store
        .publish_snapshot(&snapshot, &destination, "backup-20260827T204000Z.sqlite3")
        .unwrap();

    assert_eq!(fs::read(&published.path).unwrap(), b"consistent snapshot");
    assert!(!destination
        .join("backup-20260827T204000Z.sqlite3.part")
        .exists());
    assert_eq!(
        store
            .publish_snapshot(&snapshot, &destination, "backup-20260827T204000Z.sqlite3")
            .unwrap_err(),
        StorageError::DestinationExists
    );
    assert_eq!(
        store
            .publish_selected_snapshot(None, &destination, "backup-20260827T204000Z.sqlite3")
            .unwrap_err(),
        StorageError::SelectionCancelled
    );
    assert_eq!(
        store
            .publish_snapshot(&snapshot, &destination, "not-a-backup")
            .unwrap_err(),
        StorageError::StorageUnavailable
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn stages_each_supported_schema_without_mutating_the_selected_source() {
    let directory = temporary_directory("stage");
    fs::create_dir_all(&directory).unwrap();
    for version in 1..=CURRENT_SCHEMA_VERSION {
        let source = directory.join(format!("v{version}.sqlite3"));
        let stage = directory.join(format!("stage-v{version}.sqlite3"));
        versioned_database(&source, version);
        let before = fs::read(&source).unwrap();

        let metadata = stage_and_validate(&source, &stage).unwrap();

        assert_eq!(metadata.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(fs::read(&source).unwrap(), before);
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejects_invalid_and_unsupported_candidates_without_source_mutation() {
    let directory = temporary_directory("invalid");
    fs::create_dir_all(&directory).unwrap();
    let non_sqlite = directory.join("not-a-database.sqlite3");
    fs::write(&non_sqlite, b"not sqlite").unwrap();
    let before = fs::read(&non_sqlite).unwrap();
    assert_eq!(
        stage_and_validate(&non_sqlite, &directory.join("stage.sqlite3")).unwrap_err(),
        BackupValidationError::InvalidBackup
    );
    assert_eq!(fs::read(&non_sqlite).unwrap(), before);

    let zero = directory.join("zero.sqlite3");
    Connection::open(&zero).unwrap();
    assert_eq!(
        stage_and_validate(&zero, &directory.join("zero-stage.sqlite3")).unwrap_err(),
        BackupValidationError::UnsupportedSchema
    );

    let future = directory.join("future.sqlite3");
    versioned_database(&future, 1);
    Connection::open(&future)
        .unwrap()
        .pragma_update(None, "user_version", CURRENT_SCHEMA_VERSION + 1)
        .unwrap();
    assert_eq!(
        stage_and_validate(&future, &directory.join("future-stage.sqlite3")).unwrap_err(),
        BackupValidationError::UnsupportedSchema
    );

    for (name, mutation) in [
        ("foreign-key", "DELETE FROM products WHERE id = 1;"),
        ("structural", "DROP TABLE categories;"),
    ] {
        let source = directory.join(format!("{name}.sqlite3"));
        versioned_database(&source, CURRENT_SCHEMA_VERSION);
        let connection = Connection::open(&source).unwrap();
        connection
            .pragma_update(None, "foreign_keys", false)
            .unwrap();
        connection.execute_batch(mutation).unwrap();
        let before = fs::read(&source).unwrap();
        assert_eq!(
            stage_and_validate(&source, &directory.join(format!("{name}-stage.sqlite3")))
                .unwrap_err(),
            BackupValidationError::InvalidBackup
        );
        assert_eq!(fs::read(&source).unwrap(), before);
    }
    let corrupt = directory.join("corrupt.sqlite3");
    fs::write(&corrupt, b"SQLite format 3\0corrupt").unwrap();
    assert_eq!(
        stage_and_validate(&corrupt, &directory.join("corrupt-stage.sqlite3")).unwrap_err(),
        BackupValidationError::InvalidBackup
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn creates_a_consistent_snapshot_before_releasing_the_live_database_mutex() {
    let directory = temporary_directory("snapshot");
    fs::create_dir_all(&directory).unwrap();
    let live_path = directory.join("live.sqlite3");
    versioned_database(&live_path, CURRENT_SCHEMA_VERSION);
    let live = Mutex::new(Connection::open(&live_path).unwrap());
    let snapshot = directory.join("snapshot.sqlite3");

    let metadata = {
        let connection = live.lock().unwrap();
        create_snapshot(&connection, &snapshot).unwrap()
    };
    assert!(live.try_lock().is_ok());
    assert_eq!(metadata.schema_version, CURRENT_SCHEMA_VERSION);
    assert_eq!(
        Connection::open(&snapshot)
            .unwrap()
            .query_row("SELECT COUNT(*) FROM sales", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    fs::remove_dir_all(directory).unwrap();
}
