# Apply Progress: Local Backup and Restore
## Slice 1
- [x] 1.1–1.4: storage/path seams, synced SHA-256 publishing, candidate validation, and Online Backup snapshot/staging.
## Work Unit Evidence
| Evidence | Result |
|---|---|
| Focused test | `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore` — exit 0; 4 passed, 0 failed. |
| Runtime harness | N/A — no Tauri command/UI boundary; real file-backed SQLite integration exercised snapshot, staging, and transfer. |
| Rollback boundary | Revert `infrastructure/{filesystem,sqlite}/backup*`, `tests/backup_restore.rs`, and backup/SHA-256 dependencies. |
Outcome: success; evidence revision `sha256:3691806920e04f3906a85630b0d502b3b60e7934bed0aca75e5a01496c016188`; stacked-to-main Slice 1 only; no commit, branch, push, PR, or publication.

## Slice 2 — Preserved Failed Combined Attempt
- [ ] The combined coordinator, replacement state, crash recovery, and lifecycle work exceeded the 400-line budget.
Outcome: failed; evidence revision `sha256:f2a222bf3078549e5022e668dafc5223f94bcbf53d2367ecb13921c8caa060f3`; superseded only for the authorized 2A scope.

## Slice 2A: Restore Coordinator
- [x] 2.1 RED: behavior-first coverage for confirmation omission, token expiry/reuse/mismatch, changed candidates, protective failure, and operational-facts contract.
- [x] 2.2 GREEN: coordinator/contracts bind expiring single-use tokens to candidates, recheck before protection, and return stable outcomes without replacement.

## Work Unit Evidence
| Evidence | Result |
|---|---|
| Focused test | `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore` — exit 0; 7 passed, 0 failed. |
| Formatting/static | `cargo fmt --manifest-path src-tauri/Cargo.toml --check` — exit 0; focused Clippy — exit 0 with only pre-existing `too_many_arguments` and `needless_borrows_for_generic_args` lints excluded. |
| Runtime harness | N/A — Slice 2A has no Tauri command/UI or destructive replacement boundary; the real file-backed SQLite integration remains covered by the focused suite. |
| Rollback boundary | Revert `application/backup/*`, its `application/mod.rs` export, and Slice 2A tests; no live database replacement, marker, state, command, or UI behavior exists. |

## Behavior-First Evidence
| Task | RED | GREEN |
|---|---|---|
| 2.1/2.2 | `backup_restore` failed before implementation because `application::backup` did not exist. | Same focused command passed 7/7 after coordinator implementation. |

Outcome: passed; fresh evidence revision `sha256:ffcded1c07da89c4828f584e0cb945a0560cc9094fde70c1b80678ed039f9e16`; stacked-to-main Slice 2A only; no commit, push, PR, issue, branch, rebase, force, or publication.

## Slice 2B: Replacement and Recovery
- [ ] 2.3–2.4 not completed: the work unit reached 410 changed lines before SDD artifacts, exceeding the 400-line cap. All Slice 2B Rust/test changes were rolled back; task checkboxes remain unchanged.
Outcome: rolled back; no implementation evidence retained and no settlement attempted; preserve prior Slice 1, failed combined Slice 2, and successful Slice 2A evidence unchanged.

## Slice 2B1: Closed-Handle Replacement
- [x] 2B1 sub-slice: `DatabaseState` exclusively owns an optional live connection; it validates `pre-restore.sqlite3`, writes durable replacement transitions, drops the live connection before renames, installs the staged candidate, reopens and validates canonical data, and retains the protective database.
- [ ] Tasks 2.3–2.4 remain unchecked because their full text includes startup recovery, canonical-first selection, rollback/unavailable startup handling, and crash-state tests reserved for Slice 2B2.

## Work Unit Evidence
| Evidence | Result |
|---|---|
| Focused test | `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore` — exit 0; 9 passed, 0 failed. Fresh command-output evidence: `sha256:27cdd510b2ffcac2177a251aeea1907e9c39d849a8221a72eb18c54559a175e2`. |
| Formatting/static | `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo check --manifest-path src-tauri/Cargo.toml --features desktop`, and focused Clippy — exit 0. Clippy excludes only pre-existing `too_many_arguments` and `needless_borrows_for_generic_args` lints. |
| Runtime harness | N/A — no command/UI integration exists in 2B1 and no development process was launched; file-backed SQLite integration proves the closed-handle rename, reopen, and retention path. |
| Rollback boundary | Revert `DatabaseState`, restore-state file operations, restored-database validation, and the two 2B1 tests in the six changed Rust files; this removes no existing sales formatting or 2A coordinator behavior. |

