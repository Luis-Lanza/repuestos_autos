#[allow(dead_code)]
#[path = "../src/main.rs"]
mod cli;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct TestDirs(PathBuf);

impl TestDirs {
    fn new() -> Self {
        let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "license-cli-safety-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirs {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn key_and_output_paths_reject_repository_paths() {
    let dirs = TestDirs::new();
    let repo = dirs.path().join("controlled-repo");
    let external = dirs.path().join("external");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&external).unwrap();
    let in_repo = repo.join("placeholder");
    fs::write(&in_repo, b"not a key").unwrap();

    assert_eq!(
        cli::ensure_outside_repository(&in_repo, &repo).unwrap_err(),
        "selected key file must be outside the repository"
    );
    assert_eq!(
        cli::prompt_new_external_file(&repo.join("private.key"), &repo).unwrap_err(),
        "private-key file must be outside the repository"
    );
    assert_eq!(
        cli::ensure_lic_output_outside_repository(&repo.join("license.lic"), &repo).unwrap_err(),
        "output file must be outside the repository"
    );

    let external_key = external.join("placeholder");
    fs::write(&external_key, b"not a real key").unwrap();
    assert_eq!(
        cli::ensure_outside_repository(&external_key, &repo).unwrap(),
        external_key.canonicalize().unwrap()
    );
}

#[test]
fn output_validation_requires_lic_extension_and_nonexistent_destination() {
    let dirs = TestDirs::new();
    let repo = dirs.path().join("controlled-repo");
    let external = dirs.path().join("external");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&external).unwrap();

    assert_eq!(
        cli::ensure_lic_output_outside_repository(&external.join("license.txt"), &repo).unwrap_err(),
        "output file must use the .lic extension"
    );
    let existing = external.join("existing.lic");
    fs::write(&existing, b"keep me").unwrap();
    assert_eq!(
        cli::ensure_lic_output_outside_repository(&existing, &repo).unwrap_err(),
        "output file already exists"
    );
}

#[test]
fn new_private_key_destination_rejects_existing_file() {
    let dirs = TestDirs::new();
    let repo = dirs.path().join("controlled-repo");
    let external = dirs.path().join("external");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&external).unwrap();
    let existing = external.join("placeholder.key");
    fs::write(&existing, b"not a real key").unwrap();

    assert_eq!(
        cli::prompt_new_external_file(&existing, &repo).unwrap_err(),
        "private-key file already exists"
    );
}

#[test]
fn private_key_file_read_is_bounded_and_requires_exactly_64_lowercase_hex_bytes() {
    let dirs = TestDirs::new();
    let repo = dirs.path().join("controlled-repo");
    let external = dirs.path().join("external");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&external).unwrap();
    let oversized = external.join("placeholder");
    fs::write(&oversized, vec![b'a'; 65]).unwrap();

    assert_eq!(
        cli::read_private_seed(&oversized, &repo).unwrap_err(),
        "private-key file must contain exactly 64 lowercase hexadecimal bytes"
    );
    fs::write(&oversized, vec![b'a'; 64]).unwrap();
    assert!(cli::read_private_seed(&oversized, &repo).is_ok());
}

#[test]
fn create_new_file_never_overwrites_existing_output() {
    let dirs = TestDirs::new();
    let path = dirs.path().join("license.lic");
    cli::create_new_file(&path, b"first").unwrap();

    assert_eq!(cli::create_new_file(&path, b"replacement").unwrap_err(), "could not create license output file");
    assert_eq!(fs::read(path).unwrap(), b"first");
}
