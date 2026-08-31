## Exploration: Local Backup and Restore

### Current State
The product is an offline, single-computer Tauri 2 application with one process and one SQLite source of truth. At startup, `src-tauri/src/lib.rs` resolves Tauri's platform-specific application-data directory, derives `repuestos-autos.sqlite3`, opens one `rusqlite::Connection`, runs forward migrations through schema version 6, and stores that connection behind `Mutex<Connection>`. Every current database command takes the same mutex, so application commands are serialized and no in-process write can overlap a backup or restore while that lock is held. SQLite foreign keys are enabled, but WAL mode and a busy timeout are not configured.

The product and architecture documents already define the intended capability: a database-safe export selected by the operator, validation and explicit confirmation before restore, and a controlled database close/reopen. No backup command, application module, filesystem adapter, dialog plugin, capability file, or UI exists yet; command-surface tests deliberately prove backup and restore are unavailable.

The live path is owned by `production_database_config(app.path().app_data_dir())`, not by the UI. On Fedora and Windows this lets Tauri resolve the appropriate per-application data directory without hardcoded separators or drive assumptions. A USB drive or external disk is therefore just another operator-selected writable directory. The official Tauri dialog plugin supports Linux and Windows; Fedora can use its default GTK backend or the optional XDG Portal backend, while Windows uses the native picker. The project currently has neither `tauri-plugin-dialog` nor `@tauri-apps/plugin-dialog`, and Tauri v2 plugin permissions must be granted explicitly.

The current migration seam rejects future `user_version` values above 6 and validates selected legacy shapes and foreign keys while migrating. It does not expose a read-only “is this a Repuestos Autos backup?” validator, and opening a candidate through `open_database` would mutate it by running migrations. Restore therefore needs an app-owned staged copy: validate and migrate the stage, never the operator's source file.

SQLite's Online Backup API is the safe snapshot primitive. `rusqlite` exposes it behind the `backup` feature, which is not enabled today. A raw filesystem copy is unsafe as a general design because SQLite may have active transactions or journal/WAL sidecars, even though the present single connection and default journal mode reduce that risk.

### Affected Areas
- `src-tauri/src/lib.rs` — App state must own both the database path and a replaceable connection; register backup/restore commands and the dialog plugin.
- `src-tauri/src/application/backup/` — New deep module for create, prepare-restore, confirmed restore, stable outcomes, and orchestration.
- `src-tauri/src/infrastructure/sqlite/mod.rs` — Expose the current schema version and reusable open/validation behavior without mutating an operator-selected source.
- `src-tauri/src/infrastructure/filesystem/` — App-local staging, destination `.part` files, atomic rename, protective backup, restore marker, and startup recovery.
- `src-tauri/src/commands/backup.rs` — Narrow Tauri request/response adapter with opaque error codes and a two-step restore contract.
- `src-tauri/Cargo.toml` — Enable `rusqlite/backup` and add the Tauri dialog plugin under the desktop feature.
- `src-tauri/capabilities/default.json` — Grant only the native dialog permissions needed by the frontend; backend Rust performs file I/O, so broad frontend filesystem permissions are unnecessary.
- `package.json` and `src/commands/backup.ts` — Add dialog bindings and typed, allowlisted IPC adapters.
- `src/ui/app.ts` and `src/ui/backup/` — Add a backup/restore screen, directory/file pickers, validation summary, explicit destructive confirmation, busy state, and outcomes.
- `src-tauri/tests/backup_restore.rs` — File-backed lifecycle, corruption/version rejection, failure recovery, and catalog/stock/sales survival proof.
- `src-tauri/tests/command_seam.rs` and `src-tauri/src/lib.rs` command-surface tests — Replace the intentional exclusion with registered-command and opaque-contract evidence.
- `src/commands/backup.test.ts` and `src/ui/backup/*.test.ts` — Payload allowlisting, cancellation, confirmation gating, and UI state tests through injected dialog/command adapters.

### Approaches
1. **Online snapshot plus staged, recoverable restore** — Snapshot the live connection to an app-local temporary SQLite file, validate it, then copy the completed file to the selected directory. For restore, copy the selected source into app-owned staging through SQLite's backup API, validate/migrate the stage, return an opaque restore token for confirmation, then create a protective snapshot, close, swap, and reopen under the database mutex.
   - Pros: Consistent snapshots; source backup is never mutated; USB failure cannot damage the live database; confirmation is TOCTOU-safe because it refers to a staged candidate; supports known older schemas through existing migrations; enables rollback and startup recovery; works on Fedora and Windows.
   - Cons: Requires replaceable connection state, staging cleanup, a restore marker, careful multi-step filesystem recovery, and more tests.
   - Effort: High

2. **Direct online backup into and out of the live connection** — Back up directly to the selected destination and restore the selected file directly into the current connection with SQLite's backup API.
   - Pros: Less filesystem swap code and no raw database-file replacement.
   - Cons: Slow/removable storage holds the database lock; source changes or removal between validation and confirmation remain possible; controlled close/reopen is not explicit; failure and power-loss recovery are harder to reason about; destination may be partially created.
   - Effort: Medium

