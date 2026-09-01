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

A successful live restore MUST retain its existing observable behavior. This change MUST NOT alter database schema, migration behavior, backup format, backup destination, WAL handling, UI behavior, IPC shapes, public error codes, or user-facing restart/retry controls. It MUST preserve ticket 01 validation, connection ownership, stale-connection gating, `Ready`/`Restoring`/`Unavailable` state behavior, recovery-source ordering, and bounded outcomes. Durable filesystem transition ordering, required barrier failure handling, and fallback-source retention are explicitly in scope for this ticket and MUST remain within the bounded Windows/NTFS contract specified here.

(Previously: Ticket 02 concerns, including durable file-transition ordering, durability, and recovery-source retention, were excluded from the ticket 01 behavior.)

#### Scenario: Successful restore remains unchanged

- GIVEN a requested restore candidate passes the existing restore validation and the durable transition succeeds
- WHEN the live restore completes successfully
- THEN the candidate becomes the active validated database
- AND the restore reports its existing success result
- AND no unrelated schema, backup, or recovery behavior is changed

#### Scenario: Ticket 01 behavior remains unchanged while ticket 02 durability is exercised

- GIVEN the application performs a post-disruption recovery for a failed live restore
- WHEN the change is exercised
- THEN it preserves canonical-first, rollback-second, protective-third validation and the existing `Ready`, `Restoring`, and `Unavailable` behavior
- AND it changes only the durable filesystem ordering, required-barrier handling, and fallback retention specified by this delta
- AND it does not redesign backup formats or destinations, WAL handling, marker representation or state names, or user-facing restart/retry controls

### Requirement: Durable Prepared Evidence Before Disruption

Before disturbing the live database connection, the system MUST have a validated restore candidate and an independent validated protective source with their required file-durability operations completed. The `Prepared` marker MUST also have been durably published before the connection is dropped. A failed preparation MUST NOT be reported as a completed restore transition.

#### Scenario: Prepare candidate and protective evidence before dropping the connection

- GIVEN a restore candidate passes the existing validation rules
- WHEN the system prepares a live restore
- THEN the candidate and protective source have completed their required supported file-durability operations before the live connection is disturbed
- AND the existing `Prepared` marker is durably published before the connection is dropped
- AND at least one validated recovery source remains available at the disruption boundary

#### Scenario: Reject an unsupported or failed preparation barrier

- GIVEN a required candidate, protective-source, or `Prepared` durability operation is unsupported or fails
- WHEN the system prepares a live restore
- THEN it does not report the corresponding transition as durable
- AND it preserves the pre-disruption canonical database and any recovery evidence where possible
- AND it returns only the existing bounded restore outcome

### Requirement: Marker Progress Never Leads Durable Transitions

The durable marker state MUST be absent, equal to, or behind the completed durable filesystem transition. The system MUST NOT durably publish a marker state that describes a transition whose required barriers have not completed.

#### Scenario: Publish `LiveMoved` only after the rollback barrier

- GIVEN `Prepared` is durable and the canonical database is being moved to rollback storage
- WHEN the canonical-to-rollback namespace transition completes
- THEN the required namespace barrier for that transition completes before `LiveMoved` is durably published
- AND an interruption before that barrier leaves the marker at `Prepared` or an earlier completed state

#### Scenario: Publish `CandidateInstalled` only after both directory barriers

- GIVEN the validated candidate is being installed from the staging directory into canonical app-data storage
- WHEN the candidate installation is considered complete
- THEN the required barrier for the staging source-directory change has completed
- AND the required barrier for the app-data destination-directory change has completed
- AND only then may `CandidateInstalled` be durably published

#### Scenario: Allow a marker to lag a completed transition

- GIVEN a database namespace transition has completed its required barriers
- AND publication of the next marker value is interrupted or fails
- WHEN recovery evaluates the filesystem
- THEN the stale earlier marker is not treated as evidence that a later incomplete transition occurred
- AND canonical, rollback, and protective recovery continue through the existing validation and ordering policy

### Requirement: Marker Removal Follows Validated Readiness

Completion MUST validate the supported local fixed-NTFS, same-volume layout, exact canonical/rollback/protective paths, and safe regular-file and reparse conditions after ticket 01 has published the newly opened validated connection as `Ready` and before any marker mutation. If no marker exists, completion MUST be idempotent success only after that validation. If a marker exists, it MUST be valid and safe before the system may durably remove it. Marker removal MUST NOT be required to make the connection valid.

#### Scenario: Remove the marker after durable validated readiness

- GIVEN the candidate is durably installed as canonical
- AND the newly opened canonical connection passes existing integrity, schema, and compatibility validation and is published as `Ready`
- AND supported-layout and safe-file validation passes
- AND the existing marker is valid and safe
- WHEN completion runs
- THEN the system may durably remove the marker
- AND marker removal occurs after readiness publication and completion validation

