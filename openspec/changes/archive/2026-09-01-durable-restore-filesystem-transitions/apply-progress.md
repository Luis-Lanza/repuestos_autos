# Apply Progress: Durable Restore Filesystem Transitions

## Status

- Phase status: partial — W1A and W1B complete; four of five W2 rows complete, with real-Windows product runtime acceptance still pending.
- Structured status consumed: authoritative hybrid/OpenSpec status reported apply ready, 20/25 complete, exactly five W2 rows pending, no blocked reasons, and W0 approval supplied by the parent.
- Action context: repository-local with workspace and allowed edit root `/home/luis/velay/repuestos_autos`; edits stayed within the six supplied W2 surfaces and no action-context warning was present.
- Delivery boundary: stacked-to-main PR 3 / W2 native adapter, target dependency, and Windows-gated product tests. No branch, PR, commit, or push was created.
- Review budget: W2 is 366 changed lines against the supplied W1 baseline (357 additions and 9 deletions by reconstructed per-file accounting), below the 400-line maximum. W1A and W1B remain independent slices.
- W0 approval consumed: Windows 11 Home x64 local fixed NTFS bundle `20260901T054440Z-1ba258df.zip`, SHA-256 `e54110016b3b3cbdfb9e4b97cfe760ef553125caee41e4e3d5eeaa6f05ff1ff2`, with all supplied integrity/runtime/cleanup checks passing. Current Linux results remain compile/protocol evidence only, not real-Windows product runtime or physical power-loss evidence.

## Completed tasks and persisted checkboxes

All 20 W1 implementation rows are visibly checked in `tasks.md`. This apply checked exactly the 10 W1B rows at the assigned task region:

1. Integration expectations for marker compatibility, installation/recovery outcomes, stale temporary retry, retained fallback, cleanup failure, and unavailable settlement.
2. Settlement expectations for pre-disruption failure and bounded post-disruption outcomes.
3. BackupStore delegation through the four restore-only operations.
4. Prepared-before-Restoring/drop and single-operation installation wiring.
5. Ready-before-completion with Ready retained after cleanup failure.
6. Startup/post-disruption canonical → rollback → protective durable recovery wiring.
7. Temporary-filesystem/SQLite transition, retention, and failure evidence.
8. Non-Windows fail-before-disruption coverage and Windows-only successful transition boundaries.
9. Focused/full tests, formatting, lint attempt, and scope/diff inspection.
10. DatabaseState ownership and private filesystem-ordering refactor without broad lint suppression.

The later W2 update checks four rows; one compile/runtime row remains visibly unchecked.

## Files changed

- `src-tauri/src/infrastructure/filesystem/restore_transitions.rs` — private production selection, private test-only real-host adapter, and module-level four-operation entry points; unsupported production selection fails closed.
- `src-tauri/src/infrastructure/filesystem/backup_store.rs` — removed bypassable restore mutation primitives and delegated the four restore-only operations while leaving backup publication unchanged.
- `src-tauri/src/lib.rs` — prepares before disruption, installs through one operation, publishes Ready before completion, retains Ready on cleanup failure, and uses durable fallback in canonical → rollback → protective settlement.
- `src-tauri/tests/backup_restore.rs` — exact marker bytes, non-Windows pre-disruption failure, stale temporary/source retention, canonical-first startup behavior, and Windows-only successful transition cases.
- `openspec/changes/durable-restore-filesystem-transitions/tasks.md` — checked the 10 W1B rows and, in the later W2 update, four W2 rows.
- `openspec/changes/durable-restore-filesystem-transitions/apply-progress.md` — cumulative W1A+W1B+W2 progress.

`src-tauri/src/infrastructure/filesystem/mod.rs` retains W1A's private declaration and needed no W1B change.

## TDD Cycle Evidence

| Cycle | Safety net / RED | GREEN | TRIANGULATE | REFACTOR |
|---|---|---|---|---|
| W1A protocol core | Existing W1A evidence: focused compile failed on unresolved private seam after 16-test library safety net. | Four private protocol tests passed. | Exact markers, failpoints, retained sources, layout failure, and readiness ordering passed. | Private deep seam remained bounded to 384 lines. |
| W1B integration | Safety net: `backup_restore` passed 26/26 before W1B edits. RED: focused integration compile failed because `BackupStore::recover_canonical_durably` did not exist. | Four-operation delegation and DatabaseState wiring made the new non-Windows recovery test pass. | Added exact bytes, unsupported prepare, stale-temporary/source retention, canonical-marker startup, post-disruption Ready recovery, cleanup-failure Ready retention, and unavailable/Windows-gated paths; final focused tests passed 22/22 and library tests 17/17. | Removed old public restore mutation paths, kept the real-host adapter private under `cfg(test)`, formatted, reran focused tests, and reduced the final W1B count to 393 lines. |

