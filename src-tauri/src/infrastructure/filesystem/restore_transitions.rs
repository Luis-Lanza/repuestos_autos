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
}
