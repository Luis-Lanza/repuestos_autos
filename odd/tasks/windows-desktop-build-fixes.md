# Windows desktop build fixes

## Goal
Restore Windows desktop compilation for the hardened backup/restore branch so runtime validation can begin.

## Evidence
Windows `npm run tauri:dev` on commit `d055e75` failed before launch:
- `DRIVE_FIXED` was imported from the wrong `windows-sys` module.
- `is_internal_recovery_migration_stage` was unavailable to Windows production verification.
- Async Tauri picker commands returned bare enums and captured borrowed command state, violating the required owned `Result` command contract.

## Tasks
- [x] W1 Correct Windows platform imports and recovery-stage verifier scope with focused tests.
- [x] W2 Make desktop picker commands return owned, bounded IPC `Result` responses; retain path-free contracts.
- [x] W3 Windows-native desktop compilation/runtime startup passed on the user host; it exposed a recovery-policy regression, not a compilation failure.
- [x] W4 Open a valid canonical database when only valid retained transition sidecars remain; preserve those artifacts and still fail closed for malformed/unexpected sidecars or a missing/invalid canonical.
- [ ] W5 Diagnose and correct Windows snapshot cleanup/directory durability so backup creation does not return `storage_unavailable` after a safe publication; implementation is locally verified, but fixed-NTFS Windows runtime confirmation remains pending.

## W5 evidence
- Windows snapshot cleanup opened directories with read-only access and then called `sync_all`; the store publication path used `File::open(...).sync_all()` directly. Both bypass the Windows directory-flush contract requiring a write-capable directory handle and `FlushFileBuffers`.
- Both Windows paths now open the directory with `GENERIC_WRITE`, backup-semantics and open-reparse-point flags, share access for readers/writers/deletion, and call `FlushFileBuffers`; failed or unaccounted cleanup continues to return bounded `storage_unavailable`.
- Linux local verification passed: `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore`, `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `git diff --check`.
- Pending Windows confirmation: run `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore fixed_ntfs_backup_syncs_publication_and_snapshot_cleanup_directories -- --exact --nocapture` on fixed NTFS and observe a `Created` response with both warnings false, the published file present, and no snapshot or `.cleanup-needed` entries.

## Acceptance
- `npm run tauri:dev` compiles on Windows.
- Picker IPC responses remain token-only/path-free and preserve bounded error codes.
- Windows recovery-stage provenance remains restricted to internal UUID staging files.
- No Linux backup/restore regression is introduced.
