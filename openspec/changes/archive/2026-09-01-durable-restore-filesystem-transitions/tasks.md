# Tasks: Durable Restore Filesystem Transitions

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | W0 external feasibility: 0–100 evidence-only; W1A: 220–320; W1B: 280–380; W2: 220–340; total implementation: 720–1,040 |
| 400-line budget risk | High for the whole change; Low/Medium for each proposed implementation slice |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 W1A → PR 2 W1B → PR 3 W2; W0 and final Windows acceptance remain external gates |
| Delivery strategy | ask-on-risk |
| Chain strategy | stacked-to-main |

Decision needed before apply: No — maintainer selected the stacked-to-main chain.
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

Every proposed implementation PR remains at or below the 400 changed-line review budget: W1A 220–320, W1B 280–380, and W2 220–340. The total forecast is high because the slices cover a new private protocol module, application integration, host tests, and the separately gated native adapter. W0 feasibility and real Windows acceptance produce evidence rather than product-code changes and are not counted as implementation PRs.

## Dependency order and scope guard

1. W1A may begin without a Windows dependency and delivers the host-independent protocol core, recorder, failpoints, and private unsupported adapter.
2. W1B depends on W1A and wires `BackupStore` and `DatabaseState`, keeping real host filesystem/SQLite tests beside that integration.
3. W0 is an external feasibility gate. It may run while W1A/W1B proceed, but its approved native contract is required before W2.
4. W2 depends on W0 approval and W1B's green host behavior. It contains only the proven Windows adapter, target configuration, and Windows-specific product tests.
5. Real Windows/local-NTFS evidence and maintainer acceptance remain external when they only execute tests, inspect evidence, or decide acceptance; they cannot be marked complete by Linux host tests.

Preserve the existing marker filename, exact JSON bytes, `Prepared`/`LiveMoved`/`CandidateInstalled` meanings, recovery filenames, canonical → rollback → protective order, SQLite validation, public errors, and `Ready`/`Restoring`/`Unavailable` ownership. Keep `commands/backup.rs`, schema/migrations, backup contents/destinations, WAL policy, UI/IPC, and unrelated filesystem publishing out of scope. Do not introduce a repository-wide filesystem abstraction or a reduced-guarantee fallback.

## W0 — real Windows/local-NTFS feasibility gate (external, pending)

These are maintainer/target-environment actions, not implementation-owned checkboxes. They must remain separate from W1/W2 code work and must not be satisfied by Linux tests, Wine, cross-compilation, API documentation, a recorder, or process-kill tests.

1. On a real supported Windows host with app-data on local NTFS, build and run a minimal repository-local feasibility spike for file flush, directory/namespace flush, marker create/replace/remove, no-replace rename, volume/filesystem detection, reparse/layout checks, and handle closure before namespace mutation.
2. Record the exact crate, version, features, native calls, access masks, directory-open flags, sharing modes, delete-on-close choice, replacement flags, error mapping, local-volume/NTFS checks, same-volume rule, and observed success/failure behavior.
3. Attach reproducible commands and raw evidence for compile/runtime results, clearly separating requested ordering from physical power-loss persistence.
4. Record maintainer approval of the exact W2 contract, or record that apply stops because an operation is unsupported or unproven. No W2 dependency or native sequence may be selected before that decision.

**Windows evidence placeholders supplied only by the target environment:**

```text
<WINDOWS_TARGET> cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore -- --nocapture
<WINDOWS_TARGET> cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings
<WINDOWS_TARGET> <filesystem/volume inspection proving local NTFS for the test app-data path>
<WINDOWS_TARGET> <feasibility-spike command producing native-call and failure evidence>
```

**W0 rollback boundary:** discard only the feasibility spike and target evidence if the contract is not proven; do not add `src-tauri/Cargo.toml` target dependencies or product adapter code.

## W1A — protocol core, recorder, and failpoints (220–320 changed lines)

W1A is the first implementation PR. It is host-independent, does not select a Windows API or dependency, and is reviewable through private protocol tests. Its tests stay beside `restore_transitions.rs` behavior.

**Start:** current restore primitives in `src-tauri/src/infrastructure/filesystem/backup_store.rs` and current `DatabaseState` flow in `src-tauri/src/lib.rs`; no native Windows selection.

**Finish:** private `RestoreTransitions<F: DurableFs>` owns restore-only ordering, private `RecordingFs` models durable namespace state and failpoints, and non-Windows production rejects unsupported durability before marker publication or connection disruption. W1A does not claim Windows durability.