## Behavior-First Evidence
| Sub-slice | RED | GREEN |
|---|---|---|
| 2B1 replacement | `backup_restore` failed because `DatabaseState`, `RestoreState`, and marker transition operations did not exist. | The focused suite passed 9/9 after implementing closed-handle replacement and durable transition behavior. |

## Native Settlement Binding
- Acquire request: `local-backup-restore-slice2b1-acquire-20260827-1`; continuation token: `sha256:db9cae8ef05b2387c46b9e3135f5b4ac2c37bd8aae3e9db55dc73ae22e59ad7f`.
- Objective revision: `sha256:deac2e24a877878ff951c627f312dafa53b7643626cc6593f395ae27d984acab`; failed evidence remediated: `sha256:80d6c2ab27612b22742f5acb83ee2e1e4adcca0fcb1bd812854fabeaf6758aa5`.
- Candidate scope is 312 Rust source/test changed lines plus this progress delta, below the immutable 400-line cap; no size exception, task checkbox, commit, push, PR, issue, branch, or publication occurred.

## Slice 2B2: Startup Recovery
- [x] 2.3 RED: added file-backed crash-state coverage for `prepared`, `live_moved`, and `candidate_installed`; proves canonical-first selection, rollback then protective fallback, unavailable state with no canonical creation, marker cleanup only after convergence, and protective retention.
- [x] 2.4 GREEN: added `DatabaseState::recover_on_startup` before desktop state registration; recovery validates canonical read-only first, then retained rollback and protective databases, copies a safe source through a synced temporary file, reopens/revalidates, and leaves the state unavailable on failure.

## Work Unit Evidence
| Evidence | Result |
|---|---|
| Focused test | `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore` — exit 0; 13 passed, 0 failed. Exact fresh command-output SHA-256: `sha256:0854fc7fe84eeec874f5f88eae60413b88974b04b2ae5a4acaf0aeea1aed0167`. |
| Formatting/static | `cargo fmt --manifest-path src-tauri/Cargo.toml --check` and `cargo check --manifest-path src-tauri/Cargo.toml --features desktop` — exit 0. Focused `cargo clippy --manifest-path src-tauri/Cargo.toml --test backup_restore --features desktop -- -D warnings -A clippy::too_many_arguments -A clippy::needless_borrows_for_generic_args` — exit 0. Workspace-wide all-target Clippy remains blocked by untouched `tests/inventory_sqlite.rs:110` `clippy::drop_non_drop`. |
| Runtime harness | N/A — no Tauri command/UI or development process is in this slice. Real file-backed SQLite integration executed all crash-state scenarios; desktop command-surface test command exited 0 with 2 passed. |
| Rollback boundary | Revert `src-tauri/src/lib.rs` recovery wiring, `BackupStore::restore_canonical_from`, and the four 2B2 recovery tests in `src-tauri/tests/backup_restore.rs`; this preserves 2B1 replacement, 2A coordinator, sales formatting, and unrelated assets. |

## Behavior-First Evidence
| Task | RED | GREEN |
|---|---|---|
| 2.3/2.4 recovery states | `backup_restore` exited 101 before production recovery code because `DatabaseState::recover_on_startup` was absent. | The focused suite passed 13/13 after canonical-first recovery, unavailable-state handling, and startup wiring. |
| Protective fallback retention | The dedicated protective-fallback test failed 1/1 because recovery moved `pre-restore.sqlite3`. | It passed 1/1 after recovery copied through a synced temporary file, retaining the protective database. |

