# Proposal: Recover After Live Restore Failure

## Intent

Ensure a failed live database restore does not leave the running application indefinitely stuck in the transitional `Restoring` state. After any restore failure that occurs once the active connection has been disturbed, the application must either establish a newly opened and validated database connection or enter an explicit unavailable state.

The requested restore must still be reported as failed when service recovery succeeds. Recovery protects continued service; it does not turn a rejected or incomplete restore into a successful restore.

## Problem

The live restore flow currently drops the active SQLite connection before moving and installing database files. Any later error can return early while the in-memory state remains `status = Restoring` with no connection. The existing command gate prevents stale access, but all subsequent reads and writes remain unavailable until the process restarts, even when the canonical, rollback, or protective database is valid and recoverable.

This creates an avoidable operational outage after a bounded restore failure and leaves the process in a transitional state rather than a deliberate terminal state.

## Proposed scope

### Post-disruption recovery

- Detect restore failures that occur after the active connection has been disturbed.
- While retaining exclusive ownership of database state, attempt in-process recovery using the existing startup recovery policy and source ordering: validated canonical database first, rollback database second, and protective database third.
- Open and fully validate the selected database before installing its connection and setting state to `Ready`.
- Never reuse the previously dropped connection and never expose `Ready` without a validated connection.
- Preserve the existing mutex and status gate so concurrent commands cannot pass through a transitional, stale, or invalid connection.

### Bounded outcomes

- If a validated connection is established, keep the service usable but return the original bounded `restore_failed` result because the requested restore did not complete successfully.
- If no recovery source can establish a validated connection, explicitly set `connection = None` and state to `Unavailable`, and return the bounded `database_unavailable` result.
- Keep user-facing failures free of filesystem paths, SQLite diagnostics, marker details, and other storage internals.
- Ensure subsequent reads and writes operate through the recovered validated connection or consistently return `database_unavailable`.

### Compatibility with startup recovery

- Reuse or share the existing validated recovery behavior rather than introduce a competing recovery policy.
- Preserve current recovery-source ordering and marker semantics so an application restart remains safe after an in-process recovery attempt.
- Avoid any normal database-open path that could silently create a fresh empty canonical database while recoverable evidence exists.

## Core invariants

1. **Validated readiness:** database state is `Ready` only when it owns a newly opened connection whose database passed the existing integrity, schema, and compatibility validation.
2. **No stale access:** the dropped pre-restore connection is never reused, and no command executes while state is `Restoring` or `Unavailable`.
3. **Truthful restore result:** recovering service after a failed restore does not report the requested restore as successful.
4. **Explicit terminal failure:** inability to recover transitions the process from `Restoring` to `Unavailable` with no connection.
5. **Consistent source policy:** in-process recovery follows the same canonical-first, rollback-second, protective-third ordering as startup recovery.
6. **Bounded disclosure:** command errors expose only stable safe codes and no database or filesystem internals.

## Affected areas

- `DatabaseState` live-restore orchestration around `install_validated_stage`.
- Startup recovery logic where a small refactor may be needed to share validated source selection and connection establishment.
- Tauri backup/restore command error mapping for the distinct `restore_failed` and `database_unavailable` outcomes.
- Focused Rust restore tests covering post-disruption recovery, recovered data visibility, unavailable behavior, and bounded errors.

No database schema, migration, backup format, or frontend workflow change is required.

## Out of scope

- Redesigning durable filesystem transitions, rename/copy durability, or directory synchronization.
- Changing restore marker states, meanings, lifecycle, or durable transition protocol.
- Changing backup formats, backup destinations, recovery file names, or recovery-source ordering.
- Redesigning WAL handling or recovery-source retention.
- Adding a restart, retry, or other new recovery control to the user interface.
- Broad restructuring of `BackupStore` or the restore protocol; those concerns remain in ticket 02.

## Risks and mitigations

### Fresh empty database creation

A generic open path could create a new canonical database and hide recoverable evidence. Recovery must select and validate existing candidates before any operation capable of creating storage.

### Divergent live and startup recovery

Separate implementations could choose different sources or handle markers inconsistently. Share the existing validated recovery policy or extract only the smallest common recovery seam.

### Incorrect success reporting

A usable fallback connection does not mean the requested candidate was installed. Preserve the original bounded `restore_failed` outcome whenever service recovery succeeds after restore failure.

### Transitional state leakage

An unhandled recovery error could leave state as `Restoring` again. Every post-disruption failure path must finish in either `Ready` with a validated connection or `Unavailable` with no connection.

### Invalid candidate exposure

The failed restore candidate could be observable if readiness is set too early. Complete opening and all existing validation before publishing the connection or changing state to `Ready`.

### Marker compatibility

In-process recovery could clear or retain durable evidence incorrectly and make restart behavior unsafe. Marker handling must remain governed by the existing startup-compatible protocol; this change must not redefine marker semantics.

## Rollback

Revert the in-process recovery orchestration, any minimal shared-recovery refactor, and the focused tests together. The rollback requires no database migration, data rewrite, backup conversion, or marker-format change because this proposal does not alter persistent schemas or formats.

If rollback occurs after release, the prior behavior returns: a post-disruption restore failure may require an application restart for startup recovery. Existing databases, backups, rollback files, protective snapshots, and restore markers remain compatible.

## Success criteria

- A deterministic focused test forces an error after the active connection has been disturbed.
- When canonical, rollback, or protective storage can provide a valid database under the existing ordering, the process installs only a newly opened and validated connection and returns to `Ready`.
- After successful service recovery, the restore request still returns the bounded `restore_failed` result.
- A normal command succeeds after the recoverable failure and observes the recovered prior database rather than the rejected restore candidate.
- When no recovery source validates or can be opened, state becomes explicitly `Unavailable`, has no connection, and the restore boundary returns `database_unavailable`.
- Subsequent reads and writes in the unavailable state consistently return `database_unavailable`.
- No command can execute through the dropped connection or through any connection while state is `Restoring` or `Unavailable`.
- User-facing failure responses contain no filesystem path, SQLite text, marker detail, or other storage internals.
- Existing durable marker semantics, backup formats, destinations, WAL behavior, recovery-source ordering, and startup recovery compatibility remain unchanged.
