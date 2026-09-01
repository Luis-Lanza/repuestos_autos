use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::restore_transitions;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreState {
    Prepared,
    LiveMoved,
    CandidateInstalled,
}

impl RestoreState {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "prepared" => Some(Self::Prepared),
            "live_moved" => Some(Self::LiveMoved),
            "candidate_installed" => Some(Self::CandidateInstalled),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum StorageError {
    SelectionCancelled,
    DestinationExists,
    StorageUnavailable,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PublishedBackup {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub sha256: String,
}

pub struct BackupStore {
    root: PathBuf,
}

impl BackupStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn publish_snapshot(
        &self,
        snapshot: &Path,
        destination: &Path,
        file_name: &str,
    ) -> Result<PublishedBackup, StorageError> {
        if !file_name.ends_with(".sqlite3") || Path::new(file_name).components().count() != 1 {
            return Err(StorageError::StorageUnavailable);
        }
        let Some(selected_root) = destination.parent() else {
            return Err(StorageError::StorageUnavailable);
        };
        if !selected_root.is_dir() {
            return Err(StorageError::StorageUnavailable);
        }
        // Create only the application subdirectory: never recreate a selected root.
        match fs::create_dir(destination) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists && destination.is_dir() => {}
            Err(_) => return Err(StorageError::StorageUnavailable),
        }
        let published = destination.join(file_name);
        if published.exists() {
            return Err(StorageError::DestinationExists);
        }
        let part = destination.join(format!("{file_name}.part"));
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&part)
            .map_err(|_| StorageError::StorageUnavailable)?;
        io::copy(
            &mut File::open(snapshot).map_err(|_| StorageError::StorageUnavailable)?,
            &mut output,
        )
        .map_err(|_| StorageError::StorageUnavailable)?;
        output
            .sync_all()
            .map_err(|_| StorageError::StorageUnavailable)?;
        drop(output);
        let checksum = sha256(snapshot).map_err(|_| StorageError::StorageUnavailable)?;
        if checksum != sha256(&part).map_err(|_| StorageError::StorageUnavailable)? {
            let _ = fs::remove_file(&part);
            return Err(StorageError::StorageUnavailable);
        }
        fs::hard_link(&part, &published).map_err(|error| {
            if error.kind() == io::ErrorKind::AlreadyExists {
                StorageError::DestinationExists
            } else {
                StorageError::StorageUnavailable
            }
        })?;
        fs::remove_file(part).map_err(|_| StorageError::StorageUnavailable)?;
        Ok(PublishedBackup {
            size_bytes: fs::metadata(&published)
                .map_err(|_| StorageError::StorageUnavailable)?
                .len(),
            path: published,
            sha256: checksum,
        })
    }

    pub fn publish_selected_snapshot(
        &self,
        snapshot: Option<&Path>,
        destination: &Path,
        file_name: &str,
    ) -> Result<PublishedBackup, StorageError> {
        self.publish_snapshot(
            snapshot.ok_or(StorageError::SelectionCancelled)?,
            destination,
            file_name,
        )
    }

    pub fn prepare_durable_restore(
        &self,
        stage: &Path,
        protective: &Path,
    ) -> Result<(), StorageError> {
        restore_transitions::prepare(&self.root, stage, protective)
    }

    pub fn install_durable_restore(
        &self,
        stage: &Path,
        canonical: &Path,
    ) -> Result<(), StorageError> {
        restore_transitions::install(&self.root, stage, canonical)
    }

    pub fn recover_canonical_durably(
        &self,
        source: &Path,
        canonical: &Path,
    ) -> Result<(), StorageError> {
        restore_transitions::recover(&self.root, source, canonical)
    }

    pub fn complete_durable_restore(&self) -> Result<(), StorageError> {
        restore_transitions::complete(&self.root)
    }

    pub fn read_restore_state(&self) -> Result<Option<RestoreState>, StorageError> {
        let marker = self.root.join("restore-state.json");
        if !marker.exists() {
            return Ok(None);
        }
        let contents = fs::read_to_string(marker).map_err(|_| StorageError::StorageUnavailable)?;
        let state = contents
            .strip_prefix(r#"{"state":""#)
            .and_then(|value| value.strip_suffix(r#""}"#))
            .and_then(RestoreState::parse)
            .ok_or(StorageError::StorageUnavailable)?;
        Ok(Some(state))
    }
}

fn sha256(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
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