#### Scenario: Treat an absent marker as idempotent completion only after validation

- GIVEN canonical is durable and a newly opened canonical connection has been validated and published as `Ready`
- AND no restore marker exists
- WHEN completion runs
- THEN it validates the supported layout and safe-file conditions before deciding the result
- AND it returns idempotent success without requiring marker creation or removal

#### Scenario: Reject an unsafe completion before marker mutation

- GIVEN a newly opened canonical connection has been published as `Ready`
- AND the supported layout is invalid or the present marker is malformed, non-regular, a reparse point, or otherwise invalid
- WHEN completion runs
- THEN it fails before mutating the marker
- AND it does not expose the restore as successful

#### Scenario: Retain a marker when final removal fails

- GIVEN canonical is durable and a newly opened canonical connection has been validated and published as `Ready`
- AND completion validation has passed
- WHEN marker removal or its required namespace barrier fails
- THEN the system attempts to preserve the exact marker as recovery evidence where the OS permits
- AND the service does not expose the failed restore as successful
- AND ticket 01's bounded `restore_failed` behavior remains in force when the service is usable

### Requirement: Retain Fallback Sources Through Recovery Copy

During fallback recovery by copy, the selected rollback or protective source MUST remain intact and recoverable until the replacement canonical database has completed its required file and namespace durability operations, has been opened, and has passed existing validation. The system MUST never delete rollback or protective database evidence, including after a validated replacement; preserved marker sidecars are separate evidence and MAY be recycled only under the marker-sidecar lifecycle requirement.

#### Scenario: Preserve a source across an interrupted fallback copy

- GIVEN canonical is unavailable or invalid and rollback or protective storage is a valid recovery source
- WHEN copying or installing that source into canonical storage is interrupted or fails
- THEN the source is not moved or deleted as part of the incomplete recovery
- AND the existing marker and the rollback or protective database evidence remain available
- AND recovery does not report a successful canonical replacement

#### Scenario: Retain database evidence after validated replacement

- GIVEN a fallback source has been copied into canonical storage
- WHEN the canonical replacement is durable, newly opened, and fully validated
- THEN the rollback or protective database evidence remains intact
- AND no cleanup operation deletes that database evidence

### Requirement: Fail Closed on Required Durability Failure

A failed or unsupported required file or namespace durability operation MUST fail closed. The system MUST NOT advance a marker, expose a false restore success, or discard required recovery evidence as though the operation succeeded. After disruption, the failure MUST settle through ticket 01's existing `restore_failed` outcome when a validated connection is recovered, or `database_unavailable` when none can be established.

#### Scenario: Fail a restore while preserving a recoverable source

- GIVEN a required barrier fails after the live connection has been disturbed
- AND canonical, rollback, or protective storage can establish a validated connection
- WHEN restore recovery completes
- THEN the process is `Ready` only through that newly opened validated connection
- AND the restore result is `restore_failed`
- AND no later marker state is durably published for the failed transition

#### Scenario: Become unavailable when no source can be validated

- GIVEN a required barrier fails after disruption
- AND no canonical, rollback, or protective source can establish a validated connection
- WHEN recovery completes
- THEN the process is `Unavailable` with no active connection
- AND the result is `database_unavailable`
- AND no required durability failure is represented as restore success

### Requirement: Preserve Exact Restore Marker Compatibility

The restore marker and every preserved marker sidecar MUST preserve the exact existing JSON representation and marker bytes, file name, state meanings, and the three state values `Prepared`, `LiveMoved`, and `CandidateInstalled` (persisted as the existing `prepared`, `live_moved`, and `candidate_installed` values). This change MUST NOT add a marker state, field, version negotiation, or recovery-file rename. Existing marker data MUST remain understandable to ticket 01 startup and in-process recovery.

#### Scenario: Read every existing marker state without migration

- GIVEN a marker contains any existing `Prepared`, `LiveMoved`, or `CandidateInstalled` state in the current JSON representation
- WHEN startup or in-process recovery reads it
- THEN the marker is interpreted using the existing meaning and recovery-source order
- AND no migration or alternate marker protocol is required

#### Scenario: Write compatible marker payloads

- GIVEN the restore protocol publishes marker progress or preserves a prior marker during replacement
- WHEN the marker or its sidecar becomes observable
- THEN its bytes remain the exact existing JSON payload for `prepared`, `live_moved`, or `candidate_installed`
- AND temporary publication mechanics do not change the persisted marker schema or file name

### Requirement: Bound the Durability Guarantee to Windows Local NTFS

