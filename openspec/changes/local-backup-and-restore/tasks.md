# Tasks: Local Backup and Restore

## Review Workload Forecast

| Field | Value |
|---|---|
| Estimated changed lines | Slice 1: +220/-20; Slice 2: +300/-30; Slice 3: +230/-20; Slice 4: +110/-10; total ~940 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | Slice 1 → Slice 2 → Slice 3 → Slice 4 |
| Delivery strategy | ask-on-risk |
| Chain strategy | stacked-to-main (maintainer selected) |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

### Authorized Slice 2 Split

- Slice 2A: tasks 2.1–2.2 only — coordinator, contracts, and behavior-first evidence.
- Slice 2B: tasks 2.3–2.4 only — replacement, state, markers, and recovery; excluded from 2A.

### Suggested Work Units

| Unit | Goal | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|---|---|---|---|---|---|
| 1 | SQLite/FileStore seams, snapshot, validation | Slice 1 | `cargo test --manifest-path src-tauri/Cargo.toml backup` | N/A: file-backed integration only | Revert `infrastructure/{sqlite,filesystem}/backup*` and fixtures |
| 2 | Coordinator, replaceable state, restore/recovery | Slice 2 | `cargo test --manifest-path src-tauri/Cargo.toml backup_restore` | N/A: injected crash markers exercise lifecycle | Revert `application/backup/*`, state, and lifecycle wiring |
| 3 | Tauri commands, dialog permission, React flow | Slice 3 | `npx tsx --test src/commands/backup.test.ts src/ui/backup/*.test.ts` | `npm run tauri:dev`: picker → prepare → confirm | Revert command/UI/plugin wiring only |
| 4 | Cross-platform smoke and preservation evidence | Slice 4 | `npm test && cargo test --manifest-path src-tauri/Cargo.toml` | Fedora and Windows `npm run tauri:dev`; drive/UNC/Unicode/removal/lock | Revert smoke tests/evidence without product code |

## Phase 1: Foundation and RED Seams

- [x] 1.1 RED: add `src-tauri/tests/backup_restore.rs` cases for stable codes, Fedora/drive/UNC/Unicode paths, cancellation, destination failure/existing file, synced `.part`, and no live mutation.
- [x] 1.2 GREEN: create `src-tauri/src/infrastructure/filesystem/{mod.rs,backup_store.rs}` with SHA-256, sync, non-overwrite publish, marker fault seam, and platform replacement primitives.
- [x] 1.3 RED: add v1–v6, version-zero/future, non-SQLite, corrupt, tampered, foreign-key-invalid, structurally invalid, source-unchanged, and coherent-active-work snapshot tests; prove lock release before transfer.
- [x] 1.4 GREEN: modify `src-tauri/src/infrastructure/sqlite/{mod.rs,backup.rs}` for current-version checks, required schema, integrity/FK checks, stage-only migration, Online Backup, and metadata.

## Phase 2: Restore Lifecycle

- [x] 2.1 RED: test `src-tauri/tests/backup_restore.rs` for token expiry/reuse/mismatch, confirmation omission, protective failure, and catalog/sales/balances/movements/schema-history preservation.
- [x] 2.2 GREEN: create `src-tauri/src/application/backup/{contracts.rs,mod.rs}` with coordinator, token binding/recheck, stable outcomes, and data-preservation assertions.
- [x] 2.3 RED: inject crashes at `prepared`, `live_moved`, and `candidate_installed`; assert close/swap/reopen, canonical-first recovery, rollback, unavailable state, no empty replacement, marker clearing, and retained `pre-restore.sqlite3`.
- [x] 2.4 GREEN: modify `src-tauri/src/lib.rs` with `DatabaseState` (`Option<Connection>`), recovery-before-open, protective replacement, command exclusion, and revalidation.

## Phase 3: IPC and React

- [x] 3.1 RED: add `src-tauri/tests/backup_restore.rs` command-surface tests for serde contracts, allowlisted operations, stable sanitized errors, and unavailable/restore rejection.
- [x] 3.2 GREEN: create/modify `src-tauri/src/commands/{backup.rs,mod.rs}`, `src-tauri/Cargo.toml`, `src-tauri/capabilities/default.json`, and `package.json`; register dialog plugin and commands.
- [x] 3.3 RED: add `src/commands/backup.test.ts` and `src/ui/backup/*.test.ts` for native picker cancellation, stable errors, summary/confirmation, loading, success, and failure flow.
- [x] 3.4 GREEN: create `src/commands/backup.ts`, `src/ui/backup/*`, and modify `src/ui/app.ts` using typed IPC and native dialogs only.

## Phase 4: Verification

- [ ] 4.1 Run Rust/TypeScript suites and Fedora/Windows smoke evidence for paths, media removal, antivirus/handle locks, rename release, and preserved operational facts.
