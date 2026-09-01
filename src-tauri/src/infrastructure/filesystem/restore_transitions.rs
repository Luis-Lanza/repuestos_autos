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

#[cfg(windows)]
mod platform {
    use std::ffi::{c_void, OsStr};
    use std::io;
    use std::mem::{offset_of, size_of};
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::path::{Path, PathBuf};
    use std::ptr::{null, null_mut};

    use windows_sys::Win32::Foundation::{
        CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, DeleteFileW, FlushFileBuffers, GetDriveTypeW, GetFileInformationByHandleEx,
        GetVolumeInformationW, GetVolumePathNameW, ReadFile, ReplaceFileW,
        SetFileInformationByHandle, WriteFile, CREATE_NEW, DELETE, FILE_ATTRIBUTE_DIRECTORY,
        FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_REPARSE_POINT, FILE_ATTRIBUTE_TAG_INFO,
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_DELETE,
        FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    };

    use super::{
        DurableFs, FsOperation, VerificationPhase, CANDIDATE_INSTALLED, LIVE_MOVED,
        MARKER_BACKUP_LIMIT, PREPARED,
    };

    const SHARES: u32 = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
    const DRIVE_FIXED: u32 = 3;
    const FILE_RENAME_INFO_EX_CLASS: i32 = 22;
    const BUFFER_SIZE: usize = 64 * 1024;

    pub(super) struct WindowsDurableFs;

    impl DurableFs for WindowsDurableFs {
        fn execute(&self, operation: FsOperation<'_>) -> io::Result<()> {
            match operation {
                FsOperation::Verify(root, phase) => verify_layout(root, phase),
                FsOperation::SyncFile(path) => sync_file(path),
                FsOperation::SyncDirectory(path) => sync_directory(path),
                FsOperation::WriteExclusive(path, bytes) => write_exclusive(path, bytes),
                FsOperation::CopyExclusive(from, to) => copy_exclusive(from, to),
                FsOperation::Remove(path) => delete_file(path),
                FsOperation::RenameNoReplace(from, to) => rename_no_replace(from, to),
                FsOperation::Replace(from, to, backup) => replace_file(from, to, backup),
            }
        }

        fn is_present(&self, path: &Path) -> io::Result<bool> {
            inspect_presence(path)
        }

        fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
            read_regular(path)
        }
    }

    fn wide(path: &Path) -> io::Result<Vec<u16>> {
        if !path.is_absolute() {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        let mut value: Vec<_> = path.as_os_str().encode_wide().collect();
        if value.contains(&0) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        value.push(0);
        Ok(value)
    }

    fn with_handle<T>(
        handle: HANDLE,
        action: impl FnOnce(HANDLE) -> io::Result<T>,
    ) -> io::Result<T> {
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let result = action(handle);
        // Handles are closed before any following namespace transition.
        let closed = unsafe { CloseHandle(handle) };
        if result.is_ok() && closed == 0 {
            return Err(io::Error::last_os_error());
        }
        result
    }

    #[rustfmt::skip]
    fn open(path: &Path, access: u32, creation: u32, flags: u32) -> io::Result<HANDLE> {
        let path = wide(path)?;
        let handle = unsafe { CreateFileW(path.as_ptr(), access, SHARES, null(), creation, flags, null_mut()) };
        if handle == INVALID_HANDLE_VALUE { Err(io::Error::last_os_error()) } else { Ok(handle) }
    }

    #[rustfmt::skip]
    fn sync_file(path: &Path) -> io::Result<()> {
        let handle = open(path, GENERIC_READ | GENERIC_WRITE, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL)?;
        with_handle(handle, |handle| bool_result(unsafe { FlushFileBuffers(handle) }))
    }

    fn sync_directory(path: &Path) -> io::Result<()> {
        let handle = open(
            path,
            GENERIC_WRITE,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
        )?;
        with_handle(handle, |handle| {
            bool_result(unsafe { FlushFileBuffers(handle) })
        })
    }

    #[rustfmt::skip]
    fn write_exclusive(path: &Path, bytes: &[u8]) -> io::Result<()> {
        let handle = open(path, GENERIC_READ | GENERIC_WRITE, CREATE_NEW, FILE_ATTRIBUTE_NORMAL)?;
        with_handle(handle, |handle| write_all(handle, bytes))
    }

    fn write_all(handle: HANDLE, mut bytes: &[u8]) -> io::Result<()> {
        while !bytes.is_empty() {
            let amount = u32::try_from(bytes.len().min(u32::MAX as usize))
                .map_err(|_| io::ErrorKind::InvalidInput)?;
            let mut written = 0;
            bool_result(unsafe {
                WriteFile(
                    handle,
                    bytes.as_ptr().cast(),
                    amount,
                    &mut written,
                    null_mut(),
                )
            })?;
            if written == 0 {
                return Err(io::ErrorKind::WriteZero.into());
            }
            bytes = &bytes[written as usize..];
        }
        Ok(())
    }

    fn copy_exclusive(from: &Path, to: &Path) -> io::Result<()> {
        let input = open(from, GENERIC_READ, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL)?;
        with_handle(input, |input| {
            let output = open(
                to,
                GENERIC_READ | GENERIC_WRITE,
                CREATE_NEW,
                FILE_ATTRIBUTE_NORMAL,
            )?;
            with_handle(output, |output| {
                let mut buffer = vec![0_u8; BUFFER_SIZE];
                loop {
                    let mut read = 0;
                    bool_result(unsafe {
                        ReadFile(
                            input,
                            buffer.as_mut_ptr().cast(),
                            buffer.len() as u32,
                            &mut read,
                            null_mut(),
                        )
                    })?;
                    if read == 0 {
                        return Ok(());
                    }
                    write_all(output, &buffer[..read as usize])?;
                }
            })
        })
    }

    fn delete_file(path: &Path) -> io::Result<()> {
        let path = wide(path)?;
        bool_result(unsafe { DeleteFileW(path.as_ptr()) })
    }

    #[repr(C)]
    struct RenameInformation {
        flags: u32,
        root_directory: HANDLE,
        file_name_length: u32,
        file_name: [u16; 1],
    }

    fn rename_no_replace(from: &Path, to: &Path) -> io::Result<()> {
        if inspect_presence(to)? {
            return Err(io::ErrorKind::AlreadyExists.into());
        }
        let destination = wide(to)?;
        let name_bytes = (destination.len() - 1)
            .checked_mul(size_of::<u16>())
            .and_then(|length| u32::try_from(length).ok())
            .ok_or(io::ErrorKind::InvalidInput)?;
        let buffer_length =
            offset_of!(RenameInformation, file_name) + destination.len() * size_of::<u16>();
        let mut storage = vec![0_usize; buffer_length.div_ceil(size_of::<usize>())];
        let information = storage.as_mut_ptr().cast::<RenameInformation>();
        unsafe {
            (*information).flags = 0;
            (*information).root_directory = null_mut();
            (*information).file_name_length = name_bytes;
            destination
                .as_ptr()
                .copy_to_nonoverlapping((*information).file_name.as_mut_ptr(), destination.len());
        }
        let source = open(
            from,
            DELETE | GENERIC_READ,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
        )?;
        with_handle(source, |source| {
            bool_result(unsafe {
                SetFileInformationByHandle(
                    source,
                    FILE_RENAME_INFO_EX_CLASS,
                    information.cast::<c_void>(),
                    buffer_length as u32,
                )
            })
        })?;
        if inspect_presence(from)? || !inspect_presence(to)? {
            return Err(io::Error::other("rename result was ambiguous"));
        }
        Ok(())
    }

    #[rustfmt::skip]
    fn replace_file(from: &Path, to: &Path, backup: &Path) -> io::Result<()> {
        let from_wide = wide(from)?;
        let to_wide = wide(to)?;
        let backup_wide = wide(backup)?;
        let reservation = open(backup, GENERIC_READ | GENERIC_WRITE, CREATE_NEW, FILE_ATTRIBUTE_NORMAL)?;
        with_handle(reservation, |_| Ok(()))?;
        // The known-empty path stays reserved by convention after close; this assumes
        // cooperating product processes and does not protect against adversarial mutation.
        bool_result(unsafe { ReplaceFileW(to_wide.as_ptr(), from_wide.as_ptr(), backup_wide.as_ptr(), 0, null(), null()) })?;
        if inspect_presence(from)? || !inspect_presence(to)? || !inspect_presence(backup)? {
            return Err(io::Error::other("replacement result was ambiguous"));
        }
        Ok(())
    }

    fn has_lexically_ambiguous_component(path: &OsStr) -> bool {
        let raw: Vec<_> = path.encode_wide().collect();
        raw.split(|unit| *unit == b'\\' as u16 || *unit == b'/' as u16)
            .any(|component| component == [b'.' as u16] || component == [b'.' as u16, b'.' as u16])
    }

    #[rustfmt::skip]
    fn verify_layout(root: &Path, phase: VerificationPhase<'_>) -> io::Result<()> {
        if !root.is_absolute() || has_lexically_ambiguous_component(root.as_os_str()) { return Err(io::ErrorKind::Unsupported.into()); }
        let canonical_expected = root.join("repuestos-autos.sqlite3"); let marker_expected = root.join("restore-state.json");
        let root_volume = volume_path(root)?;
        if volume_filesystem(&root_volume)? != OsStr::new("NTFS") || unsafe { GetDriveTypeW(root_volume.as_ptr()) } != DRIVE_FIXED { return Err(io::ErrorKind::Unsupported.into()); }
        let mut sidecars = Vec::new();
        for entry in std::fs::read_dir(root)? {
            let path = entry?.path(); let name = path.file_name().map(|name| name.to_string_lossy()).unwrap_or_default();
            if name.starts_with("restore-state.json.previous-") {
                let known = (0..MARKER_BACKUP_LIMIT).any(|slot| path == root.join(format!("restore-state.json.previous-{slot}")));
                if !known || !valid_sidecar(&path)? { return Err(io::ErrorKind::InvalidData.into()); }
                sidecars.push(path);
            }
        }
        let mut required = Vec::<PathBuf>::new(); let mut stage_path = None;
        match phase {
            VerificationPhase::Prepare { stage, protective, canonical } => {
                let staging = root.join("backup-restore/staging");
                if !stage.starts_with(&staging) || stage == staging || protective != root.join("pre-restore.sqlite3") || canonical != canonical_expected || inspect_presence(&marker_expected)? { return Err(io::ErrorKind::Unsupported.into()); }
                required.extend([stage.to_path_buf(), protective.to_path_buf(), canonical.to_path_buf()]); stage_path = Some(stage);
            }
            VerificationPhase::Recovery { marker, source, canonical } => {
                let sources = [root.join("restore-rollback.sqlite3"), root.join("pre-restore.sqlite3")];
                if marker != marker_expected || canonical != canonical_expected || !sources.iter().any(|path| source == path) || !valid_marker(marker)? { return Err(io::ErrorKind::Unsupported.into()); }
                required.extend([marker.to_path_buf(), source.to_path_buf()]); if inspect_presence(canonical)? { required.push(canonical.to_path_buf()); }
            }
            VerificationPhase::Completion { marker, canonical } => {
                if marker != marker_expected || canonical != canonical_expected || !inspect_presence(canonical)? || (inspect_presence(marker)? && !valid_marker(marker)?) { return Err(io::ErrorKind::Unsupported.into()); }
                required.push(canonical.to_path_buf()); if inspect_presence(marker)? { required.push(marker.to_path_buf()); }
            }
        }
        let mut ancestor = Some(root); while let Some(path) = ancestor { reject_reparse(path, true)?; ancestor = path.parent(); }
        for path in required { inspect_required_file(&path, &root_volume)?; }
        let mut optional = vec![root.join("restore-rollback.sqlite3"), root.join("pre-restore.sqlite3"), root.join("restore-state.json.part"), root.join("restore-recovery.sqlite3.part")]; optional.extend(sidecars);
        for path in optional { if inspect_presence(&path)? { inspect_required_file(&path, &root_volume)?; } }
        if let Some(stage) = stage_path {
            let mut ancestor = stage.parent(); while let Some(path) = ancestor { reject_reparse(path, true)?; if path == root { return Ok(()); } ancestor = path.parent(); }
            return Err(io::ErrorKind::Unsupported.into());
        }
        Ok(())
    }

    fn inspect_required_file(path: &Path, root_volume: &[u16]) -> io::Result<()> {
        if !path.is_absolute() || has_lexically_ambiguous_component(path.as_os_str()) {
            return Err(io::ErrorKind::Unsupported.into());
        }
        require_same_volume(root_volume, &volume_path(path)?)?;
        reject_reparse(path, false)
    }

    fn inspect_presence(path: &Path) -> io::Result<bool> {
        match std::fs::symlink_metadata(path) {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    fn valid_marker(path: &Path) -> io::Result<bool> {
        Ok(matches!(
            read_regular(path)?.as_slice(),
            PREPARED | LIVE_MOVED | CANDIDATE_INSTALLED
        ))
    }

    fn valid_sidecar(path: &Path) -> io::Result<bool> {
        let bytes = read_regular(path)?;
        Ok(bytes.is_empty()
            || matches!(
                bytes.as_slice(),
                PREPARED | LIVE_MOVED | CANDIDATE_INSTALLED
            ))
    }

    #[rustfmt::skip]
    fn read_regular(path: &Path) -> io::Result<Vec<u8>> {
        let handle = open(path, GENERIC_READ, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT)?;
        with_handle(handle, |handle| {
            validate_handle_kind(handle, false)?; let mut bytes = Vec::new(); let mut buffer = [0_u8; 4096];
            loop {
                let mut read = 0; bool_result(unsafe { ReadFile(handle, buffer.as_mut_ptr().cast(), buffer.len() as u32, &mut read, null_mut()) })?;
                if read == 0 { return Ok(bytes); } bytes.extend_from_slice(&buffer[..read as usize]);
            }
        })
    }

    fn volume_path(path: &Path) -> io::Result<Vec<u16>> {
        let path = wide(path)?;
        let mut capacity = 260_usize;
        loop {
            let mut volume = vec![0_u16; capacity];
            if unsafe {
                GetVolumePathNameW(path.as_ptr(), volume.as_mut_ptr(), volume.len() as u32)
            } != 0
            {
                let length = volume
                    .iter()
                    .position(|value| *value == 0)
                    .ok_or(io::ErrorKind::InvalidData)?;
                volume.truncate(length + 1);
                return Ok(volume);
            }
            let error = io::Error::last_os_error();
            if error.raw_os_error() != Some(122) || capacity >= 32_768 {
                return Err(error);
            }
            capacity = (capacity * 2).min(32_768);
        }
    }

    fn require_same_volume(expected: &[u16], actual: &[u16]) -> io::Result<()> {
        if actual == expected {
            Ok(())
        } else {
            Err(io::ErrorKind::Unsupported.into())
        }
    }

    fn volume_filesystem(volume: &[u16]) -> io::Result<std::ffi::OsString> {
        let mut capacity = 32_usize;
        loop {
            let mut filesystem = vec![0_u16; capacity];
            if unsafe {
                GetVolumeInformationW(
                    volume.as_ptr(),
                    null_mut(),
                    0,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    filesystem.as_mut_ptr(),
                    filesystem.len() as u32,
                )
            } != 0
            {
                let length = filesystem
                    .iter()
                    .position(|value| *value == 0)
                    .ok_or(io::ErrorKind::InvalidData)?;
                return Ok(std::ffi::OsString::from_wide(&filesystem[..length]));
            }
            let error = io::Error::last_os_error();
            if !matches!(error.raw_os_error(), Some(122) | Some(234)) || capacity >= 256 {
                return Err(error);
            }
            capacity = (capacity * 2).min(256);
        }
    }

    fn reject_reparse(path: &Path, directory: bool) -> io::Result<()> {
        let flags = if directory {
            FILE_FLAG_BACKUP_SEMANTICS
        } else {
            FILE_ATTRIBUTE_NORMAL
        };
        let handle = open(
            path,
            GENERIC_READ,
            OPEN_EXISTING,
            flags | FILE_FLAG_OPEN_REPARSE_POINT,
        )?;
        with_handle(handle, |handle| validate_handle_kind(handle, directory))
    }

    fn validate_handle_kind(handle: HANDLE, directory: bool) -> io::Result<()> {
        let mut information = FILE_ATTRIBUTE_TAG_INFO::default();
        bool_result(unsafe {
            GetFileInformationByHandleEx(
                handle,
                9,
                (&mut information as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
                size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
            )
        })?;
        let attributes = information.FileAttributes;
        if attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
            || (attributes & FILE_ATTRIBUTE_DIRECTORY != 0) != directory
        {
            return Err(io::ErrorKind::Unsupported.into());
        }
        Ok(())
    }

    fn bool_result(result: i32) -> io::Result<()> {
        if result == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use std::fs;

        use super::*;

        fn directory(name: &str) -> std::path::PathBuf {
            std::env::temp_dir().join(format!("r-a-native-{name}-{}", uuid::Uuid::new_v4()))
        }

        fn verify_prepare(
            root: &Path,
            stage: &Path,
            protective: &Path,
            canonical: &Path,
        ) -> io::Result<()> {
            verify_layout(
                root,
                VerificationPhase::Prepare {
                    stage,
                    protective,
                    canonical,
                },
            )
        }

        #[test]
        fn closed_reservation_allows_preserved_replacement_and_keeps_collision_evidence() {
            let root = directory("exclusive-replace");
            fs::create_dir_all(&root).unwrap();
            let source = root.join("source");
            let destination = root.join("destination");
            let backup = root.join("preserved-old");
            fs::write(&source, b"new").unwrap();
            fs::write(&destination, b"old").unwrap();

            assert!(write_exclusive(&destination, b"overwrite").is_err());
            assert!(copy_exclusive(&source, &destination).is_err());
            assert!(rename_no_replace(&source, &destination).is_err());
            replace_file(&source, &destination, &backup).unwrap();
            assert_eq!(fs::read(&destination).unwrap(), b"new");
            assert_eq!(fs::read(&backup).unwrap(), b"old");

            fs::write(&source, b"newer").unwrap();
            assert_eq!(
                replace_file(&source, &destination, &backup)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::AlreadyExists
            );
            assert_eq!(fs::read(&destination).unwrap(), b"new");
            assert_eq!(fs::read(&source).unwrap(), b"newer");
            assert_eq!(fs::read(&backup).unwrap(), b"old");
            fs::remove_dir_all(root).unwrap();
        }

        #[test]
        fn layout_rejects_parent_ambiguity_and_the_actual_missing_protective_file() {
            let root = directory("layout");
            let staging = root.join("backup-restore").join("staging");
            fs::create_dir_all(&staging).unwrap();
            let canonical = root.join("repuestos-autos.sqlite3");
            let protective = root.join("pre-restore.sqlite3");
            let stage = staging.join("candidate.sqlite3");
            for path in [&canonical, &protective, &stage] {
                fs::write(path, b"data").unwrap();
            }
            verify_prepare(&root, &stage, &protective, &canonical).unwrap();
            assert!(require_same_volume(&[b'C' as u16, 0], &[b'D' as u16, 0]).is_err());

            let ambiguous = staging.join("..").join("escape.sqlite3");
            fs::write(root.join("backup-restore").join("escape.sqlite3"), b"data").unwrap();
            assert!(verify_prepare(&root, &ambiguous, &protective, &canonical).is_err());
            fs::remove_file(&protective).unwrap();
            assert!(verify_prepare(&root, &stage, &protective, &canonical).is_err());
            fs::remove_dir_all(root).unwrap();
        }

        #[test]
        fn layout_rejects_lexical_interior_dot_component() {
            let root = directory("layout-interior-dot");
            let staging = root.join("backup-restore").join("staging");
            fs::create_dir_all(&staging).unwrap();
            let canonical = root.join("repuestos-autos.sqlite3");
            let protective = root.join("pre-restore.sqlite3");
            let stage = staging.join(".").join("candidate.sqlite3");
            for path in [&canonical, &protective, &stage] {
                fs::write(path, b"data").unwrap();
            }

            assert!(verify_prepare(&root, &stage, &protective, &canonical).is_err());

            fs::remove_dir_all(root).unwrap();
        }

        #[test]
        fn all_share_handles_allow_rename_and_native_failures_remain_errors() {
            let root = directory("sharing");
            fs::create_dir_all(&root).unwrap();
            let source = root.join("source");
            let destination = root.join("destination");
            fs::write(&source, b"data").unwrap();
            let held = open(&source, GENERIC_READ, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL).unwrap();
            with_handle(held, |_| rename_no_replace(&source, &destination)).unwrap();
            assert!(sync_file(&source).is_err());
            let consumed = root.join("backup");
            assert!(replace_file(&source, &destination, &consumed).is_err());
            assert_eq!(fs::read(consumed).unwrap(), b"");
            fs::remove_dir_all(root).unwrap();
        }
    }
}

#[cfg(not(windows))]
struct UnsupportedDurableFs;

#[cfg(not(windows))]
impl DurableFs for UnsupportedDurableFs {
    fn execute(&self, operation: FsOperation<'_>) -> io::Result<()> {
        match operation {
            FsOperation::Verify(root, phase) => {
                let _ = root;
                touch_phase(phase);
            }
            FsOperation::SyncFile(a) | FsOperation::SyncDirectory(a) | FsOperation::Remove(a) => {
                let _ = a;
            }
            FsOperation::WriteExclusive(a, b) => {
                let _ = (a, b);
            }
            FsOperation::CopyExclusive(a, b) | FsOperation::RenameNoReplace(a, b) => {
                let _ = (a, b);
            }
            FsOperation::Replace(a, b, c) => {
                let _ = (a, b, c);
            }
        }
        Err(io::ErrorKind::Unsupported.into())
    }

    fn is_present(&self, path: &Path) -> io::Result<bool> {
        Ok(inspect_host_path(path)?.is_some())
    }
    fn read(&self, _path: &Path) -> io::Result<Vec<u8>> {
        Err(io::ErrorKind::Unsupported.into())
    }
}

#[cfg(not(windows))]
#[rustfmt::skip]
fn touch_phase(phase: VerificationPhase<'_>) {
    match phase {
        VerificationPhase::Prepare { stage, protective, canonical } => { let _ = (stage, protective, canonical); }
        VerificationPhase::Recovery { marker, source, canonical } => { let _ = (marker, source, canonical); }
        VerificationPhase::Completion { marker, canonical } => { let _ = (marker, canonical); }
    }
}

#[cfg(not(windows))]
fn inspect_host_path(path: &Path) -> io::Result<Option<bool>> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata.file_type().is_file())),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(all(test, not(windows)))]
struct TestDurableFs;

