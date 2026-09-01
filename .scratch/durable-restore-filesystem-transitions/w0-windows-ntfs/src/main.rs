#[cfg(not(windows))]
fn main() {
    eprintln!(
        "REFUSED: this W0 feasibility harness must run on real Windows with a local fixed NTFS root"
    );
    std::process::exit(2);
}

#[cfg(windows)]
mod windows_harness {
    use std::ffi::{c_void, OsStr};
    use std::fs;
    use std::os::windows::ffi::OsStrExt;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, ERROR_FILE_EXISTS, HANDLE,
        INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, DeleteFileW, FlushFileBuffers, GetDriveTypeW, GetFileAttributesW,
        GetFileInformationByHandleEx, GetVolumeInformationW, GetVolumePathNameW, ReplaceFileW,
        SetFileInformationByHandle, WriteFile, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL,
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
        FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    };

    const DELETE_ACCESS: u32 = 0x0001_0000;
    const GENERIC_READ: u32 = 0x8000_0000;
    const GENERIC_WRITE: u32 = 0x4000_0000;
    const DRIVE_FIXED: u32 = 3;
    const INVALID_ATTRIBUTES: u32 = u32::MAX;
    const FILE_ATTRIBUTE_TAG_INFO_CLASS: i32 = 9;
    const FILE_RENAME_INFO_EX_CLASS: i32 = 22;
    const FILE_RENAME_FLAG_WRITE_THROUGH: u32 = 0x0000_0002;
    const ALL_SHARES: u32 = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;

    #[repr(C)]
    struct AttributeTagInfo {
        attributes: u32,
        reparse_tag: u32,
    }

    struct Recorder {
        root: PathBuf,
        sequence: u64,
        failures: usize,
    }

    impl Recorder {
        fn new(root: PathBuf) -> Self {
            Self {
                root,
                sequence: 0,
                failures: 0,
            }
        }

        fn event(
            &mut self,
            case_name: &str,
            status: &str,
            operation: &str,
            role: &str,
            path: Option<&Path>,
            success: Option<bool>,
            os_error: u32,
            facts: &str,
        ) {
            self.sequence += 1;
            let logical_path = path.map_or_else(|| "<none>".to_owned(), |value| self.label(value));
            let success_json = success.map_or_else(|| "null".to_owned(), |value| value.to_string());
            println!(
                "{{\"sequence\":{},\"case\":\"{}\",\"status\":\"{}\",\"operation\":\"{}\",\"role\":\"{}\",\"path\":\"{}\",\"success\":{},\"os_error\":{},{} }}",
                self.sequence,
                json_escape(case_name),
                json_escape(status),
                json_escape(operation),
                json_escape(role),
                json_escape(&logical_path),
                success_json,
                os_error,
                facts
            );
        }

        fn fail(
            &mut self,
            case_name: &str,
            operation: &str,
            role: &str,
            path: Option<&Path>,
            os_error: u32,
            facts: &str,
        ) {
            self.failures += 1;
            self.event(
                case_name,
                "FAIL",
                operation,
                role,
                path,
                Some(false),
                os_error,
                facts,
            );
        }

        fn unproven(
            &mut self,
            case_name: &str,
            operation: &str,
            role: &str,
            path: Option<&Path>,
            os_error: u32,
            facts: &str,
        ) {
            self.failures += 1;
            self.event(
                case_name,
                "UNPROVEN",
                operation,
                role,
                path,
                Some(false),
                os_error,
                facts,
            );
        }

        fn label(&self, path: &Path) -> String {
            match path.strip_prefix(&self.root) {
                Ok(relative) if relative.as_os_str().is_empty() => "<ROOT>".to_owned(),
                Ok(relative) => format!("<ROOT>/{}", relative.to_string_lossy().replace('\\', "/")),
                Err(_) => "<OUTSIDE_ROOT_REDACTED>".to_owned(),
            }
        }

        fn inventory(&mut self, case_name: &str, entries: &[(&str, &Path)]) {
            let mut facts = String::from("\"entries\":[");
            for (index, (role, path)) in entries.iter().enumerate() {
                if index > 0 {
                    facts.push(',');
                }
                let wide = wide(path);
                // SAFETY: `wide` is a live, NUL-terminated UTF-16 path buffer.
                let attributes = unsafe { GetFileAttributesW(wide.as_ptr()) };
                let (exists, error) = if attributes == INVALID_ATTRIBUTES {
                    (false, last_error())
                } else {
                    (true, 0)
                };
                facts.push_str(&format!(
                    "{{\"role\":\"{}\",\"path\":\"{}\",\"exists\":{},\"attributes\":{},\"reparse\":{},\"os_error\":{}}}",
                    json_escape(role),
                    json_escape(&self.label(path)),
                    exists,
                    attributes,
                    exists && attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0,
                    error
                ));
            }
            facts.push(']');
            self.event(
                case_name,
                "INFO",
                "post_operation_inventory",
                "all_roles",
                None,
                None,
                0,
                &facts,
            );
        }
    }

    pub fn entry() -> i32 {
        let Some(root) = parse_root() else {
            eprintln!("usage: w0-windows-ntfs-feasibility.exe --root <local-app-data-test-root>");
            return 64;
        };
        let root = match root.canonicalize() {
            Ok(value) => value,
            Err(error) => {
                eprintln!(
                    "the supplied root cannot be canonicalized: {}",
                    error.kind()
                );
                return 65;
            }
        };
        let mut recorder = Recorder::new(root.clone());

        environment_case(&mut recorder);
        let layout = match prepare_layout(&mut recorder) {
            Some(value) => value,
            None => return finish(&mut recorder),
        };
        storage_case(&mut recorder, &layout);
        file_flush_case(&mut recorder, &layout);
        directory_flush_case(&mut recorder, &layout);
        marker_case(&mut recorder, &layout);
        rename_case(&mut recorder, &layout);
        sharing_case(&mut recorder, &layout);

        finish(&mut recorder)
    }

    struct Layout {
        root: PathBuf,
        staging: PathBuf,
        backup_restore: PathBuf,
        stage: PathBuf,
        protective: PathBuf,
        temp: PathBuf,
        marker: PathBuf,
        marker_part: PathBuf,
        marker_old: PathBuf,
        canonical: PathBuf,
        rollback: PathBuf,
    }

    fn prepare_layout(recorder: &mut Recorder) -> Option<Layout> {
        let case_name = "layout";
        let root = recorder.root.clone();
        let backup_restore = root.join("backup-restore");
        let staging = backup_restore.join("staging");
        for (role, path) in [
            ("backup_restore_dir", &backup_restore),
            ("staging_dir", &staging),
        ] {
            match fs::create_dir_all(path) {
                Ok(()) => recorder.event(
                    case_name,
                    "PASS",
                    "create_directory",
                    role,
                    Some(path),
                    Some(true),
                    0,
                    "\"access\":\"filesystem_create_dir_all\",\"share\":0,\"flags\":0",
                ),
                Err(error) => {
                    recorder.fail(
                        case_name,
                        "create_directory",
                        role,
                        Some(path),
                        raw_error(&error),
                        "\"access\":\"filesystem_create_dir_all\",\"share\":0,\"flags\":0",
                    );
                    return None;
                }
            }
        }
        let layout = Layout {
            stage: staging.join("stage.db"),
            protective: backup_restore.join("protective.db"),
            temp: staging.join("temp.part"),
            marker: root.join("restore.marker"),
            marker_part: root.join("restore.marker.part"),
            marker_old: backup_restore.join("restore.marker.old"),
            canonical: root.join("canonical.db"),
            rollback: backup_restore.join("rollback.db"),
            root,
            staging,
            backup_restore,
        };
        recorder.inventory(
            case_name,
            &[
                ("root", &layout.root),
                ("staging_dir", &layout.staging),
                ("backup_restore_dir", &layout.backup_restore),
            ],
        );
        Some(layout)
    }

    fn environment_case(recorder: &mut Recorder) {
        let case_name = "ordinary_windows_environment";
        let ordinary = std::env::var("W0_ORDINARY_NON_ADMIN").as_deref() == Ok("true");
        if ordinary {
            recorder.event(
                case_name,
                "PASS",
                "verify_non_admin_runner_attestation",
                "process",
                None,
                Some(true),
                0,
                "\"ordinary_non_admin\":true,\"access\":0,\"share\":0,\"flags\":0",
            );
        } else {
            recorder.fail(
                case_name,
                "verify_non_admin_runner_attestation",
                "process",
                None,
                0,
                "\"ordinary_non_admin\":false,\"access\":0,\"share\":0,\"flags\":0",
            );
        }

        for (role, program, argument) in [
            ("windows_version", "cmd", "/c ver"),
            ("rust_toolchain", "rustc", "-V"),
            ("cargo_toolchain", "cargo", "-V"),
        ] {
            let result = Command::new(program)
                .args(argument.split_whitespace())
                .output();
            match result {
                Ok(output) => {
                    let text = String::from_utf8_lossy(&output.stdout);
                    recorder.event(
                        case_name,
                        if output.status.success() {
                            "PASS"
                        } else {
                            "FAIL"
                        },
                        "capture_environment_fact",
                        role,
                        None,
                        Some(output.status.success()),
                        output.status.code().unwrap_or(-1) as u32,
                        &format!(
                            "\"value\":\"{}\",\"access\":0,\"share\":0,\"flags\":0",
                            json_escape(text.trim())
                        ),
                    );
                    if !output.status.success() {
                        recorder.failures += 1;
                    }
                }
                Err(error) => recorder.fail(
                    case_name,
                    "capture_environment_fact",
                    role,
                    None,
                    raw_error(&error),
                    "\"value\":\"unavailable\",\"access\":0,\"share\":0,\"flags\":0",
                ),
            }
        }
    }

    fn storage_case(recorder: &mut Recorder, layout: &Layout) {
        let case_name = "local_fixed_ntfs_same_volume_reparse";
        let root_wide = wide(&layout.root);
        let mut volume_path = vec![0_u16; 1024];
        // SAFETY: input/output buffers are live and sized as declared.
        let volume_ok = unsafe {
            GetVolumePathNameW(
                root_wide.as_ptr(),
                volume_path.as_mut_ptr(),
                volume_path.len() as u32,
            )
        } != 0;
        let volume_error = if volume_ok { 0 } else { last_error() };
        recorder.event(
            case_name,
            if volume_ok { "PASS" } else { "FAIL" },
            "GetVolumePathNameW",
            "root",
            Some(&layout.root),
            Some(volume_ok),
            volume_error,
            "\"result\":\"<VOLUME_MOUNT>\",\"access\":0,\"share\":0,\"flags\":0",
        );
        if !volume_ok {
            recorder.failures += 1;
            return;
        }
        truncate_at_nul(&mut volume_path);
        volume_path.push(0);

        let mut fs_name = vec![0_u16; 64];
        let mut serial = 0_u32;
        let mut max_component = 0_u32;
        let mut fs_flags = 0_u32;
        // SAFETY: all optional and output pointers refer to live buffers/integers.
        let info_ok = unsafe {
            GetVolumeInformationW(
                volume_path.as_ptr(),
                std::ptr::null_mut(),
                0,
                &mut serial,
                &mut max_component,
                &mut fs_flags,
                fs_name.as_mut_ptr(),
                fs_name.len() as u32,
            )
        } != 0;
        let info_error = if info_ok { 0 } else { last_error() };
        let filesystem = utf16_to_string(&fs_name);
        let is_ntfs = info_ok && filesystem.eq_ignore_ascii_case("NTFS");
        recorder.event(
            case_name,
            if is_ntfs { "PASS" } else { "FAIL" },
            "GetVolumeInformationW",
            "volume",
            None,
            Some(info_ok),
            info_error,
            &format!(
                "\"filesystem\":\"{}\",\"is_ntfs\":{},\"serial\":{},\"max_component\":{},\"filesystem_flags\":{},\"access\":0,\"share\":0,\"flags\":0",
                json_escape(&filesystem), is_ntfs, serial, max_component, fs_flags
            ),
        );
        if !is_ntfs {
            recorder.failures += 1;
        }

        // SAFETY: `volume_path` is a live, NUL-terminated volume-root buffer.
        let drive_type = unsafe { GetDriveTypeW(volume_path.as_ptr()) };
        let fixed = drive_type == DRIVE_FIXED;
        recorder.event(
            case_name,
            if fixed { "PASS" } else { "FAIL" },
            "GetDriveTypeW",
            "volume",
            None,
            Some(fixed),
            0,
            &format!(
                "\"drive_type\":{},\"required_drive_type\":{},\"access\":0,\"share\":0,\"flags\":0",
                drive_type, DRIVE_FIXED
            ),
        );
        if !fixed {
            recorder.failures += 1;
        }

        let mut all_same = true;
        for (role, path) in [
            ("root", &layout.root),
            ("staging", &layout.staging),
            ("backup_restore", &layout.backup_restore),
        ] {
            let path_wide = wide(path);
            let mut candidate = vec![0_u16; 1024];
            // SAFETY: input/output buffers are live and sized as declared.
            let ok = unsafe {
                GetVolumePathNameW(
                    path_wide.as_ptr(),
                    candidate.as_mut_ptr(),
                    candidate.len() as u32,
                )
            } != 0;
            truncate_at_nul(&mut candidate);
            truncate_at_nul(&mut volume_path);
            let same = ok
                && utf16_to_string(&candidate).eq_ignore_ascii_case(&utf16_to_string(&volume_path));
            all_same &= same;
            recorder.event(
                case_name,
                if same { "PASS" } else { "FAIL" },
                "verify_same_volume",
                role,
                Some(path),
                Some(ok),
                if ok { 0 } else { last_error() },
                &format!(
                    "\"same_volume\":{},\"volume\":\"<VOLUME_MOUNT>\",\"access\":0,\"share\":0,\"flags\":0",
                    same
                ),
            );
        }
        if !all_same {
            recorder.failures += 1;
        }

        for (role, path) in [
            ("root", &layout.root),
            ("staging", &layout.staging),
            ("backup_restore", &layout.backup_restore),
        ] {
            inspect_reparse(recorder, case_name, role, path);
        }
        recorder.event(
            case_name,
            "SKIP",
            "unsupported_storage_matrix",
            "optional_matrix",
            None,
            None,
            0,
            "\"reason\":\"removable, network, virtual, and reparse-backed roots are unavailable by design on this required local-fixed-NTFS run\",\"access\":0,\"share\":0,\"flags\":0",
        );
    }

    fn inspect_reparse(recorder: &mut Recorder, case_name: &str, role: &str, path: &Path) {
        let handle = open_handle(
            recorder,
            case_name,
            role,
            path,
            GENERIC_READ,
            ALL_SHARES,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
        );
        if handle == INVALID_HANDLE_VALUE {
            return;
        }
        let mut info = AttributeTagInfo {
            attributes: 0,
            reparse_tag: 0,
        };
        // SAFETY: handle is live and `info` is a correctly sized writable C-layout buffer.
        let ok = unsafe {
            GetFileInformationByHandleEx(
                handle,
                FILE_ATTRIBUTE_TAG_INFO_CLASS,
                (&mut info as *mut AttributeTagInfo).cast::<c_void>(),
                std::mem::size_of::<AttributeTagInfo>() as u32,
            )
        } != 0;
        let error = if ok { 0 } else { last_error() };
        let not_reparse = ok && info.attributes & FILE_ATTRIBUTE_REPARSE_POINT == 0;
        recorder.event(
            case_name,
            if not_reparse { "PASS" } else { "FAIL" },
            "GetFileInformationByHandleEx_FileAttributeTagInfo",
            role,
            Some(path),
            Some(ok),
            error,
            &format!(
                "\"attributes\":{},\"reparse_tag\":{},\"is_reparse\":{},\"access\":{},\"share\":{},\"flags\":{}",
                info.attributes,
                info.reparse_tag,
                info.attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0,
                GENERIC_READ,
                ALL_SHARES,
                FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT
            ),
        );
        if !not_reparse {
            recorder.failures += 1;
        }
        close_handle(recorder, case_name, role, path, handle);
    }

    fn file_flush_case(recorder: &mut Recorder, layout: &Layout) {
        let case_name = "file_flush_stage_protective_temp";
        for (role, path, bytes) in [
            ("stage", &layout.stage, b"stage-v1".as_slice()),
            (
                "protective",
                &layout.protective,
                b"protective-v1".as_slice(),
            ),
            ("temp", &layout.temp, b"temp-v1".as_slice()),
        ] {
            write_and_flush(recorder, case_name, role, path, bytes);
            recorder.inventory(case_name, &[(role, path)]);
        }
    }

    fn directory_flush_case(recorder: &mut Recorder, layout: &Layout) {
        let case_name = "required_directory_handle_flush";
        for (role, path) in [
            ("root", &layout.root),
            ("staging", &layout.staging),
            ("backup_restore", &layout.backup_restore),
        ] {
            if !flush_directory(recorder, case_name, role, path) {
                recorder.unproven(
                    case_name,
                    "required_directory_barrier",
                    role,
                    Some(path),
                    0,
                    "\"reason\":\"candidate directory-handle FlushFileBuffers is unsupported or failed; W0 cannot pass\",\"access\":0,\"share\":0,\"flags\":0",
                );
            }
            recorder.inventory(case_name, &[(role, path)]);
        }
    }

    fn marker_case(recorder: &mut Recorder, layout: &Layout) {
        let case_name = "marker_replace_stale_part_remove";
        write_and_flush(
            recorder,
            case_name,
            "marker_part",
            &layout.marker_part,
            b"marker-v1",
        );
        rename_no_replace(
            recorder,
            case_name,
            "marker_part_to_marker",
            &layout.marker_part,
            &layout.marker,
            Expectation::Success,
        );
        flush_directory(recorder, case_name, "root", &layout.root);
        recorder.inventory(
            case_name,
            &[
                ("marker", &layout.marker),
                ("marker_part", &layout.marker_part),
                ("marker_old", &layout.marker_old),
            ],
        );

        write_and_flush(
            recorder,
            case_name,
            "marker_part",
            &layout.marker_part,
            b"marker-v2",
        );
        replace_file(
            recorder,
            case_name,
            &layout.marker,
            &layout.marker_part,
            &layout.marker_old,
        );
        flush_file(
            recorder,
            case_name,
            "marker",
            &layout.marker,
            GENERIC_READ | GENERIC_WRITE,
            ALL_SHARES,
        );
        flush_directory(recorder, case_name, "root", &layout.root);
        flush_directory(
            recorder,
            case_name,
            "backup_restore",
            &layout.backup_restore,
        );
        recorder.inventory(
            case_name,
            &[
                ("marker", &layout.marker),
                ("marker_part", &layout.marker_part),
                ("marker_old_preserved", &layout.marker_old),
            ],
        );

        write_and_flush(
            recorder,
            case_name,
            "stale_marker_part",
            &layout.marker_part,
            b"stale",
        );
        write_and_flush(
            recorder,
            case_name,
            "retried_marker_part",
            &layout.marker_part,
            b"marker-v3",
        );
        let old_retry = layout.backup_restore.join("restore.marker.retry-old");
        replace_file(
            recorder,
            case_name,
            &layout.marker,
            &layout.marker_part,
            &old_retry,
        );
        flush_directory(recorder, case_name, "root", &layout.root);
        flush_directory(
            recorder,
            case_name,
            "backup_restore",
            &layout.backup_restore,
        );
        recorder.inventory(
            case_name,
            &[
                ("marker", &layout.marker),
                ("stale_part_removed_by_replace", &layout.marker_part),
                ("retry_old_preserved", &old_retry),
            ],
        );

        delete_file(recorder, case_name, "marker", &layout.marker, true);
        flush_directory(recorder, case_name, "root", &layout.root);
        recorder.inventory(
            case_name,
            &[
                ("marker_removed", &layout.marker),
                ("old_marker", &layout.marker_old),
                ("retry_old_marker", &old_retry),
            ],
        );
    }

    fn rename_case(recorder: &mut Recorder, layout: &Layout) {
        let case_name = "no_replace_cross_directory_and_barriers";
        write_and_flush(
            recorder,
            case_name,
            "marker",
            &layout.marker,
            b"transition-in-progress",
        );
        write_and_flush(
            recorder,
            case_name,
            "canonical",
            &layout.canonical,
            b"canonical-v1",
        );
        write_and_flush(
            recorder,
            case_name,
            "rollback_existing",
            &layout.rollback,
            b"rollback-existing",
        );
        rename_no_replace(
            recorder,
            case_name,
            "canonical_to_rollback_destination_exists",
            &layout.canonical,
            &layout.rollback,
            Expectation::DestinationExistsFailure,
        );
        recorder.inventory(
            case_name,
            &[
                ("canonical_retained", &layout.canonical),
                ("rollback_retained", &layout.rollback),
                ("protective_retained", &layout.protective),
                ("marker_retained", &layout.marker),
                ("temp_retained", &layout.temp),
            ],
        );
        delete_file(
            recorder,
            case_name,
            "rollback_existing",
            &layout.rollback,
            true,
        );
        rename_no_replace(
            recorder,
            case_name,
            "canonical_to_rollback",
            &layout.canonical,
            &layout.rollback,
            Expectation::Success,
        );
        flush_directory(recorder, case_name, "root", &layout.root);
        flush_directory(
            recorder,
            case_name,
            "backup_restore",
            &layout.backup_restore,
        );
        recorder.inventory(
            case_name,
            &[
                ("canonical_moved", &layout.canonical),
                ("rollback", &layout.rollback),
            ],
        );

        write_and_flush(
            recorder,
            case_name,
            "stage",
            &layout.stage,
            b"stage-install",
        );
        rename_no_replace(
            recorder,
            case_name,
            "stage_to_canonical_cross_directory",
            &layout.stage,
            &layout.canonical,
            Expectation::Success,
        );
        let staging_barrier = flush_directory(recorder, case_name, "staging", &layout.staging);
        let root_barrier = flush_directory(recorder, case_name, "root", &layout.root);
        if !(staging_barrier && root_barrier) {
            recorder.unproven(
                case_name,
                "both_cross_directory_barriers",
                "staging_and_root",
                None,
                0,
                "\"reason\":\"both source and destination directory barriers are required\",\"access\":0,\"share\":0,\"flags\":0",
            );
        }
        recorder.inventory(
            case_name,
            &[
                ("canonical_installed", &layout.canonical),
                ("stage_moved", &layout.stage),
                ("rollback_retained", &layout.rollback),
                ("protective_retained", &layout.protective),
                ("temp_retained", &layout.temp),
            ],
        );
    }

    fn sharing_case(recorder: &mut Recorder, layout: &Layout) {
        let case_name = "sharing_and_explicit_close_order";
        delete_file(
            recorder,
            case_name,
            "rollback_cleanup",
            &layout.rollback,
            false,
        );
        let incompatible = open_handle(
            recorder,
            case_name,
            "canonical_incompatible_live",
            &layout.canonical,
            GENERIC_READ,
            FILE_SHARE_READ,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
        );
        if incompatible != INVALID_HANDLE_VALUE {
            rename_no_replace(
                recorder,
                case_name,
                "rename_blocked_by_incompatible_live_handle",
                &layout.canonical,
                &layout.rollback,
                Expectation::SharingFailure,
            );
            recorder.inventory(
                case_name,
                &[
                    ("canonical_retained", &layout.canonical),
                    ("rollback_absent", &layout.rollback),
                    ("protective_retained", &layout.protective),
                    ("marker_retained", &layout.marker),
                    ("temp_retained", &layout.temp),
                ],
            );
            close_handle(
                recorder,
                case_name,
                "canonical_incompatible_live",
                &layout.canonical,
                incompatible,
            );
            rename_no_replace(
                recorder,
                case_name,
                "rename_after_incompatible_handle_close",
                &layout.canonical,
                &layout.rollback,
                Expectation::Success,
            );
        }
        recorder.inventory(
            case_name,
            &[
                ("canonical_moved", &layout.canonical),
                ("rollback", &layout.rollback),
            ],
        );

        let compatible = open_handle(
            recorder,
            case_name,
            "rollback_share_read_write_delete",
            &layout.rollback,
            GENERIC_READ | GENERIC_WRITE | DELETE_ACCESS,
            ALL_SHARES,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
        );
        if compatible != INVALID_HANDLE_VALUE {
            rename_no_replace(
                recorder,
                case_name,
                "compatible_handle_does_not_self_block",
                &layout.rollback,
                &layout.canonical,
                Expectation::Success,
            );
            close_handle(
                recorder,
                case_name,
                "renamed_compatible_handle",
                &layout.canonical,
                compatible,
            );
        }
        flush_directory(recorder, case_name, "root", &layout.root);
        flush_directory(
            recorder,
            case_name,
            "backup_restore",
            &layout.backup_restore,
        );
        recorder.inventory(
            case_name,
            &[
                ("canonical", &layout.canonical),
                ("rollback", &layout.rollback),
                ("protective", &layout.protective),
                ("marker", &layout.marker),
                ("temp", &layout.temp),
            ],
        );
    }

    fn write_and_flush(
        recorder: &mut Recorder,
        case_name: &str,
        role: &str,
        path: &Path,
        bytes: &[u8],
    ) -> bool {
        let access = GENERIC_READ | GENERIC_WRITE;
        let handle = open_handle(
            recorder,
            case_name,
            role,
            path,
            access,
            ALL_SHARES,
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
        );
        if handle == INVALID_HANDLE_VALUE {
            return false;
        }
        let mut written = 0_u32;
        // SAFETY: handle is live; byte buffer and output integer remain valid for the call.
        let write_ok = unsafe {
            WriteFile(
                handle,
                bytes.as_ptr(),
                bytes.len() as u32,
                &mut written,
                std::ptr::null_mut(),
            )
        } != 0;
        let write_error = if write_ok { 0 } else { last_error() };
        recorder.event(
            case_name,
            if write_ok && written as usize == bytes.len() { "PASS" } else { "FAIL" },
            "WriteFile",
            role,
            Some(path),
            Some(write_ok),
            write_error,
            &format!(
                "\"requested_bytes\":{},\"written_bytes\":{},\"access\":{},\"share\":{},\"flags\":{}",
                bytes.len(), written, access, ALL_SHARES, FILE_ATTRIBUTE_NORMAL
            ),
        );
        if !write_ok || written as usize != bytes.len() {
            recorder.failures += 1;
        }
        let flush_ok = flush_open_handle(
            recorder,
            case_name,
            role,
            path,
            handle,
            access,
            ALL_SHARES,
            FILE_ATTRIBUTE_NORMAL,
        );
        close_handle(recorder, case_name, role, path, handle);
        write_ok && written as usize == bytes.len() && flush_ok
    }

    fn flush_file(
        recorder: &mut Recorder,
        case_name: &str,
        role: &str,
        path: &Path,
        access: u32,
        share: u32,
    ) -> bool {
        let handle = open_handle(
            recorder,
            case_name,
            role,
            path,
            access,
            share,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
        );
        if handle == INVALID_HANDLE_VALUE {
            return false;
        }
        let ok = flush_open_handle(
            recorder,
            case_name,
            role,
            path,
            handle,
            access,
            share,
            FILE_ATTRIBUTE_NORMAL,
        );
        close_handle(recorder, case_name, role, path, handle);
        ok
    }

    fn flush_directory(recorder: &mut Recorder, case_name: &str, role: &str, path: &Path) -> bool {
        let flags = FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT;
        let handle = open_handle(
            recorder,
            case_name,
            role,
            path,
            GENERIC_READ,
            ALL_SHARES,
            OPEN_EXISTING,
            flags,
        );
        if handle == INVALID_HANDLE_VALUE {
            return false;
        }
        let ok = flush_open_handle(
            recorder,
            case_name,
            role,
            path,
            handle,
            GENERIC_READ,
            ALL_SHARES,
            flags,
        );
        close_handle(recorder, case_name, role, path, handle);
        ok
    }

    fn flush_open_handle(
        recorder: &mut Recorder,
        case_name: &str,
        role: &str,
        path: &Path,
        handle: HANDLE,
        access: u32,
        share: u32,
        flags: u32,
    ) -> bool {
        // SAFETY: handle is live and is closed by the caller after this call.
        let ok = unsafe { FlushFileBuffers(handle) } != 0;
        let error = if ok { 0 } else { last_error() };
        recorder.event(
            case_name,
            if ok { "PASS" } else { "FAIL" },
            "FlushFileBuffers",
            role,
            Some(path),
            Some(ok),
            error,
            &format!(
                "\"access\":{},\"share\":{},\"flags\":{}",
                access, share, flags
            ),
        );
        if !ok {
            recorder.failures += 1;
        }
        ok
    }

    fn open_handle(
        recorder: &mut Recorder,
        case_name: &str,
        role: &str,
        path: &Path,
        access: u32,
        share: u32,
        disposition: u32,
        flags: u32,
    ) -> HANDLE {
        let path_wide = wide(path);
        // SAFETY: path is a live NUL-terminated buffer and optional pointers are null.
        let handle = unsafe {
            CreateFileW(
                path_wide.as_ptr(),
                access,
                share,
                std::ptr::null(),
                disposition,
                flags,
                std::ptr::null_mut(),
            )
        };
        let ok = handle != INVALID_HANDLE_VALUE;
        let error = if ok { 0 } else { last_error() };
        recorder.event(
            case_name,
            if ok { "PASS" } else { "FAIL" },
            "CreateFileW",
            role,
            Some(path),
            Some(ok),
            error,
            &format!(
                "\"access\":{},\"share\":{},\"creation_disposition\":{},\"flags\":{}",
                access, share, disposition, flags
            ),
        );
        if !ok {
            recorder.failures += 1;
        }
        handle
    }

    fn open_handle_expect_sharing_failure(
        recorder: &mut Recorder,
        case_name: &str,
        role: &str,
        path: &Path,
        access: u32,
        share: u32,
    ) -> HANDLE {
        let path_wide = wide(path);
        // SAFETY: path is a live NUL-terminated buffer and optional pointers are null.
        let handle = unsafe {
            CreateFileW(
                path_wide.as_ptr(),
                access,
                share,
                std::ptr::null(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                std::ptr::null_mut(),
            )
        };
        let blocked = handle == INVALID_HANDLE_VALUE;
        let error = if blocked { last_error() } else { 0 };
        recorder.event(
            case_name,
            if blocked { "PASS" } else { "FAIL" },
            "CreateFileW_expected_sharing_failure",
            role,
            Some(path),
            Some(!blocked),
            error,
            &format!(
                "\"expected\":\"sharing failure while incompatible handle is live\",\"access\":{},\"share\":{},\"creation_disposition\":{},\"flags\":{}",
                access, share, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL
            ),
        );
        if !blocked {
            recorder.failures += 1;
        }
        handle
    }

    fn close_handle(
        recorder: &mut Recorder,
        case_name: &str,
        role: &str,
        path: &Path,
        handle: HANDLE,
    ) {
        // SAFETY: caller transfers one live handle and never uses it after this call.
        let ok = unsafe { CloseHandle(handle) } != 0;
        let error = if ok { 0 } else { last_error() };
        recorder.event(
            case_name,
            if ok { "PASS" } else { "FAIL" },
            "CloseHandle",
            role,
            Some(path),
            Some(ok),
            error,
            "\"ordering\":\"explicit_close_before_next_transition\",\"access\":0,\"share\":0,\"flags\":0",
        );
        if !ok {
            recorder.failures += 1;
        }
    }

    enum Expectation {
        Success,
        DestinationExistsFailure,
        SharingFailure,
    }

    fn rename_no_replace(
        recorder: &mut Recorder,
        case_name: &str,
        role: &str,
        source: &Path,
        destination: &Path,
        expectation: Expectation,
    ) -> bool {
        let access = DELETE_ACCESS | GENERIC_READ;
        let handle = if matches!(expectation, Expectation::SharingFailure) {
            open_handle_expect_sharing_failure(
                recorder, case_name, role, source, access, ALL_SHARES,
            )
        } else {
            open_handle(
                recorder,
                case_name,
                role,
                source,
                access,
                ALL_SHARES,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return matches!(expectation, Expectation::SharingFailure);
        }

        let destination_wide: Vec<u16> = destination.as_os_str().encode_wide().collect();
        let pointer_align = std::mem::align_of::<HANDLE>();
        let handle_offset = align_up(std::mem::size_of::<u32>(), pointer_align);
        let length_offset = handle_offset + std::mem::size_of::<HANDLE>();
        let filename_offset = length_offset + std::mem::size_of::<u32>();
        let byte_len = filename_offset + destination_wide.len() * std::mem::size_of::<u16>();
        let words = byte_len.div_ceil(std::mem::size_of::<usize>());
        let mut buffer = vec![0_usize; words];
        let base = buffer.as_mut_ptr().cast::<u8>();
        // SAFETY: offsets follow the C FILE_RENAME_INFO layout; the usize buffer supplies pointer
        // alignment and has enough initialized storage for the header and UTF-16 filename.
        unsafe {
            base.cast::<u32>().write(FILE_RENAME_FLAG_WRITE_THROUGH);
            base.add(handle_offset)
                .cast::<HANDLE>()
                .write(std::ptr::null_mut());
            base.add(length_offset)
                .cast::<u32>()
                .write((destination_wide.len() * 2) as u32);
            std::ptr::copy_nonoverlapping(
                destination_wide.as_ptr().cast::<u8>(),
                base.add(filename_offset),
                destination_wide.len() * 2,
            );
        }
        // SAFETY: handle and FILE_RENAME_INFO_EX-compatible buffer remain live for the call.
        let call_ok = unsafe {
            SetFileInformationByHandle(
                handle,
                FILE_RENAME_INFO_EX_CLASS,
                base.cast::<c_void>(),
                byte_len as u32,
            )
        } != 0;
        let error = if call_ok { 0 } else { last_error() };
        let expected = match expectation {
            Expectation::Success => call_ok,
            Expectation::DestinationExistsFailure => {
                !call_ok && (error == ERROR_ALREADY_EXISTS || error == ERROR_FILE_EXISTS)
            }
            Expectation::SharingFailure => !call_ok && error != 0,
        };
        let expectation_name = match expectation {
            Expectation::Success => "success",
            Expectation::DestinationExistsFailure => "destination_exists_failure",
            Expectation::SharingFailure => "sharing_failure",
        };
        recorder.event(
            case_name,
            if expected { "PASS" } else { "UNPROVEN" },
            "SetFileInformationByHandle_FileRenameInfoEx",
            role,
            Some(source),
            Some(call_ok),
            error,
            &format!(
                "\"destination\":\"{}\",\"expected\":\"{}\",\"no_replace\":true,\"access\":{},\"share\":{},\"flags\":{},\"information_class\":{}",
                json_escape(&recorder.label(destination)),
                expectation_name,
                access,
                ALL_SHARES,
                FILE_RENAME_FLAG_WRITE_THROUGH,
                FILE_RENAME_INFO_EX_CLASS
            ),
        );
        if !expected {
            recorder.failures += 1;
        }
        close_handle(
            recorder,
            case_name,
            role,
            if call_ok { destination } else { source },
            handle,
        );
        expected
    }

    fn replace_file(
        recorder: &mut Recorder,
        case_name: &str,
        replaced: &Path,
        replacement: &Path,
        preserved_old: &Path,
    ) -> bool {
        let replaced_wide = wide(replaced);
        let replacement_wide = wide(replacement);
        let backup_wide = wide(preserved_old);
        // SAFETY: all three paths are live NUL-terminated UTF-16 buffers.
        let ok = unsafe {
            ReplaceFileW(
                replaced_wide.as_ptr(),
                replacement_wide.as_ptr(),
                backup_wide.as_ptr(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        } != 0;
        let error = if ok { 0 } else { last_error() };
        recorder.event(
            case_name,
            if ok { "PASS" } else { "FAIL" },
            "ReplaceFileW",
            "marker",
            Some(replaced),
            Some(ok),
            error,
            &format!(
                "\"replacement\":\"{}\",\"preserved_old\":\"{}\",\"access\":0,\"share\":0,\"flags\":0,\"replacefile_write_through\":\"documented_unsupported_not_used_not_durability_evidence\"",
                json_escape(&recorder.label(replacement)),
                json_escape(&recorder.label(preserved_old))
            ),
        );
        if !ok {
            recorder.failures += 1;
        }
        ok
    }

    fn delete_file(
        recorder: &mut Recorder,
        case_name: &str,
        role: &str,
        path: &Path,
        required_present: bool,
    ) -> bool {
        let path_wide = wide(path);
        // SAFETY: path is a live NUL-terminated UTF-16 buffer.
        let ok = unsafe { DeleteFileW(path_wide.as_ptr()) } != 0;
        let error = if ok { 0 } else { last_error() };
        let accepted = ok || !required_present;
        recorder.event(
            case_name,
            if accepted { "PASS" } else { "FAIL" },
            "DeleteFileW",
            role,
            Some(path),
            Some(ok),
            error,
            &format!(
                "\"required_present\":{},\"access\":{},\"share\":{},\"flags\":0",
                required_present, DELETE_ACCESS, ALL_SHARES
            ),
        );
        if !accepted {
            recorder.failures += 1;
        }
        accepted
    }

    fn finish(recorder: &mut Recorder) -> i32 {
        let passed = recorder.failures == 0;
        recorder.event(
            "summary",
            if passed { "PASS" } else { "FAIL" },
            "required_w0_cases",
            "harness",
            None,
            Some(passed),
            0,
            &format!(
                "\"failure_count\":{},\"claim\":\"external feasibility evidence only; not production W2 and not a crash-durability proof\",\"access\":0,\"share\":0,\"flags\":0",
                recorder.failures
            ),
        );
        if passed {
            0
        } else {
            1
        }
    }

    fn parse_root() -> Option<PathBuf> {
        let mut args = std::env::args_os().skip(1);
        if args.next().as_deref() != Some(OsStr::new("--root")) {
            return None;
        }
        let root = args.next().map(PathBuf::from)?;
        if args.next().is_some() {
            return None;
        }
        Some(root)
    }

    fn wide(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain(Some(0)).collect()
    }

    fn truncate_at_nul(value: &mut Vec<u16>) {
        if let Some(index) = value.iter().position(|unit| *unit == 0) {
            value.truncate(index);
        }
    }

    fn utf16_to_string(value: &[u16]) -> String {
        let end = value
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(value.len());
        String::from_utf16_lossy(&value[..end])
    }

    fn align_up(value: usize, alignment: usize) -> usize {
        (value + alignment - 1) & !(alignment - 1)
    }

    fn last_error() -> u32 {
        // SAFETY: GetLastError has no preconditions and is read immediately after the failing call.
        unsafe { GetLastError() }
    }

    fn raw_error(error: &std::io::Error) -> u32 {
        error.raw_os_error().unwrap_or_default() as u32
    }

    fn json_escape(value: &str) -> String {
        let mut escaped = String::with_capacity(value.len());
        for character in value.chars() {
            match character {
                '"' => escaped.push_str("\\\""),
                '\\' => escaped.push_str("\\\\"),
                '\n' => escaped.push_str("\\n"),
                '\r' => escaped.push_str("\\r"),
                '\t' => escaped.push_str("\\t"),
                character if character.is_control() => {
                    escaped.push_str(&format!("\\u{:04x}", character as u32));
                }
                character => escaped.push(character),
            }
        }
        escaped
    }
}

#[cfg(windows)]
fn main() {
    std::process::exit(windows_harness::entry());
}
