# Exploration: Recover After Live Restore Failure

## Scope

Roadmap ticket 01 requires a failed live restore to leave the running process attached to a validated usable database or in an explicit bounded unavailable state. It excludes redesign of the durable filesystem transition protocol, backup formats, and destinations; those remain ticket 02 concerns.

## Current flow

`commands::backup::confirm_restore` consumes the pending token, rechecks the staged file checksum, and calls `DatabaseState::install_validated_stage`. The state method holds the database mutex for the full transition and currently:

1. creates and validates `pre-restore.sqlite3` from the active connection;
2. writes the `Prepared` marker;
3. sets status to `Restoring` and drops the active connection;
4. moves the canonical file to `restore-rollback.sqlite3`;
5. records `LiveMoved`, installs the stage, records `CandidateInstalled`;
6. opens and validates the canonical candidate, marks the state `Ready`, and clears the marker.

`with_read` and `with_write` reject every state other than `Ready`, so the mutex/status gate already prevents commands from using the dropped connection during restore. Startup recovery validates canonical storage first, then rollback and protective sources, and otherwise constructs an `Unavailable` state.

## Failure finding

Every `?` after the connection is dropped exits without repairing the in-memory state. The mutex then retains `status = Restoring` and `connection = None` for the lifetime of the process. This can happen when moving the live file, updating a marker, installing the staged file, opening or validating the candidate, or completing related storage work. The command returns the bounded `restore_failed` response, but the process has not attempted to recover and subsequent commands return `database_unavailable` until restart.

The significant post-disruption cases are:

| Failure point | Durable evidence likely available | Current running state |
| --- | --- | --- |
| Move live to rollback fails | Canonical usually remains valid; marker is `Prepared` | Stuck `Restoring` |
| Record `LiveMoved` fails after move | Rollback and protective copies exist; marker may still be `Prepared` | Stuck `Restoring` |
| Install stage fails | Rollback and protective copies exist; marker is `LiveMoved` | Stuck `Restoring` |
| Record/open/validate candidate fails | Canonical candidate may exist; rollback and protective copies exist | Stuck `Restoring` |
| Clear marker fails after candidate validation | Valid candidate connection is already `Ready` | Usable, though restore reports failure |

No stale `rusqlite::Connection` is currently exposed: the old connection is taken and dropped while the mutex is held, and all other operations wait for the mutex. The defect is failure to establish a valid replacement connection or intentionally transition from `Restoring` to `Unavailable` before returning.

## Recommended direction for proposal

Keep the existing durable marker/file sequence unchanged. Add an in-process recovery step for errors that occur after the active connection is disturbed, reusing the same validated candidate ordering as startup recovery:

1. while retaining exclusive state ownership, attempt to recover/open a validated canonical database using the existing canonical-first, rollback-second, protective-third policy;
2. install only that newly opened and validated connection and set status to `Ready`;
3. return the original bounded restore failure, so a failed restore is not reported as successful merely because recovery succeeded;
4. if recovery cannot establish a valid connection, explicitly set `connection = None` and `status = Unavailable`, then return `database_unavailable` at the command boundary;
5. never restore the previously dropped connection object or permit `Ready` without a validated connection.

This is recovery orchestration around the current protocol, not a change to marker meanings, rename/copy durability, directory syncing, WAL transition handling, or recovery-source retention.

## Likely seams and tests

Likely implementation seams are `DatabaseState::install_validated_stage` and a small refactor of `recover_marked_database` in `src-tauri/src/lib.rs`; `commands/backup.rs` should continue mapping only stable safe codes. Focused coverage belongs in `src-tauri/tests/backup_restore.rs`.

Tests should deterministically force at least one error after the active connection is dropped, then assert:

- the restore response remains a safe bounded error with no path, SQLite text, or storage internals;
- a normal command/read succeeds against the recovered prior database when a valid canonical, rollback, or protective source is available;
- the rejected candidate is not observed through the recovered connection;
- when every recovery source is invalid/unavailable, later reads and writes consistently return `database_unavailable`;
- no command can pass through a stale connection while status is `Restoring` or `Unavailable`.

A direct `install_validated_stage` test can induce a post-move installation failure with a missing/unusable stage after the live database has been moved. If production-path fault precision is needed, introduce only a narrow test seam rather than redesigning `BackupStore`.

Verification should use `cargo test --manifest-path src-tauri/Cargo.toml` with focused restore tests first, then the relevant Rust suite.

## Risks and boundaries

- Recovery code must avoid `open_database` creating a fresh empty canonical database when recoverable evidence exists; validation and source selection must precede any normal open that could create storage.
- Recovery failure must overwrite `Restoring` with `Unavailable`; leaving the transitional state is not an explicit terminal outcome.
- Clearing or retaining markers must remain consistent with the reused startup recovery behavior so restart remains safe after an in-process recovery attempt.
- Ticket 02 owns stronger filesystem durability and transition redesign. This change should not alter backup schema, marker states, file names, or source ordering.

## Product decisions

No unresolved product decision blocks proposal. The approved acceptance criteria support returning `restore_failed` when the prior database was recovered and `database_unavailable` only when no validated connection can be established. Any richer UI action such as an explicit restart/retry control is optional follow-up and is not required for this backend recovery defect.
