use std::io;
use std::path::{Path, PathBuf};

#[cfg(all(test, not(windows)))]
use std::fs::{self, File, OpenOptions};
#[cfg(all(test, not(windows)))]
use std::io::Write;

use super::StorageError;

const PREPARED: &[u8] = br#"{"state":"prepared"}"#;
const LIVE_MOVED: &[u8] = br#"{"state":"live_moved"}"#;
const CANDIDATE_INSTALLED: &[u8] = br#"{"state":"candidate_installed"}"#;
const MARKER_BACKUP_LIMIT: usize = 8;

#[rustfmt::skip]
#[derive(Clone, Copy)]
enum VerificationPhase<'a> {
    Prepare { stage: &'a Path, protective: &'a Path, canonical: &'a Path },
    Recovery { marker: &'a Path, source: &'a Path, canonical: &'a Path },
    Completion { marker: &'a Path, canonical: &'a Path },
}

#[derive(Clone, Copy)]
enum FsOperation<'a> {
    Verify(&'a Path, VerificationPhase<'a>),
    SyncFile(&'a Path),
    SyncDirectory(&'a Path),
    WriteExclusive(&'a Path, &'a [u8]),
    CopyExclusive(&'a Path, &'a Path),
    Remove(&'a Path),
    RenameNoReplace(&'a Path, &'a Path),
    Replace(&'a Path, &'a Path, &'a Path),
}

trait DurableFs {
    fn execute(&self, operation: FsOperation<'_>) -> io::Result<()>;
    fn is_present(&self, path: &Path) -> io::Result<bool>;
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
}

#[cfg(test)]
#[rustfmt::skip]
fn verify_test_phase(root: &Path, phase: VerificationPhase<'_>, inspect: impl Fn(&Path) -> io::Result<Option<bool>>, read: impl Fn(&Path) -> io::Result<Vec<u8>>, entries: impl Fn(&Path) -> io::Result<Vec<PathBuf>>) -> io::Result<()> {
    let canonical = root.join("repuestos-autos.sqlite3"); let marker = root.join("restore-state.json");
    let required = |path: &Path| -> io::Result<bool> { Ok(inspect(path)? == Some(true)) };
    let optional = |path: &Path| -> io::Result<bool> { Ok(inspect(path)?.unwrap_or(true)) };
    let active = |path: &Path| -> io::Result<bool> { Ok(required(path)? && matches!(read(path)?.as_slice(), PREPARED | LIVE_MOVED | CANDIDATE_INSTALLED)) };
    let sidecar = |path: &Path| -> io::Result<bool> { let bytes = read(path)?; Ok(required(path)? && (bytes.is_empty() || matches!(bytes.as_slice(), PREPARED | LIVE_MOVED | CANDIDATE_INSTALLED))) };
    for path in entries(root)? {
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
        if name.starts_with("restore-state.json.previous-") {
            let known = (0..MARKER_BACKUP_LIMIT).any(|slot| path == root.join(format!("restore-state.json.previous-{slot}")));
            if !known || !sidecar(&path)? { return Err(io::ErrorKind::InvalidData.into()); }
        }
    }
    for path in [root.join("restore-rollback.sqlite3"), root.join("pre-restore.sqlite3"), root.join("restore-state.json.part"), root.join("restore-recovery.sqlite3.part")] {
        if !optional(&path)? { return Err(io::ErrorKind::Unsupported.into()); }
    }
    let supported = match phase {
        VerificationPhase::Prepare { stage, protective, canonical: actual } => stage.starts_with(root.join("backup-restore/staging")) && protective == root.join("pre-restore.sqlite3") && actual == canonical && inspect(&marker)?.is_none() && required(stage)? && required(protective)? && required(actual)?,
        VerificationPhase::Recovery { marker: actual, source, canonical: target } => actual == marker && target == canonical && [root.join("restore-rollback.sqlite3"), root.join("pre-restore.sqlite3")].contains(&source.to_path_buf()) && required(source)? && optional(target)? && active(actual)?,
        VerificationPhase::Completion { marker: actual, canonical: target } => actual == marker && target == canonical && required(target)? && (inspect(&marker)?.is_none() || active(&marker)?),
    };
    if supported { Ok(()) } else { Err(io::ErrorKind::Unsupported.into()) }
}

struct RestoreTransitions<F> {
    root: PathBuf,
    fs: F,
}

impl<F: DurableFs> RestoreTransitions<F> {
    fn new(root: PathBuf, fs: F) -> Self {
        Self { root, fs }
    }

    fn prepare_durable_restore(
        &self,
        stage: &Path,
        protective: &Path,
        canonical: &Path,
    ) -> Result<(), StorageError> {
        self.run(FsOperation::Verify(
            &self.root,
            VerificationPhase::Prepare {
                stage,
                protective,
                canonical,
            },
        ))?;
        self.run(FsOperation::SyncDirectory(&self.root))?;
        for slot in 0..MARKER_BACKUP_LIMIT {
            let sidecar = self
                .root
                .join(format!("restore-state.json.previous-{slot}"));
            if self.present(&sidecar)? {
                self.run(FsOperation::Remove(&sidecar))?;
                self.run(FsOperation::SyncDirectory(&self.root))?;
            }
        }
        self.run(FsOperation::SyncFile(stage))?;
        self.sync_stage_ancestry(stage)?;
        self.run(FsOperation::SyncFile(protective))?;
        self.run(FsOperation::SyncDirectory(&self.root))?;
        self.publish_marker(PREPARED)
    }

    fn install_durable_restore(&self, stage: &Path, canonical: &Path) -> Result<(), StorageError> {
        let rollback = self.root.join("restore-rollback.sqlite3");
        if self.present(&rollback)? {
            self.run(FsOperation::Remove(&rollback))?;
            self.run(FsOperation::SyncDirectory(&self.root))?;
        }
        self.run(FsOperation::RenameNoReplace(canonical, &rollback))?;
        self.run(FsOperation::SyncDirectory(&self.root))?;
        self.publish_marker(LIVE_MOVED)?;
        self.run(FsOperation::RenameNoReplace(stage, canonical))?;
        self.run(FsOperation::SyncDirectory(&self.root))?;
        self.sync_stage_ancestry(stage)?;
        self.publish_marker(CANDIDATE_INSTALLED)
    }

    fn recover_canonical_durably(
        &self,
        source: &Path,
        canonical: &Path,
    ) -> Result<(), StorageError> {
        let marker = self.marker();
        self.run(FsOperation::Verify(
            &self.root,
            VerificationPhase::Recovery {
                marker: &marker,
                source,
                canonical,
            },
        ))?;
        let temporary = self.root.join("restore-recovery.sqlite3.part");
        if self.present(&temporary)? {
            self.run(FsOperation::Remove(&temporary))?;
            self.run(FsOperation::SyncDirectory(&self.root))?;
        }
        self.run(FsOperation::CopyExclusive(source, &temporary))?;
        self.run(FsOperation::SyncFile(&temporary))?;
        self.run(FsOperation::SyncDirectory(&self.root))?;
        if self.present(canonical)? {
            self.run(FsOperation::Remove(canonical))?;
            self.run(FsOperation::SyncDirectory(&self.root))?;
        }
        self.run(FsOperation::RenameNoReplace(&temporary, canonical))?;
        self.run(FsOperation::SyncDirectory(&self.root))
    }

    fn complete_durable_restore(
        &self,
        publish_ready: impl FnOnce() -> Result<(), StorageError>,
    ) -> Result<(), StorageError> {
        publish_ready()?;
        let marker = self.marker();
        let canonical = self.root.join("repuestos-autos.sqlite3");
        self.run(FsOperation::Verify(
            &self.root,
            VerificationPhase::Completion {
                marker: &marker,
                canonical: &canonical,
            },
        ))?;
        if !self.present(&marker)? {
            return Ok(());
        }
        let exact_marker = self
            .fs
            .read(&marker)
            .map_err(|_| StorageError::StorageUnavailable)?;
        self.run(FsOperation::Remove(&marker))?;
        if self.run(FsOperation::SyncDirectory(&self.root)).is_err() {
            let _ = self.publish_marker(&exact_marker);
            return Err(StorageError::StorageUnavailable);
        }
        Ok(())
    }

    fn sync_stage_ancestry(&self, stage: &Path) -> Result<(), StorageError> {
        let mut directory = stage.parent().ok_or(StorageError::StorageUnavailable)?;
        loop {
            self.run(FsOperation::SyncDirectory(directory))?;
            if directory == self.root {
                return Ok(());
            }
            directory = directory.parent().ok_or(StorageError::StorageUnavailable)?;
        }
    }

    fn publish_marker(&self, bytes: &[u8]) -> Result<(), StorageError> {
        let marker = self.marker();
        let temporary = self.root.join("restore-state.json.part");
        if self.present(&temporary)? {
            self.run(FsOperation::Remove(&temporary))?;
            self.run(FsOperation::SyncDirectory(&self.root))?;
        }
        self.run(FsOperation::WriteExclusive(&temporary, bytes))?;
        self.run(FsOperation::SyncFile(&temporary))?;
        if self.present(&marker)? {
            for slot in 0..MARKER_BACKUP_LIMIT {
                let backup = self
                    .root
                    .join(format!("restore-state.json.previous-{slot}"));
                if self.present(&backup)? {
                    continue;
                }
                match self
                    .fs
                    .execute(FsOperation::Replace(&temporary, &marker, &backup))
                {
                    Ok(()) => return self.run(FsOperation::SyncDirectory(&self.root)),
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(_) => {
                        let _ = self.fs.execute(FsOperation::SyncDirectory(&self.root));
                        return Err(StorageError::StorageUnavailable);
                    }
                }
            }
            Err(StorageError::StorageUnavailable)
        } else {
            self.run(FsOperation::RenameNoReplace(&temporary, &marker))?;
            self.run(FsOperation::SyncDirectory(&self.root))
        }
    }

    fn run(&self, operation: FsOperation<'_>) -> Result<(), StorageError> {
        self.fs
            .execute(operation)
            .map_err(|_| StorageError::StorageUnavailable)
    }

    fn present(&self, path: &Path) -> Result<bool, StorageError> {
        self.fs
            .is_present(path)
            .map_err(|_| StorageError::StorageUnavailable)
    }

    fn marker(&self) -> PathBuf {
        self.root.join("restore-state.json")
    }
}
