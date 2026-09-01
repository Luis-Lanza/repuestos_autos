# Backup and Restore Specification

## Purpose

Keep the running application in a deliberate, safe database state when a live restore fails after disturbing its active connection, while preserving the existing restore and startup-recovery contract.

## Requirements

### Requirement: Recover After a Post-Disruption Restore Failure

After a live restore failure has disturbed the active database connection, the system MUST attempt in-process recovery using the existing recovery-source policy. If a recovery source can provide a valid database, the running service MUST become usable again through that recovered database. The failed restore request MUST remain a failure even when service recovery succeeds.

#### Scenario: Recover service from an available source

- GIVEN a live restore has disturbed the active connection and the requested restore cannot complete
- AND at least one existing recovery source can provide a valid database
- WHEN the restore operation finishes
- THEN the process returns to `Ready` with the recovered database available for normal reads and writes
- AND the restore result is the bounded `restore_failed` outcome
- AND the rejected restore candidate is not observable through subsequent normal commands

#### Scenario: Preserve a usable validated connection when final cleanup fails

- GIVEN the restore candidate has been opened and validated
- WHEN related restore completion work fails before the restore operation can report success
- THEN the service remains usable through that newly validated database connection
- AND the restore result is still the bounded `restore_failed` outcome

### Requirement: Validated Readiness

The system MUST expose database state as `Ready` only when it owns a newly opened connection that has passed the existing integrity, schema, and compatibility validation. A candidate that cannot be opened or validated MUST NOT be exposed as the active connection or make the service `Ready`.

#### Scenario: Publish only a validated recovered database

- GIVEN a post-disruption recovery source is present
- WHEN the system evaluates that source for service recovery
- THEN it opens and fully validates the source before publishing readiness
- AND a normal command can observe only the validated source after `Ready` is published

#### Scenario: Skip an invalid recovery candidate

- GIVEN the first available recovery candidate fails existing integrity, schema, or compatibility validation
- WHEN post-disruption recovery selects a source
- THEN that candidate is not installed or exposed as `Ready`
- AND recovery continues according to the existing source ordering

### Requirement: Explicit Unavailable Terminal State

If no recovery source can establish a validated database connection after a post-disruption restore failure, the system MUST transition from `Restoring` to `Unavailable` and MUST own no active database connection. Commands requiring database access MUST consistently return the bounded `database_unavailable` outcome while the state is `Unavailable`.

#### Scenario: Enter unavailable when every source fails

- GIVEN the active connection has been disturbed by a live restore
- AND canonical, rollback, and protective recovery sources are absent, cannot be opened, or fail validation
- WHEN recovery finishes
- THEN the process state is `Unavailable`
- AND no database connection is owned
- AND the restore boundary returns `database_unavailable`

#### Scenario: Reject reads and writes while unavailable

- GIVEN the process is in `Unavailable` with no active connection
- WHEN a read or write command is requested
- THEN the command returns `database_unavailable`
- AND it performs no database operation or business-data mutation

### Requirement: No Stale Connection Use During Recovery

The system MUST NOT reuse the pre-restore connection after it has been disturbed. No command MUST execute against a connection while database state is `Restoring` or `Unavailable`; commands MAY execute only through the newly opened and validated connection after state is `Ready`.

#### Scenario: Gate commands during the transitional state

- GIVEN a live restore has disturbed the active connection
- WHEN another read or write command arrives while state is `Restoring`
- THEN the command is rejected with the existing bounded unavailable outcome
- AND no command executes through the dropped connection

#### Scenario: Use the recovered database after readiness

- GIVEN recovery has returned the process to `Ready`
- WHEN a normal read or write command is requested
- THEN it executes through the newly opened validated recovery connection
- AND its result reflects the recovered database rather than the failed restore candidate

### Requirement: Compatibility with Existing Recovery-Source Ordering