The ordered durability guarantee MUST be limited to supported Windows runtime use with local app-data stored on a fixed local NTFS volume and successful required file and namespace barriers. Before startup or in-process recovery performs any filesystem mutation, recovery MUST require an existing valid marker, an exact validated rollback or protective source at its expected path, the exact canonical path, the same-volume relationship, and safe regular-file and reparse checks. If any precondition is absent, invalid, ambiguous, or unsupported, recovery MUST fail before mutation. Completion MUST apply the supported-layout validation before marker mutation, including when no marker is present. The system MUST fail closed rather than claim the same guarantee for unsupported filesystems, removable or network storage, or unsupported cross-volume installation. Successful barriers MUST NOT be described as immunity from arbitrary hardware, controller-cache, device, or power-loss behavior.

#### Scenario: Apply the declared boundary to recovery

- GIVEN an existing valid marker is present
- AND the exact canonical path and exact validated rollback or protective source path are present
- AND storage is local Windows app-data on a fixed NTFS volume with the required same-volume relationship
- AND all required files are safe regular non-reparse files and all required barriers succeed
- WHEN startup or in-process recovery evaluates the restore
- THEN recovery may perform its ordered filesystem transitions
- AND it preserves the existing `Ready`, recovery ordering, and bounded result behavior

#### Scenario: Reject recovery preconditions before mutation

- GIVEN the marker is absent, malformed, non-regular, or a reparse point
- OR the canonical or rollback/protective path is not exact, safe, or valid
- OR storage is not supported local fixed NTFS on the same volume
- WHEN startup or in-process recovery is requested
- THEN it fails closed before remove, copy, rename, replacement, or other filesystem mutation
- AND it does not silently substitute an unverified reduced-guarantee path

#### Scenario: Do not silently weaken the guarantee outside the boundary

- GIVEN the storage environment is outside the declared Windows local NTFS boundary or a required operation is unsupported there
- WHEN a restore transition requires that operation
- THEN the transition fails closed with no false durable success
- AND the system does not silently substitute an unverified reduced-guarantee path

### Requirement: Recycle Completed Restore Marker Sidecars

The system MUST recycle preserved marker sidecars only during a new prepare, after supported-layout validation succeeds, while no active restore marker exists, and before disruption of the live connection. Sidecar cleanup MUST be fail-closed and protected by root barriers. Malformed, non-regular, or reparse sidecars MUST be retained and MUST block preparation. The eight-slot sidecar bound MUST apply only to one active restore/retry episode; recycling completed-restore sidecars MUST keep at least five sequential successful restores available.

#### Scenario: Recycle safe completed-restore sidecars before disruption

- GIVEN supported-layout validation succeeds
- AND no active restore marker exists
- AND completed-restore marker sidecars are safe regular non-reparse files under the expected roots
- WHEN a new prepare begins
- THEN sidecars are recycled only during that prepare and before the live connection is disrupted
- AND root barriers protect the cleanup
- AND preparation does not proceed as durable success if required sidecar cleanup or its barriers fail

#### Scenario: Retain unsafe sidecars and block preparation

- GIVEN a preserved marker sidecar is malformed, non-regular, or a reparse point
- WHEN a new prepare inspects completed-restore sidecars
- THEN the sidecar is retained
- AND preparation fails closed before disruption
- AND no cleanup bypasses the root barrier or treats the sidecar as recyclable

#### Scenario: Do not recycle while an active marker exists

- GIVEN an active restore marker exists
- WHEN preparation or recovery inspects preserved marker sidecars
- THEN it does not recycle those sidecars as completed-restore history
- AND the active marker and recovery evidence remain intact

#### Scenario: Preserve repeated successful restore availability

- GIVEN each prior restore completed successfully and left preserved marker sidecars
- AND each new prepare passes supported-layout and safe-sidecar validation with no active marker
- WHEN at least five restores are completed sequentially
- THEN every restore remains available through the completed-restore sidecar recycling lifecycle
- AND the eight-slot bound limits only one active restore or retry episode, not the lifetime number of successful restores

### Requirement: Require Real Windows NTFS Runtime Evidence

Acceptance of the bounded durability guarantee MUST require runtime evidence from a supported Windows environment using local NTFS app-data. Host-only tests, API documentation, operation recording, or process-kill tests MAY verify protocol ordering and restart behavior, but MUST NOT substitute for the required Windows/NTFS evidence.

#### Scenario: Accept only after the Windows evidence gate passes

- GIVEN the selected platform operations have been implemented
- WHEN release acceptance evaluates this change
- THEN real Windows local-NTFS evidence demonstrates successful file and directory durability operations
- AND it demonstrates ordered marker create, replacement, and removal plus canonical, rollback, and stage transitions
- AND it demonstrates both stage-source and app-data-destination directory coverage, handle closure before namespace mutation, failure retention, and ticket 01 recovery outcomes

#### Scenario: Block the durability claim without runtime evidence

- GIVEN repository tests or process-kill tests pass
- AND the required real Windows local-NTFS runtime evidence is absent or incomplete
- WHEN acceptance is evaluated
- THEN the change is not accepted as providing the bounded durability guarantee
- AND no test result is represented as proof of physical power-loss persistence