## Native Settlement Binding: Slice 2B2
- Request: `local-backup-restore-slice2b2-20260827-2220`; continuation token: `sha256:6141d85b49b4f236eb75aef2074f9c6bd3ffa427c58e58a58dc54a958e4dea6c`.
- Objective: `slice-2b2-startup-recovery`, generation `6`, max `400` changed lines, revision `sha256:6141d85b49b4f236eb75aef2074f9c6bd3ffa427c58e58a58dc54a958e4dea6c`.
- Candidate scope: 200 additions and 3 deletions in the three 2B2 Rust source/test files before SDD artifact updates; 203 changed lines, below the cap. No size exception.
- Settlement: **passed** via `sdd-attempt finish` request `local-backup-restore-slice2b2-20260827-2220-finish`. Native status is `complete: true`, `next_action: complete`, generation `6`, native changed lines `231`, candidate identity `sha256:6c07eae89ba8570102ff50655760fa79d40f496b83217341852e52ed84b26415`, candidate tree `1999fa273055708df49956ae4010b344f43f551a`, and final runtime revision `sha256:b2597c24cd2e15355c8705786f2bb7ff9a3d3e066f131223adcf33a530184fb2`.
- Settlement evidence: `sha256:0854fc7fe84eeec874f5f88eae60413b88974b04b2ae5a4acaf0aeea1aed0167`; harness disposition `reused`. No commit, push, PR, issue, branch, merge, or publication.

## Slice 3A: Tauri Command Surface
- [x] 3.1 RED: added command-boundary evidence for serde `deny_unknown_fields`, the five-command allowlist, exact tagged stable errors, unavailable-state exclusion, and invalid restore-token rejection.
- [x] 3.2 GREEN: added typed create/prepare/confirm commands, native destination/source picker commands, dialog plugin registration, least-privilege `dialog:allow-open` capability, and Rust/Node dialog dependencies.

