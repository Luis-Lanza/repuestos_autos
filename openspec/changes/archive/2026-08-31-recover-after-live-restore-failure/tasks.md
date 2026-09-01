# Implementation Tasks: Recover After Live Restore Failure

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 280–360 changed production, test, and change-support lines |
| 400-line budget risk | Medium |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Medium

**Forecast assumptions:** only `src-tauri/src/lib.rs` and `src-tauri/tests/backup_restore.rs` require edits; `src-tauri/src/commands/backup.rs` is verification-only; no schema, migration, frontend, IPC, `BackupStore`, generated-artifact, or documentation changes are included. If implementation evidence raises the forecast above 400 changed lines, stop and request a new delivery decision before continuing.

## Scope and boundaries

This task set implements only post-disruption in-process recovery for live restore failure. Preserve marker meanings, durable transition ordering, backup formats and destinations, WAL behavior, recovery-file retention, startup source ordering, and the ticket 02 boundary. Do not add a public fault-injection interface, a new recovery control, or a fresh-database fallback while recoverable evidence exists.

The implementation route is one dependency-ordered change set with tests beside the behavior. The existing command gate in `DatabaseState::with_read` and `DatabaseState::with_write` remains the mechanism that rejects `Restoring` and `Unavailable` access.

## Work Unit 1 — RED: Add deterministic post-disruption integration coverage

**Start:** Existing live-restore coverage in `src-tauri/tests/backup_restore.rs` passes, and the current `DatabaseState::install_validated_stage` API is available.

**Tasks:**

- [x] Add `live_restore_failure_recovers_valid_rollback_and_returns_restore_failed` in `src-tauri/tests/backup_restore.rs`. Build a current-schema canonical database containing prior data, create a malformed staged file under the existing staging location, invoke `install_validated_stage`, and assert the bounded `restore_failed` result, `Ready`-observable prior data through `with_read`, absence of the rejected candidate, and normal marker cleanup. <!-- sdd-owner: implementation -->
- [x] Add `live_restore_failure_enters_unavailable_when_recovery_cannot_establish_canonical` in `src-tauri/tests/backup_restore.rs`. Reuse the malformed-stage fixture, create the deterministic `restore-recovery.sqlite3.part` directory before the restore, assert the bounded `database_unavailable` result, and assert both `with_read` and `with_write` return `database_unavailable` without invoking their closures. <!-- sdd-owner: implementation -->
- [x] Extend the focused backup command boundary assertions in `src-tauri/tests/backup_restore.rs` to cover the stable `restore_failed` and `database_unavailable` response codes/messages and to reject filesystem paths, SQLite diagnostics, marker details, and source internals from serialized failures; do not edit `src-tauri/src/commands/backup.rs` for this test coverage. <!-- sdd-owner: implementation -->

**Verification:** The two new behavior tests should fail by assertion against the current stuck-`Restoring` implementation, while the boundary test identifies the existing safe mapping that must remain intact.

```sh
cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore live_restore_failure_recovers_valid_rollback_and_returns_restore_failed
cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore live_restore_failure_enters_unavailable_when_recovery_cannot_establish_canonical
```

**Finish:** Failing tests reproduce both required terminal outcomes without adding a production test hook or changing `BackupStore`.

**Rollback boundary:** Revert only the RED additions in `src-tauri/tests/backup_restore.rs`; no product or persistence behavior is changed by this unit.

## Work Unit 2 — GREEN: Capture settlement and share validated recovery

**Start:** Work Unit 1 contains deterministic failing coverage, and the existing startup recovery tests document canonical-first, rollback-second, and protective-third behavior.

**Tasks:**

- [x] In `src-tauri/src/lib.rs`, extract the private validated-recovery seam behind `open_validated_recovery_database(&DatabaseConfig, &BackupStore) -> Result<rusqlite::Connection, ()>`, preserving read-only validation of canonical first, then rollback and protective candidates, fallback copy through `restore_canonical_from`, writable `open_database`, and final `validate_restored_database`; keep marker cleanup outside this helper. <!-- sdd-owner: implementation -->
- [x] In `src-tauri/src/lib.rs`, make `recover_on_startup` call the shared helper for marked startup recovery while retaining its existing requirement that marker cleanup succeeds before constructing `Ready`; preserve `Unavailable` construction and all existing marker/source semantics on any startup failure. <!-- sdd-owner: implementation -->
- [x] In `DatabaseState::install_validated_stage` in `src-tauri/src/lib.rs`, leave preparation and pre-disruption work unchanged, then capture every post-`connection.take()` replacement/open/validation error instead of returning through `?`, so the mutex remains held until a terminal state is published. <!-- sdd-owner: implementation -->
- [x] In `src-tauri/src/lib.rs`, settle a validated installed candidate by assigning the newly opened connection and `Ready` before fallible marker cleanup; return `restore_failed` if cleanup fails while retaining that usable validated connection and marker. <!-- sdd-owner: implementation -->
- [x] In `src-tauri/src/lib.rs`, settle a failed post-disruption replacement by invoking the shared validated-recovery seam while the state mutex is held, publishing `Ready` only with its newly opened and validated connection and returning the original `restore_failed`, or publishing `Unavailable` with `connection = None` and returning `database_unavailable` when recovery fails. <!-- sdd-owner: implementation -->
- [x] In `src-tauri/src/lib.rs`, preserve the public `DatabaseState` method shapes, the `with_read`/`with_write` status gate, non-displayable internal recovery errors, and the existing no-path/no-SQLite-diagnostic command error contract; verify `src-tauri/src/commands/backup.rs` still maps only exact `database_unavailable` distinctly and all other installation failures to `restore_failed`, editing it only if source evidence proves the mapping changed. <!-- sdd-owner: implementation -->

