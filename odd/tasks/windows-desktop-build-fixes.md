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
- [x] W7 Add bounded Windows debug-only native backup diagnostics without guessing or changing production behavior; use the next Windows run to identify the failing operation before any corrective change.
- [x] W8 Restore release cleanup-evidence reconciliation behavior on metadata-inspection failure; retain the bounded evidence-recreation attempt and prove it with a regression test.
- [x] W9 Add minimal debug-only IPC boundary labels for picker selection and create-backup command entry/returned response; fresh APPDATA is empty, so distinguish backend routing from internal failure before widening branch diagnostics.
- [x] W10 Add targeted debug-only labels to the remaining high-level create-backup failure gates (prune, state, snapshot size/result, publication/cleanup outcome) to identify the exact branch without exposing data.

## W10 evidence
- Added fixed high-level `create_backup_*` outcome labels for expired and snapshot pruning, destination-token consumption, state/read/snapshot outcomes, size-limit rejection, publication failure, and final cleanup. Destination-token, internal-result, storage-error, and cleanup classes are explicitly bounded; the output contains no paths, tokens, sizes, database values, or request data.
- Labels compile only under `cfg(all(windows, debug_assertions))`; non-Windows and release behavior, including IPC responses, is unchanged. The W9 audit checks every gate label, debug-only scope, and bounded classifiers.
- Windows runtime execution is still required to identify which gate is reached in the reported environment.

## W5 evidence
- Windows snapshot cleanup opened directories with read-only access and then called `sync_all`; the store publication path used `File::open(...).sync_all()` directly. Both bypass the Windows directory-flush contract requiring a write-capable directory handle and `FlushFileBuffers`.
- Both Windows paths now open the directory with `GENERIC_WRITE`, backup-semantics and open-reparse-point flags, share access for readers/writers/deletion, and call `FlushFileBuffers`; failed or unaccounted cleanup continues to return bounded `storage_unavailable`.
- Linux local verification passed: `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore`, `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `git diff --check`.
- Windows evidence: the fixed-NTFS test passed in a 47/51 Windows suite run. W6 addresses the four observed failures without changing production behavior; rerun the full Windows suite with `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore`.
- Linux W6 verification: `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore` passed (42 tests), `cargo test --manifest-path src-tauri/Cargo.toml` passed, and `npm test` passed (392 tests) after the exact W9 hashes were updated. `git diff --check` passed.

## W7 diagnostic handoff
- Windows development diagnostics are emitted only under `cfg(all(windows, debug_assertions))` as `backup_diagnostic operation=<stable-label> os_code=<number-or-none> kind=<ErrorKind>`; release builds retain the existing IPC errors and produce no diagnostic output.
- Snapshot creation/metadata, publication, cleanup-evidence create/write/sync/remove, artifact removal, and directory sync boundaries report labels only and never paths, database values, license data, tokens, or user data. SQLite errors that do not expose a Windows OS error report `os_code=none kind=Other`.
- To identify the remaining failure, run `npm run tauri:dev` on Windows, create a backup through the app, and copy exactly one `backup_diagnostic ...` line from the Tauri terminal. Use that operation and code/kind as evidence before proposing a production fix.

## W9 evidence
- Added terminal-only picker selection outcome diagnostics (`cancelled`, `selected`, or `error` with an allowlisted code) and create-backup entry/final response diagnostics (`kind` plus allowlisted code), compiled to output only under `cfg(all(windows, debug_assertions))`.
- Diagnostics contain fixed operation/outcome/kind labels and bounded codes only; paths, tokens, requests, and response payload values are not formatted. IPC responses are unchanged.
- Rust unit coverage verifies response-kind mapping and unknown-code bounding. Windows runtime capture is still required to distinguish the observed routing/failure; no wider branch instrumentation was added.

## Acceptance
- `npm run tauri:dev` compiles on Windows.
- Picker IPC responses remain token-only/path-free and preserve bounded error codes.
- Windows recovery-stage provenance remains restricted to internal UUID staging files.
- No Linux backup/restore regression is introduced.