## Commands run

- `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore` — safety net passed 26/26; final passed 22/22 after honest platform cfg boundaries.
- `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore unsupported_recovery_retains_the_validated_source_and_stale_temporary --no-fail-fast` — RED failed on the missing four-operation method, then GREEN passed 1/1.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib` — final passed 17/17.
- `cargo test --manifest-path src-tauri/Cargo.toml` — full Rust suite passed before the final assertion-only refactor; the final affected integration and library suites were rerun green afterward.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings` — unavailable because host `glib-2.0`, `gio-2.0`, `gobject-2.0`, and `gdk-3.0` pkg-config packages are missing.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings` — reached project linting but failed on three pre-existing out-of-scope warnings: two `too_many_arguments` findings and one `large_enum_variant` finding.
- `git diff --check` — passed.
- Independent baseline comparison across the five W1B implementation/test surfaces — 393 changed lines.

## Deviations and risks

- The approved non-Windows production adapter fails closed before disruption. A private `cfg(test)` real-host adapter preserves meaningful SQLite/settlement tests without creating a public or general filesystem abstraction.
- Successful durable-transition integration cases are Windows-only. A truthful local-NTFS runtime precondition cannot be implemented until W0 selects and proves filesystem/volume detection; therefore no Linux or recorder result is represented as Windows evidence.
- The configured all-features lint command is environment-blocked, and the no-default-feature lint attempt exposes only pre-existing out-of-scope findings. No broad lint suppression was introduced.
- W1B changed-line count is 393, leaving only seven lines of budget; no further work belongs in this slice.
- That W1B slice touched no Windows API or Cargo dependency; the later W2 slice adds only the approved private adapter and target dependency while still avoiding commands, schema/migrations, frontend, branch, PR, commit, and push changes.

## W2 apply update

### Completed tasks and persisted checkboxes

Four W2 implementation rows are now visibly checked in `tasks.md`: the Windows-gated product-test boundary, private `WindowsDurableFs`, direct target dependency/lock update with bounded contract, and private fail-closed refactor. Existing `cfg(windows)` integration cases now use the required `backup-restore/staging` layout and exercise the real production adapter on Windows; modeled failpoints continue to cover flush/replacement failures and retained evidence.

### Files changed in W2

- `src-tauri/src/infrastructure/filesystem/restore_transitions.rs` — private Windows adapter using the approved handles, sharing, file/directory flush, no-replace rename, replacement, delete, volume/NTFS/fixed-drive checks, reparse rejection, same-volume checks, and explicit closure.
- `src-tauri/tests/backup_restore.rs` — corrected Windows-only product cases to the supported staging layout so they execute the native adapter rather than an unsupported layout.
- `src-tauri/Cargo.toml` — direct Windows-only `windows-sys = 0.61.2` dependency with exactly the four approved features.
- `src-tauri/Cargo.lock` — attached the already locked `windows-sys 0.61.2` package to the application package.
- `openspec/changes/durable-restore-filesystem-transitions/tasks.md` — checked exactly four completed W2 rows.
- `openspec/changes/durable-restore-filesystem-transitions/apply-progress.md` — merged this W2 evidence with W1 progress.

### W2 commands and evidence

- `cargo fetch --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-gnu --locked` — passed and fetched the approved target dependency.
- `rustup target add x86_64-pc-windows-gnu` — passed.
- `cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-gnu --locked` — reached `libsqlite3-sys` but was environment-blocked before the product crate by missing `x86_64-w64-mingw32-gcc`; therefore the compile/runtime row remains unchecked.
- A Windows-target `rustc --emit metadata` harness over the actual private transition module and fetched `windows-sys` metadata passed, type-checking the adapter API usage without executing it.
- `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore` — passed 22/22 after W2.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib` — passed 17/17 after W2.
- `cargo test --manifest-path src-tauri/Cargo.toml` — full current-host Rust suite passed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` and `git diff --check` — passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings` — environment-blocked by missing Linux `glib-2.0`, `gobject-2.0`, `gio-2.0`, `gdk-3.0`, and `atk` pkg-config packages.

