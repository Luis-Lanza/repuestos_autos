# Local Backup and Restore Specification

## Purpose

Provide safe offline snapshots and validated recovery for SQLite operational data.

## Requirements

### Requirement: Selected paths and stable outcomes

The system MUST use native pickers for writable local or removable directories and backup files. Fedora and Windows drive-letter, UNC, and Unicode paths MUST work without separator assumptions or auto-detection. Results MUST expose stable actionable categories for cancellation, storage access, validation, confirmation, restore, and recovery failures without SQL or internal paths.

#### Scenario: Selection or storage succeeds or fails

- GIVEN the operator selects a valid Fedora or Windows path, or cancellation/media failure occurs
- WHEN the operation completes
- THEN the exact valid path is used, or a matching stable outcome is returned with live data unchanged

### Requirement: Consistent safe backup

The system MUST use SQLite Online Backup for one consistent snapshot, hold the live database lock only during snapshot creation, and transfer after releasing it. It MUST publish only a synced, validated, non-overwriting file with a safe timestamped `.sqlite3` name.

#### Scenario: Backup succeeds during activity

- GIVEN catalog, sales, or inventory commands surround snapshot creation
- WHEN backup completes
- THEN one coherent committed state is stored and its result includes path, creation time, size, and schema version

### Requirement: Staged validation and compatibility

The system MUST copy a selected source into app-owned staging before checking or migrating it and MUST never mutate the source. It MUST accept only recognized application SQLite schemas `1..=current`; version `0`, future, corrupt, tampered identity/metadata, foreign-key-invalid, or structurally invalid candidates MUST fail. `integrity_check`, `foreign_key_check`, required-schema, and post-migration version checks MUST pass; migration MAY affect only staging.

#### Scenario: Unsafe backup

- GIVEN a source is non-SQLite, inaccessible, incompatible, corrupt, tampered, or fails a check
- WHEN preparation runs
- THEN a stable validation error is returned and no live file or record changes

### Requirement: Explicit staged confirmation

Preparation MUST return a summary and opaque, single-use token bound to the validated stage. Restore MUST require that token plus explicit operator confirmation; cancellation, omission, expiry, or reuse MUST do nothing destructive.

#### Scenario: Confirm prepared stage

- GIVEN preparation succeeded and the operator confirms its summary
- WHEN the valid token is submitted
- THEN restoration starts for that exact stage

### Requirement: Protective replacement lifecycle

Before replacement, the system MUST create and validate one latest app-local `pre-restore.sqlite3`; failure MUST abort before closing live data. On success it MUST mark the operation, exclude database commands, close all handles, swap the validated stage, reopen normally, revalidate, and clear the marker only after success.

#### Scenario: Confirmed restore succeeds

- GIVEN the protective backup and stage validate
- WHEN replacement completes
- THEN the database is reopened and validated, the marker is cleared, and the protective backup remains

### Requirement: Deterministic crash recovery

At startup with an uncleared marker, the system MUST validate the canonical database and keep it if valid; otherwise it MUST restore a valid rollback database. If neither validates, it MUST leave the database unopened and report recovery failure, never creating an empty replacement.

#### Scenario: Crash during replacement

- GIVEN the process stops during a marked restore
- WHEN the application starts
- THEN it keeps valid canonical data, otherwise restores valid rollback data, reopens it, and reports recovery status

### Requirement: Operational data preservation

A successful restore MUST preserve catalog records, confirmed sales, stock balances, immutable movement history, and supported schema history with their relationships and values.

#### Scenario: Restore known operational facts

- GIVEN the validated backup contains catalog, sale, balance, movement, and schema-history facts
- WHEN restore is confirmed and reopened
- THEN those facts and relationships match the backup snapshot