#[cfg(all(test, not(windows)))]
impl DurableFs for TestDurableFs {
    fn execute(&self, operation: FsOperation<'_>) -> io::Result<()> {
        match operation {
            FsOperation::Verify(root, phase) => verify_test_layout(root, phase),
            FsOperation::SyncFile(path) => File::open(path)?.sync_all(),
            FsOperation::SyncDirectory(path) => File::open(path)?.sync_all(),
            FsOperation::WriteExclusive(path, bytes) => {
                let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
                file.write_all(bytes)?;
                file.sync_all()
            }
            FsOperation::CopyExclusive(from, to) => {
                let mut output = OpenOptions::new().write(true).create_new(true).open(to)?;
                io::copy(&mut File::open(from)?, &mut output)?;
                Ok(())
            }
            FsOperation::Remove(path) => fs::remove_file(path),
            FsOperation::RenameNoReplace(from, to) => {
                if to.exists() {
                    return Err(io::ErrorKind::AlreadyExists.into());
                }
                fs::rename(from, to)
            }
            FsOperation::Replace(from, to, backup) => {
                drop(
                    OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(backup)?,
                );
                fs::rename(to, backup).map_err(io::Error::other)?;
                fs::rename(from, to).map_err(io::Error::other)
            }
        }
    }

    fn is_present(&self, path: &Path) -> io::Result<bool> {
        Ok(inspect_host_path(path)?.is_some())
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }
}