### W2 deviations and residual risks

- The local host cannot run Windows binaries, and the GNU cross-check lacks the MinGW C compiler required by bundled SQLite. Real Windows product tests and runtime acceptance were not claimed.
- The bounded claim remains limited to successful requested ordering on supported local Windows app-data/NTFS; no arbitrary physical power-loss immunity is claimed.
- All adapter errors remain private and map through `StorageError::StorageUnavailable`; no public error, marker schema, database schema, migration, IPC, UI, or unrelated backup publication behavior changed.

## Remaining implementation tasks (1)

- [ ] Compile the Windows target and run the W2-focused tests after W0 approval; confirm W1 host tests remain green and inspect the final diff for dependency, marker-schema, public-error, handle-closure, and unrelated-backup regressions. <!-- sdd-owner: implementation -->

## Corrective W2 rerun

### Status and task reconciliation

- Structured status consumed: authoritative OpenSpec apply-ready status, repository-local action context at `/home/luis/velay/repuestos_autos`, no blocked reasons, and an audited `w2-correction` attempt with a 400-line renewed limit.
- Delivery boundary stayed stacked-to-main PR 3 / W2 only. No branch, commit, push, PR, child agent, or out-of-scope file write occurred.
- The checked W2 native-test row now states explicitly that it records Windows-gated test code and Windows-target metadata compilation, not observed Windows execution. The compile/runtime row remains visibly unchecked.
- No implementation checkbox changed state during this correction; 24/25 remain checked and the exact remaining row is repeated below.

### Corrective implementation and tests

- `Replace` now carries an explicit preserved-old path through the private protocol/adapter seam. Marker replacement selects one of eight bounded same-directory backup slots, never deletes an occupied slot, and fails closed when all slots are occupied or a racing backup appears.
- Stale marker-part removal now has a protocol-owned root-directory barrier before exclusive recreation. Windows marker/recovery temporary creation uses `CREATE_NEW`, eliminating the prior existence-check plus `CREATE_ALWAYS` race.
- Preparation passes the actual protective path into verification. Verification checks exact canonical/protective locations, all exact evidence files, same-volume membership, local fixed NTFS, every existing stage/root ancestor, and reparse/type ambiguity.
- Raw Windows UTF-16 path scanning rejects components exactly `.` or `..` across both Windows separators before native operations. Volume and filesystem-name buffers use bounded growth retries; long paths remain fail-closed and are not claimed as supported.
- New private `cfg(windows)` tests directly cover kernel-exclusive create/copy/no-replace behavior, preserved replacement backup and existing-backup failure, supported-layout and ambiguous-layout decisions, same-volume decision rejection, all-share rename behavior, and native sync/replacement errors. These tests metadata-compile but were not executed on Windows.

### Corrective verification

- `cargo test --manifest-path src-tauri/Cargo.toml --lib restore_transitions --no-fail-fast` — 5/5 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore --no-fail-fast` — 22/22 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` — full current-host Rust suite passed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` and `git diff --check` — passed.
- Windows-target `rustc --test --emit=metadata` over the actual private module and `windows-sys 0.61.2` metadata — passed with dead-code-only harness warnings; this is compile evidence, not product runtime evidence.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings` — environment-blocked by missing Linux GLib/GObject/GIO/GDK/ATK pkg-config packages.

### Changed-line and risk accounting

- Corrective implementation/test delta against attempt-start tree `ea5e5356a7da09c58fb38f568baaf531471a839b`: 245 additions + 67 deletions = 312 lines, all in `restore_transitions.rs`.
- Corrective OpenSpec bookkeeping: 39 additions + 2 deletions = 41 changed lines across this cumulative progress update and the one-line task clarification. Total corrective attempt: 353 changed lines, within the renewed 400-line objective.
- Corrected cumulative W2 implementation/test/config delta against W1B tree `83c5f171143514931730c6816a9c3ca1ee99ae8f`: 548 additions + 22 deletions = 570 lines. This historical cumulative W2 footprint exceeds 400, while the separately authorized corrective attempt remains within its renewed limit; no size exception or delivery claim is invented.
- Residual risk: no real Windows product binary or native test executed in this environment. The bounded local-Windows-NTFS acceptance claim and archive remain blocked on the final unchecked task and external runtime evidence.

## Remaining implementation tasks (1)

