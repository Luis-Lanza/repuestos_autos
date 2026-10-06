#[cfg(feature = "desktop")]
use tauri::{Manager, Runtime};
#[cfg(all(feature = "desktop", not(test)))]
use tauri_plugin_dialog::DialogExt;

pub mod application;
pub mod commands;
pub mod domain;
pub mod infrastructure;

use std::sync::Mutex;

use infrastructure::{
    filesystem::BackupStore,
    sqlite::{create_snapshot, open_database, validate_restored_database, DatabaseConfig},
};
use rusqlite::OpenFlags;

pub use infrastructure::filesystem::RestoreState;

#[derive(Clone, Copy, PartialEq, Eq)]
enum DatabaseStatus {
    Ready,
    Restoring,
    Unavailable,
}

struct DatabaseStateInner {
    config: DatabaseConfig,
    connection: Option<rusqlite::Connection>,
    status: DatabaseStatus,
}

pub struct DatabaseState(Mutex<DatabaseStateInner>);

impl DatabaseState {
    pub fn open(config: DatabaseConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let connection = open_database(&config)?;
        Ok(Self::from_connection(config, connection))
    }

    pub fn from_connection(config: DatabaseConfig, connection: rusqlite::Connection) -> Self {
        Self(Mutex::new(DatabaseStateInner {
            config,
            connection: Some(connection),
            status: DatabaseStatus::Ready,
        }))
    }

    pub fn recover_on_startup(config: DatabaseConfig, store: &BackupStore) -> Self {
        let recovery_evidence = has_recovery_evidence(config.path(), store);
        match store.read_restore_state() {
            Ok(None) if recovery_evidence => {
                if has_ambiguous_temporary_artifacts(config.path())
                    || !retained_recovery_evidence_is_valid(config.path())
                    || !is_valid_recovery_evidence(config.path())
                {
                    return Self::unavailable(config);
                }
                // All retained evidence was classified read-only before canonical migration.
                match infrastructure::sqlite::open_recovery_database(&config) {
                    Ok(connection) => Self::from_connection(config, connection),
                    Err(_) => Self::unavailable(config),
                }
            }
            Ok(None) => {
                match open_database(&config).map_err(|_| ()).and_then(|connection| {
                    validate_restored_database(&connection).map_err(|_| ())?;
                    Ok(connection)
                }) {
                    Ok(connection) => {
                        if !cleanup_abandoned_restore_artifacts(config.path()) {
                            return Self::unavailable(config);
                        }
                        Self::from_connection(config, connection)
                    }
                    Err(_) => Self::unavailable(config),
                }
            }
            Ok(_) if has_ambiguous_recovery_artifacts(config.path()) => Self::unavailable(config),
            Ok(marker) => {
                let canonical_was_valid = is_valid_database(config.path());
                match open_validated_recovery_database(&config, store, marker) {
                    Ok(connection) => {
                        let marker_completed = !canonical_was_valid || cfg!(windows);
                        if marker_completed && store.complete_durable_restore().is_err() {
                            return Self::unavailable(config);
                        }
                        if marker_completed
                            && (!reconcile_abandoned_cleanup(config.path())
                                || !cleanup_abandoned_stages(config.path()))
                        {
                            return Self::unavailable(config);
                        }
                        Self::from_connection(config, connection)
                    }
                    Err(()) => Self::unavailable(config),
                }
            }
            Err(_) => Self::unavailable(config),
        }
    }

    fn unavailable(config: DatabaseConfig) -> Self {
        Self(Mutex::new(DatabaseStateInner {
            config,
            connection: None,
            status: DatabaseStatus::Unavailable,
        }))
    }

    pub fn with_read<T>(
        &self,
        operation: impl FnOnce(&rusqlite::Connection) -> Result<T, String>,
    ) -> Result<T, String> {
        let state = self.0.lock().map_err(|_| "persistence_failure")?;
        if state.status != DatabaseStatus::Ready {
            return Err("database_unavailable".into());
        }
        operation(state.connection.as_ref().ok_or("database_unavailable")?)
    }

    pub fn with_write<T>(
        &self,
        operation: impl FnOnce(&mut rusqlite::Connection) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut state = self.0.lock().map_err(|_| "persistence_failure")?;
        if state.status != DatabaseStatus::Ready {
            return Err("database_unavailable".into());
        }
        operation(state.connection.as_mut().ok_or("database_unavailable")?)
    }

    pub fn install_validated_stage(
        &self,
        stage: &std::path::Path,
        store: &BackupStore,
    ) -> Result<(), String> {
        self.install_validated_stage_with_cleanup(stage, store, || {
            store.complete_durable_restore().map_err(|_| ())
        })
    }

    fn install_validated_stage_with_cleanup(
        &self,
        stage: &std::path::Path,
        store: &BackupStore,
        clear_restore_state: impl FnOnce() -> Result<(), ()>,
    ) -> Result<(), String> {
        let mut state = self.0.lock().map_err(|_| "persistence_failure")?;
        if state.status != DatabaseStatus::Ready {
            return Err("database_unavailable".into());
        }
        let protective = state
            .config
            .path()
            .parent()
            .ok_or("restore_failed")?
            .join("pre-restore.sqlite3");
        create_snapshot(
            state.connection.as_ref().ok_or("database_unavailable")?,
            &protective,
        )
        .map_err(|_| "restore_failed")?;
        {
            let protective_connection =
                rusqlite::Connection::open(&protective).map_err(|_| "restore_failed")?;
            validate_restored_database(&protective_connection).map_err(|_| "restore_failed")?;
        }
        store
            .prepare_durable_restore(stage, &protective)
            .map_err(|_| "restore_failed")?;
        state.status = DatabaseStatus::Restoring;
        drop(state.connection.take());

        let replacement = (|| {
            store
                .install_durable_restore(stage, state.config.path())
                .map_err(|_| ())?;
            let connection = open_database(&state.config).map_err(|_| ())?;
            validate_restored_database(&connection).map_err(|_| ())?;
            Ok::<_, ()>(connection)
        })();

        match replacement {
            Ok(connection) => {
                state.connection = Some(connection);
                state.status = DatabaseStatus::Ready;
                clear_restore_state().map_err(|_| "restore_failed".to_string())
            }
            Err(()) => match store.read_restore_state() {
                Ok(marker) => match open_validated_recovery_database(&state.config, store, marker) {
                    Ok(connection) => {
                        state.connection = Some(connection);
                        state.status = DatabaseStatus::Ready;
                        Err("restore_failed".into())
                    }
                    Err(()) => {
                        state.connection = None;
                        state.status = DatabaseStatus::Unavailable;
                        Err("database_unavailable".into())
                    }
                },
                Err(_) => {
                    state.connection = None;
                    state.status = DatabaseStatus::Unavailable;
                    Err("database_unavailable".into())
                }
            },
        }
    }
}

fn open_validated_recovery_database(
    config: &DatabaseConfig,
    store: &BackupStore,
    marker: Option<RestoreState>,
) -> Result<rusqlite::Connection, ()> {
    let canonical = config.path();
    if !is_valid_database(canonical) {
        let rollback = canonical.with_file_name("restore-rollback.sqlite3");
        let protective = canonical.parent().ok_or(())?.join("pre-restore.sqlite3");
        let candidates: [&std::path::Path; 2] = match marker {
            Some(RestoreState::Prepared) => [&protective, &rollback],
            Some(RestoreState::LiveMoved | RestoreState::CandidateInstalled) | None => {
                [&rollback, &protective]
            }
        };
        let staging = canonical
            .parent()
            .ok_or(())?
            .join("backup-restore/staging");
        std::fs::create_dir_all(&staging).map_err(|_| ())?;
        let mut recovered = false;
        for source in candidates {
            let stage = staging.join(format!("{}.sqlite3", uuid::Uuid::new_v4()));
            match infrastructure::sqlite::stage_and_validate(source, &stage) {
                Ok(_) => {
                    store
                        .recover_canonical_durably(&stage, canonical)
                        .map_err(|_| ())?;
                    recovered = true;
                    break;
                }
                Err(_) => {
                    if !commands::backup::remove_stage_with_evidence(&stage) {
                        return Err(());
                    }
                }
            }
        }
        if !recovered {
            return Err(());
        }
    }
    open_existing_validated_database(config)
}

fn open_existing_validated_database(
    config: &DatabaseConfig,
) -> Result<rusqlite::Connection, ()> {
    let connection = rusqlite::Connection::open_with_flags(
        config.path(),
        OpenFlags::SQLITE_OPEN_READ_WRITE,
    )
    .map_err(|_| ())?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|_| ())?;
    validate_restored_database(&connection).map_err(|_| ())?;
    Ok(connection)
}

fn path_has_entry_or_error(path: &std::path::Path) -> bool {
    match std::fs::symlink_metadata(path) {
        Ok(_) => true,
        Err(error) => error.kind() != std::io::ErrorKind::NotFound,
    }
}

fn has_ambiguous_temporary_artifacts(canonical: &std::path::Path) -> bool {
    let Some(root) = canonical.parent() else { return true };
    [
        root.join("restore-state.json.part"),
        root.join("restore-recovery.sqlite3.part"),
    ]
    .iter()
    .any(|path| path_has_entry_or_error(path))
}

fn has_ambiguous_recovery_artifacts(canonical: &std::path::Path) -> bool {
    let Some(root) = canonical.parent() else { return true };
    has_ambiguous_temporary_artifacts(canonical)
        || match recovery_sidecar_entries(root) {
            Ok(entries) => !entries.is_empty(),
            Err(()) => true,
        }
}

fn recovery_sidecar_entries(root: &std::path::Path) -> Result<Vec<std::path::PathBuf>, ()> {
    let mut sidecars = Vec::new();
    for entry in std::fs::read_dir(root).map_err(|_| ())? {
        let entry = entry.map_err(|_| ())?;
        let name = entry.file_name();
        if name.to_string_lossy().starts_with("restore-state.json.previous-") {
            sidecars.push(entry.path());
        }
    }
    Ok(sidecars)
}

fn retained_recovery_evidence_is_valid(canonical: &std::path::Path) -> bool {
    let Some(root) = canonical.parent() else { return false };
    let sidecars = match recovery_sidecar_entries(root) {
        Ok(sidecars) => sidecars,
        Err(()) => return false,
    };
    for path in sidecars {
        let Some(slot) = path.file_name().and_then(|name| name.to_str())
            .and_then(|name| name.strip_prefix("restore-state.json.previous-"))
            .and_then(|slot| slot.parse::<u8>().ok())
        else {
            return false;
        };
        if slot >= 8 {
            return false;
        }
        let Ok(metadata) = std::fs::symlink_metadata(&path) else { return false };
        if !metadata.file_type().is_file() {
            return false;
        }
        let Ok(bytes) = std::fs::read(&path) else { return false };
        if !bytes.is_empty()
            && ![
                br#"{"state":"prepared"}"#.as_slice(),
                br#"{"state":"live_moved"}"#.as_slice(),
                br#"{"state":"candidate_installed"}"#.as_slice(),
            ].contains(&bytes.as_slice())
        {
            return false;
        }
    }
    for name in ["restore-rollback.sqlite3", "pre-restore.sqlite3"] {
        let path = root.join(name);
        match std::fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_file() && is_valid_recovery_evidence(&path) => {}
            Ok(_) => return false,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return false,
        }
    }
    true
}

fn has_recovery_evidence(canonical: &std::path::Path, store: &BackupStore) -> bool {
    let Some(root) = canonical.parent() else { return true };
    [
        root.join("restore-rollback.sqlite3"),
        root.join("pre-restore.sqlite3"),
        root.join("restore-state.json.part"),
        root.join("restore-recovery.sqlite3.part"),
    ]
    .iter()
    .any(|path| path_has_entry_or_error(path))
        || has_ambiguous_recovery_artifacts(canonical)
        || store.read_restore_state().is_err()
}

fn cleanup_abandoned_stages(canonical: &std::path::Path) -> bool {
    cleanup_abandoned_stages_using(canonical, &mut commands::backup::RealStageCleanup)
}

fn cleanup_abandoned_stages_using(canonical: &std::path::Path, cleanup: &mut impl commands::backup::StageCleanup) -> bool {
    let Some(root) = canonical.parent() else { return false };
    cleanup_abandoned_directory_using(&root.join("backup-restore/staging"), false, cleanup)
}

fn cleanup_abandoned_restore_artifacts(canonical: &std::path::Path) -> bool {
    let Some(root) = canonical.parent() else { return false };
    reconcile_abandoned_cleanup(canonical)
        && cleanup_abandoned_directory(&root.join("backup-restore/staging"), false)
        && cleanup_abandoned_directory(&root.join("backup-restore/snapshots"), true)
}

fn reconcile_abandoned_cleanup(canonical: &std::path::Path) -> bool {
    reconcile_abandoned_cleanup_using(canonical, &mut commands::backup::RealStageCleanup)
}

fn reconcile_abandoned_cleanup_using(canonical: &std::path::Path, cleanup: &mut impl commands::backup::StageCleanup) -> bool {
    let Some(root) = canonical.parent() else { return false };
    commands::backup::reconcile_cleanup_evidence_using(&root.join("backup-restore/staging"), cleanup)
        && commands::backup::reconcile_cleanup_evidence_using(&root.join("backup-restore/snapshots"), cleanup)
}

fn cleanup_abandoned_directory(directory: &std::path::Path, snapshots: bool) -> bool {
    cleanup_abandoned_directory_using(directory, snapshots, &mut commands::backup::RealStageCleanup)
}

fn cleanup_abandoned_directory_using(directory: &std::path::Path, snapshots: bool, cleanup: &mut impl commands::backup::StageCleanup) -> bool {
    // This is called only after its caller established the artifact is abandoned. In-memory
    // confirmation tokens do not survive restart, so every unowned stage is abandoned.
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return true,
        Err(_) => return false,
    };
    let mut count = 0usize;
    for entry in entries {
        let Ok(entry) = entry else { return false };
        let path = entry.path();
        match std::fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_file() => {}
            _ => return false,
        };
        let filename = path.file_name().and_then(|value| value.to_str()).unwrap_or_default();
        let recognized = if snapshots {
            filename.strip_suffix(".sqlite3").and_then(|stem| stem.split_once('-')).is_some_and(|(timestamp, id)| timestamp.parse::<u64>().is_ok() && uuid::Uuid::parse_str(id).is_ok())
        } else {
            filename.strip_suffix(".sqlite3").is_some_and(|id| uuid::Uuid::parse_str(id).is_ok())
        };
        if !recognized { return false; }
        count += 1;
        if count > 8 { return false; }
        let removed = commands::backup::remove_stage_with_evidence_using(&path, cleanup);
        if !removed { return false; }
    }
    true
}

fn is_valid_recovery_evidence(path: &std::path::Path) -> bool {
    if !std::fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.file_type().is_file())
    {
        return false;
    }
    rusqlite::Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .as_ref()
        .is_ok_and(|connection| {
            infrastructure::sqlite::backup::validate_recovery_evidence(connection).is_ok()
        })
}

fn is_valid_database(path: &std::path::Path) -> bool {
    let connection = rusqlite::Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY);
    connection
        .as_ref()
        .is_ok_and(|connection| validate_restored_database(connection).is_ok())
}

