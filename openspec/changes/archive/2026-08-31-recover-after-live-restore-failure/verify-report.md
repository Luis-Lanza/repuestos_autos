```yaml
schema: gentle-ai.verify-result/v1
evidence_revision: sha256:4a27ae57d114849578427f86423de6605b1582d80d175926b076fee7bc202c98
verdict: pass_with_warnings
blockers: 0
critical_findings: 0
requirements: 8/8
scenarios: 18/18
test_command: cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore live_restore_failure_recovers_valid_rollback_and_returns_restore_failed && cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore live_restore_failure_enters_unavailable_when_recovery_cannot_establish_canonical && cargo test --manifest-path src-tauri/Cargo.toml --lib validated_candidate_remains_ready_when_final_marker_cleanup_fails && cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore startup_recovery && cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore live_restore && cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore backup_command_boundary_serializes_only_allowlisted_stable_and_safe_outcomes && cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore && cargo test --manifest-path src-tauri/Cargo.toml
test_exit_code: 0
test_output_hash: sha256:1c52aedf3d5e4e04ef9c180fbafbb62c96c881d4223dbed78dd1878bd31ddb8d
build_command: cargo fmt --manifest-path src-tauri/Cargo.toml -- --check && git diff --check -- src-tauri/src/lib.rs src-tauri/tests/backup_restore.rs
build_exit_code: 0
build_output_hash: sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```

# Verification Report: Recover After Live Restore Failure

## Result

**PASS WITH WARNINGS.** The one permitted corrective rerun found no implementation-critical failure. All 8 requirements and 18 scenarios are covered, all required focused and full tests pass, the two Windows cleanup-order defects are corrected, and there is no archive/sync blocker. The warnings are limited to unavailable Linux desktop system libraries, unrelated pre-existing Clippy findings, and the absence of an installed Windows Rust target.

## Structured Status and Action Context

- Change: `recover-after-live-restore-failure`.
- Authoritative status: 18/18 tasks complete; apply `all_done`; `dependencies.verify` ready; `nextRecommended: verify`.
- Artifact store: hybrid/both. Spec, tasks, and apply-progress were read from OpenSpec and their matching Engram observations.
- Action context: repo-local `/home/luis/velay/repuestos_autos`; implementation ownership and both changed target files are inside this workspace.
- Allowed verification edit surface: only this report. No implementation, test, task, design, or apply-progress file was altered during verification.
- Unchecked implementation task scan matching `^\s*- \[ \]`: **none**.
- Strict TDD: inactive (`openspec/config.yaml` sets `strict_tdd: false`). Apply-progress still contains a `TDD Cycle Evidence` table and all referenced test files exist.

## Corrective Gate Evidence

- `src-tauri/src/lib.rs:273-274` now executes `drop(state);` before `fs::remove_dir_all(directory)`, closing the validated candidate connection before temporary-directory deletion.
- `src-tauri/tests/backup_restore.rs:594-595` now executes `drop(state);` before temporary-directory deletion in the recovered-ready test.
- `git diff` contains exactly these two added `drop(state);` cleanup corrections. The unavailable test needs no equivalent drop because settlement explicitly owns no connection.
- Only `x86_64-unknown-linux-gnu` is installed, so Windows execution was unavailable. Ownership inspection proves the two ready-state SQLite handles are now closed before deletion, resolving the previously reported Windows lifetime defect.

## Requirement and Scenario Coverage

| Requirement | Status | Evidence |
|---|---|---|
| Recover After a Post-Disruption Restore Failure | PASS | Post-drop replacement errors are captured and settled at `src-tauri/src/lib.rs:141-180`. The recoverable test proves prior-data read/write service returns with `restore_failed`; the cleanup-only test proves validated service remains usable on final cleanup failure. |
| Validated Readiness | PASS | `open_validated_recovery_database` validates read-only evidence, opens canonical storage, and revalidates it before returning (`src-tauri/src/lib.rs:185-203`). `Ready` is published only with the returned connection (`:163-171`). |
| Explicit Unavailable Terminal State | PASS | Failed recovery writes `connection = None`, then `Unavailable`, then returns `database_unavailable` (`src-tauri/src/lib.rs:175-178`). The focused test proves read/write closures do not execute. |
| No Stale Connection Use During Recovery | PASS | The state mutex remains held through replacement and settlement (`src-tauri/src/lib.rs:118-181`); the old connection is taken and dropped at `:141-142`; read/write gates accept only `Ready` (`:80-99`). |
| Compatibility with Existing Recovery-Source Ordering | PASS | Canonical is checked first, followed by rollback and protective evidence (`src-tauri/src/lib.rs:189-203`). Five startup recovery tests pass, including canonical retention, rollback restoration, protective fallback, and no-safe-candidate behavior. No writable open occurs before valid existing evidence is selected. |
| Bounded and Truthful Restore Results | PASS | Recovered service publishes `Ready` but returns `restore_failed`; failed recovery returns `database_unavailable`. `confirm_restore` maps only exact `database_unavailable` distinctly (`src-tauri/src/commands/backup.rs:238-243`). |
| Safe Error Disclosure | PASS | Changed orchestration exposes fixed bounded codes only. The command-boundary test verifies fixed messages and rejects SQLite, path, marker, filename, and recovery-source detail. |
| Preserve Existing Restore and Ticket 02 Boundaries | PASS | The implementation diff is limited to `src-tauri/src/lib.rs` and `src-tauri/tests/backup_restore.rs`. No schema, migration, frontend, IPC shape, `BackupStore`, backup format/destination, marker meaning, WAL behavior, retention, UI control, or durable protocol redesign changed. |

