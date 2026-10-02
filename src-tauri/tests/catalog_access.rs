use std::fs;

use repuestos_autos::application::catalog::access::{CatalogAccessSession, CatalogAccessStatus};
use repuestos_autos::infrastructure::filesystem::catalog_access::CatalogAccessStore;
#[cfg(windows)]
use repuestos_autos::infrastructure::filesystem::BackupStore;
#[cfg(windows)]
use repuestos_autos::infrastructure::sqlite::{create_snapshot, production_database_config};
#[cfg(windows)]
use repuestos_autos::DatabaseState;

fn workspace() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("catalog-access-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&path).unwrap();
    path
}

fn session(path: &std::path::Path) -> CatalogAccessSession {
    CatalogAccessSession::open(CatalogAccessStore::new(path))
}

#[test]
fn setup_displays_recovery_once_and_persists_only_distinct_argon2id_hashes_after_confirmation() {
    let root = workspace();
    let access = session(&root);
    assert_eq!(access.status(), CatalogAccessStatus::SetupRequired);
    let recovery = access.begin_setup("private-password").unwrap();
    assert!(!access.is_authorized());
    assert!(!root.join("catalog-access.json").exists());
    assert!(access.finish_setup(false).is_err());
    assert!(access.finish_setup(true).is_ok());
    assert!(access.is_authorized());

    let raw = fs::read_to_string(root.join("catalog-access.json")).unwrap();
    assert!(!raw.contains("private-password"));
    assert!(!raw.contains(&recovery));
    let config = CatalogAccessStore::new(&root).load().unwrap().unwrap();
    assert_ne!(config.password_hash, config.recovery_hash);
    assert!(config.password_hash.starts_with("$argon2id$v=19$"));
    assert!(config.recovery_hash.starts_with("$argon2id$v=19$"));

    drop(access);
    let restarted = session(&root);
    assert_eq!(restarted.status(), CatalogAccessStatus::Locked);
    assert!(restarted.unlock("private-password").is_ok());
    assert_eq!(restarted.status(), CatalogAccessStatus::Unlocked);
    restarted.lock().unwrap();
    assert_eq!(restarted.status(), CatalogAccessStatus::Locked);
    assert!(!restarted.is_authorized());
    assert!(restarted.unlock("private-password").is_ok());
    assert!(restarted.unlock("wrong-password").is_err());
    drop(restarted);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn password_change_requires_current_password_and_recovery_replaces_and_rotates_both_secrets() {
    let root = workspace();
    let access = session(&root);
    let first_recovery = access.begin_setup("first-password").unwrap();
    access.finish_setup(true).unwrap();

    assert!(access.change_password("wrong-password", "second-password").is_err());
    access.change_password("first-password", "second-password").unwrap();
    assert!(access.unlock("first-password").is_err());
    access.unlock("second-password").unwrap();

    let rotated = access.begin_recovery(&first_recovery, "recovered-password").unwrap();
    assert_ne!(first_recovery, rotated);
    assert!(!access.is_authorized());
    access.finish_recovery(true).unwrap();
    assert!(access.unlock("second-password").is_err());
    access.unlock("recovered-password").unwrap();
    assert!(access.begin_recovery(&first_recovery, "other-password").is_err());
    let raw = fs::read_to_string(root.join("catalog-access.json")).unwrap();
    assert!(!raw.contains("first-password"));
    assert!(!raw.contains("second-password"));
    assert!(!raw.contains("recovered-password"));
    assert!(!raw.contains(&first_recovery));
    assert!(!raw.contains(&rotated));
    drop(access);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn installing_a_restored_sqlite_snapshot_does_not_replace_device_local_access_configuration() {
    let root = workspace();
    let access = session(&root);
    let recovery = access.begin_setup("device-password").unwrap();
    access.finish_setup(true).unwrap();
    let access_path = root.join("catalog-access.json");
    let original = fs::read(&access_path).unwrap();

    let database_config = production_database_config(&root);
    let database = DatabaseState::open(database_config).unwrap();
    let stage = root.join(format!("backup-restore/staging/{}.sqlite3", uuid::Uuid::new_v4()));
    fs::create_dir_all(stage.parent().unwrap()).unwrap();
    database.with_read(|connection| create_snapshot(connection, &stage).map_err(|_| "snapshot_failed".to_string())).unwrap();
    database.install_validated_stage(&stage, &BackupStore::new(&root)).unwrap();

    assert_eq!(fs::read(access_path).unwrap(), original);
    assert!(!original.windows(recovery.len()).any(|window| window == recovery.as_bytes()));
    drop(database);
    drop(access);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn replacing_local_access_config_is_readable_and_never_leaves_a_plaintext_or_partial_file() {
    let root = workspace();
    let access = session(&root);
    access.begin_setup("first-password").unwrap();
    access.finish_setup(true).unwrap();
    let before = fs::read(root.join("catalog-access.json")).unwrap();
    access.change_password("first-password", "replacement-password").unwrap();
    let after = fs::read(root.join("catalog-access.json")).unwrap();
    assert_ne!(before, after);
    assert!(!String::from_utf8_lossy(&after).contains("replacement-password"));
    assert!(CatalogAccessStore::new(&root).load().unwrap().is_some());
    assert_eq!(fs::read_dir(&root).unwrap().filter_map(Result::ok).filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp")).count(), 0);
    drop(access);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn absent_local_configuration_on_another_device_starts_setup_without_touching_the_original() {
    let first_device = workspace();
    let second_device = workspace();
    let first = session(&first_device);
    let recovery = first.begin_setup("device-one-password").unwrap();
    first.finish_setup(true).unwrap();
    let original = fs::read(first_device.join("catalog-access.json")).unwrap();

    let restored_device = session(&second_device);
    assert_eq!(restored_device.status(), CatalogAccessStatus::SetupRequired);
    let second_recovery = restored_device.begin_setup("device-two-password").unwrap();
    restored_device.finish_setup(true).unwrap();
    assert_ne!(recovery, second_recovery);
    assert_eq!(fs::read(first_device.join("catalog-access.json")).unwrap(), original);
    assert!(second_device.join("catalog-access.json").exists());

    drop(first);
    drop(restored_device);
    fs::remove_dir_all(first_device).unwrap();
    fs::remove_dir_all(second_device).unwrap();
}
