use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
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
    UnsupportedDestination,
    StorageUnavailable,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PublishedBackup {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub sha256: String,
    pub durability_warning: bool,
}

pub struct BackupStore {
    root: PathBuf,
}

impl BackupStore {
    pub fn validate_destination(path: &Path) -> Result<(), StorageError> {
        validate_backup_destination(path)
    }

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
        let parent = destination.parent().ok_or(StorageError::StorageUnavailable)?;
        validate_backup_destination(parent)?;
        match fs::create_dir(destination) {
            Ok(()) => {
                // If this sync fails, publication never starts. The directory is retained;
                // callers receive the original bounded storage error and can retry safely.
                sync_directory(parent)?;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                validate_backup_destination(destination)?;
            }
            Err(_) => return Err(StorageError::StorageUnavailable),
        }
        validate_backup_destination(destination)?;
        publish_validated(snapshot, destination, file_name, &mut RealPublicationFs)

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
        match fs::symlink_metadata(&marker) {
            Ok(metadata) if metadata.file_type().is_file() => {}
            Ok(_) => return Err(StorageError::StorageUnavailable),
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(StorageError::StorageUnavailable),
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

trait PublicationFs {
    fn create_temp(&mut self, path: &Path) -> io::Result<File>;
    fn copy(&mut self, source: &Path, output: &mut File) -> io::Result<u64>;
    fn flush(&mut self, output: &mut File) -> io::Result<()>;
    fn sync_file(&mut self, output: &File) -> io::Result<()>;
    fn checksum(&mut self, path: &Path, source: bool) -> io::Result<String>;
    fn validate_sqlite(&mut self, path: &Path) -> Result<(), StorageError>;
    fn finalize(&mut self, from: &Path, to: &Path) -> Result<(), StorageError>;
    fn exists_no_follow(&mut self, path: &Path) -> bool;
    fn is_regular_file(&mut self, path: &Path) -> bool;
    fn remove_temp(&mut self, path: &Path) -> io::Result<()>;
    fn sync_directory(&mut self, path: &Path) -> Result<(), StorageError>;
    fn create_cleanup_record(&mut self, path: &Path) -> io::Result<()>;
}

struct RealPublicationFs;

impl PublicationFs for RealPublicationFs {
    fn create_temp(&mut self, path: &Path) -> io::Result<File> {
        OpenOptions::new().write(true).create_new(true).open(path)
    }
    fn copy(&mut self, source: &Path, output: &mut File) -> io::Result<u64> {
        io::copy(&mut File::open(source)?, output)
    }
    fn flush(&mut self, output: &mut File) -> io::Result<()> { output.flush() }
    fn sync_file(&mut self, output: &File) -> io::Result<()> { output.sync_all() }
    fn checksum(&mut self, path: &Path, _source: bool) -> io::Result<String> { sha256(path) }
    fn validate_sqlite(&mut self, path: &Path) -> Result<(), StorageError> { validate_sqlite_snapshot(path) }
    fn finalize(&mut self, from: &Path, to: &Path) -> Result<(), StorageError> {
        rename_no_replace(from, to)
    }
    fn exists_no_follow(&mut self, path: &Path) -> bool { fs::symlink_metadata(path).is_ok() }
    fn is_regular_file(&mut self, path: &Path) -> bool {
        matches!(fs::symlink_metadata(path), Ok(metadata) if metadata.file_type().is_file())
    }
    fn remove_temp(&mut self, path: &Path) -> io::Result<()> { fs::remove_file(path) }
    fn sync_directory(&mut self, path: &Path) -> Result<(), StorageError> { sync_directory(path) }
    fn create_cleanup_record(&mut self, path: &Path) -> io::Result<()> {
        let mut record = OpenOptions::new().write(true).create_new(true).open(path)?;
        writeln!(record, "temporary-file-cleanup-required")?;
        record.sync_all()
    }
}

fn publish_validated<F: PublicationFs>(
    snapshot: &Path,
    destination: &Path,
    file_name: &str,
    fs: &mut F,
) -> Result<PublishedBackup, StorageError> {
    publish_with_pre_finalize_validation(snapshot, destination, file_name, fs, validate_backup_destination)
}

#[cfg(test)]
fn publish_after_destination_validation_for_test<F: PublicationFs>(
    snapshot: &Path,
    destination: &Path,
    file_name: &str,
    fs: &mut F,
) -> Result<PublishedBackup, StorageError> {
    // Fault-injection tests enter here only after their fixture's destination is established;
    // this seam is unavailable to production and cannot stand in for NTFS/reparse validation.
    publish_with_pre_finalize_validation(snapshot, destination, file_name, fs, |_| Ok(()))
}

fn publish_with_pre_finalize_validation<F: PublicationFs>(
    snapshot: &Path,
    destination: &Path,
    file_name: &str,
    fs: &mut F,
    validate_before_finalize: impl FnOnce(&Path) -> Result<(), StorageError>,
) -> Result<PublishedBackup, StorageError> {
    let published = destination.join(file_name);
    let part = destination.join(format!(".{file_name}.{}.part", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut output = fs.create_temp(&part).map_err(map_create_temp_error)?;
        let copied_bytes = match fs.copy(snapshot, &mut output) {
            Ok(bytes) => bytes,
            Err(_) => { drop(output); return Err(StorageError::StorageUnavailable); }
        };
        fs.flush(&mut output).map_err(|_| StorageError::StorageUnavailable)?;
        fs.sync_file(&output).map_err(|_| StorageError::StorageUnavailable)?;
        drop(output);
        let checksum = fs.checksum(snapshot, true).map_err(|_| StorageError::StorageUnavailable)?;
        if checksum != fs.checksum(&part, false).map_err(|_| StorageError::StorageUnavailable)? {
            return Err(StorageError::StorageUnavailable);
        }
        fs.validate_sqlite(&part)?;
        validate_before_finalize(destination)?;
        let mut durability_warning = false;
        if let Err(error) = fs.finalize(&part, &published) {
            if fs.exists_no_follow(&part) || !valid_published_file(fs, &published, &checksum) {
                return Err(error);
            }
            // A consumed temp plus a valid final file resolves an ambiguous rename as Created.
            durability_warning = true;
        }
        // Finalization is confirmed; directory sync failure is a warning, not a failed creation.
        durability_warning |= fs.sync_directory(destination).is_err();
        Ok(PublishedBackup { path: published.clone(), size_bytes: copied_bytes, sha256: checksum, durability_warning })
    })();
    if result.is_err() && fs.exists_no_follow(&part) {
        if fs.remove_temp(&part).is_err() {
            let evidence = destination.join(format!(".{file_name}.cleanup-needed"));
            if fs.create_cleanup_record(&evidence).is_ok() {
                let _ = fs.sync_directory(destination);
            }
        } else {
            let _ = fs.sync_directory(destination);
        }
    }
    result
}

fn map_create_temp_error(error: io::Error) -> StorageError {
    if error.kind() == io::ErrorKind::AlreadyExists { StorageError::DestinationExists }
    else { StorageError::StorageUnavailable }
}

fn valid_published_file<F: PublicationFs>(fs: &mut F, path: &Path, expected_sha256: &str) -> bool {
    fs.is_regular_file(path)
        && matches!(fs.checksum(path, false), Ok(actual) if actual == expected_sha256)
        && fs.validate_sqlite(path).is_ok()
}

pub fn validate_backup_destination(path: &Path) -> Result<(), StorageError> {
    platform::validate_destination(path).map_err(|_| StorageError::UnsupportedDestination)
}

fn validate_sqlite_snapshot(path: &Path) -> Result<(), StorageError> {
    let connection = rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| StorageError::StorageUnavailable)?;
    crate::infrastructure::sqlite::validate_restored_database(&connection)
        .map_err(|_| StorageError::StorageUnavailable)
}

fn sync_directory(path: &Path) -> Result<(), StorageError> {
    File::open(path).and_then(|directory| directory.sync_all())
        .map_err(|_| StorageError::StorageUnavailable)
}

#[cfg(windows)]
fn rename_no_replace(from: &Path, to: &Path) -> Result<(), StorageError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::MoveFileW;
    let from: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let to: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    if unsafe { MoveFileW(from.as_ptr(), to.as_ptr()) } == 0 {
        let error = io::Error::last_os_error();
        return Err(if error.kind() == io::ErrorKind::AlreadyExists { StorageError::DestinationExists } else { StorageError::StorageUnavailable });
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn rename_no_replace(from: &Path, to: &Path) -> Result<(), StorageError> {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    const AT_FDCWD: i32 = -100;
    const RENAME_NOREPLACE: u32 = 1;
    unsafe extern "C" {
        fn renameat2(old_dirfd: i32, old_path: *const i8, new_dirfd: i32, new_path: *const i8, flags: u32) -> i32;
    }
    let from = CString::new(from.as_os_str().as_bytes()).map_err(|_| StorageError::StorageUnavailable)?;
    let to = CString::new(to.as_os_str().as_bytes()).map_err(|_| StorageError::StorageUnavailable)?;
    if unsafe { renameat2(AT_FDCWD, from.as_ptr(), AT_FDCWD, to.as_ptr(), RENAME_NOREPLACE) } == 0 {
        Ok(())
    } else {
        let error = io::Error::last_os_error();
        Err(if error.kind() == io::ErrorKind::AlreadyExists { StorageError::DestinationExists } else { StorageError::StorageUnavailable })
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
fn rename_no_replace(_from: &Path, _to: &Path) -> Result<(), StorageError> {
    Err(StorageError::UnsupportedDestination)
}

#[cfg(windows)]
mod platform {
    use std::{ffi::OsStr, io, os::windows::{ffi::{OsStrExt, OsStringExt}, fs::MetadataExt}, path::Path};
    use windows_sys::Win32::Storage::FileSystem::{GetDriveTypeW, GetVolumeInformationW, GetVolumePathNameW, FILE_ATTRIBUTE_REPARSE_POINT};
    use windows_sys::Win32::System::WindowsProgramming::DRIVE_FIXED;
    use super::*;

    pub(super) fn validate_destination(path: &Path) -> io::Result<()> {
        if !path.is_absolute() { return Err(io::ErrorKind::InvalidInput.into()); }
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut volume = vec![0u16; 32768];
        if unsafe { GetVolumePathNameW(wide.as_ptr(), volume.as_mut_ptr(), volume.len() as u32) } == 0 { return Err(io::Error::last_os_error()); }
        if unsafe { GetDriveTypeW(volume.as_ptr()) } != DRIVE_FIXED { return Err(io::ErrorKind::Unsupported.into()); }
        let mut filesystem = vec![0u16; 32];
        if unsafe { GetVolumeInformationW(volume.as_ptr(), std::ptr::null_mut(), 0, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), filesystem.as_mut_ptr(), filesystem.len() as u32) } == 0 { return Err(io::Error::last_os_error()); }
        let end = filesystem.iter().position(|unit| *unit == 0).ok_or(io::ErrorKind::InvalidData)?;
        if OsStr::new("NTFS") != std::ffi::OsString::from_wide(&filesystem[..end]) { return Err(io::ErrorKind::Unsupported.into()); }
        let mut final_metadata = None;
        for component in path.ancestors().collect::<Vec<_>>().into_iter().rev() {
            let metadata = fs::symlink_metadata(component)?;
            if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return Err(io::ErrorKind::Unsupported.into());
            }
            final_metadata = Some(metadata);
        }
        if !final_metadata.is_some_and(|metadata| metadata.is_dir()) {
            return Err(io::ErrorKind::NotADirectory.into());
        }
        Ok(())
    }
}

#[cfg(not(windows))]
mod platform {
    use std::{io, path::Path};
    pub(super) fn validate_destination(_path: &Path) -> io::Result<()> { Err(io::ErrorKind::Unsupported.into()) }
}

#[cfg(test)]
mod publication_tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Fault {
        CreateTemp, CopyPartial, Flush, SyncFile, SourceChecksum, TempChecksum,
        SqliteValidation, Collision, AmbiguousRename, AmbiguousRenameTempRetained, AmbiguousRenameInvalidFinal,
        DirectorySyncAfterFinalize, CleanupFailure, CleanupRecordFailure,
    }

    struct FaultFs { fault: Fault, directory_syncs: usize }

    impl PublicationFs for FaultFs {
        fn create_temp(&mut self, path: &Path) -> io::Result<File> {
            if self.fault == Fault::CreateTemp { return Err(io::ErrorKind::PermissionDenied.into()); }
            RealPublicationFs.create_temp(path)
        }
        fn copy(&mut self, source: &Path, output: &mut File) -> io::Result<u64> {
            if self.fault == Fault::CopyPartial {
                let mut input = File::open(source)?;
                let mut partial = [0; 16];
                let count = input.read(&mut partial)?;
                output.write_all(&partial[..count])?;
                return Err(io::ErrorKind::Other.into());
            }
            RealPublicationFs.copy(source, output)
        }
        fn flush(&mut self, output: &mut File) -> io::Result<()> {
            if self.fault == Fault::Flush { return Err(io::ErrorKind::Other.into()); }
            RealPublicationFs.flush(output)
        }
        fn sync_file(&mut self, output: &File) -> io::Result<()> {
            if self.fault == Fault::SyncFile { return Err(io::ErrorKind::Other.into()); }
            RealPublicationFs.sync_file(output)
        }
        fn checksum(&mut self, path: &Path, source: bool) -> io::Result<String> {
            if (source && self.fault == Fault::SourceChecksum)
                || (!source && self.fault == Fault::TempChecksum) {
                return Err(io::ErrorKind::Other.into());
            }
            RealPublicationFs.checksum(path, source)
        }
        fn validate_sqlite(&mut self, path: &Path) -> Result<(), StorageError> {
            if self.fault == Fault::SqliteValidation { return Err(StorageError::StorageUnavailable); }
            RealPublicationFs.validate_sqlite(path)
        }
        fn finalize(&mut self, from: &Path, to: &Path) -> Result<(), StorageError> {
            if matches!(self.fault, Fault::Collision | Fault::CleanupFailure | Fault::CleanupRecordFailure) {
                return Err(StorageError::DestinationExists);
            }
            if self.fault == Fault::AmbiguousRename {
                RealPublicationFs.finalize(from, to)?;
                return Err(StorageError::StorageUnavailable);
            }
            if self.fault == Fault::AmbiguousRenameTempRetained {
                fs::copy(from, to).map_err(|_| StorageError::StorageUnavailable)?;
                return Err(StorageError::StorageUnavailable);
            }
            if self.fault == Fault::AmbiguousRenameInvalidFinal {
                fs::remove_file(from).map_err(|_| StorageError::StorageUnavailable)?;
                fs::write(to, b"not the validated snapshot").map_err(|_| StorageError::StorageUnavailable)?;
                return Err(StorageError::StorageUnavailable);
            }
            RealPublicationFs.finalize(from, to)
        }
        fn exists_no_follow(&mut self, path: &Path) -> bool { RealPublicationFs.exists_no_follow(path) }
        fn is_regular_file(&mut self, path: &Path) -> bool { RealPublicationFs.is_regular_file(path) }
        fn remove_temp(&mut self, path: &Path) -> io::Result<()> {
            if matches!(self.fault, Fault::CleanupFailure | Fault::CleanupRecordFailure) {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
            RealPublicationFs.remove_temp(path)
        }
        fn sync_directory(&mut self, path: &Path) -> Result<(), StorageError> {
            self.directory_syncs += 1;
            if self.fault == Fault::DirectorySyncAfterFinalize && self.directory_syncs == 1 {
                return Err(StorageError::StorageUnavailable);
            }
            RealPublicationFs.sync_directory(path)
        }
        fn create_cleanup_record(&mut self, path: &Path) -> io::Result<()> {
            if self.fault == Fault::CleanupRecordFailure { return Err(io::ErrorKind::PermissionDenied.into()); }
            RealPublicationFs.create_cleanup_record(path)
        }
    }

    struct Fixture { root: PathBuf, source: PathBuf, destination: PathBuf }

    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!("backup-publication-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&root).unwrap();
            let source = root.join("source.sqlite3");
            let connection = crate::infrastructure::sqlite::open_database(
                &crate::infrastructure::sqlite::database_config(&source),
            ).unwrap();
            crate::infrastructure::sqlite::validate_restored_database(&connection).unwrap();
            drop(connection);
            let destination = root.join("destination");
            fs::create_dir(&destination).unwrap();
            Self { root, source, destination }
        }
        fn part(&self) -> Option<PathBuf> {
            fs::read_dir(&self.destination).unwrap().filter_map(Result::ok)
                .map(|entry| entry.path())
                .find(|path| path.file_name().unwrap().to_string_lossy().ends_with(".part"))
        }
        fn final_path(&self) -> PathBuf { self.destination.join("backup.sqlite3") }
        fn cleanup_record(&self) -> PathBuf { self.destination.join(".backup.sqlite3.cleanup-needed") }
    }

    impl Drop for Fixture {
        fn drop(&mut self) { let _ = fs::remove_dir_all(&self.root); }
    }

    fn run_fault(fault: Fault) -> (Fixture, Result<PublishedBackup, StorageError>) {
        let fixture = Fixture::new();
        if matches!(fault, Fault::CleanupFailure | Fault::CleanupRecordFailure) {
            fs::write(fixture.final_path(), b"occupied").unwrap();
        }
        let result = publish_after_destination_validation_for_test(
            &fixture.source, &fixture.destination, "backup.sqlite3", &mut FaultFs { fault, directory_syncs: 0 },
        );
        (fixture, result)
    }

    #[test]
    fn prefinalization_failures_return_bounded_error_and_clean_temp_best_effort() {
        for fault in [Fault::CreateTemp, Fault::CopyPartial, Fault::Flush, Fault::SyncFile,
            Fault::SourceChecksum, Fault::TempChecksum, Fault::SqliteValidation] {
            let (fixture, result) = run_fault(fault);
            assert_eq!(result, Err(StorageError::StorageUnavailable), "fault {fault:?}");
            assert!(!fixture.final_path().exists(), "fault {fault:?}");
            assert!(fixture.part().is_none(), "fault {fault:?}");
            assert!(!fixture.cleanup_record().exists(), "fault {fault:?}");
        }
    }

    #[test]
    fn collision_is_not_created_and_temp_is_removed() {
        let fixture = Fixture::new();
        fs::write(fixture.final_path(), b"occupied").unwrap();
        let result = publish_after_destination_validation_for_test(&fixture.source, &fixture.destination, "backup.sqlite3", &mut FaultFs {
            fault: Fault::Collision, directory_syncs: 0,
        });
        assert_eq!(result, Err(StorageError::DestinationExists));
        assert_eq!(fs::read(fixture.final_path()).unwrap(), b"occupied");
        assert!(fixture.part().is_none());
        assert!(!fixture.cleanup_record().exists());
    }

    #[test]
    fn ambiguous_rename_is_created_only_after_valid_final_verification() {
        let (fixture, result) = run_fault(Fault::AmbiguousRename);
        let published = result.unwrap();
        assert_eq!(published.path, fixture.final_path());
        assert!(published.durability_warning);
        assert!(fixture.final_path().is_file());
        assert!(fixture.part().is_none());
        assert!(!fixture.cleanup_record().exists());
    }

    #[test]
    fn valid_final_is_not_created_when_the_temporary_still_exists() {
        let (fixture, result) = run_fault(Fault::AmbiguousRenameTempRetained);
        assert_eq!(result, Err(StorageError::StorageUnavailable));
        assert!(fixture.final_path().is_file());
        assert!(fixture.part().is_none());
        assert!(!fixture.cleanup_record().exists());
    }

    #[test]
    fn ambiguous_error_with_mismatching_final_is_not_created() {
        let (fixture, result) = run_fault(Fault::AmbiguousRenameInvalidFinal);
        assert_eq!(result, Err(StorageError::StorageUnavailable));
        assert_eq!(fs::read(fixture.final_path()).unwrap(), b"not the validated snapshot");
        assert!(fixture.part().is_none());
        assert!(!fixture.cleanup_record().exists());
    }

    #[test]
    fn directory_sync_failure_after_finalization_is_created_with_warning() {
        let (fixture, result) = run_fault(Fault::DirectorySyncAfterFinalize);
        assert!(result.unwrap().durability_warning);
        assert!(fixture.final_path().is_file());
        assert!(fixture.part().is_none());
        assert!(!fixture.cleanup_record().exists());
    }

    #[test]
    fn failed_temp_removal_records_cleanup_evidence() {
        let (fixture, result) = run_fault(Fault::CleanupFailure);
        assert_eq!(result, Err(StorageError::DestinationExists));
        assert_eq!(fs::read(fixture.final_path()).unwrap(), b"occupied");
        assert!(fixture.part().is_some());
        assert!(fixture.cleanup_record().is_file());
    }

    #[test]
    fn failed_cleanup_record_creation_preserves_temp_without_claiming_created() {
        let (fixture, result) = run_fault(Fault::CleanupRecordFailure);
        assert_eq!(result, Err(StorageError::DestinationExists));
        assert_eq!(fs::read(fixture.final_path()).unwrap(), b"occupied");
        assert!(fixture.part().is_some());
        assert!(!fixture.cleanup_record().exists());
    }

    #[test]
    fn destination_validation_policy_remains_a_real_external_boundary() {
        #[cfg(not(windows))]
        assert_eq!(validate_backup_destination(&std::env::temp_dir()), Err(StorageError::UnsupportedDestination));
        #[cfg(windows)]
        assert_eq!(validate_backup_destination(Path::new("relative\\backup")), Err(StorageError::UnsupportedDestination));
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
