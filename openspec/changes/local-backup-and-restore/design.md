# Design: Local Backup and Restore

## Technical Approach

Add a deep Rust `backup` application module with three operator operations—create, prepare, confirm—and startup recovery. React selects paths and renders outcomes; Tauri adapts IPC; Rust owns path validation, SQLite work, replacement, and stable errors. All copies preserve the complete SQLite image, including immutable inventory movements, sales, balances, catalog, and migration history.

## Architecture Decisions

| Option | Tradeoff | Decision |
|---|---|---|
| Local Online Backup snapshot, then removable-media copy | Uses app-local temporary space but minimizes live lock time | Snapshot to `backup-restore/snapshots/*.sqlite3` under the database mutex; validate, release the mutex, copy to a synced `.part`, validate, then non-overwriting rename. |
| Staged restore with opaque token | More lifecycle state but removes source mutation and confirmation TOCTOU | Copy the read-only selected database through Online Backup into `backup-restore/staging/<uuid>.sqlite3`; validate/migrate only that stage. Bind an in-memory single-use, expiring token to stage path, SHA-256, size, and schema version; recheck before restore. No sidecar metadata file. |
| Replaceable database holder | Existing `Mutex<Connection>` cannot represent closed/recovery-failed states | `DatabaseState` owns `DatabaseConfig`, `Option<Connection>`, and status behind one mutex. Its small `read`/`write` interfaces reject commands while restoring or unavailable; no sentinel database is created. |
| Marker-driven swap | No portable atomic exchange exists; Windows refuses replacement while handles remain open | Persist `restore-state.json` (`prepared`, `live_moved`, `candidate_installed`) via temp-write, `sync_all`, and rename. Drop every statement/connection, rename canonical to `restore-rollback.sqlite3`, then stage to canonical. Linux/Fedora uses atomic same-directory rename; Windows uses non-overwriting moves and opens only after all handles are released. |
| Minimal native-dialog permission | Frontend sees selected paths, but broad filesystem access is avoided | Register `tauri-plugin-dialog`; grant `core:default` and `dialog:allow-open` only. Rust performs all file I/O; no shell or frontend filesystem permission. |

## Data Flow

```text
React picker -> typed command -> BackupCoordinator -> SQLite/FileStore
 create: lock -> local snapshot -> unlock -> .part copy/sync/check -> publish
 restore: source -> stage/check/migrate -> token -> confirm -> protect/mark/close/swap/reopen/check
 startup: marker -> valid canonical ? keep : valid rollback/pre-restore ? recover : stay closed
```

The protective `pre-restore.sqlite3` is an Online Backup snapshot refreshed through the platform file-replacement adapter and fully validated before any live handle closes. Startup never uses create-capable SQLite open flags during recovery. It validates canonical first, then `restore-rollback.sqlite3`, then the protective backup; failure leaves `DatabaseState` unavailable and retains evidence. A successful reopen uses normal PRAGMAs/migrations, reruns integrity, foreign-key, version, and required-schema checks, clears the marker, and retains `pre-restore.sqlite3`.

Candidate validation requires `user_version` in `1..=CURRENT_SCHEMA_VERSION`, recognized tables/columns/triggers, `integrity_check`, empty `foreign_key_check`, and the same checks after staged migration. Version zero, future, non-SQLite, structurally foreign, and checksum-changed stages are rejected.

## File Changes

| File | Action | Description |
|---|---|---|
| `src-tauri/src/application/backup/{mod.rs,contracts.rs}` | Create | Coordinator, token registry, metadata, stable outcomes. |
| `src-tauri/src/infrastructure/sqlite/{mod.rs,backup.rs}` | Modify/Create | Current version, read-only validation, migration-on-stage, Online Backup. |
| `src-tauri/src/infrastructure/filesystem/{mod.rs,backup_store.rs}` | Create | Synced files, SHA-256, platform replacement, marker, fault seam. |
| `src-tauri/src/commands/backup.rs`, `src-tauri/src/commands/mod.rs` | Create/Modify | Narrow serde adapters. |
| `src-tauri/src/lib.rs` | Modify | `DatabaseState`, recovery-before-open, plugin and command wiring. |
| `src-tauri/Cargo.toml`, `src-tauri/capabilities/default.json`, `package.json` | Modify/Create | `rusqlite/backup`, checksum/dialog dependencies, least privilege. |
| `src/commands/backup.ts`, `src/ui/backup/*`, `src/ui/app.ts` | Create/Modify | Allowlisted IPC, picker seam, confirmation flow. |
| `src-tauri/tests/backup_restore.rs`, `src/commands/backup.test.ts`, `src/ui/backup/*.test.ts` | Create | Lifecycle and UI contract evidence. |

## Interfaces / Contracts

`BackupCoordinator::{create(destination), prepare(source), confirm(token), recover_on_startup}` returns tagged results. Create returns final path, UTC creation time, size, and schema version; prepare returns those facts plus the token. Stable codes are `selection_cancelled`, `storage_unavailable`, `destination_exists`, `invalid_backup`, `unsupported_schema`, `confirmation_required`, `token_invalid`, `token_expired`, `restore_failed`, `recovery_failed`, and `database_unavailable`; responses expose no SQL or internal app paths. Paths remain native `PathBuf` values, preserving Fedora Unicode and Windows drive, UNC, and Unicode forms without separator rewriting.

## Testing Strategy

| Layer | What to Test | Approach |
|---|---|---|
| Unit | Names, tokens, codes, marker transitions | Inject clock, token generator, and faulting `FileStore`. |
| Integration | Coherent backup; v1–v6 staging; rejection checks; full fact preservation; every crash point | Real file-backed SQLite and failure injection; assert USB copy occurs after mutex release. |
| Platform/E2E | Pickers, locks, paths, removal | Fedora GTK/portal smoke; Windows drive/UNC/Unicode, antivirus lock, handle release, and rename release gate. |

## Threat Matrix

N/A — no routing, shell, subprocess, VCS/PR automation, executable-file classification, or process-integration boundary.

## Migration / Rollout

No schema migration required. Ship behind the new screen/commands; implementation should be sliced to stay reviewable under the 400-line budget. No cloud, scheduling, encryption, compression, retention, or backup history is introduced.

## Open Questions

None.
