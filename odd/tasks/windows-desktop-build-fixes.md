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
- [x] W5 Diagnose and correct Windows snapshot cleanup/directory durability so backup creation does not return `storage_unavailable` after a safe publication; the fixed-NTFS test passed on the Windows host.
- [x] W6 Align backup/restore tests with platform-safe destination and markerless fail-closed policy; close SQLite handles before fixture deletion.

## W5 evidence
- Windows snapshot cleanup opened directories with read-only access and then called `sync_all`; the store publication path used `File::open(...).sync_all()` directly. Both bypass the Windows directory-flush contract requiring a write-capable directory handle and `FlushFileBuffers`.
- Both Windows paths now open the directory with `GENERIC_WRITE`, backup-semantics and open-reparse-point flags, share access for readers/writers/deletion, and call `FlushFileBuffers`; failed or unaccounted cleanup continues to return bounded `storage_unavailable`.
- Linux local verification passed: `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore`, `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `git diff --check`.
- Windows evidence: the fixed-NTFS test passed in a 47/51 Windows suite run. W6 addresses the four observed failures without changing production behavior; rerun the full Windows suite with `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore`.
- Linux W6 verification: `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore` passed (42 tests), `cargo test --manifest-path src-tauri/Cargo.toml` passed, and `npm test` passed (392 tests) after the exact W9 hashes were updated. `git diff --check` passed.

## Acceptance
- `npm run tauri:dev` compiles on Windows.
- Picker IPC responses remain token-only/path-free and preserve bounded error codes.
- Windows recovery-stage provenance remains restricted to internal UUID staging files.
- No Linux backup/restore regression is introduced.