In-process recovery MUST use the same validated recovery policy and source ordering as startup recovery: canonical database first, rollback database second, and protective database third. The change MUST preserve existing marker semantics so a subsequent restart remains governed by the same recovery behavior. Recovery MUST NOT create a fresh empty canonical database while recoverable evidence is available.

#### Scenario: Prefer canonical over rollback and protective sources

- GIVEN canonical, rollback, and protective sources are all available and valid
- WHEN post-disruption recovery runs
- THEN it establishes the canonical source
- AND it does not choose rollback or protective storage instead

#### Scenario: Fall through the existing ordering

- GIVEN the canonical source is unavailable or invalid
- AND the rollback source is valid
- WHEN post-disruption recovery runs
- THEN it establishes the rollback source
- AND it does not use the protective source

#### Scenario: Use protective storage only after earlier sources fail

- GIVEN canonical and rollback sources cannot establish validated connections
- AND the protective source is valid
- WHEN post-disruption recovery runs
- THEN it establishes the protective source

#### Scenario: Preserve restart-compatible marker behavior

- GIVEN a post-disruption restore failure has been handled in process
- WHEN the application is restarted
- THEN marker handling and recovery follow the existing startup-compatible meanings and lifecycle
- AND no new marker state, file name, or recovery ordering is required by this change

### Requirement: Bounded and Truthful Restore Results

The restore command MUST expose only stable bounded outcomes for post-disruption failure. It MUST return `restore_failed` when the requested restore fails but a validated service connection is recovered, and MUST return `database_unavailable` when no validated connection can be established. A recovered service MUST NOT cause the rejected restore to be reported as successful.

#### Scenario: Report failed restore after successful service recovery

- GIVEN the requested restore candidate is rejected or incomplete
- AND a validated recovery connection is established
- WHEN the restore command returns
- THEN the response contains `restore_failed`
- AND it does not contain a success result for the requested restore

#### Scenario: Report unavailable when service recovery fails

- GIVEN the requested restore fails after disturbing the active connection
- AND no recovery source can establish a validated connection
- WHEN the restore command returns
- THEN the response contains `database_unavailable`
- AND the process is in `Unavailable` with no connection

### Requirement: Safe Error Disclosure

User-facing restore and database-access failures MUST contain only stable safe error codes and bounded user-facing information. They MUST NOT disclose filesystem paths, SQLite diagnostics, marker details, recovery-source internals, or other storage implementation details.

#### Scenario: Bound a recovered restore failure

- GIVEN service recovery succeeds after the requested restore fails
- WHEN the restore error is returned to the caller
- THEN it contains the stable `restore_failed` code
- AND it contains no filesystem path, SQLite text, marker detail, or storage-internal diagnostic

#### Scenario: Bound an unavailable failure

- GIVEN no validated recovery source can be established
- WHEN the restore or a subsequent database command returns an error
- THEN it contains the stable `database_unavailable` code
- AND it contains no filesystem path, SQLite text, marker detail, or storage-internal diagnostic

### Requirement: Preserve Existing Restore and Ticket 02 Boundaries

A successful live restore MUST retain its existing observable behavior. This change MUST NOT alter database schema, migration behavior, backup format, backup destination, recovery file names, durable filesystem transition ordering or durability, WAL handling, recovery-source retention, marker meanings, or startup recovery ordering. It MUST NOT add a restart, retry, or other recovery control to the user interface; durable restore-protocol redesign remains excluded to ticket 02.

#### Scenario: Successful restore remains unchanged

- GIVEN a requested restore candidate passes the existing restore validation and durable transition
- WHEN the live restore completes successfully
- THEN the candidate becomes the active validated database
- AND the restore reports its existing success result
- AND no unrelated schema, backup, or recovery behavior is changed

#### Scenario: Ticket 02 concerns remain excluded

- GIVEN the application performs a post-disruption recovery for a failed live restore
- WHEN the change is exercised
- THEN it changes only the running-state recovery outcome and bounded command result
- AND it does not redesign durable file transitions, marker protocol, backup formats or destinations, WAL handling, recovery retention, or user-facing restart/retry controls