All **18/18 specification scenarios** are covered by source inspection plus focused startup, live-recovery, cleanup-failure, unavailable-gating, command-boundary, successful-restore, and full regression tests.

## Test and Validation Commands

| Exact command | Outcome |
|---|---|
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | PASS, exit 0 |
| `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore live_restore_failure_recovers_valid_rollback_and_returns_restore_failed` | PASS, 1 passed |
| `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore live_restore_failure_enters_unavailable_when_recovery_cannot_establish_canonical` | PASS, 1 passed |
| `cargo test --manifest-path src-tauri/Cargo.toml --lib validated_candidate_remains_ready_when_final_marker_cleanup_fails` | PASS, 1 passed |
| `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore startup_recovery` | PASS, 5 passed |
| `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore live_restore` | PASS, 2 passed |
| `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore backup_command_boundary_serializes_only_allowlisted_stable_and_safe_outcomes` | PASS, 1 passed |
| `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore` | PASS, 26 passed |
| `cargo test --manifest-path src-tauri/Cargo.toml` | PASS, all 166 unit/integration tests passed; doc tests had 0 failures |
| `git diff --check -- src-tauri/src/lib.rs src-tauri/tests/backup_restore.rs` | PASS, exit 0 |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings` | Environment-limited, exit 101 before project linting because `glib-2.0`, `gobject-2.0`, `gio-2.0`, `gdk-3.0`, and `atk` pkg-config libraries are unavailable |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings` | Baseline-limited, exit 101 from three pre-existing findings outside changed files |

The non-desktop Clippy findings are `too_many_arguments` at `src-tauri/src/application/catalog/repository.rs:36`, `large_enum_variant` at `src-tauri/src/commands/sales_history.rs:30`, and `too_many_arguments` at `src-tauri/src/domain/inventory.rs:126`. `git show HEAD:<path>` confirms all three predate this change. These lint limitations are warnings, not candidate blockers.

## Assertion Quality and TDD

Strict TDD is disabled, so strict-TDD compliance is not a gate. The changed tests nevertheless make substantive assertions against real SQLite and filesystem behavior: exact bounded outcomes, recovered data reads and writes, non-observation of the rejected candidate, unavailable closure non-execution, marker state, and allowlisted serialization. No tautology, ghost loop, type-only assertion, smoke-only check, or implementation-detail CSS assertion was found. The bounded-disclosure loop checks every fixed forbidden token and is not a ghost loop.

## Review Workload and Scope Boundary

- Current implementation source/test diff: 227 additions and 22 deletions, **249 changed lines**, below the 400-line budget.
- The two-line increase from apply-progress is exactly the corrective `drop(state);` additions.
- Chained PRs recommended: No; implemented boundary remains one bounded PR/work unit.
- `size:exception`: not used and not required.
- Chain strategy: `pending`; no chain boundary applies or was violated.
- `src-tauri/src/commands/backup.rs` remained verification-only, as forecast.
- No scope creep into ticket 02 or unrelated repository areas was found.

## Blockers and Warnings

**Blockers: none. Critical findings: none.**

Non-blocking warnings:

1. A Windows target is not installed locally, so the corrected cleanup order was verified by Rust ownership/source inspection and Linux execution rather than Windows execution.
2. All-features Clippy cannot build without unavailable GTK/GLib-family system packages.
3. Non-desktop Clippy reaches project linting but fails on three unrelated findings already present at `HEAD`.
4. Existing filesystem validation/copy races and crash-durability limitations remain unchanged and stay within ticket 02.

## Sync Readiness

**Sync is ready.** All implementation tasks are checked, specification coverage is complete, corrective verification passed, and no archive/sync blocker remains.
