# SQLite Crash Safety

## Forward Migrations

Follow `src-tauri/src/infrastructure/sqlite/mod.rs` and `src-tauri/src/infrastructure/sqlite/migrations/`:

1. Read `PRAGMA user_version`.
2. Reject a version greater than `CURRENT_SCHEMA_VERSION`.
3. Apply only the next numbered migration.
4. Validate compatibility before any restrictive change.
5. Execute schema SQL, postconditions, and the new `user_version` in one transaction.
6. Commit only after required columns, indexes, triggers, and foreign keys validate.
7. Add a new migration for corrections; never edit an applied migration.

Test a fresh database, every supported predecessor, incompatible legacy data, foreign-key violations, and a newer unsupported version.

## Transaction Recovery

Let SQLite own atomic rollback. Keep every related write on the same `rusqlite::Transaction`; return errors before commit and never continue with compensating writes outside that transaction. Verify failed operations leave counts, balances, lifecycle facts, and request reservations unchanged.

## Online Backup

Follow `src-tauri/src/infrastructure/sqlite/backup.rs`:

- Create a consistent live snapshot with `rusqlite::backup::Backup`.
- Run backup in bounded page steps when responsiveness matters.
- Validate `PRAGMA integrity_check`, schema version, required schema objects, and foreign keys on the destination.
- Never use a raw filesystem copy of an open WAL-backed database.

## Restore and Crash Recovery

Follow `DatabaseState::install_validated_stage` and `DatabaseState::recover_on_startup` in `src-tauri/src/lib.rs`, plus `src-tauri/src/infrastructure/filesystem/backup_store.rs`:

1. Copy the selected source into a stage through SQLite backup and validate it.
2. Create and validate a protective snapshot of the live connection.
3. Persist a restore marker before moving the canonical database.
4. Move the live database to a rollback location.
5. Persist marker progress before installing the staged candidate.
6. Open and fully validate the canonical candidate.
7. Mark the database ready, then clear the marker.

On startup with a marker, validate the canonical database first. If invalid or absent, recover from the first validated rollback or protective source. If no candidate validates, keep storage unavailable; never silently create a fresh database over recoverable evidence.

Test interruption after each durable marker transition and verify startup selects only a valid database.