- [ ] Compile the Windows target and run the W2-focused tests after W0 approval; confirm W1 host tests remain green and inspect the final diff for dependency, marker-schema, public-error, handle-closure, and unrelated-backup regressions. <!-- sdd-owner: implementation -->

## Surgical W2 lexical-path remediation

- Structured status produced from the authoritative hybrid/OpenSpec artifacts: proposal/spec/design/tasks/apply-progress are present, task progress remains 24/25, apply is ready, and the exact parent-authorized edit roots are the transition module and this progress file with no warnings.
- Delivery boundary remained `w2-path-remediation`, max one attempt / 120 changed lines; no checkbox changed and the final Windows runtime row remains unchecked.
- Replaced normalized `Path::components()` inspection with raw `OsStr` UTF-16 component scanning across `\\` and `/`, rejecting components exactly `.` or `..` before any native layout operation.
- Added the focused `cfg(windows)` test `layout_rejects_lexical_interior_dot_component`; the actual module and test metadata compile for `x86_64-pc-windows-gnu`, but no Windows test binary was executed.
- Current-host evidence passed: focused restore transitions 5/5, focused backup restore 22/22, and the full Rust suite. `cargo fmt --check` and `git diff --check` passed after the formatting correction.
- Exact source delta against attempt-start tree `d23355e3a642878f98545598005a86f0b8684805`: 25 additions + 6 deletions = 31 changed lines. Existing W2 native mechanics and public behavior were otherwise untouched.

### Commands

- `cargo test --manifest-path src-tauri/Cargo.toml --lib restore_transitions --no-fail-fast`
- `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore --no-fail-fast`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `git diff --check`
- Windows-target `rustc --test --emit=metadata` harness over the actual private module and fetched `windows-sys` metadata; the temporary metadata output was removed.

## Next

Run the real-Windows local-NTFS product compile/test matrix, then reconcile the final unchecked W2 row. Apply remains partial and is not ready for `sdd-verify`.

## Windows evidence remediation (`w2-windows-remediation`)

- Structured status consumed/produced: authoritative OpenSpec artifacts are present, 24/25 implementation tasks remain checked, apply is ready/partial, and the parent-authorized edit boundary was limited to the transition module, `backup_restore.rs`, and this cumulative progress file. No action-context warning was present.
- Review boundary: stacked-to-main W2 remediation; delta against attempt-start tree `266523482f7ba9a8c8e28aa1df0137b17308bde7` is 111 additions + 22 deletions in `restore_transitions.rs`, 11 additions + 1 deletion in `backup_restore.rs`, and 16 additions in this note: 161 changed lines total, within the 300-line limit.
- RED: the new modeled reservation-race case failed because an atomically occupied slot aborted publication instead of advancing to the next slot.
- GREEN/TRIANGULATE: marker replacement now skips known occupied/stale slots and atomically reserves each candidate with `CREATE_NEW`; Windows keeps that empty reservation open with read/write/delete sharing through `ReplaceFileW`. Only `AlreadyExists` advances to another slot; failed or ambiguous replacement retains and consumes the reservation/evidence, and eight occupied slots fail without changing the current marker. Existing post-replacement root barriers remain unchanged. This is not adversarial-process protection.
- Corrected Windows-only native expectations and model tests cover arbitrary occupied evidence unchanged, stale empty reservations skipped, fresh reservation replacement with old marker evidence, exhaustion, and ambiguous-failure slot retention/non-reuse.
- Explicitly dropped live `DatabaseState`/SQLite owners before the seven evidence-identified cleanup calls, and corrected failed-restore recovery to retain `Some(CandidateInstalled)`.
- Verification passed: focused restore transitions 6/6; focused `backup_restore` 22/22; full current-host Rust suite; `cargo fmt --check`; and `git diff --check`.
- Windows-target `rustc --test --emit=metadata` over the actual private module and fetched `windows-sys 0.61.2` metadata passed with dead-code-only harness warnings. No Windows binary or test executed on this Linux host, so task 25 remains visibly unchecked and Windows acceptance is not claimed.
- External failed evidence reviewed from `/tmp/w2-evidence-review` corresponds to supplied ZIP SHA-256 `0b29f2552e54ec2adaa9f00e3d4264afa17f3ce1b866ceca9f09412c902f4ebc`. The approved `src-tauri/icons/icon.ico` remained byte-identical at SHA-256 `73da5451fa4724001bdaafe39ea3f0d41a205e64cb57ccbe0357e6990494dcfd`.

### Remaining implementation task (unchanged)