**Rollback boundary:** revert only `src-tauri/src/infrastructure/filesystem/restore_transitions.rs` and its private declaration in `src-tauri/src/infrastructure/filesystem/mod.rs`, including the adjacent private tests. W1B and W2 must not retain calls or native assumptions that depend on the reverted module.

### W1A-RED — specify protocol behavior

- [x] Add private `RecordingFs`/modeled-namespace fixtures in `src-tauri/src/infrastructure/filesystem/restore_transitions.rs` with operation-kind/ordinal failpoints, durable-marker state, canonical/rollback/protective evidence, and interruption-before/after modeling. <!-- sdd-owner: implementation -->
- [x] Add failing private cases for preparation-before-disruption, marker progress, fallback-source retention, stale `.part` retry, existing-marker replacement, unsupported layout, cross-volume rejection, and required-barrier failure. <!-- sdd-owner: implementation -->

### W1A-GREEN — implement the deep private protocol module

- [x] Add private `DurableFs` primitives and `RestoreTransitions<F>` in `src-tauri/src/infrastructure/filesystem/restore_transitions.rs`; keep sync, directory, rename, replacement, removal, and adapter details out of public interfaces and map adapter failures to `StorageError::StorageUnavailable`. <!-- sdd-owner: implementation -->
- [x] Implement the non-Windows `UnsupportedDurableFs` so supported-layout verification fails before marker publication or live-connection disruption; keep recorder and real-filesystem adapters private to the module tests. <!-- sdd-owner: implementation -->
- [x] Implement `prepare_durable_restore` and marker publication with closed validated inputs, layout/same-volume checks, stage/protective file barriers, exact existing JSON bytes, exclusive `.part` handling, close-before-namespace mutation, atomic create/replace, and the app-data directory barrier. <!-- sdd-owner: implementation -->
- [x] Implement `install_durable_restore`, `recover_canonical_durably`, and `complete_durable_restore` with canonical-to-rollback ordering, both stage-install directory barriers, retained fallback sources, durable canonical installation before validation, and marker removal only after the caller's readiness callback. <!-- sdd-owner: implementation -->

### W1A-TRIANGULATE — prove modeled ordering beside the implementation

- [x] Table-drive interruption immediately before and after every file sync, directory barrier, stale-part removal, rename/replacement, canonical removal, marker publication, and marker removal; assert exact order, marker absence/equality/lag, and at least one modeled validated recovery source. <!-- sdd-owner: implementation -->
- [x] Assert private protocol tests preserve `{"state":"prepared"}`, `{"state":"live_moved"}`, and `{"state":"candidate_installed"}` bytes and never expose native paths or error text through the storage seam. <!-- sdd-owner: implementation -->
- [x] Run the focused private Rust tests and format check for W1A; label recorder/process interruption results as protocol evidence rather than Windows durability evidence. <!-- sdd-owner: implementation -->

### W1A-REFACTOR — keep the slice bounded

- [x] Refactor `restore_transitions.rs` for a small deep interface, policy/order in the private module, mechanics in adapters, borrowed paths where practical, typed `Result` propagation, and no production `unwrap`/`expect`; inspect the diff for W1A-only files. <!-- sdd-owner: implementation -->