#[cfg(test)]
mod database_state_tests {
    use std::{cell::Cell, fs};

    use super::*;

    fn schema_upgrade_fixture(path: &std::path::Path) {
        let connection = rusqlite::Connection::open(path).unwrap();
        // Apply the shipped history, not a version stamp on a current schema.
        for migration in [
            include_str!("infrastructure/sqlite/migrations/0001_confirm_sale.sql"),
            include_str!("infrastructure/sqlite/migrations/0002_fixed_price_checkout.sql"),
            include_str!("infrastructure/sqlite/migrations/0003_sale_line_product_snapshots.sql"),
            include_str!("infrastructure/sqlite/migrations/0004_product_onboarding.sql"),
            include_str!("infrastructure/sqlite/migrations/0005_catalog_onboarding_hardening.sql"),
            include_str!("infrastructure/sqlite/migrations/0006_operational_inventory_control.sql"),
            include_str!("infrastructure/sqlite/migrations/0007_catalog_maintenance.sql"),
            include_str!("infrastructure/sqlite/migrations/0008_catalog_metadata_name_uniqueness.sql"),
            include_str!("infrastructure/sqlite/migrations/0009_sales_history_index.sql"),
            include_str!("infrastructure/sqlite/migrations/0010_post_sale_lifecycle.sql"),
            include_str!("infrastructure/sqlite/migrations/0011_sale_idempotency_conflicts.sql"),
            include_str!("infrastructure/sqlite/migrations/0012_inventory_idempotency_conflicts.sql"),
            include_str!("infrastructure/sqlite/migrations/0013_catalog_dual_pricing.sql"),
            include_str!("infrastructure/sqlite/migrations/0014_sale_list_price_snapshot.sql"),
            include_str!("infrastructure/sqlite/migrations/0015_catalog_price_cap.sql"),
            include_str!("infrastructure/sqlite/migrations/0016_product_images.sql"),
            include_str!("infrastructure/sqlite/migrations/0017_product_image_thumbnails.sql"),
            include_str!("infrastructure/sqlite/migrations/0018_global_product_purchase_price.sql"),
            include_str!("infrastructure/sqlite/migrations/0019_category_field_lifecycle.sql"),
            include_str!("infrastructure/sqlite/migrations/0020_product_locations.sql"),
            include_str!("infrastructure/sqlite/migrations/0021_sale_line_cost_snapshot.sql"),
            include_str!("infrastructure/sqlite/migrations/0022_product_low_stock_threshold.sql"),
            include_str!("infrastructure/sqlite/migrations/0023_remove_untouched_demo_catalog.sql"),
        ] {
            connection.execute_batch(migration).unwrap();
        }
        connection.pragma_update(None, "user_version", 23).unwrap();
        connection.execute_batch(
            "INSERT INTO categories (id, name) VALUES (41, 'Synthetic category');
             INSERT INTO products (id, category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos)
             VALUES (73, 41, 'SYN-73', 'Synthetic product', 1, 2500, 1800);
             INSERT INTO stock_balances (product_id, quantity) VALUES (73, 9);"
        ).unwrap();
        assert!(connection.execute(
            "INSERT INTO products (category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos)
             VALUES (41, 'OTHER', 'Synthetic product', 1, 2500, 1800)", []
        ).is_err(), "schema 23 must enforce unique normalized names");
        assert!(validate_restored_database(&connection).is_err(), "finalized candidates still require schema 24");
    }

    fn schema_upgrade_evidence(directory: &std::path::Path) -> Vec<(std::path::PathBuf, Vec<u8>)> {
        fs::create_dir_all(directory).unwrap();
        for name in ["restore-rollback.sqlite3", "pre-restore.sqlite3"] {
            schema_upgrade_fixture(&directory.join(name));
        }
        fs::write(directory.join("restore-state.json.previous-0"), br#"{"state":"prepared"}"#).unwrap();
        fs::write(directory.join("restore-state.json.previous-1"), br#"{"state":"candidate_installed"}"#).unwrap();
        ["restore-rollback.sqlite3", "pre-restore.sqlite3", "restore-state.json.previous-0", "restore-state.json.previous-1"]
            .iter().map(|name| {
                let path = directory.join(name);
                let bytes = fs::read(&path).unwrap();
                (path, bytes)
            }).collect()
    }

    #[test]
    fn schema_upgrade_startup_migrates_canonical_and_preserves_historical_evidence() {
        let directory = std::env::temp_dir().join(format!("r-a-schema-upgrade-{}", uuid::Uuid::new_v4()));
        let evidence = schema_upgrade_evidence(&directory);
        let config = infrastructure::sqlite::production_database_config(&directory);
        schema_upgrade_fixture(config.path());

        let recovered = DatabaseState::recover_on_startup(config, &BackupStore::new(&directory));

        let business = recovered.with_read(|connection| {
            validate_restored_database(connection).map_err(|_| "invalid_current_schema".to_string())?;
            connection.query_row(
                "SELECT p.id, p.sku, p.name, p.list_price_centavos, s.quantity
                 FROM products p JOIN stock_balances s ON s.product_id = p.id WHERE p.id = 73",
                [], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?, row.get::<_, i64>(4)?))
            ).map_err(|error| error.to_string())
        });
        assert_eq!(business, Ok((73, "SYN-73".into(), "Synthetic product".into(), 2500, 9)));
        for (path, bytes) in evidence {
            assert_eq!(fs::read(path).unwrap(), bytes);
        }
    }

    #[test]
    fn schema_upgrade_startup_accepts_current_canonical_with_old_snapshots() {
        let directory = std::env::temp_dir().join(format!("r-a-current-old-{}", uuid::Uuid::new_v4()));
        let evidence = schema_upgrade_evidence(&directory);
        let config = infrastructure::sqlite::production_database_config(&directory);
        let initialized = DatabaseState::open(config.clone()).unwrap();
        initialized.with_write(|connection| {
            connection.execute_batch(
                "INSERT INTO categories (id, name) VALUES (41, 'Synthetic category');
                 INSERT INTO products (id, category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos)
                 VALUES (73, 41, 'SYN-73', 'Repeated name', 1, 2500, 1800),
                        (74, 41, 'SYN-74', 'Repeated name', 1, 2500, 1800);"
            ).map_err(|error| error.to_string())
        }).unwrap();
        drop(initialized);
        let canonical_before = fs::read(config.path()).unwrap();

        let recovered = DatabaseState::recover_on_startup(config.clone(), &BackupStore::new(&directory));

        assert_eq!(recovered.with_read(|connection| {
            validate_restored_database(connection).map_err(|_| "invalid_schema".to_string())?;
            connection.query_row("SELECT COUNT(*) FROM products WHERE name = 'Repeated name' AND id IN (73, 74)", [], |row| row.get::<_, i64>(0))
                .map_err(|error| error.to_string())
        }), Ok(2));
        assert_eq!(fs::read(config.path()).unwrap(), canonical_before);
        for (path, bytes) in evidence {
            assert_eq!(fs::read(path).unwrap(), bytes);
        }
    }

    #[test]
    fn schema_upgrade_startup_rejects_invalid_or_ambiguous_evidence_without_writes() {
        for case in ["corrupt", "future_snapshot", "unknown_snapshot", "invalid_schema", "foreign_key", "ambiguous", "invalid_marker", "future_canonical", "missing_canonical"] {
            let directory = std::env::temp_dir().join(format!("r-a-negative-{case}-{}", uuid::Uuid::new_v4()));
            schema_upgrade_evidence(&directory);
            let config = infrastructure::sqlite::production_database_config(&directory);
            if case != "missing_canonical" {
                schema_upgrade_fixture(config.path());
            }
            let rollback = directory.join("restore-rollback.sqlite3");
            match case {
                "corrupt" => fs::write(&rollback, b"not a SQLite database").unwrap(),
                "future_snapshot" | "unknown_snapshot" => {
                    let connection = rusqlite::Connection::open(&rollback).unwrap();
                    connection.pragma_update(None, "user_version", if case == "future_snapshot" { 25 } else { 0 }).unwrap();
                }
                "invalid_schema" => {
                    rusqlite::Connection::open(&rollback).unwrap().execute_batch("PRAGMA foreign_keys = OFF; DROP TABLE products;").unwrap();
                }
                "foreign_key" => {
                    let connection = rusqlite::Connection::open(&rollback).unwrap();
                    connection.execute_batch("PRAGMA foreign_keys = OFF; UPDATE products SET category_id = 999 WHERE id = 73;").unwrap();
                }
                "ambiguous" => fs::write(directory.join("restore-state.json.part"), b"ambiguous").unwrap(),
                "invalid_marker" => fs::write(directory.join("restore-state.json.previous-0"), b"unknown state").unwrap(),
                "future_canonical" => {
                    rusqlite::Connection::open(config.path()).unwrap().pragma_update(None, "user_version", 25).unwrap();
                }
                "missing_canonical" => {}
                _ => unreachable!(),
            }
            let before: Vec<_> = fs::read_dir(&directory).unwrap().map(|entry| {
                let path = entry.unwrap().path();
                let bytes = fs::read(&path).unwrap();
                (path, bytes)
            }).collect();

            let recovered = DatabaseState::recover_on_startup(config.clone(), &BackupStore::new(&directory));

            assert_eq!(recovered.with_read(|_| Ok(())), Err("database_unavailable".into()), "{case}");
            assert_eq!(fs::read_dir(&directory).unwrap().count(), before.len(), "{case}: no new files");
            for (path, bytes) in before {
                assert_eq!(fs::read(path).unwrap(), bytes, "{case}: preserve canonical and evidence");
            }
            if case == "missing_canonical" {
                assert!(!config.path().exists(), "recovery must not create canonical storage");
            }
        }
    }

    fn schema_upgrade_missing_balances_fixture(
        missing_file: &str,
        current_canonical: bool,
    ) -> (std::path::PathBuf, DatabaseConfig, Vec<(std::path::PathBuf, Vec<u8>)>) {
        let directory = std::env::temp_dir().join(format!("r-a-missing-balances-{}", uuid::Uuid::new_v4()));
        schema_upgrade_evidence(&directory);
        let config = infrastructure::sqlite::production_database_config(&directory);
        if current_canonical {
            drop(DatabaseState::open(config.clone()).unwrap());
        } else {
            schema_upgrade_fixture(config.path());
        }
        rusqlite::Connection::open(directory.join(missing_file)).unwrap()
            .execute_batch("DROP TABLE stock_balances;").unwrap();
        let before = fs::read_dir(&directory).unwrap().map(|entry| {
            let path = entry.unwrap().path();
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        }).collect();
        (directory, config, before)
    }

    #[test]
    fn schema_upgrade_startup_rejects_missing_balances_in_historical_canonical() {
        let (directory, config, before) = schema_upgrade_missing_balances_fixture("repuestos-autos.sqlite3", false);

        let recovered = DatabaseState::recover_on_startup(config, &BackupStore::new(&directory));

        assert_eq!(recovered.with_read(|_| Ok(())), Err("database_unavailable".into()));
        assert_eq!(fs::read_dir(&directory).unwrap().count(), before.len());
        for (path, bytes) in before {
            assert_eq!(fs::read(path).unwrap(), bytes, "canonical and all evidence must remain unchanged");
        }
    }

    #[test]
    fn schema_upgrade_startup_rejects_missing_balances_in_historical_rollback() {
        let (directory, config, before) = schema_upgrade_missing_balances_fixture("restore-rollback.sqlite3", false);

        let recovered = DatabaseState::recover_on_startup(config, &BackupStore::new(&directory));

        assert_eq!(recovered.with_read(|_| Ok(())), Err("database_unavailable".into()));
        assert_eq!(fs::read_dir(&directory).unwrap().count(), before.len());
        for (path, bytes) in before {
            assert_eq!(fs::read(path).unwrap(), bytes, "canonical and all evidence must remain unchanged");
        }
    }

    #[test]
    fn schema_upgrade_startup_rejects_missing_balances_in_historical_protector() {
        let (directory, config, before) = schema_upgrade_missing_balances_fixture("pre-restore.sqlite3", false);

        let recovered = DatabaseState::recover_on_startup(config, &BackupStore::new(&directory));

        assert_eq!(recovered.with_read(|_| Ok(())), Err("database_unavailable".into()));
        assert_eq!(fs::read_dir(&directory).unwrap().count(), before.len());
        for (path, bytes) in before {
            assert_eq!(fs::read(path).unwrap(), bytes, "canonical and all evidence must remain unchanged");
        }
    }

    #[test]
    fn schema_upgrade_startup_rejects_current_canonical_with_missing_historical_balances() {
        let (directory, config, before) = schema_upgrade_missing_balances_fixture("restore-rollback.sqlite3", true);

        let recovered = DatabaseState::recover_on_startup(config, &BackupStore::new(&directory));

        assert_eq!(recovered.with_read(|_| Ok(())), Err("database_unavailable".into()));
        assert_eq!(fs::read_dir(&directory).unwrap().count(), before.len());
        for (path, bytes) in before {
            assert_eq!(fs::read(path).unwrap(), bytes, "canonical and all evidence must remain unchanged");
        }
    }

    #[test]
    fn schema_upgrade_startup_rejects_unusable_balance_columns_or_view() {
        for replacement in [
            "CREATE TABLE stock_balances (product_id INTEGER PRIMARY KEY);",
            "CREATE TABLE stock_balances (quantity INTEGER NOT NULL);",
            "CREATE VIEW stock_balances AS SELECT id AS product_id, 0 AS quantity FROM products;",
        ] {
            let (directory, config, _) = schema_upgrade_missing_balances_fixture("restore-rollback.sqlite3", false);
            rusqlite::Connection::open(directory.join("restore-rollback.sqlite3")).unwrap()
                .execute_batch(replacement).unwrap();
            let before: Vec<_> = fs::read_dir(&directory).unwrap().map(|entry| {
                let path = entry.unwrap().path();
                let bytes = fs::read(&path).unwrap();
                (path, bytes)
            }).collect();

            let recovered = DatabaseState::recover_on_startup(config, &BackupStore::new(&directory));

            assert_eq!(recovered.with_read(|_| Ok(())), Err("database_unavailable".into()), "{replacement}");
            assert_eq!(fs::read_dir(&directory).unwrap().count(), before.len());
            for (path, bytes) in before {
                assert_eq!(fs::read(path).unwrap(), bytes, "canonical and all evidence must remain unchanged");
            }
        }
    }

    #[test]
    fn backup_diagnostic_classification_uses_only_bounded_kinds_and_codes() {
        assert_eq!(bounded_backup_diagnostic_code("storage_unavailable"), "storage_unavailable");
        assert_eq!(bounded_backup_diagnostic_code("unsupported_destination"), "unsupported_destination");
        assert_eq!(bounded_backup_diagnostic_code("user-controlled-value"), "other");

        assert_eq!(backup_response_diagnostic(&commands::backup::BackupResponse::Restored), ("restored", "none"));
        assert_eq!(backup_response_diagnostic(&commands::backup::BackupResponse::error("destination_exists")), ("error", "destination_exists"));
        assert_eq!(backup_response_diagnostic(&commands::backup::BackupResponse::error("unlisted-code")), ("error", "other"));
    }

