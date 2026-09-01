```yaml
schema: gentle-ai.verify-result/v1
evidence_revision: sha256:c8cfb644e1d7905f190583a1a53da9e91cb4a9456a971433a65204443fbe329b
verdict: pass_with_warnings
blockers: 0
critical_findings: 0
requirements: 10/10
scenarios: 26/26
test_command: cargo test --manifest-path src-tauri/Cargo.toml && cargo test --manifest-path src-tauri/Cargo.toml --lib restore_transitions -- --nocapture && cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore -- --nocapture
test_exit_code: 0
test_output_hash: sha256:54a3662a4c52d16370ce9def3ea8ef0e1bfb41bc8030f2954ee9b4e01e849e26
build_command: cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings && cargo fmt --manifest-path src-tauri/Cargo.toml -- --check && git diff --check
build_exit_code: 0
build_output_hash: sha256:4f27ac380b22e1d975eee20ce727fa8cbc09061177f6253af0777fd4de7b7506
```

# Verification Report: Durable Restore Filesystem Transitions

## Result

**PASS_WITH_WARNINGS.** The amended contract is complete at **31/31 implementation tasks, 10/10 requirements, and 26/26 scenarios**. Fresh host verification passed. Corrected real-Windows evidence passed the focused native and backup/restore suites on fixed local NTFS. The prior sidecar-slot exhaustion and recovery/completion precondition blockers are resolved. No archive blocker or critical finding remains.

Two accepted warnings remain: the all-target/all-feature Windows runner compiled and then failed four unrelated pre-existing Tauri mock IPC tests, and the historical cumulative W2 review footprint exceeded the 400-line budget without a recorded `size:exception`.

## Structured Status and Action Context

- Active change: `durable-restore-filesystem-transitions`, explicitly selected and present under OpenSpec.
- Native authoritative status: hybrid/OpenSpec; proposal, spec, design, tasks, apply-progress, and prior verify-report are present; apply is `all_done`; verify is `ready`; task progress is 31/31; `nextRecommended` is `verify`.
- The status blocker, “failed verification evidence is incomplete; rerun SDD verification,” referred to the superseded failed report and is resolved by this complete rerun.
- Action context: `repo-local`, workspace `/home/luis/velay/repuestos_autos`, native allowed root the workspace. Delegated write authority was narrower: only this verify report was writable. All other repository files and `/tmp/w2-evidence-v6-review/**` were read-only.
- Runtime token `sha256:73c63dadb87c47356b7e18d614c6b859d9d3d099c9dd02a24c743cacfa2cb6c2` authenticated to the existing attempt and returned `proceed` without a new acquisition.
- No settlement, product-code edit, planning-artifact edit, commit, push, sync, or archive was performed.

## Artifact and Task Completion

- Proposal: present and coherent with the bounded local-Windows/NTFS durability claim.
- Spec: present; authoritative amended count is 10 requirements and 26 scenarios.
- Design: present; private deep restore-transition module, phase verification, ordered barriers, bounded sidecars, and application settlement remain coherent with the implementation.
- Tasks: **31/31 checked**.
- Apply-progress: present and includes cumulative implementation, test, Windows evidence, and correction-slice records.
- Unchecked implementation markers matching `^\s*- \[ \]`: **none**.
- Exact unchecked implementation lines: **none**.

## Prior Blocker Resolution

### Sidecar lifecycle and slot reuse — resolved