- [ ] Compile the Windows target and run the W2-focused tests after W0 approval; confirm W1 host tests remain green and inspect the final diff for dependency, marker-schema, public-error, handle-closure, and unrelated-backup regressions. <!-- sdd-owner: implementation -->

## ReplaceFileW ambiguity barrier remediation

- Structured status produced from authoritative hybrid/OpenSpec artifacts: proposal/spec/design/tasks/apply-progress are present, implementation progress remains 24/25, apply remains ready/partial, and the parent-authorized edit roots were exactly the transition module and this progress file with no warnings.
- Runtime authority was authenticated as the existing `w2-windows-remediation` attempt token `sha256:3569a6a6506bcc707df9256c0501ac14ab8dd5f8f34ac987b829e1c786405f68`; this correction stayed within that objective and did not alter task 25.
- RED: `cargo test --manifest-path src-tauri/Cargo.toml --lib restore_transitions -- --nocapture` failed because the ambiguous replacement path ended with `Replace` rather than attempting `SyncDirectory(root)`.
- GREEN: a non-`AlreadyExists` replacement error now attempts the root directory barrier best-effort, then returns `StorageUnavailable` regardless of the barrier result. The consumed reservation remains retained and is not reused.
- TRIANGULATE: the ambiguity model now proves the root barrier attempt and models the post-replacement marker plus preserved old-marker evidence as durable before retrying through the next reservation slot.
- Verification passed: focused restore transitions 6/6; focused `backup_restore` 22/22; full current-host Rust suite; `cargo fmt --check`; and `git diff --check`.
- No Windows metadata type-check was run in this correction, and no Windows runtime claim is made.
- Scope remained limited to `restore_transitions.rs` and this cumulative progress artifact; the native contract, task 25, icon, read-only integration test, and unrelated tests were unchanged.

### Remaining implementation task (unchanged)

- [ ] Compile the Windows target and run the W2-focused tests after W0 approval; confirm W1 host tests remain green and inspect the final diff for dependency, marker-schema, public-error, handle-closure, and unrelated-backup regressions. <!-- sdd-owner: implementation -->

## Closed-reservation native remediation

- Reviewed failed native Windows evidence SHA-256 `4856eb908731548a74d5797bdfac4219b5a22c895c7c9d8311aa372b636753e6`: `ReplaceFileW` failed with error 1175 while the `CREATE_NEW` backup reservation handle remained open, cascading four restore tests from a retained `Prepared` marker.
- Runtime authority was re-authenticated with the active `w2-windows-remediation` token. Authoritative OpenSpec status remains apply-ready/partial at 24/25 implementation tasks, with no action-context warning and the stacked W2 work-unit boundary retained.
- Windows replacement now atomically creates the known-empty backup path, requires successful handle closure, and then invokes `ReplaceFileW` without deleting the placeholder. Reservation collisions still surface as `AlreadyExists`; the close-to-replace convention is explicitly bounded to cooperating product processes, not adversarial mutation.
- The native test now names closed-reservation behavior and retains success, preserved old destination, second-call collision, and unchanged source/destination/evidence assertions. Non-Windows-only adapters and imports now use exact `not(windows)` cfgs.
- Added exactly three definition-local justified `#[expect]` annotations for the authorized existing Clippy baseline findings; no API, serialization, layout, or broad lint allowance changed.
- Host verification passed: focused restore transitions 6/6, focused `backup_restore` 22/22, full Rust suite, format check, and `git diff --check`.
- Strongest all-features Clippy remained host-dependency-blocked by missing GLib/GObject/GIO/GDK pkg-config packages. `cargo clippy --all-targets --locked -- -D warnings` reached project linting and found only additional pre-existing test-target lints outside the authorized surfaces; `cargo clippy --lib --locked -- -D warnings` passed with no product-library lint findings.
- Estimated correction delta before this progress note is 36 changed product lines; cumulative `w2-windows-remediation` delta is 223 changed lines against objective-start tree `266523482f7ba9a8c8e28aa1df0137b17308bde7`, below the 300-line authority limit. No Windows pass is claimed.

### Remaining implementation task (unchanged)

- [ ] Compile the Windows target and run the W2-focused tests after W0 approval; confirm W1 host tests remain green and inspect the final diff for dependency, marker-schema, public-error, handle-closure, and unrelated-backup regressions. <!-- sdd-owner: implementation -->

## Test-only Clippy correction