#[cfg(all(test, not(windows)))]
#[rustfmt::skip]
fn verify_test_layout(root: &Path, phase: VerificationPhase<'_>) -> io::Result<()> {
    verify_test_phase(root, phase, inspect_host_path, |path| fs::read(path), |path| fs::read_dir(path)?.map(|entry| Ok(entry?.path())).collect())
}

#[cfg(all(test, not(windows)))]
type ProductionDurableFs = TestDurableFs;
#[cfg(all(not(test), not(windows)))]
type ProductionDurableFs = UnsupportedDurableFs;
#[cfg(windows)]
type ProductionDurableFs = platform::WindowsDurableFs;

fn production(root: &Path) -> RestoreTransitions<ProductionDurableFs> {
    #[cfg(all(test, not(windows)))]
    let fs = TestDurableFs;
    #[cfg(all(not(test), not(windows)))]
    let fs = UnsupportedDurableFs;
    #[cfg(windows)]
    let fs = platform::WindowsDurableFs;
    RestoreTransitions::new(root.to_path_buf(), fs)
}

pub(super) fn prepare(root: &Path, stage: &Path, protective: &Path) -> Result<(), StorageError> {
    production(root).prepare_durable_restore(
        stage,
        protective,
        &root.join("repuestos-autos.sqlite3"),
    )
}