3. **Close and copy database files with filesystem operations** — Lock and close the connection, copy the main database file, replace it on restore, and reopen.
   - Pros: Few dependencies and simple happy-path code.
   - Cons: Fragile around journal/WAL sidecars, active external access, partial writes, USB removal, Windows open-file semantics, and crash windows; copies can look successful while being inconsistent.
   - Effort: Low initially, High to make safe

### Recommendation
Use **online snapshot plus staged, recoverable restore** behind one small application interface. The UI should know only how to select a directory/file, request preparation, display the validated summary, and submit the returned opaque restore token after explicit confirmation. The backup module should hide SQLite snapshotting, version compatibility, staging, protective backup, path handling, and recovery.

Bound the first slice as follows:

- **Create backup:** choose any writable directory; create an app-local online snapshot while holding the database mutex; release the mutex before copying to slow/removable storage; copy to a unique `.part` path, sync/close it, validate it, and rename it to a non-overwriting timestamped `.sqlite3` filename. Return the final path, schema version, size, and creation time.
- **Prepare restore:** choose one backup file; reject the live database path; open the source read-only; copy it to app-owned staging with the Online Backup API; reject `user_version == 0`, future versions, corruption, foreign-key violations, or an unrecognized schema; migrate only the stage through the existing migration path; then run full `integrity_check`, `foreign_key_check`, current-version, and required-schema checks. Return a summary and opaque in-memory restore token. Do not use a boolean `confirmed` flag as the confirmation contract.
- **Confirmed restore:** lock the one database state so all catalog, inventory, and sales commands are excluded; create and validate one fixed app-local `pre-restore.sqlite3` protective snapshot; write a restore marker; close the live connection; rename the live file to a rollback path and the staged candidate to the canonical path using Windows-compatible non-overwriting steps; reopen through `open_database`; revalidate; and only then clear the marker. On any error, restore and reopen the prior database before returning an opaque failure.
- **Startup recovery:** if a restore marker remains after a crash, select only a validated canonical or rollback database according to deterministic recovery rules; never create a fresh empty database over recoverable data.
- **Compatibility:** accept only recognized application schema versions `1..=CURRENT_SCHEMA_VERSION`; reject version 0 and future versions. Older recognized backups are upgraded only in staging. This is migration compatibility, not advanced backup history.
- **Scope:** include manual backup, manual restore, one latest protective pre-restore backup, native pickers, validation, reopen, and survival tests. Exclude cloud sync, schedules, encryption, compression, retention management, backup browsing/history, automatic removable-drive detection, and cross-device merge.

The current single `Mutex<Connection>` is an advantage for correctness, but the state should become a database holder with `Option<Connection>` and its `DatabaseConfig`, so close/swap/reopen is representable without exposing a temporary sentinel connection. Long USB copying must occur after releasing the database lock. A small internal filesystem seam is justified because production and fault-injecting adapters are both needed to prove rename, disk-full, permission, and interruption recovery.

Testing should use real file-backed temporary SQLite databases rather than only in-memory fixtures. One vertical test should create an operator product, record stock movement, confirm a sale, create a backup, mutate all three areas, prepare and confirm restore, reopen the canonical database, then prove catalog search, stock balance/movements, and the persisted sale/idempotent retry match the backup snapshot. Additional tests should cover corrupt/non-SQLite files, wrong/future/version-zero schemas, foreign-key failures, USB/source disappearance, unwritable/full destinations, destination-name collision, reopen failure, interrupted swap recovery, and preservation of the protective backup. Fedora needs a native-dialog smoke test now; Windows needs a release-gate smoke test for drive-letter/UNC-style paths, Unicode paths, removable-drive removal, antivirus/file locks, and rename behavior.

### Risks
- Restore is destructive and spans SQLite plus filesystem state; without staging, a marker, and deterministic startup recovery, a crash between renames can make the canonical path disappear.
- The existing `Mutex<Connection>` serializes correctness but can freeze all database commands if removable-storage I/O occurs while locked.
- The selected backup may change or disappear between validation and confirmation; staging plus an opaque token is required to eliminate that race.
- `integrity_check` does not detect foreign-key violations, so both it and `foreign_key_check` are required, along with application schema/version recognition.
- Windows will not replace an open destination file with Unix-style rename semantics; every SQLite handle and statement must be dropped before the non-overwriting swap sequence.
- Fedora dialog behavior depends on the packaged GTK or XDG Portal backend; Windows behavior cannot be considered proven by Linux tests.
- App-local staging and the protective backup require enough free space for multiple database-sized files; capacity failures must happen before closing the live connection.
- `openspec/config.yaml` names the prior change and contains a stale 350-line task-rule sentence despite the active 400-line budget; later planning should use the session's 400-line budget without modifying that unrelated configuration during exploration.

### Ready for Proposal
Yes. The proposal should preserve the bounded staged-snapshot approach, explicit two-step confirmation, one protective pre-restore backup, deterministic reopen/recovery, recognized-version policy, and cross-platform smoke-test obligations. No current evidence requires cloud, scheduling, encryption, or advanced history.