    #[test]
    fn fallback_recovery_after_live_moved_crash_reclaims_stage_and_retains_sources() {
        let directory = std::env::temp_dir().join(format!("r-a-recovery-stage-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(directory.join("backup-restore/staging")).unwrap();
        let config = infrastructure::sqlite::production_database_config(&directory);
        let initialized = DatabaseState::open(config.clone()).unwrap();
        let rollback = directory.join("restore-rollback.sqlite3");
        let protective = directory.join("pre-restore.sqlite3");
        initialized.with_read(|connection| {
            create_snapshot(connection, &rollback).map_err(|_| "snapshot_failed".to_string())?;
            create_snapshot(connection, &protective).map_err(|_| "snapshot_failed".to_string())?;
            Ok(())
        }).unwrap();
        drop(initialized);
        fs::remove_file(config.path()).unwrap(); // Simulate the durable live-to-rollback rename.
        let rollback_before = fs::read(&rollback).unwrap();
        let protective_before = fs::read(&protective).unwrap();
        let stage = directory.join(format!("backup-restore/staging/{}.sqlite3", uuid::Uuid::new_v4()));
        fs::write(&stage, b"candidate before stage-to-canonical rename").unwrap();
        fs::write(directory.join("restore-state.json"), br#"{"state":"live_moved"}"#).unwrap();

        let recovered = DatabaseState::recover_on_startup(config.clone(), &BackupStore::new(&directory));

        assert!(recovered.with_read(|_| Ok(())).is_ok());
        assert!(config.path().is_file());
        assert!(!stage.exists());
        assert_eq!(fs::read(&rollback).unwrap(), rollback_before);
        assert_eq!(fs::read(&protective).unwrap(), protective_before);
        assert_eq!(BackupStore::new(&directory).read_restore_state().unwrap(), None);
        drop(recovered);
        fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn startup_abandoned_stage_cleanup_failure_fails_closed_and_keeps_evidence() {
        use std::os::unix::fs::symlink;

        let directory = std::env::temp_dir().join(format!("r-a-abandoned-failure-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(directory.join("backup-restore/staging")).unwrap();
        let config = infrastructure::sqlite::production_database_config(&directory);
        let initialized = DatabaseState::open(config.clone()).unwrap();
        drop(initialized);
        let target = directory.join("source");
        fs::write(&target, b"must remain").unwrap();
        let stage = directory.join(format!("backup-restore/staging/{}.sqlite3", uuid::Uuid::new_v4()));
        symlink(&target, &stage).unwrap();
        let evidence = stage.with_file_name(format!(".{}.cleanup-needed", stage.file_name().unwrap().to_string_lossy()));
        fs::write(&evidence, b"restore-stage-cleanup-required\n").unwrap();

        let recovered = DatabaseState::recover_on_startup(config, &BackupStore::new(&directory));

        assert!(recovered.with_read(|_| Ok(())).is_err(), "startup must fail closed");
        assert!(fs::symlink_metadata(&stage).unwrap().file_type().is_symlink());
        assert_eq!(fs::read(&target).unwrap(), b"must remain");
        assert!(evidence.exists(), "startup reconciliation must retain existing cleanup evidence");
        drop(recovered);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn startup_removes_abandoned_stage_and_snapshot_only_without_recovery_evidence() {
        let directory = std::env::temp_dir().join(format!("r-a-abandoned-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(directory.join("backup-restore/staging")).unwrap();
        fs::create_dir_all(directory.join("backup-restore/snapshots")).unwrap();
        let config = infrastructure::sqlite::production_database_config(&directory);
        let initialized = DatabaseState::open(config.clone()).unwrap();
        drop(initialized);
        let stage = directory.join(format!("backup-restore/staging/{}.sqlite3", uuid::Uuid::new_v4()));
        let snapshot = directory.join(format!("backup-restore/snapshots/{}-{}.sqlite3", 1, uuid::Uuid::new_v4()));
        fs::write(&stage, b"stage").unwrap();
        fs::write(&snapshot, b"snapshot").unwrap();

        let recovered = DatabaseState::recover_on_startup(config, &BackupStore::new(&directory));

        assert!(recovered.with_read(|_| Ok(())).is_ok());
        assert!(!stage.exists());
        assert!(!snapshot.exists());
        drop(recovered);
        fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(not(windows))]
    #[test]
    fn startup_preserves_abandoned_stage_while_restore_marker_is_active() {
        let directory = std::env::temp_dir().join(format!("r-a-active-stage-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(directory.join("backup-restore/staging")).unwrap();
        let config = infrastructure::sqlite::production_database_config(&directory);
        let initialized = DatabaseState::open(config.clone()).unwrap();
        drop(initialized);
        let stage = directory.join("backup-restore/staging/active.sqlite3");
        fs::write(&stage, b"recovery-owned").unwrap();
        fs::write(directory.join("restore-state.json"), br#"{"state":"candidate_installed"}"#).unwrap();

        let recovered = DatabaseState::recover_on_startup(config, &BackupStore::new(&directory));

        assert!(recovered.with_read(|_| Ok(())).is_ok());
        assert!(stage.exists());
        assert!(directory.join("restore-state.json").exists());
        drop(recovered);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn post_disruption_failure_recovers_ready_without_clearing_evidence() {
        let directory = std::env::temp_dir().join(format!(
            "r-a-post-disruption-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(directory.join("backup-restore/staging")).unwrap();
        let config = infrastructure::sqlite::production_database_config(&directory);
        let state = DatabaseState::open(config).unwrap();
        let stage = directory.join("backup-restore/staging/candidate.sqlite3");
        fs::write(&stage, b"invalid candidate after preparation").unwrap();
        let store = BackupStore::new(&directory);

        assert_eq!(
            state.install_validated_stage(&stage, &store),
            Err("restore_failed".into())
        );
        assert!(state.with_read(|_| Ok(())).is_ok());
        assert!(directory.join("restore-rollback.sqlite3").exists());
        assert_eq!(
            store.read_restore_state().unwrap(),
            Some(RestoreState::CandidateInstalled)
        );

        drop(state);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn validated_candidate_remains_ready_when_final_marker_cleanup_fails() {
        let directory = std::env::temp_dir().join(format!(
            "r-a-cleanup-failure-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&directory).unwrap();
        let config = infrastructure::sqlite::production_database_config(&directory);
        let state = DatabaseState::open(config).unwrap();
        let stage = directory.join("backup-restore/staging/candidate.sqlite3");
        fs::create_dir_all(stage.parent().unwrap()).unwrap();
        state
            .with_read(|connection| {
                create_snapshot(connection, &stage).map_err(|_| "snapshot_failed".into())
            })
            .unwrap();
        rusqlite::Connection::open(&stage)
            .unwrap()
            .execute(
                "INSERT INTO categories (name) VALUES (?1)",
                ["validated-candidate"],
            )
            .unwrap();
        let store = BackupStore::new(&directory);
        let cleanup_called = Cell::new(false);

        assert_eq!(
            state.install_validated_stage_with_cleanup(&stage, &store, || {
                cleanup_called.set(true);
                Err(())
            }),
            Err("restore_failed".into())
        );
        assert!(cleanup_called.get());
        assert_eq!(
            state
                .with_read(|connection| {
                    connection
                        .query_row(
                            "SELECT COUNT(*) FROM categories WHERE name = ?1",
                            ["validated-candidate"],
                            |row| row.get::<_, i64>(0),
                        )
                        .map_err(|_| "database_unavailable".into())
                })
                .unwrap(),
            1
        );
        assert_eq!(
            store.read_restore_state().unwrap(),
            Some(RestoreState::CandidateInstalled)
        );

        drop(state);
        fs::remove_dir_all(directory).unwrap();
    }
}

#[cfg(feature = "desktop")]
type AppState = DatabaseState;

fn bounded_backup_diagnostic_code(code: &str) -> &'static str {
    match code {
        "storage_unavailable" => "storage_unavailable",
        "unsupported_destination" => "unsupported_destination",
        "destination_exists" => "destination_exists",
        "destination_token_invalid" => "destination_token_invalid",
        "destination_token_expired" => "destination_token_expired",
        "database_unavailable" => "database_unavailable",
        _ => "other",
    }
}

fn backup_response_diagnostic(response: &commands::backup::BackupResponse) -> (&'static str, &'static str) {
    match response {
        commands::backup::BackupResponse::Created { .. } => ("created", "none"),
        commands::backup::BackupResponse::Prepared { .. } => ("prepared", "none"),
        commands::backup::BackupResponse::Restored => ("restored", "none"),
        commands::backup::BackupResponse::Error { code, .. } => {
            ("error", bounded_backup_diagnostic_code(code))
        }
    }
}

#[cfg(all(windows, debug_assertions))]
fn report_backup_picker_selection(outcome: &str, code: &str) {
    eprintln!("backup_diagnostic operation=picker_selection outcome={outcome} code={code}");
}

#[cfg(not(all(windows, debug_assertions)))]
fn report_backup_picker_selection(_outcome: &str, _code: &str) {}

#[cfg(all(windows, debug_assertions))]
fn report_create_backup_entry() {
    eprintln!("backup_diagnostic operation=create_backup_command phase=entry");
}

#[cfg(not(all(windows, debug_assertions)))]
fn report_create_backup_entry() {}

#[cfg(all(windows, debug_assertions))]
fn report_create_backup_response(response: &commands::backup::BackupResponse) {
    let (kind, code) = backup_response_diagnostic(response);
    eprintln!("backup_diagnostic operation=create_backup_response kind={kind} code={code}");
}

#[cfg(not(all(windows, debug_assertions)))]
fn report_create_backup_response(_response: &commands::backup::BackupResponse) {}

#[cfg(feature = "desktop")]
fn catalog_access_authorized(license: &commands::license::LicenseCommandState, access: &application::catalog::access::CatalogAccessSession) -> bool {
    catalog_authorization_failure(license, access).is_none()
}

#[cfg(feature = "desktop")]
fn catalog_authorization_failure(license: &commands::license::LicenseCommandState, access: &application::catalog::access::CatalogAccessSession) -> Option<(&'static str, &'static str)> {
    if license.authorize_business_operation().is_err() { Some(("license_required", "A valid license is required to access the catalog.")) }
    else if !access.is_authorized() { Some(("catalog_access_required", "Desbloqueá el catálogo para continuar.")) }
    else { None }
}

#[cfg(feature = "desktop")]
fn command_builder<R: Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.invoke_handler(tauri::generate_handler![
        catalog_access_status_command,
        catalog_access_begin_setup_command,
        catalog_access_finish_setup_command,
        catalog_access_unlock_command,
        catalog_access_lock_command,
        catalog_access_change_password_command,
        catalog_access_begin_recovery_command,
        catalog_access_finish_recovery_command,
        search_products_command,
        browse_products_command,
        browse_inventory_products_command,
        browse_sale_products_command,
        dashboard_command,
        gross_profit_report_command,
        gross_profit_operations_command,
        export_gross_profit_operations_command,
        confirm_sale_command,
        create_sale_return_command,
        cancel_sale_command,
        confirm_stock_entry_command,
        confirm_physical_count_command,
        list_inventory_alerts_command,
        list_catalog_maintenance_command,
        list_catalog_categories_command,
        maintain_catalog_command,
        edit_catalog_command,
        edit_category_schema_command,
        catalog_metadata_detail_command,
        choose_product_image_command,
        license_status_command,
        license_installation_code_command,
        choose_license_file_command,
        import_license_command,
        remove_product_image_command,
        catalog_product_image_thumbnail_command,
        sales_product_image_thumbnail_command,
        sales_product_image_original_command,
        location_schema_command,
        onboarding_location_schema_command,
        onboarding_list_product_locations_command,
        onboarding_assign_product_primary_location_command,
        save_location_schema_command,
        list_product_locations_command,
        create_product_location_command,
        activate_product_location_command,
        deactivate_product_location_command,
        delete_product_location_command,
        assign_product_primary_location_command,
        list_categories_command,
        create_category_command,
        create_product_command,
        list_sales_history_command,
        sale_history_detail_command,
        list_movement_ledger_command,
        list_movement_ledger_product_options_command,
        export_movement_ledger_command,
        choose_backup_destination_command,
        choose_restore_source_command,
        create_backup_command,
        prepare_restore_command,
        confirm_restore_command
    ])
}

#[cfg(feature = "desktop")]
fn desktop_command_builder(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    command_builder(builder.plugin(tauri_plugin_dialog::init()))
}

#[cfg(feature = "desktop")]
pub fn run() -> Result<(), tauri::Error> {
    desktop_command_builder(tauri::Builder::default())
        .setup(|app: &mut tauri::App<tauri::Wry>| {
            let app_data_directory = app.path().app_data_dir()?;
            let database_config =
                infrastructure::sqlite::production_database_config(&app_data_directory);
            let store = BackupStore::new(&app_data_directory);
            let state = DatabaseState::recover_on_startup(database_config, &store);
            app.manage(state);
            app.manage(application::catalog::access::CatalogAccessSession::open(
                infrastructure::filesystem::catalog_access::CatalogAccessStore::new(&app_data_directory),
            ));
            app.manage(commands::license::LicenseCommandState::new(
                application::license::LicenseService::new(
                    infrastructure::windows_machine_identity::SystemMachineIdentity,
                    application::license::FileLicenseStorage::new(&app_data_directory),
                ),
            ));
            app.manage(Mutex::new(commands::backup::BackupCommandState::new(
                app_data_directory,
            )));
            Ok(())
        })
        .run(tauri::generate_context!())
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn license_status_command(state: tauri::State<'_, commands::license::LicenseCommandState>) -> commands::license::LicenseStatusResponse {
    commands::license::status(&state.service)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn license_installation_code_command(state: tauri::State<'_, commands::license::LicenseCommandState>) -> commands::license::InstallationCodeResponse {
    commands::license::installation_code(&state.service)
}

#[cfg(feature = "desktop")]
#[tauri::command]
async fn choose_license_file_command<R: Runtime>(
    state: tauri::State<'_, commands::license::LicenseCommandState>,
    window: tauri::WebviewWindow<R>,
) -> Result<commands::license::LicenseFileSelection, String> {
    let selected_file = state.selected_file.clone();
    drop(state);
    let app_handle = window.app_handle().clone();
    drop(window);
    let selection = commands::backup::select_callback_path(|complete| {
        #[cfg(test)] { let _ = app_handle; complete(None); }
        #[cfg(not(test))] { app_handle.dialog().file().add_filter("License file", &["lic"]).pick_file(move |path| {
            complete(path.and_then(|path| path.into_path().ok()));
        }); }
    }).await;
    let selected = match selection {
        commands::backup::PathSelection::Selected { path } => Some(path),
        commands::backup::PathSelection::Cancelled => None,
    };
    let Ok(mut pending) = selected_file.lock() else { return Ok(commands::license::LicenseFileSelection::Cancelled); };
    *pending = selected;
    Ok(if pending.is_some() { commands::license::LicenseFileSelection::Selected } else { commands::license::LicenseFileSelection::Cancelled })
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn import_license_command(state: tauri::State<'_, commands::license::LicenseCommandState>) -> commands::license::LicenseImportResponse {
    let Ok(mut pending) = state.selected_file.lock() else { return commands::license::LicenseImportResponse::Error { code: "license_storage_unavailable" }; };
    let selected = pending.take();
    commands::license::import_selected(&state.service, selected)
}

#[cfg(feature = "desktop")]
#[tauri::command]
async fn choose_backup_destination_command<R: Runtime>(
    window: tauri::WebviewWindow<R>,
) -> Result<commands::backup::BackupDestinationSelection, String> {
    let app_handle = window.app_handle().clone();
    drop(window);
    let picker_app_handle = app_handle.clone();
    let selection = commands::backup::select_callback_path(|complete| {
        #[cfg(test)]
        {
            let _ = picker_app_handle;
            complete(None);
        }
        #[cfg(not(test))]
        {
            picker_app_handle
                .dialog()
                .file()
                .pick_folder(move |path| {
                    complete(path.and_then(|path| path.into_path().ok()));
                });
        }
    })
    .await;
    let commands::backup::PathSelection::Selected { path } = selection else {
        report_backup_picker_selection("cancelled", "none");
        return Ok(commands::backup::BackupDestinationSelection::Cancelled);
    };
    let commands = app_handle.state::<Mutex<commands::backup::BackupCommandState>>();
    let Ok(mut commands) = commands.lock() else {
        report_backup_picker_selection("error", "storage_unavailable");
        return Ok(commands::backup::BackupDestinationSelection::Error {
            code: "storage_unavailable",
            message: "Backup storage is unavailable.",
        });
    };
    let selection = commands.select_backup_destination(path);
    match &selection {
        commands::backup::BackupDestinationSelection::Selected { .. } => {
            report_backup_picker_selection("selected", "none");
        }
        commands::backup::BackupDestinationSelection::Cancelled => {
            report_backup_picker_selection("cancelled", "none");
        }
        commands::backup::BackupDestinationSelection::Error { code, .. } => {
            report_backup_picker_selection("error", bounded_backup_diagnostic_code(code));
        }
    }
    Ok(selection)
}

#[cfg(feature = "desktop")]
#[tauri::command]
async fn choose_restore_source_command<R: Runtime>(
    window: tauri::WebviewWindow<R>,
) -> Result<commands::backup::RestoreSourceSelection, String> {
    let app_handle = window.app_handle().clone();
    drop(window);
    let picker_app_handle = app_handle.clone();
    let selection = commands::backup::select_callback_path(|complete| {
        #[cfg(test)]
        {
            let _ = picker_app_handle;
            complete(None);
        }
        #[cfg(not(test))]
        {
            picker_app_handle
                .dialog()
                .file()
                .add_filter("SQLite backup", &["sqlite3"])
                .pick_file(move |path| {
                    complete(path.and_then(|path| path.into_path().ok()));
                });
        }
    })
    .await;
    let commands::backup::PathSelection::Selected { path } = selection else {
        return Ok(commands::backup::RestoreSourceSelection::Cancelled);
    };
    let commands = app_handle.state::<Mutex<commands::backup::BackupCommandState>>();
    let Ok(mut commands) = commands.lock() else {
        return Ok(commands::backup::RestoreSourceSelection::Error {
            code: "storage_unavailable",
            message: "Backup storage is unavailable.",
        });
    };
    Ok(commands.select_restore_source(path))
}

#[cfg(feature = "desktop")]
fn catalog_access_license_error() -> commands::catalog::CatalogAccessResponse {
    commands::catalog::CatalogAccessResponse::Error { code: "license_required", message: "Se requiere una licencia válida para configurar el acceso al catálogo." }
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn catalog_access_status_command(access: tauri::State<'_, application::catalog::access::CatalogAccessSession>, license: tauri::State<'_, commands::license::LicenseCommandState>) -> commands::catalog::CatalogAccessResponse {
    if license.authorize_business_operation().is_err() { return catalog_access_license_error(); }
    commands::catalog::catalog_access_status(&access)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn catalog_access_begin_setup_command(access: tauri::State<'_, application::catalog::access::CatalogAccessSession>, license: tauri::State<'_, commands::license::LicenseCommandState>, request: commands::catalog::CatalogSecretRequest) -> commands::catalog::CatalogAccessResponse {
    if license.authorize_business_operation().is_err() { return catalog_access_license_error(); }
    commands::catalog::begin_catalog_setup(&access, request)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn catalog_access_finish_setup_command(access: tauri::State<'_, application::catalog::access::CatalogAccessSession>, license: tauri::State<'_, commands::license::LicenseCommandState>, request: commands::catalog::CatalogConfirmationRequest) -> commands::catalog::CatalogAccessResponse {
    if license.authorize_business_operation().is_err() { return catalog_access_license_error(); }
    commands::catalog::finish_catalog_setup(&access, request)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn catalog_access_unlock_command(access: tauri::State<'_, application::catalog::access::CatalogAccessSession>, license: tauri::State<'_, commands::license::LicenseCommandState>, request: commands::catalog::CatalogSecretRequest) -> commands::catalog::CatalogAccessResponse {
    if license.authorize_business_operation().is_err() { return catalog_access_license_error(); }
    commands::catalog::unlock_catalog(&access, request)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn catalog_access_lock_command(access: tauri::State<'_, application::catalog::access::CatalogAccessSession>, license: tauri::State<'_, commands::license::LicenseCommandState>) -> commands::catalog::CatalogAccessResponse {
    if license.authorize_business_operation().is_err() { return catalog_access_license_error(); }
    commands::catalog::lock_catalog(&access)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn catalog_access_change_password_command(access: tauri::State<'_, application::catalog::access::CatalogAccessSession>, license: tauri::State<'_, commands::license::LicenseCommandState>, request: commands::catalog::CatalogPasswordChangeRequest) -> commands::catalog::CatalogAccessResponse {
    if !catalog_access_authorized(&license, &access) { return catalog_access_license_error(); }
    commands::catalog::change_catalog_password(&access, request)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn catalog_access_begin_recovery_command(access: tauri::State<'_, application::catalog::access::CatalogAccessSession>, license: tauri::State<'_, commands::license::LicenseCommandState>, request: commands::catalog::CatalogRecoveryRequest) -> commands::catalog::CatalogAccessResponse {
    if license.authorize_business_operation().is_err() { return catalog_access_license_error(); }
    commands::catalog::begin_catalog_recovery(&access, request)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn catalog_access_finish_recovery_command(access: tauri::State<'_, application::catalog::access::CatalogAccessSession>, license: tauri::State<'_, commands::license::LicenseCommandState>, request: commands::catalog::CatalogConfirmationRequest) -> commands::catalog::CatalogAccessResponse {
    if license.authorize_business_operation().is_err() { return catalog_access_license_error(); }
    commands::catalog::finish_catalog_recovery(&access, request)
}

#[cfg(feature = "desktop")]
#[tauri::command]
async fn choose_product_image_command<R: Runtime>(
    state: tauri::State<'_, AppState>,
    license: tauri::State<'_, commands::license::LicenseCommandState>,
    access: tauri::State<'_, application::catalog::access::CatalogAccessSession>,
    window: tauri::WebviewWindow<R>,
    request: commands::catalog::ProductImageRequest,
) -> Result<commands::catalog::ProductImageResponse, String> {
    if !catalog_access_authorized(&license, &access) {
        return Ok(commands::catalog::ProductImageResponse::Error(commands::catalog::CatalogMaintenanceError {
            code: "license_required", message: "A valid license is required to change the catalog.",
        }));
    }
    let Ok(request) = commands::catalog::parse_product_image_request(request) else {
        return Ok(commands::catalog::ProductImageResponse::Error(commands::catalog::CatalogMaintenanceError {
            code: "validation_error", message: "Review the catalog values and try again.",
        }));
    };
    let app_handle = window.app_handle().clone();
    drop(window);
    let selection = commands::backup::select_callback_path(|complete| {
        #[cfg(test)]
        { let _ = app_handle; complete(None); }
        #[cfg(not(test))]
        { app_handle.dialog().file().add_filter("Product image", &["png", "jpg", "jpeg", "webp"]).pick_file(move |path| {
            complete(path.and_then(|path| path.into_path().ok()));
        }); }
    }).await;
    let commands::backup::PathSelection::Selected { path } = selection else {
        return Ok(commands::catalog::ProductImageResponse::Cancelled);
    };
    if !catalog_access_authorized(&license, &access) {
        return Ok(commands::catalog::ProductImageResponse::Error(commands::catalog::CatalogMaintenanceError {
            code: "catalog_access_required", message: "Desbloqueá el catálogo para continuar.",
        }));
    }
    let read_result = read_selected_image(&path);
    let (mime, bytes) = match read_result {
        Ok(value) => value,
        Err(()) => return Ok(commands::catalog::ProductImageResponse::Error(commands::catalog::CatalogMaintenanceError {
            code: "image_unavailable", message: "The selected image could not be used.",
        })),
    };
    Ok(state.with_write(|connection| Ok(commands::catalog::persist_selected_product_image(
        connection, request.product_id, request.expected_revision, mime, bytes,
    ))).unwrap_or_else(|_| commands::catalog::ProductImageResponse::Error(commands::catalog::CatalogMaintenanceError {
        code: "persistence_failure", message: "The catalog could not be completed.",
    })))
}

#[cfg(feature = "desktop")]
fn read_selected_image(path: &std::path::Path) -> Result<(&'static str, Vec<u8>), ()> {
    use std::io::Read;
    const MAX_BYTES: u64 = application::catalog::MAX_PRODUCT_IMAGE_BYTES as u64;
    let file = std::fs::File::open(path).map_err(|_| ())?;
    if file.metadata().map_err(|_| ())?.len() > MAX_BYTES { return Err(()); }
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes).map_err(|_| ())?;
    if bytes.len() as u64 > MAX_BYTES { return Err(()); }
    let extension = path.extension().and_then(|value| value.to_str()).unwrap_or_default().to_ascii_lowercase();
    let mime = match extension.as_str() { "png" => "image/png", "jpg" | "jpeg" => "image/jpeg", "webp" => "image/webp", _ => return Err(()) };
    Ok((mime, bytes))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn remove_product_image_command(
    state: tauri::State<AppState>, license: tauri::State<commands::license::LicenseCommandState>, access: tauri::State<application::catalog::access::CatalogAccessSession>, request: commands::catalog::ProductImageRequest,
) -> commands::catalog::ProductImageResponse {
    if !catalog_access_authorized(&license, &access) {
        return commands::catalog::ProductImageResponse::Error(commands::catalog::CatalogMaintenanceError { code: "license_required", message: "A valid license is required to change the catalog." });
    }
    state.with_write(|connection| Ok(commands::catalog::remove_product_image(connection, request)))
        .unwrap_or_else(|_| commands::catalog::ProductImageResponse::Error(commands::catalog::CatalogMaintenanceError {
            code: "persistence_failure", message: "The catalog could not be completed.",
        }))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn catalog_product_image_thumbnail_command(
    state: tauri::State<AppState>, license: tauri::State<commands::license::LicenseCommandState>, access: tauri::State<application::catalog::access::CatalogAccessSession>, request: commands::catalog::ProductImageRequest,
) -> commands::catalog::ProductImageThumbnailResponse {
    if !catalog_access_authorized(&license, &access) { return commands::catalog::ProductImageThumbnailResponse::Error(commands::catalog::CatalogMaintenanceError { code: "catalog_access_required", message: "Desbloqueá el catálogo para continuar." }); }
    state.with_read(|connection| Ok(commands::catalog::catalog_product_image_thumbnail(connection, request)))
        .unwrap_or_else(|_| commands::catalog::ProductImageThumbnailResponse::Error(commands::catalog::CatalogMaintenanceError {
            code: "persistence_failure", message: "The catalog could not be completed.",
        }))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn sales_product_image_original_command(
    state: tauri::State<AppState>,
    request: commands::catalog::SalesProductOriginalRequest,
) -> commands::catalog::SalesProductOriginalResponse {
    state.with_read(|connection| Ok(commands::catalog::sales_product_image_original(connection, request)))
        .unwrap_or(commands::catalog::SalesProductOriginalResponse::Error {
            code: "persistence_failure", message: "The product image could not be loaded.",
        })
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn sales_product_image_thumbnail_command(
    state: tauri::State<AppState>,
    request: commands::catalog::SalesProductThumbnailRequest,
) -> commands::catalog::SalesProductThumbnailResponse {
    state.with_read(|connection| Ok(commands::catalog::sales_product_image_thumbnail(connection, request)))
        .unwrap_or(commands::catalog::SalesProductThumbnailResponse::Error {
            code: "persistence_failure", message: "The product image could not be loaded.",
        })
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn create_backup_command(
    state: tauri::State<AppState>,
    commands: tauri::State<Mutex<commands::backup::BackupCommandState>>,
    request: commands::backup::CreateBackupRequest,
) -> commands::backup::BackupResponse {
    report_create_backup_entry();
    let response = match commands.lock() {
        Ok(mut commands) => commands::backup::create_backup(&state, &mut commands, request),
        Err(_) => commands::backup::BackupResponse::error("storage_unavailable"),
    };
    report_create_backup_response(&response);
    response
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn prepare_restore_command(
    state: tauri::State<AppState>,
    license: tauri::State<commands::license::LicenseCommandState>,
    commands: tauri::State<Mutex<commands::backup::BackupCommandState>>,
    request: commands::backup::PrepareRestoreRequest,
) -> commands::backup::BackupResponse {
    if license.authorize_business_operation().is_err() { return commands::backup::BackupResponse::error("license_required"); }
    let Ok(mut commands) = commands.lock() else {
        return commands::backup::BackupResponse::error("storage_unavailable");
    };
    commands::backup::prepare_restore(&state, &mut commands, request)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn confirm_restore_command(
    state: tauri::State<AppState>,
    license: tauri::State<commands::license::LicenseCommandState>,
    commands: tauri::State<Mutex<commands::backup::BackupCommandState>>,
    request: commands::backup::ConfirmRestoreRequest,
) -> commands::backup::BackupResponse {
    if license.authorize_business_operation().is_err() { return commands::backup::BackupResponse::error("license_required"); }
    let Ok(mut commands) = commands.lock() else {
        return commands::backup::BackupResponse::error("storage_unavailable");
    };
    commands::backup::confirm_restore(&state, &mut commands, request)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn gross_profit_report_command(
    state: tauri::State<AppState>,
    request: commands::gross_profit::GrossProfitRequest,
) -> commands::gross_profit::GrossProfitResponse {
    state.with_read(|connection| Ok(commands::gross_profit::gross_profit(connection, request)))
        .unwrap_or_else(|_| commands::gross_profit::GrossProfitResponse::Error(commands::gross_profit::GrossProfitError {
            code: "persistence_failure", message: "The gross-profit report could not be loaded.",
        }))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn gross_profit_operations_command(
    state: tauri::State<AppState>,
    request: commands::gross_profit_operations::GrossProfitOperationsRequest,
) -> commands::gross_profit_operations::GrossProfitOperationsResponse {
    state.with_read(|connection| Ok(commands::gross_profit_operations::gross_profit_operations(connection, request)))
        .unwrap_or_else(|_| commands::gross_profit_operations::GrossProfitOperationsResponse::Error(
            commands::gross_profit_operations::GrossProfitOperationsError {
                code: "persistence_failure",
                message: "The gross-profit operations could not be loaded.",
            },
        ))
}

#[cfg(feature = "desktop")]
#[tauri::command]
async fn export_gross_profit_operations_command<R: Runtime>(
    app_handle: tauri::AppHandle<R>,
    request: commands::gross_profit_operations::GrossProfitOperationsExportRequest,
) -> Result<commands::gross_profit_operations::GrossProfitOperationsExportResponse, String> {
    #[cfg(test)]
    {
        let _ = (app_handle, request);
        Ok(commands::gross_profit_operations::GrossProfitOperationsExportResponse::Cancelled)
    }
    #[cfg(not(test))]
    {
        let (sender, mut receiver) = tauri::async_runtime::channel(1);
        app_handle.dialog().file().add_filter("PDF", &["pdf"])
            .set_file_name("informe-ganancia-bruta.pdf")
            .save_file(move |path| {
                let converted = match path {
                    None => Ok(None),
                    Some(path) => path.into_path().map(Some).map_err(|_| ()),
                };
                let _ = sender.try_send(converted);
            });
        let path = match receiver.recv().await {
            Some(Ok(Some(path))) => path,
            Some(Ok(None)) | None => return Ok(commands::gross_profit_operations::GrossProfitOperationsExportResponse::Cancelled),
            Some(Err(())) => return Ok(commands::gross_profit_operations::GrossProfitOperationsExportResponse::Error(
                commands::gross_profit_operations::GrossProfitOperationsError { code: "export_failed", message: "No se pudo guardar el informe PDF." },
            )),
        };
        let generated_at = time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "No disponible".to_string());
        let state = app_handle.state::<AppState>();
        let response = state.with_read(|connection| Ok(commands::gross_profit_operations::export_gross_profit_operations(
            connection, request, &generated_at, |bytes| {
                std::fs::write(&path, bytes).map_or(
                    commands::gross_profit_operations::ExportSaveResult::Failed,
                    |_| commands::gross_profit_operations::ExportSaveResult::Saved,
                )
            },
        ))).unwrap_or_else(|_| commands::gross_profit_operations::GrossProfitOperationsExportResponse::Error(
            commands::gross_profit_operations::GrossProfitOperationsError { code: "export_failed", message: "No se pudo guardar el informe PDF." },
        ));
        Ok(response)
    }
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn dashboard_command(
    state: tauri::State<AppState>,
    request: commands::dashboard::DashboardRequest,
) -> commands::dashboard::DashboardResponse {
    state
        .with_read(|connection| Ok(commands::dashboard::dashboard(connection, request)))
        .unwrap_or_else(|_| commands::dashboard::DashboardResponse::Error(commands::dashboard::persistence_failure()))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn search_products_command(
    state: tauri::State<AppState>,
    request: commands::catalog::SearchProductsRequest,
) -> Result<Vec<application::catalog::SaleProductSearchResult>, String> {
    state.with_read(|connection| commands::catalog::search_sale_products(connection, request))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn browse_products_command(
    state: tauri::State<AppState>, license: tauri::State<commands::license::LicenseCommandState>,
    access: tauri::State<application::catalog::access::CatalogAccessSession>,
    request: commands::catalog::BrowseProductsRequest,
) -> Result<commands::catalog::ProductBrowseResponse, String> {
    if !catalog_access_authorized(&license, &access) { return Ok(commands::catalog::ProductBrowseResponse::Error(commands::catalog::CatalogBrowseError { code: "catalog_access_required", message: "Desbloqueá el catálogo para continuar." })); }
    state.with_read(|connection| Ok(commands::catalog::browse_products(connection, request)))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn browse_inventory_products_command(
    state: tauri::State<AppState>,
    license: tauri::State<commands::license::LicenseCommandState>,
    request: commands::catalog::BrowseProductsRequest,
) -> Result<application::catalog::InventoryBrowsePage, String> {
    if license.authorize_business_operation().is_err() { return Err("license_required".into()); }
    state.with_read(|connection| commands::catalog::browse_inventory_products(connection, request))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn browse_sale_products_command(state: tauri::State<AppState>, request: commands::catalog::BrowseProductsRequest) -> Result<application::catalog::SaleBrowsePage, String> {
    state.with_read(|connection| commands::catalog::browse_sale_products(connection, request))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn confirm_sale_command(
    state: tauri::State<AppState>,
    license: tauri::State<commands::license::LicenseCommandState>,
    request: commands::confirm_sale::ConfirmSaleRequest,
) -> Result<commands::confirm_sale::ConfirmSaleResponse, String> {
    if license.authorize_business_operation().is_err() { return Ok(commands::confirm_sale::ConfirmSaleResponse::Error(commands::confirm_sale::CommandError { code: "license_required", message: "A valid license is required to confirm sales." })); }
    state.with_write(|connection| commands::confirm_sale::confirm_sale(connection, request))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn create_sale_return_command(
    state: tauri::State<AppState>,
    license: tauri::State<commands::license::LicenseCommandState>,
    request: commands::post_sale::CreateSaleReturnRequest,
) -> commands::post_sale::PostSaleCommandResponse {
    if license.authorize_business_operation().is_err() { return commands::post_sale::PostSaleCommandResponse::Error(commands::confirm_sale::CommandError { code: "license_required", message: "A valid license is required to correct sales." }); }
    state
        .with_write(|connection| Ok(commands::post_sale::create_sale_return(connection, request)))
        .unwrap_or_else(|_| {
            commands::post_sale::PostSaleCommandResponse::Error(
                commands::post_sale::persistence_failure(),
            )
        })
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn cancel_sale_command(
    state: tauri::State<AppState>,
    license: tauri::State<commands::license::LicenseCommandState>,
    request: commands::post_sale::CancelSaleRequest,
) -> commands::post_sale::CancelSaleCommandResponse {
    if license.authorize_business_operation().is_err() { return commands::post_sale::CancelSaleCommandResponse::Error(commands::confirm_sale::CommandError { code: "license_required", message: "A valid license is required to correct sales." }); }
    state
        .with_write(|connection| Ok(commands::post_sale::cancel_sale(connection, request)))
        .unwrap_or_else(|_| {
            commands::post_sale::CancelSaleCommandResponse::Error(
                commands::post_sale::persistence_failure(),
            )
        })
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn confirm_stock_entry_command(
    state: tauri::State<AppState>,
    license: tauri::State<commands::license::LicenseCommandState>,
    request: commands::inventory::StockEntryRequest,
) -> Result<commands::inventory::InventoryCommandResponse, String> {
    if license.authorize_business_operation().is_err() { return Ok(commands::inventory::InventoryCommandResponse::Error(commands::confirm_sale::CommandError { code: "license_required", message: "A valid license is required to change inventory." })); }
    state.with_write(|connection| {
        commands::inventory::confirm_stock_entry_command(connection, request)
    })
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn confirm_physical_count_command(
    state: tauri::State<AppState>,
    license: tauri::State<commands::license::LicenseCommandState>,
    access: tauri::State<application::catalog::access::CatalogAccessSession>,
    request: commands::inventory::PhysicalCountRequest,
) -> Result<commands::inventory::InventoryCommandResponse, String> {
    if license.authorize_business_operation().is_err() { return Ok(commands::inventory::InventoryCommandResponse::Error(commands::confirm_sale::CommandError { code: "license_required", message: "A valid license is required to change inventory." })); }
    state.with_write(|connection| {
        commands::inventory::confirm_physical_count_command(connection, &access, request)
    })
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn list_inventory_alerts_command(
    state: tauri::State<AppState>,
) -> Result<commands::inventory::InventoryCommandResponse, String> {
    state.with_read(commands::inventory::list_inventory_alerts_command)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn list_catalog_maintenance_command(
    state: tauri::State<AppState>, license: tauri::State<commands::license::LicenseCommandState>, access: tauri::State<application::catalog::access::CatalogAccessSession>,
) -> Result<commands::catalog::CatalogMaintenanceListResponse, String> {
    if !catalog_access_authorized(&license, &access) { return Ok(commands::catalog::CatalogMaintenanceListResponse::Error(commands::catalog::CatalogMaintenanceError { code: "catalog_access_required", message: "Desbloqueá el catálogo para continuar." })); }
    state.with_read(commands::catalog::list_catalog_maintenance)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn list_catalog_categories_command(
    state: tauri::State<AppState>, license: tauri::State<commands::license::LicenseCommandState>, access: tauri::State<application::catalog::access::CatalogAccessSession>,
) -> Result<commands::catalog::CatalogMaintenanceListResponse, String> {
    if !catalog_access_authorized(&license, &access) { return Ok(commands::catalog::CatalogMaintenanceListResponse::Error(commands::catalog::CatalogMaintenanceError { code: "catalog_access_required", message: "Desbloqueá el catálogo para continuar." })); }
    state.with_read(commands::catalog::list_catalog_categories)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn maintain_catalog_command(
    state: tauri::State<AppState>,
    license: tauri::State<commands::license::LicenseCommandState>,
    access: tauri::State<application::catalog::access::CatalogAccessSession>,
    request: commands::catalog::MaintainCatalogRequest,
) -> Result<commands::catalog::CatalogMaintenanceResponse, String> {
    if let Some((code, message)) = catalog_authorization_failure(&license, &access) { return Ok(commands::catalog::CatalogMaintenanceResponse::Error(commands::catalog::CatalogMaintenanceError { code, message })); }
    state.with_write(|connection| commands::catalog::maintain_catalog(connection, request))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn edit_catalog_command(
    state: tauri::State<AppState>,
    license: tauri::State<commands::license::LicenseCommandState>,
    access: tauri::State<application::catalog::access::CatalogAccessSession>,
    request: commands::catalog::EditCatalogRequest,
) -> commands::catalog::CatalogMaintenanceResponse {
    if let Some((code, message)) = catalog_authorization_failure(&license, &access) { return commands::catalog::CatalogMaintenanceResponse::Error(commands::catalog::CatalogMaintenanceError { code, message }); }
    state
        .with_write(|connection| commands::catalog::edit_catalog(connection, request))
        .unwrap_or_else(|error| {
            commands::catalog::CatalogMaintenanceResponse::Error(
                commands::catalog::map_command_state_error(&error),
            )
        })
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn edit_category_schema_command(
    state: tauri::State<AppState>,
    license: tauri::State<commands::license::LicenseCommandState>,
    access: tauri::State<application::catalog::access::CatalogAccessSession>,
    request: commands::catalog::EditCategorySchemaRequest,
) -> commands::catalog::CatalogMaintenanceResponse {
    if let Some((code, message)) = catalog_authorization_failure(&license, &access) { return commands::catalog::CatalogMaintenanceResponse::Error(commands::catalog::CatalogMaintenanceError { code, message }); }
    state
        .with_write(|connection| Ok(commands::catalog::edit_category_schema(connection, request)))
        .unwrap_or_else(|error| {
            commands::catalog::CatalogMaintenanceResponse::Error(
                commands::catalog::map_command_state_error(&error),
            )
        })
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn catalog_metadata_detail_command(
    state: tauri::State<AppState>, license: tauri::State<commands::license::LicenseCommandState>,
    access: tauri::State<application::catalog::access::CatalogAccessSession>,
    request: commands::catalog::CatalogMetadataDetailRequest,
) -> commands::catalog::CatalogMetadataDetailResponse {
    if !catalog_access_authorized(&license, &access) { return commands::catalog::CatalogMetadataDetailResponse::Error(commands::catalog::CatalogMaintenanceError { code: "catalog_access_required", message: "Desbloqueá el catálogo para continuar." }); }
    state
        .with_read(|connection| commands::catalog::catalog_metadata_detail(connection, request))
        .unwrap_or_else(|error| {
            commands::catalog::CatalogMetadataDetailResponse::Error(
                commands::catalog::map_command_state_error(&error),
            )
        })
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn list_sales_history_command(
    state: tauri::State<AppState>,
    request: commands::sales_history::ListSalesHistoryRequest,
) -> commands::sales_history::SalesHistoryListResponse {
    state
        .with_read(|connection| {
            Ok(commands::sales_history::list_sales_history(
                connection, request,
            ))
        })
        .unwrap_or_else(|_| {
            commands::sales_history::SalesHistoryListResponse::Error(
                commands::sales_history::persistence_failure(),
            )
        })
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn sale_history_detail_command(
    state: tauri::State<AppState>,
    sale_id: i64,
) -> commands::sales_history::SalesHistoryDetailResponse {
    state
        .with_read(|connection| {
            Ok(commands::sales_history::sale_history_detail(
                connection, sale_id,
            ))
        })
        .unwrap_or_else(|_| {
            commands::sales_history::SalesHistoryDetailResponse::Error(
                commands::sales_history::persistence_failure(),
            )
        })
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn list_movement_ledger_command(
    state: tauri::State<AppState>,
    request: commands::movement_ledger::MovementLedgerRequest,
) -> commands::movement_ledger::MovementLedgerResponse {
    state
        .with_read(|connection| Ok(commands::movement_ledger::list_movement_ledger(connection, request)))
        .unwrap_or_else(|_| {
            commands::movement_ledger::MovementLedgerResponse::Error(
                commands::movement_ledger::MovementLedgerCommandError {
                    code: "persistence_failure",
                    message: "The movement ledger could not be loaded.",
                },
            )
        })
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn list_movement_ledger_product_options_command(
    state: tauri::State<AppState>,
    request: commands::movement_ledger::MovementLedgerProductOptionsRequest,
) -> commands::movement_ledger::MovementLedgerProductOptionsResponse {
    state.with_read(|connection| Ok(commands::movement_ledger::list_movement_ledger_product_options(connection, request)))
        .unwrap_or_else(|_| commands::movement_ledger::MovementLedgerProductOptionsResponse::Error(
            commands::movement_ledger::MovementLedgerCommandError {
                code: "persistence_failure",
                message: "The movement ledger products could not be loaded.",
            },
        ))
}

#[cfg(feature = "desktop")]
#[tauri::command]
async fn export_movement_ledger_command<R: Runtime>(
    app_handle: tauri::AppHandle<R>,
    request: commands::movement_ledger::MovementLedgerExportRequest,
) -> Result<commands::movement_ledger::MovementLedgerExportResponse, String> {
    #[cfg(test)]
    {
        let _ = (app_handle, request);
        Ok(commands::movement_ledger::MovementLedgerExportResponse::Cancelled)
    }
    #[cfg(not(test))]
    {
        let selection = commands::backup::select_callback_path(|complete| {
            app_handle.dialog().file()
                .add_filter("PDF document", &["pdf"])
                .set_file_name("movement-ledger.pdf")
                .save_file(move |path| {
                    complete(path.and_then(|path| path.into_path().ok()));
                });
        }).await;
        let commands::backup::PathSelection::Selected { path } = selection else {
            return Ok(commands::movement_ledger::MovementLedgerExportResponse::Cancelled);
        };
        let generated_at = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "Unavailable".to_string());
        let state = app_handle.state::<AppState>();
        let response = state.with_read(|connection| Ok(commands::movement_ledger::export_movement_ledger(
            connection, request, &generated_at, |bytes| {
                std::fs::write(&path, bytes).map_or(
                    commands::movement_ledger::ExportSaveResult::Failed,
                    |_| commands::movement_ledger::ExportSaveResult::Saved,
                )
            },
        ))).unwrap_or_else(|_| commands::movement_ledger::MovementLedgerExportResponse::Error(
            commands::movement_ledger::MovementLedgerCommandError {
                code: "persistence_failure",
                message: "The movement ledger could not be loaded.",
            },
        ));
        Ok(response)
    }
}

#[cfg(feature = "desktop")]
fn license_required_location_response() -> commands::catalog::ProductLocationResponse {
    commands::catalog::ProductLocationResponse::Error(commands::catalog::CatalogLocationError { code: "license_required", message: "A valid license is required to change the catalog." })
}

#[cfg(feature = "desktop")]
fn catalog_access_required_location_response() -> commands::catalog::ProductLocationResponse {
    commands::catalog::ProductLocationResponse::Error(commands::catalog::CatalogLocationError { code: "catalog_access_required", message: "Desbloqueá el catálogo para continuar." })
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn location_schema_command(state: tauri::State<AppState>, license: tauri::State<commands::license::LicenseCommandState>, access: tauri::State<application::catalog::access::CatalogAccessSession>) -> commands::catalog::ProductLocationResponse {
    if !catalog_access_authorized(&license, &access) { return catalog_access_required_location_response(); }
    state.with_read(|connection| Ok(commands::catalog::location_schema(connection)))
        .unwrap_or_else(|_| commands::catalog::ProductLocationResponse::Error(commands::catalog::CatalogLocationError { code: "persistence_failure", message: "The location change could not be completed." }))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn onboarding_location_schema_command(state: tauri::State<AppState>, license: tauri::State<commands::license::LicenseCommandState>) -> commands::catalog::ProductLocationResponse {
    if license.authorize_business_operation().is_err() { return license_required_location_response(); }
    state.with_read(|connection| Ok(commands::catalog::location_schema(connection)))
        .unwrap_or_else(|_| commands::catalog::ProductLocationResponse::Error(commands::catalog::CatalogLocationError { code: "persistence_failure", message: "The location change could not be completed." }))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn onboarding_list_product_locations_command(state: tauri::State<AppState>, license: tauri::State<commands::license::LicenseCommandState>) -> commands::catalog::ProductLocationResponse {
    if license.authorize_business_operation().is_err() { return license_required_location_response(); }
    state.with_read(|connection| Ok(commands::catalog::list_product_locations(connection, commands::catalog::ListProductLocationsRequest { include_inactive: false })))
        .unwrap_or_else(|_| commands::catalog::ProductLocationResponse::Error(commands::catalog::CatalogLocationError { code: "persistence_failure", message: "The location change could not be completed." }))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn onboarding_assign_product_primary_location_command(state: tauri::State<AppState>, license: tauri::State<commands::license::LicenseCommandState>, request: commands::catalog::AssignProductLocationRequest) -> commands::catalog::ProductLocationResponse {
    if license.authorize_business_operation().is_err() { return license_required_location_response(); }
    state.with_write(|connection| Ok(commands::catalog::assign_product_primary_location(connection, request)))
        .unwrap_or_else(|_| commands::catalog::ProductLocationResponse::Error(commands::catalog::CatalogLocationError { code: "persistence_failure", message: "The location change could not be completed." }))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn save_location_schema_command(state: tauri::State<AppState>, license: tauri::State<commands::license::LicenseCommandState>, access: tauri::State<application::catalog::access::CatalogAccessSession>, request: commands::catalog::SaveLocationSchemaRequest) -> commands::catalog::ProductLocationResponse {
    if !catalog_access_authorized(&license, &access) { return license_required_location_response(); }
    state.with_write(|connection| Ok(commands::catalog::save_location_schema(connection, request)))
        .unwrap_or_else(|_| commands::catalog::ProductLocationResponse::Error(commands::catalog::CatalogLocationError { code: "persistence_failure", message: "The location change could not be completed." }))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn list_product_locations_command(state: tauri::State<AppState>, license: tauri::State<commands::license::LicenseCommandState>, access: tauri::State<application::catalog::access::CatalogAccessSession>, request: commands::catalog::ListProductLocationsRequest) -> commands::catalog::ProductLocationResponse {
    if !catalog_access_authorized(&license, &access) { return catalog_access_required_location_response(); }
    state.with_read(|connection| Ok(commands::catalog::list_product_locations(connection, request)))
        .unwrap_or_else(|_| commands::catalog::ProductLocationResponse::Error(commands::catalog::CatalogLocationError { code: "persistence_failure", message: "The location change could not be completed." }))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn create_product_location_command(state: tauri::State<AppState>, license: tauri::State<commands::license::LicenseCommandState>, access: tauri::State<application::catalog::access::CatalogAccessSession>, request: commands::catalog::CreateProductLocationRequest) -> commands::catalog::ProductLocationResponse {
    if !catalog_access_authorized(&license, &access) { return license_required_location_response(); }
    state.with_write(|connection| Ok(commands::catalog::create_product_location(connection, request)))
        .unwrap_or_else(|_| commands::catalog::ProductLocationResponse::Error(commands::catalog::CatalogLocationError { code: "persistence_failure", message: "The location change could not be completed." }))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn activate_product_location_command(state: tauri::State<AppState>, license: tauri::State<commands::license::LicenseCommandState>, access: tauri::State<application::catalog::access::CatalogAccessSession>, request: commands::catalog::ProductLocationLifecycleRequest) -> commands::catalog::ProductLocationResponse {
    if !catalog_access_authorized(&license, &access) { return license_required_location_response(); }
    state.with_write(|connection| Ok(commands::catalog::set_product_location_activity(connection, request, true)))
        .unwrap_or_else(|_| commands::catalog::ProductLocationResponse::Error(commands::catalog::CatalogLocationError { code: "persistence_failure", message: "The location change could not be completed." }))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn deactivate_product_location_command(state: tauri::State<AppState>, license: tauri::State<commands::license::LicenseCommandState>, access: tauri::State<application::catalog::access::CatalogAccessSession>, request: commands::catalog::ProductLocationLifecycleRequest) -> commands::catalog::ProductLocationResponse {
    if !catalog_access_authorized(&license, &access) { return license_required_location_response(); }
    state.with_write(|connection| Ok(commands::catalog::set_product_location_activity(connection, request, false)))
        .unwrap_or_else(|_| commands::catalog::ProductLocationResponse::Error(commands::catalog::CatalogLocationError { code: "persistence_failure", message: "The location change could not be completed." }))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn delete_product_location_command(state: tauri::State<AppState>, license: tauri::State<commands::license::LicenseCommandState>, access: tauri::State<application::catalog::access::CatalogAccessSession>, request: commands::catalog::ProductLocationLifecycleRequest) -> commands::catalog::ProductLocationResponse {
    if !catalog_access_authorized(&license, &access) { return license_required_location_response(); }
    state.with_write(|connection| Ok(commands::catalog::delete_product_location(connection, request)))
        .unwrap_or_else(|_| commands::catalog::ProductLocationResponse::Error(commands::catalog::CatalogLocationError { code: "persistence_failure", message: "The location change could not be completed." }))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn assign_product_primary_location_command(state: tauri::State<AppState>, license: tauri::State<commands::license::LicenseCommandState>, access: tauri::State<application::catalog::access::CatalogAccessSession>, request: commands::catalog::AssignProductLocationRequest) -> commands::catalog::ProductLocationResponse {
    if !catalog_access_authorized(&license, &access) { return license_required_location_response(); }
    state.with_write(|connection| Ok(commands::catalog::assign_product_primary_location(connection, request)))
        .unwrap_or_else(|_| commands::catalog::ProductLocationResponse::Error(commands::catalog::CatalogLocationError { code: "persistence_failure", message: "The location change could not be completed." }))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn list_categories_command(
    state: tauri::State<AppState>, license: tauri::State<commands::license::LicenseCommandState>,
) -> Result<commands::onboarding::ListCategoriesResponse, String> {
    if license.authorize_business_operation().is_err() { return Err("license_required".into()); }
    state.with_read(commands::onboarding::list_categories)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn create_category_command(
    state: tauri::State<AppState>,
    license: tauri::State<commands::license::LicenseCommandState>,
    request: application::catalog::CreateCategoryInput,
) -> Result<commands::onboarding::CreateCategoryResponse, String> {
    if license.authorize_business_operation().is_err() { return Ok(commands::onboarding::CreateCategoryResponse::Error(commands::onboarding::OnboardingError { code: "license_required", message: "A valid license is required to change the catalog.", field_error: None })); }
    state.with_write(|connection| commands::onboarding::create_category(connection, request))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn create_product_command(
    state: tauri::State<AppState>,
    license: tauri::State<commands::license::LicenseCommandState>,
    request: application::catalog::CreateProductInput,
) -> Result<commands::onboarding::CreateProductResponse, String> {
    if license.authorize_business_operation().is_err() { return Ok(commands::onboarding::CreateProductResponse::Error(commands::onboarding::OnboardingError { code: "license_required", message: "A valid license is required to change the catalog.", field_error: None })); }
    state.with_write(|connection| commands::onboarding::create_product(connection, request))
}

#[cfg(test)]
mod sales_browse_contract_tests {
    use super::*;

    fn request(page_size: i64) -> commands::catalog::BrowseProductsRequest {
        commands::catalog::BrowseProductsRequest {
            query: Some("FLT".into()), category_id: Some(1),
            stock_state: "all".into(), activity: "active".into(), page: 1, page_size,
        }
    }

    #[test]
    fn sales_command_serializes_only_operational_facts_with_nullable_location() {
        use application::catalog::locations::{assign_product_location, create_product_location, save_location_schema, CreateProductLocationInput, SaveLocationSchemaInput};
        let mut connection = infrastructure::sqlite::open_seeded_catalog().unwrap();
        connection.execute("UPDATE products SET purchase_price_centavos = 3200 WHERE id = 1", []).unwrap();
        let unassigned = serde_json::to_value(commands::catalog::browse_sale_products(&connection, request(20)).unwrap()).unwrap();
        assert_eq!(unassigned["products"][0]["purchase_price_centavos"], 3200);
        assert!(unassigned["products"][0].as_object().unwrap().contains_key("primary_location_code"));
        assert_eq!(unassigned["products"][0]["primary_location_code"], serde_json::Value::Null);
        save_location_schema(&mut connection, SaveLocationSchemaInput { expected_revision: 0, segments: vec!["Zone".into()] }).unwrap();
        let location = create_product_location(&mut connection, CreateProductLocationInput { values: vec!["A-1".into()] }).unwrap();
        assign_product_location(&mut connection, 1, 0, Some(location.location_id)).unwrap();
        let changes_before = connection.total_changes();
        let assigned = serde_json::to_value(commands::catalog::browse_sale_products(&connection, request(20)).unwrap()).unwrap();
        assert_eq!(assigned["products"][0]["primary_location_code"], "A1");
        let keys = assigned["products"][0].as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>();
        assert_eq!(keys, vec!["attribute_values", "available_quantity", "category_id", "category_name", "minimum_sale_price_centavos", "name", "primary_location_code", "product_id", "purchase_price_centavos", "sale_price_centavos", "sku"]);
        assert_eq!(assigned["products"][0]["available_quantity"], unassigned["products"][0]["available_quantity"]);
        assert_eq!(connection.total_changes(), changes_before);
        connection.execute("UPDATE products SET purchase_price_centavos = NULL WHERE id = 1", []).unwrap();
        let unknown = serde_json::to_value(commands::catalog::browse_sale_products(&connection, request(20)).unwrap()).unwrap();
        assert!(unknown["products"][0].as_object().unwrap().contains_key("purchase_price_centavos"));
        assert_eq!(unknown["products"][0]["purchase_price_centavos"], serde_json::Value::Null);
    }

    #[test]
    fn sales_command_keeps_page_size_bounded() {
        let connection = infrastructure::sqlite::open_seeded_catalog().unwrap();
        assert_eq!(commands::catalog::browse_sale_products(&connection, request(51)).unwrap_err(), "validation_error");
    }
}

#[cfg(all(test, feature = "desktop"))]
mod command_surface_tests {
    use super::*;
    use tauri::{
        ipc::CallbackFn,
        test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY},
        webview::InvokeRequest,
        WebviewWindowBuilder,
    };

    #[derive(Debug, PartialEq, Eq)]
    struct PersistenceSnapshot {
        stock: Vec<(i64, i64)>,
        sales: i64,
        payments: i64,
        sale_lines: i64,
        movements: i64,
        categories: Vec<(i64, String, i64)>,
        products: Vec<(i64, String, i64)>,
        locations: i64,
        location_segments: i64,
        location_values: i64,
        attribute_definitions: i64,
        attribute_values: i64,
        images: i64,
    }

    fn snapshot(connection: &rusqlite::Connection) -> PersistenceSnapshot {
        let count = |table| {
            connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap()
        };
        let stock = connection
            .prepare("SELECT product_id, quantity FROM stock_balances ORDER BY product_id")
            .unwrap()
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<Vec<(i64, i64)>, _>>()
            .unwrap();
        PersistenceSnapshot {
            stock,
            sales: count("sales"),
            payments: count("sale_payments"),
            sale_lines: count("sale_lines"),
            movements: count("inventory_movements"),
            categories: connection.prepare("SELECT id, name, revision FROM categories ORDER BY id").unwrap().query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).unwrap().collect::<Result<Vec<_>, _>>().unwrap(),
            products: connection.prepare("SELECT id, name, revision FROM products ORDER BY id").unwrap().query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).unwrap().collect::<Result<Vec<_>, _>>().unwrap(),
            locations: count("product_locations"),
            location_segments: count("location_segments"),
            location_values: count("product_location_values"),
            attribute_definitions: count("attribute_definitions"),
            attribute_values: count("product_attribute_values"),
            images: count("product_images"),
        }
    }

    #[cfg(any(windows, target_os = "android"))]
    const IPC_URL: &str = "http://tauri.localhost";
    #[cfg(not(any(windows, target_os = "android")))]
    const IPC_URL: &str = "tauri://localhost";

    fn request(command: &str) -> InvokeRequest {
        InvokeRequest {
            cmd: command.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: IPC_URL.parse().unwrap(),
            body: Default::default(),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_owned(),
        }
    }

    fn request_with(command: &str, payload: serde_json::Value) -> InvokeRequest {
        InvokeRequest {
            body: serde_json::json!({ "request": payload }).into(),
            ..request(command)
        }
    }

    fn test_window() -> (
        tauri::App<tauri::test::MockRuntime>,
        tauri::WebviewWindow<tauri::test::MockRuntime>,
    ) {
        test_window_with_authority(true)
    }

    fn test_window_with_authority(authorized: bool) -> (
        tauri::App<tauri::test::MockRuntime>,
        tauri::WebviewWindow<tauri::test::MockRuntime>,
    ) {
        let license = commands::license::LicenseCommandState::new(
            application::license::LicenseService::new(
                infrastructure::windows_machine_identity::SystemMachineIdentity,
                application::license::FileLicenseStorage::new(&std::env::temp_dir()),
            ),
        );
        license.set_test_authorized(authorized);
        let catalog_access = application::catalog::access::CatalogAccessSession::open(
            infrastructure::filesystem::catalog_access::CatalogAccessStore::new(&std::env::temp_dir().join(format!("catalog-access-test-{}", uuid::Uuid::new_v4()))),
        );
        catalog_access.set_test_authorized(authorized);
        let app = command_builder(mock_builder())
            .manage(catalog_access)
            .manage(AppState::from_connection(
                infrastructure::sqlite::production_database_config(std::env::temp_dir()),
                infrastructure::sqlite::open_seeded_catalog().unwrap(),
            ))
            .manage(license)
            .manage(Mutex::new(commands::backup::BackupCommandState::new(
                std::env::temp_dir(),
            )))
            .build(mock_context(noop_assets()))
            .unwrap();
        let window = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        (app, window)
    }

    #[test]
    fn rejects_excluded_onboarding_operations_without_persistence_mutation() {
        let (app, window) = test_window();
        let before = app
            .state::<AppState>()
            .with_read(|connection| Ok(snapshot(connection)))
            .unwrap();

        for command in [
            "set_cart_line_price_command",
            "create_supplier_command",
            "record_supplier_cost_command",
            "update_product_command",
            "inventory_report_command",
            "backup_database_command",
            "restore_database_command",
            "create_role_command",
            "sync_cloud_command",
            "transfer_stock_command",
        ] {
            assert!(
                get_ipc_response(&window, request(command)).is_err(),
                "{command}"
            );
        }

        assert_eq!(
            app.state::<AppState>()
                .with_read(|connection| Ok(snapshot(connection)))
                .unwrap(),
            before
        );
    }

    #[test]
    fn rejecting_draft_removal_and_discard_leaves_persistence_unchanged() {
        let (app, window) = test_window();
        let before = app
            .state::<AppState>()
            .with_read(|connection| Ok(snapshot(connection)))
            .unwrap();

        for command in [
            "remove_draft_cart_line_command",
            "discard_draft_cart_command",
        ] {
            assert!(
                get_ipc_response(&window, request(command)).is_err(),
                "{command}"
            );
        }

        assert_eq!(
            app.state::<AppState>()
                .with_read(|connection| Ok(snapshot(connection)))
                .unwrap(),
            before
        );
    }

    #[test]
    fn registers_license_commands_with_stable_path_free_responses() {
        let (_app, window) = test_window();
        let status = get_ipc_response(&window, request("license_status_command")).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(status["kind"], "status");
        assert!(status["code"].as_str().is_some_and(|code| !code.is_empty()));
        let code = get_ipc_response(&window, request("license_installation_code_command")).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert!(matches!(code["kind"].as_str(), Some("code" | "error")));
        let selection = get_ipc_response(&window, request("choose_license_file_command")).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(selection, serde_json::json!({"kind":"cancelled"}));
        let imported = get_ipc_response(&window, request("import_license_command")).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(imported, serde_json::json!({"kind":"cancelled"}));
        assert!(!status.to_string().contains("path"));
        assert!(!code.to_string().contains("path"));
    }

    #[test]
    fn rejects_all_business_mutations_and_both_restore_stages_without_data_changes() {
        let (app, window) = test_window_with_authority(false);
        let before = app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap();
        let cases = [
            ("confirm_sale_command", serde_json::json!({"request_id":"550e8400-e29b-41d4-a716-446655440099","lines":[],"payment":{"amount_tendered_centavos":null,"qr_applied_centavos":null}})),
            ("create_sale_return_command", serde_json::json!({"request_id":"550e8400-e29b-41d4-a716-446655440098","sale_id":1,"lines":[]})),
            ("cancel_sale_command", serde_json::json!({"request_id":"550e8400-e29b-41d4-a716-446655440097","sale_id":1,"reason":"test"})),
            ("confirm_stock_entry_command", serde_json::json!({"request_id":"550e8400-e29b-41d4-a716-446655440096","product_id":1,"quantity":1,"unit_purchase_price_centavos":1})),
            ("confirm_physical_count_command", serde_json::json!({"request_id":"550e8400-e29b-41d4-a716-446655440095","product_id":1,"count":7,"reason":"count"})),
            ("maintain_catalog_command", serde_json::json!({"target":"product","entity_id":1,"intent":"archive","expected_revision":0})),
            ("edit_catalog_command", serde_json::json!({"target":"category","entity_id":1,"expected_revision":0,"name":"Changed"})),
            ("edit_category_schema_command", serde_json::json!({"category_id":1,"expected_revision":0,"fields":[]})),
            ("remove_product_image_command", serde_json::json!({"product_id":1,"expected_revision":0})),
            ("choose_product_image_command", serde_json::json!({"product_id":1,"expected_revision":0})),
            ("save_location_schema_command", serde_json::json!({"expected_revision":0,"segments":["Zone"]})),
            ("create_product_location_command", serde_json::json!({"values":["A1"]})),
            ("activate_product_location_command", serde_json::json!({"location_id":1,"expected_revision":0})),
            ("deactivate_product_location_command", serde_json::json!({"location_id":1,"expected_revision":0})),
            ("delete_product_location_command", serde_json::json!({"location_id":1,"expected_revision":0})),
            ("assign_product_primary_location_command", serde_json::json!({"product_id":1,"expected_revision":0,"location_id":1})),
            ("create_category_command", serde_json::json!({"name":"Blocked","fields":[]})),
            ("create_product_command", serde_json::json!({"sku":"BLOCKED","name":"Blocked","category_id":1,"purchase_price_centavos":1,"sale_price_centavos":2,"minimum_sale_price_centavos":1,"opening_quantity":1,"attribute_values":[]})),
            ("prepare_restore_command", serde_json::json!({"source_token":"unissued-token"})),
            ("confirm_restore_command", serde_json::json!({"token":"unknown","confirmed":true})),
        ];
        for (command, payload) in cases {
            let response = get_ipc_response(&window, request_with(command, payload)).unwrap_or_else(|error| panic!("{command}: {error:?}"));
            let value = response.deserialize::<serde_json::Value>().unwrap();
            assert_eq!(value["code"], "license_required", "{command}: {value}");
            assert!(!value.to_string().contains("sqlite") && !value.to_string().contains("path"), "{command}: {value}");
        }
        assert_eq!(app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap(), before);
    }

    #[test]
    fn active_test_authority_allows_a_business_write() {
        let (_app, window) = test_window_with_authority(true);
        let response = get_ipc_response(&window, request_with("confirm_sale_command", serde_json::json!({
            "request_id":"550e8400-e29b-41d4-a716-446655440094",
            "lines":[{"product_id":1,"quantity":1,"captured_unit_price_centavos":2500}],
            "payment":{"amount_tendered_centavos":null,"qr_applied_centavos":2500}
        }))).unwrap();
        assert_eq!(response.deserialize::<serde_json::Value>().unwrap()["kind"], "success");
    }

    #[test]
    fn catalog_reads_and_mutations_are_denied_until_device_access_is_unlocked() {
        let (app, window) = test_window();
        app.state::<application::catalog::access::CatalogAccessSession>().set_test_authorized(false);
        let before = app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap();
        let browse = get_ipc_response(&window, request_with("browse_products_command", serde_json::json!({"stock_state":"all","activity":"active","page":1,"page_size":20}))).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(browse["code"], "catalog_access_required");
        let listing = get_ipc_response(&window, request("list_catalog_categories_command")).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(listing["code"], "catalog_access_required");
        let edit = get_ipc_response(&window, request_with("edit_catalog_command", serde_json::json!({"target":"category","entity_id":1,"expected_revision":0,"name":"Blocked"}))).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(edit["code"], "catalog_access_required");
        assert_eq!(app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap(), before);
    }

    #[test]
    fn explicit_catalog_lock_clears_the_session_and_maintenance_stays_gated() {
        let (app, window) = test_window();
        let access = app.state::<application::catalog::access::CatalogAccessSession>();
        assert!(access.is_authorized());
        let response = get_ipc_response(&window, request("catalog_access_lock_command")).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(response["kind"], "success");
        assert!(!access.is_authorized());
        let status = get_ipc_response(&window, request("catalog_access_status_command")).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(status["status"], "locked");
        let listing = get_ipc_response(&window, request("list_catalog_categories_command")).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(listing["code"], "catalog_access_required");
        access.set_test_authorized(true);
        assert!(access.is_authorized(), "an authorized unlock can start a new session");
    }

    #[test]
    fn inventory_and_onboarding_operations_work_while_catalog_is_locked_with_safe_inventory_projection() {
        let (app, window) = test_window();
        app.state::<application::catalog::access::CatalogAccessSession>().set_test_authorized(false);
        app.state::<AppState>().with_write(|connection| {
            connection.execute("UPDATE products SET purchase_price_centavos = 7777, revision = 9 WHERE id = 1", []).map_err(|_| "test_setup_failed")?;
            connection.execute("INSERT INTO attribute_definitions (id, category_id, label, field_type, required, active) VALUES (9001, 1, 'Material', 'text', 0, 1), (9002, 1, 'Retired secret', 'text', 0, 0)", []).map_err(|_| "test_setup_failed")?;
            connection.execute("INSERT INTO product_attribute_values (product_id, definition_id, text_value, searchable_value) VALUES (1, 9001, 'Steel', 'Steel'), (1, 9002, 'retired-secret', 'retired-secret')", []).map_err(|_| "test_setup_failed")?;
            Ok(())
        }).unwrap();
        let browse = get_ipc_response(&window, request_with("browse_inventory_products_command", serde_json::json!({"query":"filtro","category_id":1,"stock_state":"all","activity":"active","page":1,"page_size":20}))).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(browse["products"].as_array().unwrap().len(), 1);
        for forbidden in ["purchase_price_centavos", "revision", "attribute_values", "retired-secret", "7777", "activity"] {
            assert!(!browse.to_string().contains(forbidden), "unexpected Inventory field {forbidden}: {browse}");
        }
        assert!(get_ipc_response(&window, request("list_categories_command")).is_ok());
        assert!(get_ipc_response(&window, request_with("create_category_command", serde_json::json!({"name":"Locked onboarding","fields":[]}))).is_ok());
        let product = get_ipc_response(&window, request_with("create_product_command", serde_json::json!({"sku":"LOCKED-1","name":"Locked onboarding product","category_id":1,"purchase_price_centavos":100,"sale_price_centavos":200,"minimum_sale_price_centavos":100,"opening_quantity":1,"attribute_values":[]}))).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(product["kind"], "success");
        assert!(get_ipc_response(&window, request("onboarding_location_schema_command")).is_ok());
        assert!(get_ipc_response(&window, request("onboarding_list_product_locations_command")).is_ok());
        let assignment = get_ipc_response(&window, request_with("onboarding_assign_product_primary_location_command", serde_json::json!({"product_id":product["product_id"],"expected_revision":0,"location_id":null}))).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(assignment["kind"], "assignment_success");
        let detail = get_ipc_response(&window, request_with("catalog_metadata_detail_command", serde_json::json!({"target":"product","entity_id":1}))).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(detail["code"], "catalog_access_required");
        let maintain = get_ipc_response(&window, request_with("maintain_catalog_command", serde_json::json!({"target":"product","entity_id":1,"intent":"archive","expected_revision":9}))).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(maintain["code"], "catalog_access_required");
    }

    #[test]
    fn physical_count_ipc_requires_the_current_catalog_password_without_unlocking_catalog() {
        let (app, window) = test_window();
        let access = app.state::<application::catalog::access::CatalogAccessSession>();
        access.begin_setup("old-password").unwrap();
        access.finish_setup(true).unwrap();
        access.lock().unwrap();
        let before = app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap();
        for payload in [
            serde_json::json!({"request_id":"550e8400-e29b-41d4-a716-446655440250","product_id":1,"count":7,"reason":"count"}),
            serde_json::json!({"request_id":"550e8400-e29b-41d4-a716-446655440251","product_id":1,"count":7,"reason":"count","catalog_password":"wrong-password"}),
        ] {
            let response = get_ipc_response(&window, request_with("confirm_physical_count_command", payload)).unwrap().deserialize::<serde_json::Value>().unwrap();
            assert_eq!(response["code"], "catalog_password_invalid");
            assert_eq!(app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap(), before);
            assert_eq!(access.status(), application::catalog::access::CatalogAccessStatus::Locked);
        }
        let success = get_ipc_response(&window, request_with("confirm_physical_count_command", serde_json::json!({
            "request_id":"550e8400-e29b-41d4-a716-446655440252","product_id":1,"count":7,"reason":"count","catalog_password":"old-password"
        }))).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(success["kind"], "success");
        assert_eq!(access.status(), application::catalog::access::CatalogAccessStatus::Locked);
        assert_ne!(app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap(), before);
    }

    #[test]
    fn sales_browse_and_category_filters_work_while_catalog_is_locked_with_approved_cost() {
        let (app, window) = test_window();
        app.state::<application::catalog::access::CatalogAccessSession>().set_test_authorized(false);
        app.state::<AppState>().with_write(|connection| {
            connection.execute("UPDATE products SET purchase_price_centavos = 7777 WHERE id = 1", []).map_err(|_| "test_setup_failed")?;
            connection.execute("INSERT INTO attribute_definitions (id, category_id, label, field_type, required, active) VALUES (9001, 1, 'Material', 'text', 0, 1), (9002, 1, 'Retired grade', 'text', 0, 0)", []).map_err(|_| "test_setup_failed")?;
            connection.execute("INSERT INTO product_attribute_values (product_id, definition_id, text_value, searchable_value) VALUES (1, 9001, 'Acero', 'Acero'), (1, 9002, 'Sensitive retired', 'Sensitive retired')", []).map_err(|_| "test_setup_failed")?;
            Ok(())
        }).unwrap();
        let response = get_ipc_response(&window, request_with("browse_sale_products_command", serde_json::json!({"query":"filtro","category_id":1,"stock_state":"all","activity":"active","page":1,"page_size":20}))).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(response["products"].as_array().unwrap().len(), 1);
        assert_eq!(response["products"][0]["category_id"], 1);
        assert_eq!(response["categories"][0]["category_id"], 1);
        assert_eq!(response["products"][0]["attribute_values"], serde_json::json!([{"definition_id":9001,"label":"Material","value":"Acero"}]));
        assert_eq!(response["products"][0]["purchase_price_centavos"], 7777);
        let before = app.state::<AppState>().with_read(|connection| Ok(connection.total_changes())).unwrap();
        app.state::<application::catalog::access::CatalogAccessSession>().set_test_authorized(true);
        let unlocked = get_ipc_response(&window, request_with("browse_sale_products_command", serde_json::json!({"query":"filtro","category_id":1,"stock_state":"all","activity":"active","page":1,"page_size":20}))).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(unlocked, response);
        app.state::<application::catalog::access::CatalogAccessSession>().set_test_authorized(false);
        assert_eq!(app.state::<AppState>().with_read(|connection| Ok(connection.total_changes())).unwrap(), before);
        app.state::<AppState>().with_write(|connection| {
            connection.execute("UPDATE products SET purchase_price_centavos = NULL WHERE id = 1", []).map_err(|_| "test_setup_failed")?;
            Ok(())
        }).unwrap();
        let unknown = get_ipc_response(&window, request_with("browse_sale_products_command", serde_json::json!({"query":"filtro","category_id":1,"stock_state":"all","activity":"active","page":1,"page_size":20}))).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert!(unknown["products"][0].as_object().unwrap().contains_key("purchase_price_centavos"));
        assert_eq!(unknown["products"][0]["purchase_price_centavos"], serde_json::Value::Null);
        assert!(!response.to_string().contains("Sensitive retired"));
        assert!(response["products"][0].as_object().unwrap().contains_key("primary_location_code"));
        assert_eq!(response["products"][0]["primary_location_code"], serde_json::Value::Null);
        for forbidden in ["primary_location_id", "active_product_count", "low_stock_threshold", "revision", "active", "profit_margin"] {
            assert!(!response.to_string().contains(forbidden), "unexpected sensitive field {forbidden}: {response}");
        }
        let listing = get_ipc_response(&window, request("list_catalog_categories_command")).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(listing["code"], "catalog_access_required");
        let all = get_ipc_response(&window, request_with("browse_sale_products_command", serde_json::json!({"query":null,"category_id":null,"stock_state":"all","activity":"active","page":1,"page_size":20}))).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert!(all["products"].as_array().unwrap().len() >= 1);
        assert!(all["categories"].as_array().unwrap().iter().any(|category| category["category_id"] == 1));
        let admin_thumbnail = get_ipc_response(&window, request_with("catalog_product_image_thumbnail_command", serde_json::json!({"product_id":1,"expected_revision":0}))).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(admin_thumbnail["code"], "catalog_access_required");
        app.state::<AppState>().with_write(|connection| {
            use application::catalog::locations::{save_location_schema, create_product_location, assign_product_location, SaveLocationSchemaInput, CreateProductLocationInput};
            save_location_schema(connection, SaveLocationSchemaInput { expected_revision: 0, segments: vec!["Zone".into(), "Shelf".into()] }).map_err(|_| "test_setup_failed")?;
            let location = create_product_location(connection, CreateProductLocationInput { values: vec!["A-1".into(), "Shelf 2".into()] }).map_err(|_| "test_setup_failed")?;
            assign_product_location(connection, 1, 0, Some(location.location_id)).map_err(|_| "test_setup_failed")?;
            let mut bytes = Vec::new();
            image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(2, 2, image::Rgb([20, 40, 60])))
                .write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Jpeg)
                .map_err(|_| "test_setup_failed")?;
            let image = application::catalog::ProductImage::new("image/jpeg", bytes).map_err(|_| "test_setup_failed")?;
            application::catalog::replace_product_image(connection, 1, 1, &image).map_err(|_| "test_setup_failed")?;
            Ok(())
        }).unwrap();
        let located = get_ipc_response(&window, request_with("browse_sale_products_command", serde_json::json!({"query":"filtro","category_id":1,"stock_state":"all","activity":"active","page":1,"page_size":20}))).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(located["products"][0]["primary_location_code"], "A1-SHELF2");
        for forbidden in ["primary_location_id", "active_product_count", "low_stock_threshold", "revision", "active", "profit_margin"] {
            assert!(!located.to_string().contains(forbidden));
        }
        assert_eq!(get_ipc_response(&window, request("list_catalog_categories_command")).unwrap().deserialize::<serde_json::Value>().unwrap()["code"], "catalog_access_required");
        let sales_thumbnail = get_ipc_response(&window, request_with("sales_product_image_thumbnail_command", serde_json::json!({"product_id":1}))).unwrap().deserialize::<serde_json::Value>().unwrap();
        let sales_original = get_ipc_response(&window, request_with("sales_product_image_original_command", serde_json::json!({"product_id":1}))).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(sales_original["kind"], "success");
        assert_eq!(sales_original["mime_type"], "image/jpeg");
        assert_eq!(sales_original.as_object().unwrap().len(), 5);
        assert_eq!(get_ipc_response(&window, request_with("catalog_product_image_thumbnail_command", serde_json::json!({"product_id":1,"expected_revision":0}))).unwrap().deserialize::<serde_json::Value>().unwrap()["code"], "catalog_access_required");
        assert_eq!(sales_thumbnail["kind"], "success");
        assert_eq!(sales_thumbnail["product_id"], 1);
        assert_eq!(sales_thumbnail["mime_type"], "image/jpeg");
        assert_eq!(sales_thumbnail["encoding"], "base64");
        assert!(sales_thumbnail["bytes"].as_str().is_some_and(|bytes| !bytes.is_empty()));
        for forbidden in ["revision", "purchase_price_centavos", "category_name", "sku", "path"] {
            assert!(!sales_thumbnail.to_string().contains(forbidden));
        }
    }

    #[test]
    fn sale_search_and_inventory_read_surfaces_stay_available_while_catalog_is_locked_without_cost() {
        let (app, window) = test_window();
        app.state::<application::catalog::access::CatalogAccessSession>().set_test_authorized(false);
        app.state::<AppState>().with_write(|connection| {
            connection.execute("UPDATE products SET purchase_price_centavos = 7777 WHERE id = 1", []).map_err(|_| "test_setup_failed")?;
            connection.execute("UPDATE stock_balances SET quantity = 0 WHERE product_id = 1", []).map_err(|_| "test_setup_failed")?;
            Ok(())
        }).unwrap();

        let search = get_ipc_response(&window, request_with("search_products_command", serde_json::json!({"query":"filtro"}))).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(search.as_array().unwrap().len(), 1);
        let product = &search[0];
        assert_eq!(product["sku"], "FLT-001");
        assert_eq!(product["sale_price_centavos"], 2500);
        assert!(product.get("purchase_price_centavos").is_none());
        assert!(product.get("minimum_sale_price_centavos").is_some());
        assert!(product.get("revision").is_none());
        assert!(product.get("attribute_values").is_none());
        assert!(product.get("category_id").is_none());

        let alerts = get_ipc_response(&window, request("list_inventory_alerts_command")).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert!(alerts.to_string().contains("FLT-001"));
        assert!(!alerts.to_string().contains("purchase_price_centavos"));
        assert!(!alerts.to_string().contains("7777"));
        let options = get_ipc_response(&window, request_with("list_movement_ledger_product_options_command", serde_json::json!({"query":"filtro","page":1}))).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(options["kind"], "success");
        assert!(!options.to_string().contains("purchase_price_centavos"));
        assert!(!options.to_string().contains("7777"));
    }

    #[test]
    fn reads_activation_backup_and_inventory_alerts_remain_available_unlicensed() {
        let (_app, window) = test_window_with_authority(false);
        for command in ["license_status_command", "license_installation_code_command", "choose_license_file_command", "import_license_command", "list_inventory_alerts_command"] {
            assert!(get_ipc_response(&window, request(command)).is_ok(), "{command}");
        }
        let catalog = get_ipc_response(&window, request("list_catalog_maintenance_command")).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(catalog["code"], "catalog_access_required");
        let alerts = get_ipc_response(&window, request("list_inventory_alerts_command")).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(alerts["kind"], "alerts");
        let backup = get_ipc_response(&window, request_with("create_backup_command", serde_json::json!({"destination_token": "webview-supplied-token"}))).unwrap().deserialize::<serde_json::Value>().unwrap();
        assert_eq!(backup["code"], "destination_token_invalid", "{backup}");
    }

    #[test]
    fn registers_catalog_image_commands_without_exposing_picker_paths() {
        let (_app, window) = test_window();
        let picker = get_ipc_response(&window, request_with("choose_product_image_command", serde_json::json!({ "product_id": 1, "expected_revision": 0 }))).unwrap();
        assert_eq!(picker.deserialize::<serde_json::Value>().unwrap(), serde_json::json!({ "kind": "cancelled" }));
        for command in ["remove_product_image_command", "catalog_product_image_thumbnail_command"] {
            let response = get_ipc_response(&window, request_with(command, serde_json::json!({ "product_id": 1, "expected_revision": 0 }))).unwrap();
            let value = response.deserialize::<serde_json::Value>().unwrap();
            assert!(!value.to_string().contains("path"));
        }
    }

    #[test]
    fn registers_product_location_contract_commands_at_the_tauri_command_seam() {
        let (_app, window) = test_window();
        assert!(get_ipc_response(&window, request("location_schema_command")).is_ok());
        assert!(get_ipc_response(&window, request_with("save_location_schema_command", serde_json::json!({ "expected_revision": 0, "segments": ["Zone"] }))).is_ok());
        assert!(get_ipc_response(&window, request_with("list_product_locations_command", serde_json::json!({ "include_inactive": false }))).is_ok());
        assert!(get_ipc_response(&window, request_with("create_product_location_command", serde_json::json!({ "values": ["A1"] }))).is_ok());
        for (command, payload) in [
            ("activate_product_location_command", serde_json::json!({ "location_id": 1, "expected_revision": 0 })),
            ("deactivate_product_location_command", serde_json::json!({ "location_id": 1, "expected_revision": 0 })),
            ("delete_product_location_command", serde_json::json!({ "location_id": 1, "expected_revision": 0 })),
            ("assign_product_primary_location_command", serde_json::json!({ "product_id": 1, "expected_revision": 0, "location_id": 1 })),
        ] {
            assert!(get_ipc_response(&window, request_with(command, payload)).is_ok(), "{command}");
        }
    }

    #[test]
    fn registers_catalog_maintenance_listing_at_the_tauri_command_seam() {
        let (_app, window) = test_window();
        assert!(get_ipc_response(&window, request("list_catalog_maintenance_command")).is_ok());
    }

    #[test]
    fn registers_post_sale_commands_at_the_tauri_command_seam() {
        let (_app, window) = test_window();
        for (command, payload) in [
            (
                "create_sale_return_command",
                serde_json::json!({
                    "request_id": "550e8400-e29b-41d4-a716-446655440041",
                    "sale_id": 1, "lines": []
                }),
            ),
            (
                "cancel_sale_command",
                serde_json::json!({
                    "request_id": "550e8400-e29b-41d4-a716-446655440042",
                    "sale_id": 1, "reason": "correction"
                }),
            ),
        ] {
            assert!(
                get_ipc_response(&window, request_with(command, payload)).is_ok(),
                "{command}"
            );
        }
    }

    #[test]
    fn registers_read_only_dashboard_command_at_the_tauri_command_seam() {
        let (app, window) = test_window();
        let before = app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap();
        assert!(get_ipc_response(&window, request_with("dashboard_command", serde_json::json!({
            "today_from_utc": "2024-03-10T05:00:00Z",
            "today_to_exclusive_utc": "2024-03-11T04:00:00Z",
            "month_from_utc": "2024-03-01T05:00:00Z",
            "month_to_exclusive_utc": "2024-04-01T04:00:00Z"
        }))).is_ok());
        assert_eq!(app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap(), before);
    }

    #[test]
    fn registers_gross_profit_operations_command_with_strict_paged_request_at_ipc_seam() {
        let (app, window) = test_window();
        let before = app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap();
        let response = get_ipc_response(&window, request_with("gross_profit_operations_command", serde_json::json!({
            "from_utc": "2024-03-10T05:00:00.000Z",
            "to_exclusive_utc": "2024-03-11T04:00:00.000Z",
            "page": 1,
            "page_size": 20
        }))).unwrap();
        let payload = response.deserialize::<serde_json::Value>().unwrap();
        assert_eq!(payload["kind"], "success");
        assert_eq!(payload["report"]["total"], 0);
        assert_eq!(app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap(), before);
    }

    #[test]
    fn registers_gross_profit_pdf_export_without_exposing_a_path_at_the_ipc_seam() {
        let (_app, window) = test_window();
        let response = get_ipc_response(&window, request_with("export_gross_profit_operations_command", serde_json::json!({
            "from": { "local_date": "2024-03-10", "utc": "2024-03-10T05:00:00.000Z", "utc_offset_minutes": 300 },
            "to_exclusive": { "local_date": "2024-03-11", "utc": "2024-03-11T04:00:00.000Z", "utc_offset_minutes": 240 }
        }))).unwrap();
        let value = response.deserialize::<serde_json::Value>().unwrap();
        assert_eq!(value, serde_json::json!({ "kind": "cancelled" }));
        assert!(!value.to_string().contains("path"));
        assert!(get_ipc_response(&window, request_with("export_gross_profit_operations_command", serde_json::json!({
            "from": { "local_date": "2024-03-10", "utc": "2024-03-10T05:00:00.000Z", "utc_offset_minutes": 300 },
            "to_exclusive": { "local_date": "2024-03-11", "utc": "2024-03-11T04:00:00.000Z", "utc_offset_minutes": 240 },
            "path": "/private/report.pdf"
        }))).is_err());
    }

    #[test]
    fn registers_read_only_sales_history_commands_at_the_tauri_command_seam() {
        let (app, window) = test_window();
        let before = app
            .state::<AppState>()
            .with_read(|connection| Ok(snapshot(connection)))
            .unwrap();
        assert!(get_ipc_response(
            &window,
            request_with(
                "list_sales_history_command",
                serde_json::json!({
                    "from_utc": "2024-03-10T05:00:00Z",
                    "to_exclusive_utc": "2024-03-11T04:00:00Z"
                }),
            ),
        )
        .is_ok());
        let detail_request = InvokeRequest {
            body: serde_json::json!({ "saleId": 999 }).into(),
            ..request("sale_history_detail_command")
        };
        assert!(get_ipc_response(&window, detail_request).is_ok());
        assert_eq!(
            app.state::<AppState>()
                .with_read(|connection| Ok(snapshot(connection)))
                .unwrap(),
            before
        );
    }

    #[test]
    fn registers_metadata_edit_and_detail_commands_at_the_tauri_command_seam() {
        let (_app, window) = test_window();
        assert!(get_ipc_response(&window, request_with("edit_catalog_command", serde_json::json!({ "target": "category", "entity_id": 1, "expected_revision": 0, "name": "Filters and oils" }))).is_ok());
        assert!(get_ipc_response(&window, request_with("edit_category_schema_command", serde_json::json!({ "category_id": 1, "expected_revision": 1, "fields": [] }))).is_ok());
        assert!(get_ipc_response(
            &window,
            request_with(
                "catalog_metadata_detail_command",
                serde_json::json!({ "target": "product", "entity_id": 1 })
            )
        )
        .is_ok());
    }

    #[test]
    fn registers_backup_commands_at_the_tauri_command_seam() {
        let (_app, window) = test_window();
        for command in [
            "choose_backup_destination_command",
            "choose_restore_source_command",
        ] {
            let response = get_ipc_response(&window, request(command)).unwrap();
            assert_eq!(
                response.deserialize::<serde_json::Value>().unwrap(),
                serde_json::json!({ "kind": "cancelled" }),
                "{command} must return the test cancellation response",
            );
        }
        assert!(get_ipc_response(
            &window,
            request_with(
                "create_backup_command",
                serde_json::json!({ "destination_token": "webview-supplied-token" }),
            ),
        )
        .is_ok());
        assert!(get_ipc_response(
            &window,
            request_with(
                "prepare_restore_command",
                serde_json::json!({ "source_token": "unissued-token" }),
            ),
        )
        .is_ok());
        assert!(get_ipc_response(
            &window,
            request_with(
                "confirm_restore_command",
                serde_json::json!({ "token": "unknown", "confirmed": true }),
            ),
        )
        .is_ok());
    }
}

pub mod catalog {
    pub use crate::application::catalog::{search_active_products, ProductSearchResult};
    pub use crate::infrastructure::sqlite::open_seeded_catalog;
}