pub(super) fn install(root: &Path, stage: &Path, canonical: &Path) -> Result<(), StorageError> {
    production(root).install_durable_restore(stage, canonical)
}

pub(super) fn recover(root: &Path, source: &Path, canonical: &Path) -> Result<(), StorageError> {
    production(root).recover_canonical_durably(source, canonical)
}

pub(super) fn complete(root: &Path) -> Result<(), StorageError> {
    production(root).complete_durable_restore(|| Ok(()))
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use super::*;

    #[rustfmt::skip]
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    enum Kind { Verify, SyncFile, SyncDirectory, Write, Copy, Remove, Rename, Replace }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Recorded {
        kind: Kind,
        paths: Vec<PathBuf>,
    }

    #[rustfmt::skip]
    impl Recorded {
        fn from(operation: FsOperation<'_>) -> Self {
            let (kind, paths) = match operation {
                FsOperation::Verify(root, phase) => { let mut paths = vec![root.into()]; match phase { VerificationPhase::Prepare { stage, protective, canonical } => paths.extend([stage.into(), protective.into(), canonical.into()]), VerificationPhase::Recovery { marker, source, canonical } => paths.extend([marker.into(), source.into(), canonical.into()]), VerificationPhase::Completion { marker, canonical } => paths.extend([marker.into(), canonical.into()]) }; (Kind::Verify, paths) },
                FsOperation::SyncFile(path) => (Kind::SyncFile, vec![path.into()]),
                FsOperation::SyncDirectory(path) => (Kind::SyncDirectory, vec![path.into()]),
                FsOperation::WriteExclusive(path, _) => (Kind::Write, vec![path.into()]),
                FsOperation::CopyExclusive(from, to) => (Kind::Copy, vec![from.into(), to.into()]),
                FsOperation::Remove(path) => (Kind::Remove, vec![path.into()]),
                FsOperation::RenameNoReplace(from, to) => (Kind::Rename, vec![from.into(), to.into()]),
                FsOperation::Replace(from, to, backup) => (Kind::Replace, vec![from.into(), to.into(), backup.into()]),
            };
            Self { kind, paths }
        }
    }

    #[derive(Clone, Copy)]
    enum Moment {
        Before,
        After,
    }

    #[derive(Clone)]
    struct Model {
        files: HashMap<PathBuf, Vec<u8>>,
        durable: HashMap<PathBuf, Vec<u8>>,
        operations: Vec<Recorded>,
        failpoint: Option<(Kind, usize, Moment)>,
        hidden_from_exists: Option<PathBuf>,
        unsafe_entries: Vec<PathBuf>,
        supported: bool,
    }

    struct RecordingFs(RefCell<Model>);

    impl RecordingFs {
        fn seeded(paths: &[(&str, &[u8])]) -> Self {
            let files: HashMap<_, _> = paths
                .iter()
                .map(|(path, bytes)| (PathBuf::from(path), bytes.to_vec()))
                .collect();
            Self(RefCell::new(Model {
                durable: files.clone(),
                files,
                operations: vec![],
                failpoint: None,
                hidden_from_exists: None,
                unsafe_entries: Vec::new(),
                supported: true,
            }))
        }
    }

    impl DurableFs for RecordingFs {
        fn execute(&self, operation: FsOperation<'_>) -> io::Result<()> {
            let recorded = Recorded::from(operation);
            let mut model = self.0.borrow_mut();
            let ordinal = model
                .operations
                .iter()
                .filter(|item| item.kind == recorded.kind)
                .count();
            model.operations.push(recorded.clone());
            if matches!(model.failpoint, Some((kind, at, Moment::Before)) if kind == recorded.kind && at == ordinal)
            {
                return Err(io::Error::other("injected before"));
            }
            match operation {
                FsOperation::Verify(root, phase) => verify_model(root, phase, &model)?,
                FsOperation::SyncDirectory(_) => model.durable = model.files.clone(),
                FsOperation::WriteExclusive(path, bytes) => {
                    model.files.insert(path.into(), bytes.into());
                }
                FsOperation::CopyExclusive(from, to) => {
                    let bytes = model.files.get(from).cloned().unwrap_or_default();
                    model.files.insert(to.into(), bytes);
                }
                FsOperation::Remove(path) => {
                    model.files.remove(path);
                }
                FsOperation::RenameNoReplace(from, to) => {
                    let bytes = model.files.remove(from).unwrap_or_default();
                    model.files.insert(to.into(), bytes);
                }
                FsOperation::Replace(from, to, backup) => {
                    if model.files.contains_key(backup) {
                        return Err(io::ErrorKind::AlreadyExists.into());
                    }
                    model.files.insert(backup.into(), Vec::new());
                    let old = model.files.remove(to).unwrap_or_default();
                    model.files.insert(backup.into(), old);
                    let bytes = model.files.remove(from).unwrap_or_default();
                    model.files.insert(to.into(), bytes);
                }
                FsOperation::SyncFile(_) => {}
            }
            if matches!(model.failpoint, Some((kind, at, Moment::After)) if kind == recorded.kind && at == ordinal)
            {
                return Err(io::Error::other("injected after"));
            }
            Ok(())
        }

        fn is_present(&self, path: &Path) -> io::Result<bool> {
            let model = self.0.borrow();
            Ok(model.hidden_from_exists.as_deref() != Some(path) && model.files.contains_key(path))
        }

        fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
            self.0
                .borrow()
                .files
                .get(path)
                .cloned()
                .ok_or_else(|| io::ErrorKind::NotFound.into())
        }
    }

    fn verify_model(root: &Path, phase: VerificationPhase<'_>, model: &Model) -> io::Result<()> {
        if !model.supported {
            return Err(io::ErrorKind::Unsupported.into());
        }
        verify_test_phase(
            root,
            phase,
            |path| {
                Ok(model
                    .files
                    .contains_key(path)
                    .then(|| !model.unsafe_entries.iter().any(|entry| entry == path)))
            },
            |path| {
                model
                    .files
                    .get(path)
                    .cloned()
                    .ok_or_else(|| io::ErrorKind::NotFound.into())
            },
            |directory| {
                Ok(model
                    .files
                    .keys()
                    .filter(|path| path.parent() == Some(directory))
                    .cloned()
                    .collect())
            },
        )
    }

    fn seeded() -> RecordingFs {
        RecordingFs::seeded(&[
            ("/app/repuestos-autos.sqlite3", b"live"),
            ("/app/backup-restore/staging/stage.sqlite3", b"stage"),
            ("/app/pre-restore.sqlite3", b"protective"),
            ("/app/restore-state.json", PREPARED),
            ("/app/restore-state.json.part", b"stale"),
            ("/app/restore-recovery.sqlite3.part", b"stale"),
        ])
    }

    fn protocol(fs: RecordingFs) -> RestoreTransitions<RecordingFs> {
        RestoreTransitions::new("/app".into(), fs)
    }

    #[rustfmt::skip]
    #[test]
    fn protocol_orders_exact_markers_barriers_installation_and_retained_recovery() {
        let stage = Path::new("/app/backup-restore/staging/stage.sqlite3");
        let canonical = Path::new("/app/repuestos-autos.sqlite3");
        let interrupted = completed_cycle();
        interrupted.prepare_durable_restore(stage, Path::new("/app/pre-restore.sqlite3"), canonical).unwrap();
        assert_eq!(interrupted.fs.0.borrow().durable.get(Path::new("/app/restore-state.json")), Some(&PREPARED.to_vec()));
        interrupted.fs.0.borrow_mut().failpoint = Some((Kind::Rename, 2, Moment::Before));
        assert_eq!(interrupted.install_durable_restore(stage, canonical), Err(StorageError::StorageUnavailable));
        assert_eq!(interrupted.fs.0.borrow().durable.get(Path::new("/app/restore-state.json")), Some(&LIVE_MOVED.to_vec()));

        let completed = completed_cycle();
        completed.prepare_durable_restore(stage, Path::new("/app/pre-restore.sqlite3"), canonical).unwrap();
        completed.install_durable_restore(stage, canonical).unwrap();
        completed.recover_canonical_durably(Path::new("/app/restore-rollback.sqlite3"), canonical).unwrap();
        let model = completed.fs.0.borrow();
        assert_eq!(model.durable.get(Path::new("/app/restore-state.json")), Some(&CANDIDATE_INSTALLED.to_vec()));
        assert_eq!(model.durable.get(Path::new("/app/restore-rollback.sqlite3")), Some(&b"live".to_vec()));
        assert!(model.operations.iter().any(|item| item.kind == Kind::Replace));
    }

    fn assert_all_failpoints(
        seed: impl Fn() -> RestoreTransitions<RecordingFs>,
        action: impl Fn(&RestoreTransitions<RecordingFs>) -> Result<(), StorageError>,
    ) {
        let baseline = seed();
        action(&baseline).unwrap();
        let operations = baseline.fs.0.borrow().operations.clone();
        for (index, operation) in operations.iter().enumerate() {
            let ordinal = operations[..index]
                .iter()
                .filter(|item| item.kind == operation.kind)
                .count();
            for moment in [Moment::Before, Moment::After] {
                let transitions = seed();
                transitions.fs.0.borrow_mut().failpoint = Some((operation.kind, ordinal, moment));
                assert_eq!(
                    action(&transitions),
                    Err(StorageError::StorageUnavailable),
                    "failpoint {:?} ordinal {ordinal} at {}",
                    operation.kind,
                    match moment {
                        Moment::Before => "before",
                        Moment::After => "after",
                    }
                );
                assert!(transitions
                    .fs
                    .0
                    .borrow()
                    .durable
                    .contains_key(Path::new("/app/pre-restore.sqlite3")));
            }
        }
    }
    fn completed_cycle() -> RestoreTransitions<RecordingFs> {
        protocol(RecordingFs::seeded(&[
            ("/app/repuestos-autos.sqlite3", b"live"),
            (STAGE, b"stage"),
            (PROTECTIVE, b"protective"),
        ]))
    }

    const STAGE: &str = "/app/backup-restore/staging/stage.sqlite3";
    const CANONICAL: &str = "/app/repuestos-autos.sqlite3";
    const PROTECTIVE: &str = "/app/pre-restore.sqlite3";
}