```bash
cargo test --manifest-path src-tauri/Cargo.toml --lib
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

## W1B — BackupStore/DatabaseState integration and real host tests (280–380 changed lines)

W1B is the second implementation PR and depends on W1A. It makes the protocol usable through the existing application seams and keeps actual host filesystem/SQLite integration tests in the same slice as the wiring they verify. It remains independent of Windows-native selection.

**Start:** W1A's private transition module and tests are green; `BackupStore` and `DatabaseState` still use the old restore paths.

**Finish:** `BackupStore` exposes only the four narrow restore operations, `DatabaseState` preserves ticket 01 settlement and startup behavior, and `src-tauri/tests/backup_restore.rs` verifies real host behavior, compatibility, fallback retention, and bounded outcomes.

**Rollback boundary:** revert W1B's delegation in `src-tauri/src/infrastructure/filesystem/backup_store.rs`, private module wiring in `src-tauri/src/infrastructure/filesystem/mod.rs` if changed by integration, settlement edits in `src-tauri/src/lib.rs`, and the coupled `src-tauri/tests/backup_restore.rs` cases together. Leave W1A's private protocol slice intact and do not leave partial caller ordering.

### W1B-RED — add integration expectations

- [x] Add failing integration cases in `src-tauri/tests/backup_restore.rs` for exact marker compatibility, successful candidate installation, canonical-first/rollback-second/protective-third recovery, stale temporary retry, fallback-source retention, cleanup failure with `Ready`, and unavailable recovery with no active connection. <!-- sdd-owner: implementation -->
- [x] Add failing settlement cases proving preparation failure leaves canonical and the live connection untouched, while post-disruption failure retains evidence and maps only to `restore_failed` or `database_unavailable`. <!-- sdd-owner: implementation -->

### W1B-GREEN — wire the existing application seams

- [x] Replace bypassable restore primitives in `src-tauri/src/infrastructure/filesystem/backup_store.rs` with delegation to `prepare_durable_restore`, `install_durable_restore`, `recover_canonical_durably`, and `complete_durable_restore`; preserve unrelated backup publication and keep `filesystem/mod.rs` private. <!-- sdd-owner: implementation -->
- [x] Refactor `DatabaseState::install_validated_stage` in `src-tauri/src/lib.rs` so stage/protective validation handles close before preparation, durable `Prepared` precedes `Restoring` and live-connection drop, and one install operation owns filesystem ordering. <!-- sdd-owner: implementation -->
- [x] Publish a newly opened and fully validated canonical connection as `Ready` before completion; preserve the validated connection and return `restore_failed` if final marker removal fails, without treating marker removal as connection validity. <!-- sdd-owner: implementation -->
- [x] Route startup and post-disruption fallback through canonical → rollback → protective validation and `recover_canonical_durably`; preserve stale-connection gating, command gating, normal marker-free startup, public errors, and `Unavailable` with no active connection. <!-- sdd-owner: implementation -->

### W1B-TRIANGULATE — verify behavior on the configured host

- [x] Extend `src-tauri/tests/backup_restore.rs` with real temporary-filesystem and SQLite cases for handle closure before namespace mutation, both candidate-install directories, marker replacement/removal, recovery-copy source retention, and modeled failure evidence. <!-- sdd-owner: implementation -->
- [x] Add non-Windows coverage proving production selection fails before disruption and does not claim completed Windows durability; gate successful durable-transition cases with `cfg(windows)` and a local-NTFS precondition. <!-- sdd-owner: implementation -->
- [x] Run focused and full Rust tests, format, and inspect the W1B diff to confirm `commands/backup.rs`, schemas, migrations, UI/IPC, and unrelated publication behavior remain unchanged. <!-- sdd-owner: implementation -->

### W1B-REFACTOR — preserve application integrity

- [x] Refactor the integrated paths so `DatabaseState` retains connection/state ownership, filesystem ordering remains in the deep private module, native details stay out of IPC, and failures propagate without broad lint suppression. <!-- sdd-owner: implementation -->

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings
```

W1A and W1B together prove host-independent protocol ordering, failpoints, settlement, compatibility, and bounded recovery only. They do not prove Windows directory durability or physical cache persistence.

## W2 — W0-proven native adapter, target configuration, and Windows tests (220–340 changed lines)

W2 is a separate implementation PR blocked by the external W0 approval record and dependent on green W1B behavior. If W0 fails, stop; do not implement a reduced-guarantee path.

**Start:** W0 names and proves the exact native contract; W1B host tests pass.

**Finish:** `cfg(windows)` production code implements exactly the approved adapter and direct target dependency, with Windows-only tests covering the proven contract. The bounded claim is not accepted until the external evidence gate passes.

**Rollback boundary:** revert the W2 adapter, `src-tauri/Cargo.toml` target dependency, Cargo lockfile changes required by that dependency, and W2 Windows-only tests together. This removes the bounded durability claim rather than leaving a partial native path.

### W2-RED — encode the approved native contract

- [x] Add `cfg(windows)` tests in `src-tauri/tests/backup_restore.rs` or the private transition test module for marker create/replace/remove, canonical-to-rollback, stage-source and app-data-destination barriers, same-volume/layout rejection, handle sharing, flush/replacement failure, and recovery-evidence retention; this checkbox records test code plus Windows-target metadata compilation, not observed Windows execution, which remains unchecked below. <!-- sdd-owner: implementation -->

### W2-GREEN — implement only W0-approved mechanics

- [x] Implement `platform::WindowsDurableFs` in `src-tauri/src/infrastructure/filesystem/restore_transitions.rs` using only W0-proven calls, flags, access masks, sharing, replacement/no-replace behavior, flush operations, filesystem detection, reparse checks, and typed error mapping. <!-- sdd-owner: implementation -->
- [x] Add only the direct W0-approved dependency under `[target.'cfg(windows)'.dependencies]` in `src-tauri/Cargo.toml`, update `Cargo.lock` as Cargo requires, and document the bounded local-Windows-NTFS contract without claiming arbitrary power-loss immunity. <!-- sdd-owner: implementation -->

