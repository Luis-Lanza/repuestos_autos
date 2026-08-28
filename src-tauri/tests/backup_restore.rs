use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use repuestos_autos::application::backup::{
    BackupCoordinator, ConfirmRestoreResult, OperationalFacts, PrepareRestoreResult,
    ProtectiveBackup, RestoreCandidate, RestoreCandidateStore, RestoreError, RestoreTokenSource,
};
use repuestos_autos::commands::backup::{
    confirm_restore, prepare_restore, BackupCommandState, BackupResponse, ConfirmRestoreRequest,
    PrepareRestoreRequest, ALLOWED_COMMANDS,
};
#[cfg(feature = "desktop")]
use repuestos_autos::commands::backup::{select_callback_path, PathSelection};
use repuestos_autos::infrastructure::filesystem::{BackupStore, StorageError};
use repuestos_autos::infrastructure::sqlite::{
    create_snapshot, production_database_config, stage_and_validate, BackupValidationError,
    CURRENT_SCHEMA_VERSION,
};
use repuestos_autos::{DatabaseState, RestoreState};
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
fn publishes_a_synced_non_overwriting_backup_beneath_an_existing_selected_root() {
    let directory = temporary_directory("publish");
    let snapshot = directory.join("snapshot.sqlite3");
    let selected_root = directory.join("USB á");
    let destination = selected_root.join("backup-restore");
    fs::create_dir_all(&selected_root).unwrap();
    fs::write(&snapshot, b"consistent snapshot").unwrap();

    let store = BackupStore::new(&directory.join("app-data"));
    let published = store
        .publish_snapshot(&snapshot, &destination, "backup-20260827T204000Z.sqlite3")
        .unwrap();

    assert!(destination.is_dir());
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
fn rejects_a_selected_root_that_disappears_before_publication_without_recreating_it() {
    let directory = temporary_directory("removed-selected-root");
    let snapshot = directory.join("snapshot.sqlite3");
    let selected_root = directory.join("removable-like-mount");
    let destination = selected_root.join("backup-restore");
    fs::create_dir_all(&selected_root).unwrap();
    fs::write(&snapshot, b"consistent snapshot").unwrap();
    fs::remove_dir_all(&selected_root).unwrap();

    let store = BackupStore::new(&directory.join("app-data"));

    assert_eq!(
        store.publish_snapshot(&snapshot, &destination, "backup-20260827T204000Z.sqlite3"),
        Err(StorageError::StorageUnavailable)
    );
    assert!(!selected_root.exists());
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

#[test]
fn records_each_durable_replacement_transition() {
    let directory = temporary_directory("restore-state");
    let store = BackupStore::new(&directory);

    for state in [
        RestoreState::Prepared,
        RestoreState::LiveMoved,
        RestoreState::CandidateInstalled,
    ] {
        store.write_restore_state(state).unwrap();
        assert_eq!(store.read_restore_state().unwrap(), Some(state));
    }

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn installs_a_validated_stage_after_closing_the_live_connection() {
    let directory = temporary_directory("replacement");
    fs::create_dir_all(&directory).unwrap();
    let config = production_database_config(&directory);
    versioned_database(config.path(), CURRENT_SCHEMA_VERSION);
    let state = DatabaseState::open(config.clone()).unwrap();
    let stage = directory.join("staging/candidate.sqlite3");
    fs::create_dir_all(stage.parent().unwrap()).unwrap();
    versioned_database(&stage, CURRENT_SCHEMA_VERSION);
    Connection::open(&stage)
        .unwrap()
        .execute(
            "INSERT INTO categories (name) VALUES (?1)",
            ["restored-state"],
        )
        .unwrap();
    let store = BackupStore::new(&directory);

    state.install_validated_stage(&stage, &store).unwrap();

    assert!(!stage.exists());
    assert!(directory.join("pre-restore.sqlite3").exists());
    assert!(directory.join("restore-rollback.sqlite3").exists());
    assert!(!directory.join("restore-state.json").exists());
    assert_eq!(
        state
            .with_read(|connection| {
                connection
                    .query_row(
                        "SELECT COUNT(*) FROM categories WHERE name = ?1",
                        ["restored-state"],
                        |row| row.get::<_, i64>(0),
                    )
                    .map_err(|_| "database_unavailable".into())
            })
            .unwrap(),
        1
    );
    fs::remove_dir_all(directory).unwrap();
}

fn database_with_category(path: &Path, category_name: &str) {
    versioned_database(path, CURRENT_SCHEMA_VERSION);
    Connection::open(path)
        .unwrap()
        .execute("INSERT INTO categories (name) VALUES (?1)", [category_name])
        .unwrap();
}

fn category_count(state: &DatabaseState, category_name: &str) -> i64 {
    state
        .with_read(|connection| {
            connection
                .query_row(
                    "SELECT COUNT(*) FROM categories WHERE name = ?1",
                    [category_name],
                    |row| row.get(0),
                )
                .map_err(|_| "database_unavailable".into())
        })
        .unwrap()
}

#[test]
fn startup_recovery_keeps_valid_canonical_data_for_prepared_and_candidate_installed_markers() {
    for (name, state) in [
        ("recover-prepared", RestoreState::Prepared),
        (
            "recover-candidate-installed",
            RestoreState::CandidateInstalled,
        ),
    ] {
        let directory = temporary_directory(name);
        fs::create_dir_all(&directory).unwrap();
        let config = production_database_config(&directory);
        database_with_category(config.path(), "canonical");
        database_with_category(&directory.join("restore-rollback.sqlite3"), "rollback");
        database_with_category(&directory.join("pre-restore.sqlite3"), "protective");
        let store = BackupStore::new(&directory);
        store.write_restore_state(state).unwrap();

        let recovered = DatabaseState::recover_on_startup(config, &store);

        assert_eq!(category_count(&recovered, "canonical"), 1, "{state:?}");
        assert_eq!(category_count(&recovered, "rollback"), 0, "{state:?}");
        assert_eq!(store.read_restore_state().unwrap(), None, "{state:?}");
        assert!(directory.join("pre-restore.sqlite3").exists(), "{state:?}");
        fs::remove_dir_all(directory).unwrap();
    }
}

#[test]
fn startup_recovery_restores_a_valid_rollback_after_live_moved_crash() {
    let directory = temporary_directory("recover-live-moved");
    fs::create_dir_all(&directory).unwrap();
    let config = production_database_config(&directory);
    database_with_category(&directory.join("restore-rollback.sqlite3"), "rollback");
    database_with_category(&directory.join("pre-restore.sqlite3"), "protective");
    let store = BackupStore::new(&directory);
    store.write_restore_state(RestoreState::LiveMoved).unwrap();

    let recovered = DatabaseState::recover_on_startup(config.clone(), &store);

    assert_eq!(category_count(&recovered, "rollback"), 1);
    assert!(config.path().exists());
    assert_eq!(store.read_restore_state().unwrap(), None);
    assert!(directory.join("pre-restore.sqlite3").exists());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn startup_recovery_uses_the_retained_protective_database_when_rollback_is_unavailable() {
    let directory = temporary_directory("recover-protective");
    fs::create_dir_all(&directory).unwrap();
    let config = production_database_config(&directory);
    fs::write(directory.join("restore-rollback.sqlite3"), b"not sqlite").unwrap();
    database_with_category(&directory.join("pre-restore.sqlite3"), "protective");
    let store = BackupStore::new(&directory);
    store.write_restore_state(RestoreState::LiveMoved).unwrap();

    let recovered = DatabaseState::recover_on_startup(config, &store);

    assert_eq!(category_count(&recovered, "protective"), 1);
    assert_eq!(store.read_restore_state().unwrap(), None);
    assert!(directory.join("pre-restore.sqlite3").exists());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn startup_recovery_leaves_database_unavailable_without_a_safe_candidate() {
    let directory = temporary_directory("recover-unavailable");
    fs::create_dir_all(&directory).unwrap();
    let config = production_database_config(&directory);
    fs::write(directory.join("restore-rollback.sqlite3"), b"not sqlite").unwrap();
    fs::write(directory.join("pre-restore.sqlite3"), b"not sqlite").unwrap();
    let store = BackupStore::new(&directory);
    store.write_restore_state(RestoreState::LiveMoved).unwrap();

    let recovered = DatabaseState::recover_on_startup(config.clone(), &store);

    assert_eq!(
        recovered.with_read(|_| Ok(())).unwrap_err(),
        "database_unavailable"
    );
    assert!(!config.path().exists());
    assert_eq!(
        store.read_restore_state().unwrap(),
        Some(RestoreState::LiveMoved)
    );
    assert!(directory.join("pre-restore.sqlite3").exists());
    fs::remove_dir_all(directory).unwrap();
}

#[derive(Clone)]
struct CandidateStore {
    candidate: RestoreCandidate,
    rechecked: RestoreCandidate,
}

impl RestoreCandidateStore for CandidateStore {
    fn prepare(&self, _: &Path) -> Result<RestoreCandidate, RestoreError> {
        Ok(self.candidate.clone())
    }

    fn recheck(&self, _: &RestoreCandidate) -> Result<RestoreCandidate, RestoreError> {
        Ok(self.rechecked.clone())
    }
}

struct BackupProtection {
    should_fail: bool,
    calls: Cell<u8>,
}

impl ProtectiveBackup for BackupProtection {
    fn create_and_validate(&self) -> Result<(), RestoreError> {
        self.calls.set(self.calls.get() + 1);
        if self.should_fail {
            Err(RestoreError::RestoreFailed)
        } else {
            Ok(())
        }
    }
}

struct FixedToken;

impl RestoreTokenSource for FixedToken {
    fn next_token(&mut self) -> String {
        "opaque-token".into()
    }
}

fn candidate() -> RestoreCandidate {
    RestoreCandidate {
        stage: PathBuf::from("staging/candidate.sqlite3"),
        schema_version: CURRENT_SCHEMA_VERSION,
        size_bytes: 4096,
        sha256: "candidate-checksum".into(),
        facts: OperationalFacts {
            catalog_records: 1,
            confirmed_sales: 1,
            stock_balances: 1,
            movement_records: 2,
            schema_history_version: CURRENT_SCHEMA_VERSION,
        },
    }
}

fn coordinator(
    rechecked: RestoreCandidate,
    protective_failure: bool,
) -> BackupCoordinator<CandidateStore, BackupProtection, FixedToken> {
    let candidate = candidate();
    BackupCoordinator::new(
        CandidateStore {
            candidate: candidate.clone(),
            rechecked,
        },
        BackupProtection {
            should_fail: protective_failure,
            calls: Cell::new(0),
        },
        FixedToken,
        60,
    )
}

#[test]
fn requires_explicit_confirmation_and_rejects_mismatched_or_reused_tokens() {
    let mut coordinator = coordinator(candidate(), false);
    let PrepareRestoreResult::Prepared { token, .. } =
        coordinator.prepare(Path::new("backup.sqlite3"), 10)
    else {
        panic!("candidate should prepare");
    };

    assert_eq!(
        coordinator.confirm(&token, false, 11),
        ConfirmRestoreResult::Failed(RestoreError::ConfirmationRequired)
    );
    assert_eq!(
        coordinator.confirm("different-token", true, 11),
        ConfirmRestoreResult::Failed(RestoreError::TokenInvalid)
    );
    let ConfirmRestoreResult::ReadyForReplacement { candidate } =
        coordinator.confirm(&token, true, 11)
    else {
        panic!("confirmed candidate should be ready for Slice 2B");
    };
    assert_eq!(candidate.facts.catalog_records, 1);
    assert_eq!(candidate.facts.confirmed_sales, 1);
    assert_eq!(candidate.facts.stock_balances, 1);
    assert_eq!(candidate.facts.movement_records, 2);
    assert_eq!(
        candidate.facts.schema_history_version,
        CURRENT_SCHEMA_VERSION
    );
    assert_eq!(
        coordinator.confirm(&token, true, 12),
        ConfirmRestoreResult::Failed(RestoreError::TokenInvalid)
    );
}

#[test]
fn rejects_expired_or_changed_candidates_before_protective_backup() {
    let mut expired = coordinator(candidate(), false);
    let PrepareRestoreResult::Prepared { token, .. } =
        expired.prepare(Path::new("backup.sqlite3"), 10)
    else {
        panic!("candidate should prepare");
    };
    assert_eq!(
        expired.confirm(&token, true, 71),
        ConfirmRestoreResult::Failed(RestoreError::TokenExpired)
    );

    let mut changed = candidate();
    changed.sha256 = "changed-checksum".into();
    let mut coordinator = coordinator(changed, true);
    let PrepareRestoreResult::Prepared { token, .. } =
        coordinator.prepare(Path::new("backup.sqlite3"), 10)
    else {
        panic!("candidate should prepare");
    };
    assert_eq!(
        coordinator.confirm(&token, true, 11),
        ConfirmRestoreResult::Failed(RestoreError::InvalidBackup)
    );
}

#[test]
fn aborts_before_replacement_when_protective_backup_fails() {
    let mut coordinator = coordinator(candidate(), true);
    let PrepareRestoreResult::Prepared { token, .. } =
        coordinator.prepare(Path::new("backup.sqlite3"), 10)
    else {
        panic!("candidate should prepare");
    };

    assert_eq!(
        coordinator.confirm(&token, true, 11),
        ConfirmRestoreResult::Failed(RestoreError::RestoreFailed)
    );
}

#[test]
fn backup_command_boundary_serializes_only_allowlisted_stable_and_safe_outcomes() {
    let request: PrepareRestoreRequest =
        serde_json::from_str(r#"{"source":"/media/USB/backup.sqlite3"}"#).unwrap();
    assert_eq!(request.source, PathBuf::from("/media/USB/backup.sqlite3"));
    assert!(serde_json::from_str::<PrepareRestoreRequest>(
        r#"{"source":"backup.sqlite3","unexpected":true}"#
    )
    .is_err());
    assert_eq!(
        serde_json::to_value(BackupResponse::error("invalid_backup")).unwrap(),
        serde_json::json!({
            "kind": "error",
            "code": "invalid_backup",
            "message": "The selected backup is invalid.",
        })
    );
    assert_eq!(
        ALLOWED_COMMANDS,
        [
            "choose_backup_destination_command",
            "choose_restore_source_command",
            "create_backup_command",
            "prepare_restore_command",
            "confirm_restore_command",
        ]
    );

    let directory = temporary_directory("command-unavailable");
    fs::create_dir_all(&directory).unwrap();
    let config = production_database_config(&directory);
    let store = BackupStore::new(&directory);
    fs::write(directory.join("restore-rollback.sqlite3"), b"not sqlite").unwrap();
    fs::write(directory.join("pre-restore.sqlite3"), b"not sqlite").unwrap();
    store.write_restore_state(RestoreState::LiveMoved).unwrap();
    let state = DatabaseState::recover_on_startup(config, &store);
    let mut commands = BackupCommandState::new(&directory);

    assert_eq!(
        prepare_restore(
            &state,
            &mut commands,
            PrepareRestoreRequest {
                source: PathBuf::from("/untrusted/internal.sqlite3"),
            }
        ),
        BackupResponse::error("database_unavailable")
    );
    assert_eq!(
        confirm_restore(
            &state,
            &mut commands,
            ConfirmRestoreRequest {
                token: "unknown-token".into(),
                confirmed: true,
            }
        ),
        BackupResponse::error("token_invalid")
    );
    fs::remove_dir_all(directory).unwrap();
}

#[cfg(feature = "desktop")]
#[test]
fn maps_nonblocking_picker_callbacks_to_the_typed_command_contract() {
    let selected = tauri::async_runtime::block_on(select_callback_path(|complete| {
        complete(Some(PathBuf::from("/media/USB á")));
    }));
    let cancelled = tauri::async_runtime::block_on(select_callback_path(|complete| {
        complete(None);
    }));

    assert_eq!(
        selected,
        PathSelection::Selected {
            path: PathBuf::from("/media/USB á")
        }
    );
    assert_eq!(cancelled, PathSelection::Cancelled);
    assert_eq!(
        serde_json::to_value(cancelled).unwrap(),
        serde_json::json!({ "kind": "cancelled" })
    );
}