- Removed ineffective test-only `drop(repository)` and `drop(writer_factory)` calls after the full Rust suite confirmed borrow checking and behavior remain unchanged.
- Removed only two needless borrows around `directory.join("app-data")` in `backup_restore.rs`; no production API or allowance changed.
- Exact verification passed: full Rust suite, format check, `git diff --check`, and all-target locked Clippy with warnings denied.
- Source delta is four deletions across the three authorized test files. Estimated cumulative `w2-windows-remediation` size is 258 changed lines including progress bookkeeping, below the 300-line limit.
- Task 25 remains visibly unchecked; no settlement, commit, push, reset, or Windows acceptance claim occurred.

### Remaining implementation task (unchanged)

- [ ] Compile the Windows target and run the W2-focused tests after W0 approval; confirm W1 host tests remain green and inspect the final diff for dependency, marker-schema, public-error, handle-closure, and unrelated-backup regressions. <!-- sdd-owner: implementation -->

## Final Windows acceptance reconciliation

- Structured authoritative OpenSpec status advanced from apply-ready 24/25 to all-done 25/25; action context allowed only `tasks.md` and this progress file, with no warnings.
- Maintainer accepted `/tmp/evidence-w2-v4.zip`, SHA-256 `a5d334cf4ab115476c61fd756fcecd2415548d4b8d9a477b005cf3c2b583613e`; all 31/31 internal checksums passed.
- Evidence patch SHA-256: `1ec9be96dd2df1747b0edf8a3ea2fef5d9743d1a76f15f753b422cdcf8bf2253`.
- Runtime: Windows 10.0.26200.0 x64, PowerShell 5.1, ordinary non-admin, fixed local `C:` on NTFS.
- Focused native restore transitions passed 9/9; `backup_restore` passed 26/26.
- All-targets/all-features compiled; its tests recorded 24 passed and four unrelated pre-existing Tauri mock IPC positive-registration failures.
- Maintainer explicitly accepted those four IPC failures as an out-of-ticket-02 warning; W2 did not change command builder or registration.
- Strict all-targets/all-features Clippy, formatting, and diff checks passed; cleanup, process status, and redaction passed with no timeout.
- Acceptance is bounded to successful requested ordering on supported local Windows/NTFS; no physical power-loss proof is claimed, and the overall runner is not represented as passing.
- Persisted task reconciliation is complete at 25/25; next action is `sdd-verify`. No product code, runtime authority, commit, push, sync, or archive action occurred.

## Corrective slice B (`sha256:e8f06bda…`)
- Tasks 26–30 are checked; task 31 remains unchecked. `restore_transitions.rs` changed 193 additions and 92 deletions from slice baseline; exact focused (10/10, 22/22), full Rust, locked all-target Clippy, format, and diff checks passed. Windows phase tests now use UUID paths and the phase-aware helper; no Windows runtime is claimed. A non-required GNU cross-check remained environment-blocked before the crate by missing `x86_64-w64-mingw32-gcc`.

## Final Windows fixture correction (`sha256:e0a5d3f4…`)
- Replaced the recovery-temporary directory fixture with a regular file held open under read-only sharing through install and assertions, then dropped before cleanup.
- Host focused/full Rust tests and locked all-target Clippy passed; formatting passed after import-order correction, and `git diff --check` passed.
- Static Windows review confirms `OpenOptionsExt::share_mode(FILE_SHARE_READ)` permits inspection but omits delete sharing; no Windows runtime claim is made.
- Task 31 remains visibly unchecked; no product behavior, settlement, commit, push, reset, sync, or archive action occurred.

## Final Windows v6 evidence bookkeeping
- Bundle `/tmp/evidence-w2-v6.zip` SHA-256 `c8cfb644e1d7905f190583a1a53da9e91cb4a9456a971433a65204443fbe329b`; matching sidecar and 31/31 internal checksums passed.
- Product patch SHA-256 `7826bb4255bf30b9fda785786d02a4fd37991eda78c92ccfcf4926a697bbe8be`.
- Ordinary non-admin Windows x64 ran on fixed local `C:` NTFS.
- Native phase-aware transitions passed 13/13; `backup_restore` passed 26/26, including unavailable post-disruption recovery.
- All targets/features compiled; 28 passed and four maintainer-accepted unrelated IPC mock tests failed.
- Strict all-feature Clippy, formatting, diff, cleanup, process, and redaction checks passed; no physical power-loss claim is made.
- Task 31 is checked, persisted implementation progress is 31/31, and next is `sdd-verify`; no settlement occurred.
