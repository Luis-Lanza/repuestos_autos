# Proposal: Local Backup and Restore

## Intent

Provide offline, database-safe protection and recovery of operational data to USB/external storage or any writable local directory.

## Scope

### In Scope
- Create a consistent manual SQLite backup and return path, creation time, size, and schema version.
- Validate and stage a selected backup, migrate recognized older schemas, show metadata, and require confirmation through an opaque restore token.
- Create one validated protective backup, then close, swap, reopen, revalidate, and recover deterministically after interruption.
- Preserve catalog, confirmed sales, inventory balances, immutable movements, and schema history.
- Add lifecycle tests plus Fedora and Windows target smoke coverage.

### Out of Scope
- Cloud sync, scheduling, encryption, compression, cross-device merge, and removable-drive auto-detection.
- Backup browsing, retention policies, or advanced backup history beyond the latest protective pre-restore backup.

## Capabilities

### New Capabilities
- `local-backup-and-restore`: Manual snapshots, validated two-step restore, compatibility, rollback, and crash recovery.

### Modified Capabilities
- None. Existing domain behavior remains unchanged; this capability preserves its records.

## Approach

Use SQLite's Online Backup API behind a Rust application module. Snapshot locally under the database mutex, then release it before copying through a synced `.part` file and non-overwriting rename. Restore validates identity, integrity, foreign keys, and schema version in app-owned staging; only the stage may migrate. Confirmation creates `pre-restore.sqlite3`, records a marker, closes handles, swaps files with Windows-compatible steps, reopens, and clears the marker after validation.

## Affected Areas

| Area | Impact | Description |
|---|---|---|
| `src-tauri/src/{application/backup,infrastructure,commands}` | New/Modified | Snapshot, staging, swap, recovery, IPC |
| `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`, `src-tauri/capabilities/` | Modified | Replaceable state, backup/dialog support, least-privilege permissions |
| `src/commands/backup.ts`, `src/ui/backup/`, tests | New/Modified | Native pickers, confirmation UX, metadata, platform evidence |

## Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| Crash or file lock during destructive swap | Med | Marker, staged candidate, protective backup, deterministic recovery |
| Corrupt/incompatible input | Med | Read-only source checks; staged integrity, foreign-key, schema validation |
| Slow/removed/full destination | Med | Keep removable-storage I/O outside the live lock; finalize atomically |
| Platform-specific dialog/path behavior | Med | Fedora smoke; Windows drive, UNC, Unicode, lock, removal, and rename release gate |

## Rollback Plan

Disable the UI/commands and revert wiring. After a failed swap, recover and reopen the validated rollback database; retain `pre-restore.sqlite3`.

## Dependencies

- `rusqlite` backup feature and Tauri v2 dialog plugin with minimal desktop permission.

## Success Criteria

- [ ] Backup and confirmed restore preserve catalog, sales, balances, movements, and supported schema history.
- [ ] Invalid, future, version-zero, corrupt, interrupted, or inaccessible cases never replace recoverable live data.
- [ ] Fedora and Windows target smoke obligations pass.