## Work Unit Evidence
| Evidence | Result |
|---|---|
| Focused test | `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore` — exit 0; 14 passed, 0 failed. Exact command-output SHA-256: `sha256:5d7ae510f0bddbd2282f9a3346642e0cb0448d848abaa3d2403a92e4c1894f68`. |
| Formatting/static | `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo check --manifest-path src-tauri/Cargo.toml --features desktop`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --test backup_restore --features desktop -- -D warnings -A clippy::too_many_arguments -A clippy::needless_borrows_for_generic_args` — exit 0. |
| Runtime harness | `cargo test --manifest-path src-tauri/Cargo.toml --lib --features desktop command_surface_tests` — exit 0; 2 passed, 0 failed. The Tauri mock command-surface harness ran without starting a development process; native platform smoke remains explicitly excluded to Slice 4. |
| Rollback boundary | Revert `src-tauri/src/commands/backup.rs`, backup command registration/state in `src-tauri/src/lib.rs`, dialog dependency/capability changes, and the Slice 3A test; this preserves sales formatting, storage, coordinator, replacement, and recovery behavior. |

## Behavior-First Evidence
| Task | RED | GREEN | REFACTOR |
|---|---|---|---|
| 3.1/3.2 command boundary | `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore` exited 101 because `commands::backup` did not exist. | The focused suite passed 14/14 after the typed command boundary and registration were implemented. | `cargo fmt --manifest-path src-tauri/Cargo.toml --check` exited 0; command responses retain only stable codes/messages and intentional selected paths. |

## Native Settlement Binding: Slice 3A
- Request: `local-backup-restore-slice3a-20260827-2245`; continuation token: `sha256:1e3adafec825c4facaefa20d875b82c6af07a06286cea9bc81d9d98a5cb49b4a`.
- Objective: `slice-3a-tauri-backup-ipc`, generation `7`, max `400` changed lines, initial revision `sha256:1e3adafec825c4facaefa20d875b82c6af07a06286cea9bc81d9d98a5cb49b4a`.
- Settlement: **passed** via `sdd-attempt finish` request `local-backup-restore-slice3a-20260827-2245-finish`. Native status is `complete: true`, `next_action: complete`, generation `7`, native changed lines `344`, candidate identity `sha256:e5371e1ae1a95cd6f0dc0251280b204643b22e8889900a5e15273698d5ff080c`, candidate tree `356739c7138db1fc12adf5df6b848a2d8a26ade4`, and final runtime revision `sha256:c8526e24d9b18939f20927e85998adf002675861a37cb4ff6f142327f701672c`.
- Settlement evidence: `sha256:5d7ae510f0bddbd2282f9a3346642e0cb0448d848abaa3d2403a92e4c1894f68`; harness disposition `reused`. No size exception, dev process, commit, push, PR, issue, branch, merge, or publication.

## Slice 3B: React Backup and Restore Flow
- [x] 3.3 RED: added behavior-first command and UI-flow evidence for native-picker cancellation, typed payloads, stable errors, candidate summary/confirmation, loading, success, and failure states.
- [x] 3.4 GREEN: added the typed Tauri adapter and React backup/restore screen, including native picker commands only, destructive confirmation, disabled busy controls, and stable outcome feedback.

## Work Unit Evidence
| Evidence | Result |
|---|---|
| Focused test | `npx tsx --test src/commands/backup.test.ts src/ui/backup/*.test.ts` — RED: exit 1, 2 failed because the adapter and flow modules did not exist; GREEN: exit 0, 4 passed, 0 failed. |
| Typecheck/build | `npx tsc --noEmit` and `npm run build` — exit 0. Vite reports only its existing mixed static/dynamic Tauri-core import warning. |
| UI regression | `npx tsx --test src/ui/**/*.test.ts` — exit 0; 13 passed, 0 failed. `npm test` — exit 0; 26 passed, 0 failed. |
| Runtime harness | N/A — the required native picker → prepare → confirm scenario is a Phase 4 Fedora/Windows smoke obligation; no development process was started for this slice. |
| Rollback boundary | Revert `src/commands/backup*`, `src/ui/backup/*`, and the backup navigation additions in `src/ui/app.ts`; this leaves sales formatting and all Rust lifecycle behavior intact. |

## Native Settlement Binding: Slice 3B
- Request: `local-backup-restore-slice3b-20260827-2340`; continuation token: `sha256:de15f296a3f525af2973acb29d4725653b9b83bafe48fa8a52a7a10d92d18bae`; max `400` changed lines.
- Candidate scope: 118 additions and 1 deletion (119 changed lines) across the six TypeScript command/UI/test files before SDD artifact updates; 141 changed lines including the two task checkbox replacements and this 18-line progress delta, below the cap with no size exception.
- Settlement: unavailable — `sdd-attempt` is not installed on this executor's PATH, so no native acquire/finish receipt was created or claimed. No commit, push, PR, issue, branch, merge, publication, Rust change, or development process occurred.

## Slice 4: Verification Attempt — BLOCKED
- [ ] 4.1 remains incomplete. No production files or task checkboxes changed.
- Full TypeScript suite: `npm test` — exit 0; 26 passed, 0 failed.
- Full locked Rust suite: `cargo test --manifest-path src-tauri/Cargo.toml --locked --features desktop` — exit 0; 98 passed, 0 failed.
- TypeScript typecheck: `npx tsc --noEmit` — exit 0.
- Isolated production build: `npm run build -- --outDir /tmp/opencode/repuestos-autos-slice4-build` — exit 0; Vite emitted only the pre-existing mixed static/dynamic Tauri-core import warning.
- Quality: `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo check --manifest-path src-tauri/Cargo.toml --locked --all-targets --features desktop`, and focused `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --test backup_restore --features desktop -- -D warnings -A clippy::too_many_arguments -A clippy::needless_borrows_for_generic_args` — exit 0.
- Focused checks: `npx tsx --test src/commands/backup.test.ts src/ui/backup/*.test.ts` — exit 0; 4 passed, 0 failed. `cargo test --manifest-path src-tauri/Cargo.toml --locked --test backup_restore` — exit 0; 14 passed, 0 failed.

## Work Unit Evidence
| Evidence | Result |
|---|---|
| Fedora runtime harness | `setsid env GDK_BACKEND=x11 WEBKIT_DISABLE_DMABUF_RENDERER=1 XDG_DATA_HOME=<temp>/data XDG_CONFIG_HOME=<temp>/config XDG_CACHE_HOME=<temp>/cache npm run tauri:dev` started Vite (ready in 268 ms), `target/debug/repuestos-autos`, and WebKit child processes against only `/tmp/opencode/repuestos-autos-slice4-runtime.TMG2yf/data/com.repuestosautos.app/repuestos-autos.sqlite3`. Exact PGID `1744017` was terminated with `TERM`; a post-cleanup group query was empty. |
| Deterministic media-removal probe | **FAILED requirement.** `cargo run --manifest-path /tmp/opencode/repuestos-autos-slice4-media-probe/Cargo.toml --offline` removed an isolated `removable-like-mount` before `BackupStore::publish_snapshot`. The call returned `Ok(PublishedBackup { ... })` and `mount_recreated=true`, proving `fs::create_dir_all(destination)` re-creates a removed mountpoint instead of returning `storage_unavailable`. |
| Visual/manual evidence | No screenshot is claimed: ImageMagick `import` failed and recorded its error at `/tmp/opencode/repuestos-autos-slice4-runtime.TMG2yf/screenshot.stderr`. Native picker selection/confirmation requires manual interaction and was not performed. |
| Windows release gate | **BLOCKING.** The proposal success criterion and Phase 4 task require Windows smoke, including drive/UNC/Unicode, antivirus/handle locks, and rename release. This Fedora-only executor did not run Windows evidence; it cannot be downgraded to a warning. |
| Preservation and lock evidence | Existing focused Rust coverage passed for SQLite snapshot mutex release, validated-stage replacement after closing the live connection, recovery states, and operational-facts coordinator contracts. End-to-end manual restore fact preservation was not attempted after the media-removal defect was proven. |
| Rollback boundary | No product code was changed. Remove only this Slice 4 progress section and external `/tmp/opencode/repuestos-autos-slice4-*` evidence if a clean verification record is required. |

## Native Settlement Binding: Slice 4
- Request: `local-backup-restore-slice4-20260828-0005`; expected revision token: `sha256:d743b4fc5c029f69bb421b1b1581e17f254aab3cd4139b302f6d90b22c2bfa0b`; max `400` changed lines.
- Settlement: **failed** via `sdd-attempt finish` request `local-backup-restore-slice4-20260828-0005-finish`. Native objective generation `9`, changed lines `0`, finish candidate tree `dde725db94030f69687bf1d9c06482a4dea7fc4a`, final runtime revision `sha256:941782bdc4d82c9760ad9072f04a15e0df141c2f13dd3e5382c53a024cde0bff`.
- Evidence revision: `sha256:40bfc1fb2e4aa57c259a56867a2f390e9430050ba85d3e2b66538f80b00343d7` for `/tmp/opencode/repuestos-autos-slice4-runtime.TMG2yf/evidence.md`; harness disposition `invalidated`.

## Bounded Remediation: Removed Selected Root
- [ ] 4.1 remains incomplete: Windows platform smoke evidence is still pending.
- The selected root must exist; publication may create only its `backup-restore` child and returns `storage_unavailable` without recreating a removed root.

### TDD Cycle Evidence
| RED | GREEN | REFACTOR |
|---|---|---|
| The removed-root regression exited 101: publication returned `Ok` and recreated the root. | The same focused test exited 0; 1 passed. Full `backup_restore` exited 0; 15 passed. | `cargo fmt --check`, desktop `cargo check`, and focused Clippy exited 0. |

### Work Unit Evidence
| Evidence | Command / result |
|---|---|
| Focused test | `cargo test --manifest-path src-tauri/Cargo.toml --locked --test backup_restore` — exit 0; 15 passed, 0 failed. |
| Runtime harness | N/A — this file-store boundary uses real filesystem integration; broad desktop runtime is intentionally deferred to independent verification. |
| Rollback boundary | Revert the selected-root child path in `commands/backup.rs`, guarded child-directory creation in `backup_store.rs`, and the two publish tests. |

- Settlement attempt: **blocked**. The single authorized `sdd-attempt settle` call included the failed-evidence revision but returned `invalid_continuation`; native status remains `running`, generation `10`, `next_action: finish`.

```yaml
schema: gentle-ai.remediation-result/v1
lineage_id: sha256:0fe8c2b6b6447e8ee943003ec442bd7abaef776ea851d42fb94f7e04b5381d6b
generation: 10
fix_batch: 1
failed_evidence_revision: sha256:40bfc1fb2e4aa57c259a56867a2f390e9430050ba85d3e2b66538f80b00343d7
evidence_revision: sha256:48445ff2b05d0d541b4177570627f258a474058dbbb3fb936eea1e7b56dbe5f5
status: blocked
mode: Standard
focused_tests: passed
runtime_harness: not_applicable
rollback_boundary: recorded
settlement_state: blocked_invalid_continuation
next_recommended: native-attempt-repair
```
```json
{"schema":"gentle-ai.remediation-evidence/v1","lineage_id":"sha256:0fe8c2b6b6447e8ee943003ec442bd7abaef776ea851d42fb94f7e04b5381d6b","generation":10,"fix_batch":1,"failed_evidence_revision":"sha256:40bfc1fb2e4aa57c259a56867a2f390e9430050ba85d3e2b66538f80b00343d7","evidence_revision":"sha256:48445ff2b05d0d541b4177570627f258a474058dbbb3fb936eea1e7b56dbe5f5","settlement":{"state":"blocked","reason":"invalid_continuation"},"commands":[{"command":"cargo test --manifest-path src-tauri/Cargo.toml --locked --test backup_restore","exit_code":0,"result":"15 passed, 0 failed"},{"command":"cargo fmt --check; cargo check --all-targets --features desktop; focused cargo clippy","exit_code":0,"result":"all passed"}],"runtime_harness":{"status":"not_applicable","na_reason":"Real filesystem integration only; desktop runtime deferred."},"rollback":{"boundary":"commands/backup.rs, backup_store.rs, backup_restore.rs","evidence":"Revert only selected-root publication behavior and its regression tests."}}
```

## Post-Remediation Fedora Verification (2026-08-28)

- Fedora-only independent verification passed for the corrected candidate; task 4.1 remains unchecked because the Windows release gate is still pending.
- Evidence revision: `sha256:6364984f1131f1e66b54292e5fa24e08c0af6b852df1cc5964c9618c433b9f82` (`/tmp/opencode/local-backup-restore-fedora-verify-20260828-1/evidence.md`).

| Evidence | Command / result |
|---|---|
| Removed-root regression | Locked focused `backup_restore` test — exit 0; 1 passed, 14 filtered. It proves `storage_unavailable` and no selected-root recreation. |
| Full integration | Locked `backup_restore` — exit 0; 15 passed. It covers the existing Unicode selected root, publish sync/non-overwrite, staged restore facts, mutex release, and closed-handle replacement. |
| Full suites and quality | Locked Rust desktop — 99 passed; TypeScript — 26 passed; typecheck, isolated production build, fmt, all-target desktop check, and focused Clippy — all exit 0. |
| Fedora runtime | Isolated XDG paths under `/tmp/opencode`; Tauri and WebKit started with `GDK_BACKEND=x11 WEBKIT_DISABLE_DMABUF_RENDERER=1`. Exact PGID 24611 was terminated with TERM; post-TERM and post-cleanup group queries were empty. No visual or picker interaction is claimed. |
| Windows gate | Still pending: drive/UNC/Unicode, antivirus-handle/rename-release evidence. Fedora success does not complete task 4.1 or overall SDD verification. |

Rollback boundary: remove only this verification record and `/tmp/opencode/local-backup-restore-fedora-verify-20260828-1`; no production edit was made by this verification work unit.

## Picker Callback Remediation (2026-08-28)
- [ ] 4.1 remains incomplete: Fedora and Windows manual picker evidence is pending.
- Both picker commands now bridge `pick_folder`/`pick_file` callbacks through an async typed `PathSelection` result; no blocking dialog APIs remain.

### Work Unit Evidence
| Evidence | Command / result |
|---|---|
| Focused test | Desktop `backup_restore` — exit 0; 16 passed, including callback selected/cancelled contract mapping. |
| Automated regression | Default `backup_restore` — 15 passed; full desktop Rust — 100 passed; TypeScript — 26 passed; typecheck, fmt, check, and focused Clippy — exit 0. |
| Runtime harness | Detached isolated Fedora session is alive (leader/PGID `40357`); Vite, Tauri, and WebKit are running for maintainer exercise of both callbacks. |
| Rollback boundary | Revert callback bridge in `lib.rs`, its helper in `commands/backup.rs`, and the callback-contract test only. |

## Merged Remediation State (2026-08-28)

- PR #86 merged remediation commit `071ca29` through merge commit `9800abf`.
- Removed selected-root handling and asynchronous native picker behavior are merged.
- Fedora manual smoke passed for the corrected backup and restore picker flows.
- Windows drive-letter, UNC, Unicode, removable-media, antivirus/handle-lock, and rename-release evidence remains pending.
- Task 4.1 remains incomplete. No final verification or archive is claimed.