- Exact marker bytes remain constants: `{"state":"prepared"}`, `{"state":"live_moved"}`, and `{"state":"candidate_installed"}`.
- Prepare performs phase verification first, requires active-marker absence, then runs an initial root barrier before any sidecar removal.
- Only `restore-state.json.previous-{0..7}` entries are accepted. Unknown names, malformed bytes, non-regular files, and reparse points fail closed before disruption.
- Empty sidecars are accepted as crash reservations. Occupied valid slots are removed in ascending order with an immediate root barrier after each removal; removal/barrier failure stops preparation while canonical and the live connection remain intact.
- The deterministic five-cycle test proves completed-cycle sidecars are recycled and only two slots remain occupied after the fifth completed cycle. Retry tests cover the initial barrier, removal, and per-removal barrier failure points.
- Recovery and Completion never recycle sidecars. Sidecar cleanup never removes rollback or protective databases. Fallback and completion retain rollback/protective evidence; the install protocol's existing old-rollback rotation remains the designed precondition to canonical-to-rollback movement while canonical and the independently durable protective source still exist.

### Recovery and Completion gates — resolved

- Recovery invokes phase-specific verification before stale-temporary removal, copy, canonical removal, or rename.
- Recovery requires the exact active marker path and exact accepted marker bytes, an exact rollback or protective source, exact canonical path, fixed local NTFS, same volume, regular non-reparse files, safe ancestors, and fallible presence checks.
- Completion invokes phase-specific verification after the newly opened validated canonical connection is published `Ready` and before marker mutation.
- Completion validates exact canonical/marker paths, local fixed NTFS, same-volume optional rollback/protective evidence, regular-file/reparse conditions, and a present marker's exact bytes.
- Marker absence is idempotent only after Completion verification succeeds. Unsupported, malformed, missing-required, wrong-path, cross-volume, reparse, and fallible-presence conditions return `StorageUnavailable` before mutation.
- SQLite source-validation connections and native inspection handles close before namespace mutation. Windows helper handles close through `with_handle`; stage/protective validation scopes close before Prepare; the live connection is dropped before install mutation.

### Windows unavailable fixture — resolved

- The Windows-only unavailable test creates `restore-recovery.sqlite3.part` as a regular file and holds it open with read-only sharing that omits delete sharing.
- The failure therefore occurs after disruption when Recovery attempts stale-temporary removal, exercising the intended native sharing failure rather than failing Prepare on an invalid fixture type.
- Corrected Windows `backup_restore` evidence passed 26/26, including this `database_unavailable` path and command gating with no stale connection.

## Requirement and Scenario Coverage

| # | Requirement | Scenarios | Result | Verification evidence |
|---:|---|---:|---|---|
| 1 | Durable Prepared Evidence Before Disruption | 2/2 | PASS | Prepare verification, stage/protective file syncs, stage ancestry/root barriers, and exact `Prepared` publication finish before `Restoring` and live-connection drop; failure remains pre-disruption. |
| 2 | Marker Progress Never Leads Durable Transitions | 3/3 | PASS | Canonical-to-rollback root barrier precedes `LiveMoved`; destination and stage-source ancestry barriers precede `CandidateInstalled`; modeled failpoints allow lag but not lead. |
| 3 | Marker Removal Follows Validated Readiness | 4/4 | PASS | Application publishes the newly opened validated connection as `Ready`; Completion then verifies the supported boundary and present/absent marker before removal; removal/barrier failure returns bounded failure while retaining Ready. |
| 4 | Retain Fallback Sources Through Recovery Copy | 2/2 | PASS | Recovery copies from exact rollback/protective source and never moves or deletes that source; fallback/completion leave database evidence intact after canonical validation. |
| 5 | Fail Closed on Required Durability Failure | 2/2 | PASS | Adapter errors map privately to `StorageUnavailable`; post-disruption settlement yields only `restore_failed` with a new validated connection or `database_unavailable` with no connection. |
| 6 | Preserve Exact Restore Marker Compatibility | 2/2 | PASS | Parser and constants retain the three exact payloads, marker filename, and state meanings; sidecars contain only exact payload bytes or empty crash reservations. |
| 7 | Bound the Durability Guarantee to Windows Local NTFS | 3/3 | PASS | Prepare, Recovery, and Completion verify fixed local NTFS, exact paths, same volume, regular-file type, reparse absence, lexical safety, and fallible presence before mutation; non-Windows production fails closed. |
| 8 | Recycle Completed Restore Marker Sidecars | 4/4 | PASS | Prepare-only validation, initial/per-removal barriers, malformed/unknown/reparse rejection, active-marker exclusion, retry behavior, empty reservation handling, and five completed cycles are covered and green. |
| 9 | Require Real Windows NTFS Runtime Evidence | 2/2 | PASS | Corrected non-admin Windows x64 evidence on fixed `C:` NTFS passed native phase transitions 13/13, `backup_restore` 26/26, Clippy, format, diff, cleanup, process, and redaction checks. |
| 10 | Preserve Existing Restore and Ticket 02 Boundaries | 2/2 | PASS | Canonical→rollback→protective recovery, Ready/Restoring/Unavailable ownership, command gating, bounded errors, backup publication, schema, migrations, marker compatibility, and IPC registration remain unchanged. |