**Verification:**

```sh
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore live_restore_failure_recovers_valid_rollback_and_returns_restore_failed
cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore live_restore_failure_enters_unavailable_when_recovery_cannot_establish_canonical
```

**Finish:** Work Unit 1 passes; every ordinary post-drop `Result` exit reaches `Ready` with a newly validated connection or `Unavailable` with no connection, without changing durable restore protocol behavior.

**Rollback boundary:** Revert the `DatabaseState` settlement/refactor in `src-tauri/src/lib.rs` together with the RED tests; no migration, database rewrite, marker-format conversion, or `BackupStore` protocol rollback is required.

## Work Unit 3 — TRIANGULATE: Prove ordering, cleanup, gating, and bounded outcomes

**Start:** GREEN behavior passes the two deterministic live-failure tests, and the shared helper is in place.

**Tasks:**

- [x] Extend `src-tauri/tests/backup_restore.rs` only where needed to exercise the shared helper through real temporary SQLite files: retain and strengthen canonical-first startup recovery, rollback fallback, protective fallback, invalid-candidate skipping, and restart-compatible marker assertions using the existing startup fixtures rather than introducing a second recovery policy. <!-- sdd-owner: implementation -->
- [x] Add coverage in `src-tauri/tests/backup_restore.rs` for the recovered service path to prove a subsequent normal read/write uses recovered prior data and cannot observe the malformed restore candidate, while the restore operation remains `restore_failed`; keep assertions at the public `DatabaseState` and command seams. <!-- sdd-owner: implementation -->
- [x] If a portable filesystem fixture cannot fail only the final `clear_restore_state` call, add the smallest private cleanup callback seam local to `src-tauri/src/lib.rs` and a colocated unit test that injects only that cleanup failure, then assert `Ready`, validated data usability, retained marker, and `restore_failed`; do not expose the seam through `DatabaseState` or `BackupStore`. <!-- sdd-owner: implementation -->
- [x] Verify the unavailable fixture leaves no active canonical connection and that subsequent `with_read` and `with_write` operations return exactly `database_unavailable` without executing database closures; verify no path capable of creating a fresh empty canonical database runs before valid evidence is selected. <!-- sdd-owner: implementation -->
- [x] Verify `src-tauri/src/commands/backup.rs` without behavioral edits: `confirm_restore` must preserve `restore_failed` after successful in-process service recovery and map only the exact `database_unavailable` internal result to the unavailable response; serialized messages must remain fixed and bounded. <!-- sdd-owner: implementation -->

**Focused verification:**

```sh
cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore startup_recovery
cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore live_restore
cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore backup_command_boundary_serializes_only_allowlisted_stable_and_safe_outcomes
```

**Finish:** Integration and colocated tests cover all recovery-source ordering outcomes, final-cleanup failure, terminal-state gating, truthful restore results, bounded disclosure, and unchanged startup behavior.

**Rollback boundary:** Revert only the triangulation tests and any strictly private `lib.rs` cleanup-test seam if implementation fails to pass; retain the minimal GREEN settlement logic for separate diagnosis.

## Work Unit 4 — REFACTOR: Locality, lint cleanliness, and scope audit

**Start:** All RED and TRIANGULATE tests pass against the GREEN implementation.

**Tasks:**

- [x] Refactor `src-tauri/src/lib.rs` only for locality and readability: keep source selection, connection opening, validation, state publication, and marker cleanup behind small honest private functions without flags or a new filesystem abstraction, and keep the mutex ownership/settlement order obvious. <!-- sdd-owner: implementation -->
- [x] Refactor `src-tauri/tests/backup_restore.rs` fixtures and test names for descriptive, DAMP stories while keeping setup reusable and each recovery action/assertion tied to a concrete scenario; do not weaken assertions or replace real SQLite/file mechanics with sleeps, races, permission assumptions, or mocks. <!-- sdd-owner: implementation -->
- [x] Run formatting, the focused integration test, the full Rust test suite, and the repository Rust lint command; fix only regressions introduced by this change. <!-- sdd-owner: implementation -->
- [x] Audit the final diff against `openspec/changes/recover-after-live-restore-failure/specs/backup-restore/spec.md` and `design.md`, confirming no edits to ticket 02 concerns, schema/migrations, frontend/IPC, backup formats/destinations, marker meanings, WAL handling, recovery retention, or user-interface controls. <!-- sdd-owner: implementation -->

**Focused verification:**

```sh
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore
```

**Full verification:**

```sh
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings
```

**Finish:** The single-PR change is formatted, tested, lint-clean, within the 400 changed-line budget, and limited to `src-tauri/src/lib.rs` plus `src-tauri/tests/backup_restore.rs` unless explicit evidence justifies otherwise.

**Rollback boundary:** Revert the complete implementation and its focused tests together; stored databases, backups, markers, and recovery files require no conversion because this change alters only in-process orchestration.

## Apply decision gate

The preflight forecast is below the 400-line budget, so `ask-on-risk` currently permits apply without an additional decision. If implementation or triangulation discovers a required source/support change that raises the forecast above 400 changed lines, pause before that work unit continues and request an explicit delivery decision; do not silently chain or expand into ticket 02.