### W2-TRIANGULATE/REFACTOR — keep product changes verifiable

- [x] Compile the Windows target and run the W2-focused tests after W0 approval; confirm W1 host tests remain green and inspect the final diff for dependency, marker-schema, public-error, handle-closure, and unrelated-backup regressions. <!-- sdd-owner: implementation -->
- [x] Refactor the native adapter to keep all platform details private, preserve the four-operation caller interface, and fail closed for unsupported filesystems, reparse ambiguity, sharing violations, invalid handles, access denial, replacement failures, and flush failures. <!-- sdd-owner: implementation -->

## External W2 runtime evidence and maintainer acceptance (pending, not implementation PR work)

These actions change no product code and therefore remain visibly external. Their evidence cannot be replaced by W1 recorder output, Linux tests, cross-compilation, Wine, API documentation, or process-kill results.

1. On real Windows with local NTFS app-data, run focused/full Rust tests and the runtime matrix for marker retries, closed-handle transitions, both candidate-install directories, injected failures, fallback-source retention, restart selection order, `restore_failed`, and `database_unavailable`.
2. Record exact commands, environment, filesystem proof, native-call execution, successful and failed barrier behavior, and raw output. Keep process-kill results labeled as restart/protocol evidence, not physical power-loss evidence.
3. Have the maintainer review W0 and W2 records and decide whether the bounded durability claim is accepted. Missing or incomplete evidence blocks acceptance and archive; it must not silently weaken the guarantee.

```text
<WINDOWS_TARGET> cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore -- --nocapture
<WINDOWS_TARGET> cargo test --manifest-path src-tauri/Cargo.toml --all-targets --all-features
<WINDOWS_TARGET> cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
<WINDOWS_TARGET> cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings
```

## Whole-change verification and rollback

Required current-host verification is `cargo test --manifest-path src-tauri/Cargo.toml`, the focused `backup_restore` test, `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, and the configured clippy command. `npm test` is only relevant if an approved shared behavior unexpectedly touches the frontend.

The complete rollback is a coordinated revert of W1A/W1B restore-transition, delegation, settlement, and test changes plus any W2 adapter/configuration/tests. Persistent marker JSON, filenames, states, recovery-source order, and schema remain compatible; rollback restores the prior weaker durability behavior and must not retain ordering calls that depend on removed adapter support.

## Corrective Work Unit — post-verification blockers

These tasks are dependency ordered and remain unchecked until the two final-verification blockers are corrected.

- [x] **RED — deterministic lifecycle and cleanup coverage:** In `src-tauri/src/infrastructure/filesystem/restore_transitions.rs`, add failing deterministic tests for five sequential restore cycles with sidecar reuse, initial/per-sidecar cleanup failpoints with retry, and malformed sidecar rejection before disruption. <!-- sdd-owner: implementation -->
- [x] **RED — phase-verification coverage:** In `src-tauri/src/infrastructure/filesystem/restore_transitions.rs` and `src-tauri/tests/backup_restore.rs`, add failing tests proving Recovery rejects a missing marker, wrong source, or wrong canonical path, and Completion rejects unsupported layouts while allowing absent-marker idempotence only after supported-boundary verification. <!-- sdd-owner: implementation -->
- [x] **GREEN — private seam correction:** In `src-tauri/src/infrastructure/filesystem/restore_transitions.rs`, implement Prepare-only completed-sidecar recycling and phase-aware Prepare/Recovery/Completion verification, preserving exact marker bytes, fail-closed mutation ordering, and retained recovery evidence. <!-- sdd-owner: implementation -->
- [x] **TRIANGULATE — native and host alignment:** Update `cfg(windows)` native tests in `src-tauri/src/infrastructure/filesystem/restore_transitions.rs` or `src-tauri/tests/backup_restore.rs`, and align the host adapter/recorder expectations with sidecar reuse, cleanup retries, and all three phase-verification gates. <!-- sdd-owner: implementation -->
- [x] **REFACTOR — host verification and scope inspection:** Run focused and full Rust tests, locked all-target Clippy, formatting, and `git diff --check`; inspect `src-tauri/src/lib.rs`, `backup_store.rs`, `backup_restore.rs`, schemas, migrations, IPC, public errors, and fallback-source handling for regressions. <!-- sdd-owner: implementation -->
- [x] **Acceptance — final Windows implementation gate:** Regenerate the real-Windows fixed-local-NTFS evidence bundle and pass its focused native and `backup_restore` acceptance matrices for the corrected implementation. <!-- sdd-owner: implementation -->