**Coverage total: 10/10 requirements and 26/26 scenarios compliant.**

## Implementation Integrity and Scope Inspection

- **Deep module:** restore ordering and Windows mechanics remain private in `restore_transitions.rs`; `BackupStore` exposes only the four narrow restore operations and `DatabaseState` retains connection/state settlement.
- **Marker states:** exact bytes and all three existing states are unchanged; no version, alternate encoding, or public recovery filename was added.
- **Fail-closed mapping:** all native/verification failures collapse to private `StorageError::StorageUnavailable`; only existing `restore_failed` and `database_unavailable` outcomes cross the application/command seam.
- **Handle closure:** validation and native inspection/copy/sync handles close before remove, rename, replacement, cleanup, or subsequent namespace operations.
- **Recovery retention:** fallback copies through an exclusive temporary, synchronizes file and namespace, retains its source and active marker, and validates a newly opened canonical connection before Ready.
- **Bounded guarantee:** successful requested operations are claimed only for supported fixed local Windows/NTFS. No arbitrary hardware, controller-cache, storage-device, or physical-power-loss immunity is claimed.
- **No schema/backup regression:** no schema or migration file changed; backup contents, destinations, SQLite online-backup behavior, WAL policy, and unrelated backup publication remain unchanged.
- **No public error/IPC regression:** no public error code, command DTO, command registration list, `command_builder`, or desktop `generate_handler!` list changed. The four Windows IPC failures occur in unchanged command-registration tests.
- **Dependency scope:** direct `windows-sys = 0.61.2` remains target-only with the four approved feature groups.

## Fresh Host Test and Validation Commands

All requested commands were run fresh from `/home/luis/velay/repuestos_autos`.

| Exact command | Exit | Result |
|---|---:|---|
| `cargo test --manifest-path src-tauri/Cargo.toml` | 0 | PASS; 173 tests passed, 0 failed, plus doc tests 0/0. |
| `cargo test --manifest-path src-tauri/Cargo.toml --lib restore_transitions -- --nocapture` | 0 | PASS; 10/10 focused host tests. |
| `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore -- --nocapture` | 0 | PASS; 22/22 on the non-Windows host. |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings` | 0 | PASS. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | 0 | PASS. |
| `git diff --check` | 0 | PASS. |

Combined fresh test-output SHA-256: `54a3662a4c52d16370ce9def3ea8ef0e1bfb41bc8030f2954ee9b4e01e849e26`.

Combined fresh Clippy/format/diff-output SHA-256: `4f27ac380b22e1d975eee20ce727fa8cbc09061177f6253af0777fd4de7b7506`.

## Corrected Windows Evidence

- ZIP: `/tmp/evidence-w2-v6.zip`.
- ZIP SHA-256: `c8cfb644e1d7905f190583a1a53da9e91cb4a9456a971433a65204443fbe329b`, matching `/tmp/evidence-w2-v6.zip.sha256`.
- ZIP safety: 32 entries; no absolute/traversal/backslash names and no symlink entries.
- Extracted review directory: `/tmp/w2-evidence-v6-review`.
- Internal manifest: **31/31 checksums passed**.
- Product patch attestation: SHA-256 `7826bb4255bf30b9fda785786d02a4fd37991eda78c92ccfcf4926a697bbe8be`.
- Runtime: Microsoft Windows `10.0.26200.0`, x64 process on x64 OS, PowerShell 5.1, ordinary non-admin.
- Storage: local fixed `C:` drive (`DriveType 3`) on NTFS.
- Native phase-aware restore transitions: **13/13 passed**.
- `backup_restore`: **26/26 passed**.
- `cargo test --manifest-path src-tauri/Cargo.toml --all-targets --all-features`: compilation succeeded; 28 tests passed and four unrelated pre-existing Tauri mock IPC tests failed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings`: passed.
- Format and `git diff --check`: passed.
- Cleanup/process/redaction: passed; every process exited; zero timeouts; zero redaction findings.
- Physical power loss: explicitly not claimed.

