# Apply Progress: Recover After Live Restore Failure

## Status

- Phase: apply
- Result: implementation complete
- Task progress: 18/18 implementation-owned tasks checked in `tasks.md`
- Next recommended: `sdd-verify`
- Artifact store: hybrid/both
- Action context: `repo-local`; workspace root and allowed edit root were both `/home/luis/velay/repuestos_autos`; no action-context warnings.

## Completed Work

- Added deterministic malformed-stage coverage for recoverable rollback restoration and explicit unavailable settlement.
- Captured every ordinary post-disruption replacement failure while retaining the database mutex through terminal state publication.
- Extracted `open_validated_recovery_database` so startup and live recovery share canonical-first, rollback-second, protective-third validation and establishment.
- Preserved truthful bounded results: recovered service returns `restore_failed`; failed service recovery returns `database_unavailable`.
- Added a private cleanup-only test seam proving a validated candidate remains usable and marked when final marker cleanup fails.
- Verified stable command response codes/messages without changing `src-tauri/src/commands/backup.rs`.

## Persisted Task Evidence

All 18 implementation-owned rows in `openspec/changes/recover-after-live-restore-failure/tasks.md` were changed from `- [ ]` to `- [x]`. No unchecked implementation task remains.

## Files Changed

- `src-tauri/src/lib.rs`
- `src-tauri/tests/backup_restore.rs`
- `openspec/changes/recover-after-live-restore-failure/tasks.md`
- `openspec/changes/recover-after-live-restore-failure/apply-progress.md`

Implementation source/test diff: 225 additions and 22 deletions, 247 changed lines, below the 400-line budget. PR boundary: one bounded single-PR work unit; no chain and no size exception.

## TDD Cycle Evidence

| Cycle | RED | GREEN | TRIANGULATE | REFACTOR |
|---|---|---|---|---|
| Recoverable post-disruption failure | New focused test failed because subsequent access returned `database_unavailable`. | Shared recovery restored rollback and returned `restore_failed`; focused test passed. | Added post-recovery write/read and marker-cleanup assertions; startup ordering suite passed. | Centralized replacement settlement and shared recovery helper; formatting passed. |
| Unrecoverable post-disruption failure | New focused test expected `database_unavailable` but current code returned `restore_failed`. | Failed recovery now publishes `Unavailable` with no connection; focused test passed. | Read/write closures were proven uncalled and marker retention was asserted. | Kept existing status gate and bounded internal errors unchanged. |
| Final cleanup failure | Private cleanup-only test was added after GREEN to distinguish validated readiness from cleanup success. | Validated connection is published before cleanup and retained on cleanup error. | Test proved candidate data remains readable and marker remains `CandidateInstalled`. | Seam remains private to `DatabaseState`; no `BackupStore` or public interface change. |

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore live_restore_failure_recovers_valid_rollback_and_returns_restore_failed` — RED failed, then GREEN passed (1/1).
- `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore live_restore_failure_enters_unavailable_when_recovery_cannot_establish_canonical` — RED failed, then GREEN passed (1/1).
- `cargo test --manifest-path src-tauri/Cargo.toml --lib validated_candidate_remains_ready_when_final_marker_cleanup_fails` — passed (1/1).
- `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore startup_recovery` — passed (5/5).
- `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore live_restore` — passed (2/2).
- `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore backup_command_boundary_serializes_only_allowlisted_stable_and_safe_outcomes` — passed (1/1).
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — passed.
- `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore` — passed (26/26).
- `cargo test --manifest-path src-tauri/Cargo.toml` — passed, including 12 library tests, 26 backup/restore tests, all other integration tests, and doc tests.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings` — blocked before crate linting because system `glib-2.0`, `gio-2.0`, and `gobject-2.0` pkg-config libraries are unavailable.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings` — reached crate linting and reported three pre-existing warnings outside allowed edit surfaces: two `too_many_arguments` findings and one `large_enum_variant`; no changed file was identified.
- `git diff --check -- src-tauri/src/lib.rs src-tauri/tests/backup_restore.rs` — passed.

## Deviations and Residual Risks

No design deviation and no ticket 02 scope expansion occurred. Schema, migrations, frontend, IPC contracts, `BackupStore`, marker meanings, durable transition ordering, WAL handling, and recovery retention were untouched. Full all-features Clippy remains environment-blocked, while non-desktop Clippy remains blocked by unrelated baseline warnings. Existing residual filesystem race and crash-durability risks from the design remain unchanged.

## Remaining Tasks

None. There are no unchecked implementation-owned `- [ ]` lines.