## Strict TDD and Assertion Quality

- Strict TDD is inactive (`openspec/config.yaml`: `strict_tdd: false`), so strict-TDD compliance is not a release gate.
- Apply-progress nevertheless contains a `TDD Cycle Evidence` table, and the reported test files exist.
- Corrective RED/GREEN/TRIANGULATE records are present for lifecycle cleanup, phase verification, reservation ambiguity, and the Windows sharing fixture.
- Changed tests assert exact marker bytes, exact operation order, fail-closed errors, mutation absence, retained evidence, repeated-cycle availability, supported-boundary decisions, Ready/Unavailable state, and command gating.
- No tautology, ghost loop, type-only assertion, smoke-only assertion, or CSS implementation-detail assertion was found.

## Review Workload and PR Boundary

- Tasks recommended chained delivery using `W1A → W1B → W2` and `stacked-to-main`; apply-progress consistently records that logical boundary and no branch/PR/commit was created by the executor.
- Recorded W1A and W1B slices remained below 400 changed lines.
- The historical cumulative W2 implementation/test/config delta reached 570 changed lines against the W1B tree, above both its forecast and the 400-line review budget, with no recorded `size:exception`.
- Later corrective slices were separately bounded and remained within their assigned correction limits; they do not erase the historical cumulative W2 review warning.
- Final correction work stayed focused on sidecar lifecycle, phase verification, Windows native behavior/fixture coverage, and required evidence bookkeeping. No command registration, IPC behavior, schema, migration, or unrelated backup behavior was added to the correction scope.

## Warnings

1. The corrected all-target/all-feature Windows runner exited 101 after successful compilation because four unchanged Tauri mock IPC positive-registration tests failed: `registers_catalog_maintenance_listing_at_the_tauri_command_seam`, `registers_metadata_edit_and_detail_commands_at_the_tauri_command_seam`, `registers_post_sale_commands_at_the_tauri_command_seam`, and `registers_read_only_sales_history_commands_at_the_tauri_command_seam`. The maintainer accepted this as an unrelated pre-existing warning; the overall runner is not represented as passing.
2. The historical cumulative W2 review footprint exceeded 400 changed lines without a recorded `size:exception`; later bounded correction slices do not retroactively remove that review-workload warning.

## Blockers

**None.** No unchecked implementation task, failed requirement, failed scenario, unsupported acceptance claim, implementation-ownership ambiguity, or action-context violation remains.

## Evidence Revision Recommendation

Use `sha256:c8cfb644e1d7905f190583a1a53da9e91cb4a9456a971433a65204443fbe329b` as the immutable evidence revision for this passing verification because it identifies the corrected Windows v6 bundle whose sidecar, ZIP safety, 31/31 internal checksums, environment proof, focused runtime suites, and validation outputs were independently checked. The attested product patch revision is `sha256:7826bb4255bf30b9fda785786d02a4fd37991eda78c92ccfcf4926a697bbe8be`.

## Next Recommendation

**Ready for parent-owned settlement, then sync/archive according to SDD policy.** This executor did not settle the attempt.
