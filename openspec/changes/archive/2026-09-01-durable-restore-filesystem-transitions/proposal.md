# Proposal: Durable Restore Filesystem Transitions

## Intent

Strengthen live restore and startup recovery so filesystem transitions request ordered file and namespace durability on supported local Windows app-data stored on NTFS, while retaining at least one validated recovery source across every protocol step.

This change closes the gap between ticket 01's safe in-process settlement and the persistence of its files and marker directory entries across process or machine interruption. It does not promise survival against every arbitrary hardware, controller-cache, device, or power-loss failure; its guarantee is bounded to successful durability operations supported and verified on Windows/NTFS.

## Problem

Ticket 01 establishes validated staging, canonical → rollback → protective recovery, `Ready` only with a newly opened validated connection, bounded `restore_failed`/`database_unavailable` outcomes, and stale-connection gating. However, the current filesystem mechanics do not explicitly make every related namespace transition durable.

Marker temporary files and recovery-copy temporary files are synchronized before rename, but containing directories are not synchronized. Canonical, rollback, stage, and marker rename/removal operations also lack explicit namespace barriers, and validated staged/protective SQLite files are not explicitly synchronized after their SQLite writers close. A process-visible rename can therefore become durable in a different order from marker progress. In the worst case, the canonical move survives while `Prepared` does not, allowing startup to follow an unmarked normal-open path instead of recovery.

## Proposed scope

### Bounded durability guarantee

- Support local Windows app-data on NTFS as the declared durable restore storage environment.
- Request and order both file durability and containing-directory namespace durability for every transition that the restore protocol relies on.
- Permit a small private Windows-native adapter or target-specific dependency under `cfg(windows)` when Rust `std` cannot express the required Windows directory or replacement semantics.
- Treat a failed or unsupported required durability operation as restore failure, never as durable success.
- Fail closed while preserving recovery evidence and settling through ticket 01's existing `restore_failed` or `database_unavailable` outcomes.
- Make no absolute claim of survival across arbitrary filesystems, removable or network storage, hardware, controller caches, or storage-device behavior.

### Durable transition ordering

- Synchronize the closed, validated staged candidate and protective database before the live connection is disturbed.
- Publish each existing marker value through a synchronized temporary file, namespace publication/replacement, and required marker-directory barrier.
- Complete the canonical-to-rollback namespace barrier before publishing `LiveMoved` durably.
- Complete both the staging-directory removal barrier and app-data-directory installation barrier before publishing `CandidateInstalled` durably.
- Remove the marker durably only after canonical installation is durable and the canonical database has been opened, validated, and published according to ticket 01 settlement rules.
- During fallback recovery, retain rollback or protective source evidence until a copied canonical database is synchronized, installed durably, opened, and validated.
- Order cleanup so removal or replacement of old evidence is never the next operation when that evidence is the last validated recovery source.

### Compatibility and application-state behavior

- Preserve the exact restore-marker JSON representation and the existing `Prepared`, `LiveMoved`, and `CandidateInstalled` states.
- Permit marker progress to lag a completed durable database transition, but never to lead an incomplete durable transition.
- Preserve startup and in-process recovery ordering: validated canonical first, rollback second, protective third.
- Preserve `Ready` only with a newly opened and validated connection, command gating while `Restoring` or `Unavailable`, and the prohibition on stale connection reuse.
- Preserve the bounded `restore_failed` result when service recovers and `database_unavailable` when no validated connection can be established.
- Prevent normal-open behavior from creating fresh canonical storage over retained marked recovery evidence.

## Protocol invariants

1. **Prepared evidence precedes disruption:** before the live connection is dropped, the validated candidate and protective database have completed required supported file-durability operations, and `Prepared` has completed its file and namespace durability operations.
2. **A validated source survives:** after disruption, every interruption boundary retains at least one validated canonical, rollback, or protective recovery source; destructive cleanup never removes the last such source.
3. **Markers never lead transitions:** a durable marker may describe an earlier completed state, but it never describes a filesystem transition whose required barriers have not completed.
4. **Candidate installation covers both directories:** moving the stage into canonical storage is complete only after required barriers cover both the staging source directory and app-data destination directory.
5. **Marker removal is last:** the marker is not durably removed until canonical is durably installed, opened, validated, and published through ticket 01's settlement behavior.
6. **Fallback sources are retained:** recovery by copy does not move or delete its rollback/protective source before the replacement canonical is durable and validated.
7. **Failure is closed and bounded:** any required durability failure retains marker and recovery evidence where possible, exposes no false success, and reaches ticket 01's bounded running-state outcome.
8. **Persistent compatibility is exact:** existing marker JSON, state meanings, file names, and canonical → rollback → protective ordering remain understandable without migration or version negotiation.
9. **Handles close before namespace mutation:** SQLite and file handles that could block Windows rename, replace, remove, or cleanup operations are closed before those operations begin.
10. **Claims remain bounded:** successful barriers establish the supported Windows/NTFS contract only; they do not establish arbitrary-hardware power-loss immunity.

## Affected areas

- Restore-transition mechanics in `src-tauri/src/infrastructure/filesystem/backup_store.rs`, deepened behind its existing narrow restore interface.
- Live restore and startup recovery orchestration in `DatabaseState` only where needed to invoke durable transitions without changing ticket 01 state ownership or settlement policy.
- Private platform-specific filesystem helpers and, if selected, Windows-only dependency/target configuration.
- Private test-only operation recording or failpoints used to prove ordering without introducing a repository-wide filesystem seam.
- Focused Rust integration tests for interruption states, recovery-copy retention, marker retries, cleanup ordering, and bounded outcomes.
- Windows/NTFS compilation and runtime verification evidence for the selected native mechanics.

## Non-goals

- Changing backup contents, backup format, backup destinations, or SQLite backup creation semantics.
- Changing database schema, migrations, compatibility validation, or WAL policy.
- Changing marker JSON, marker versioning, marker state names, recovery file names, or recovery-source ordering.
- Changing UI behavior, IPC shapes, public error codes, or adding restore/retry controls.
- Creating a general filesystem abstraction or changing unrelated filesystem publishing behavior.
- Supporting FAT, exFAT, removable media, network shares, cross-volume stage installation, or non-NTFS app-data under this durability guarantee.
- Claiming absolute power-loss survival across arbitrary hardware or storage stacks.

## Windows evidence gate

Repository and platform analysis is sufficient for this proposal, but no particular Windows-native sequence is accepted merely from API documentation or naming. Design must identify the exact private Windows operations, flags, handle-sharing rules, filesystem assumptions, and fail-closed behavior before implementation proceeds.

Verification and release acceptance require evidence on a real supported Windows environment with local NTFS app-data that:

- the selected file and directory durability operations compile and execute successfully;
- marker create/replace/remove and canonical/rollback/stage transitions request barriers in protocol order;
- source and destination directory changes are both covered for stage installation;
- open handles do not invalidate rename, replacement, removal, retry, or cleanup behavior;
- an unsupported or failed required barrier is surfaced as failure and retains recoverable evidence;
- restart recovery preserves canonical → rollback → protective selection and ticket 01 bounded outcomes.

Process-kill tests may prove restart behavior and requested operation ordering, but they are not evidence of cache persistence after physical power loss. Failure to produce the Windows/NTFS evidence blocks acceptance of the durability claim rather than weakening it silently.

## Risks and mitigations

### Windows semantics are weaker or different than assumed

Directory flushing, replacement, handle sharing, or filesystem support may not match portable assumptions. Keep native mechanics private, require the Windows evidence gate, and fail closed when a required operation is unsupported or fails.

### Marker and database transitions become durably reordered

A marker that leads its database transition can direct recovery using evidence that was never installed durably. Publish each marker only after the corresponding database namespace barriers complete; stale earlier markers remain safe because recovery validates canonical first.

### Cleanup destroys the last recovery source

Replacing an old rollback or canonical file can create a destructive gap. Establish and synchronize an independent validated protective source first, and retain fallback sources until their replacement is durable and validated.

### Cross-directory installation is treated as one transition

Stage installation removes one directory entry and creates another. Require barriers for both staging and app-data directories before advancing `CandidateInstalled`; reject unsupported cross-volume arrangements rather than inventing an unverified copy protocol.

### Windows open handles block transitions

SQLite or file handles can prevent rename, replacement, removal, or cleanup. Make closure ordering explicit and verify it on Windows before namespace mutation.

### Added mechanics alter ticket 01 outcomes

Durability errors could accidentally leak new states or public errors. Keep `DatabaseState` responsible for `Ready`/`Restoring`/`Unavailable`, preserve validation and stale-connection gating, and map failures only to existing bounded outcomes.

### Tests overstate durability

An operation recorder or child-process interruption test can prove requested ordering but not physical cache persistence. Separate protocol-order evidence from the real Windows/NTFS runtime gate and retain bounded acceptance language.

## Rollback and compatibility

Revert the durable transition mechanics, private Windows adapter/dependency configuration, and focused protocol tests together. No schema migration, backup conversion, marker migration, or UI rollback is required because this proposal preserves all persistent names, marker JSON, and marker states exactly.

Markers and recovery files created while this change is active remain understandable to ticket 01 behavior after code rollback. Rollback therefore restores the previous weaker durability implementation without stranding a new marker version. It must be documented as a reduction of the Windows/NTFS interruption guarantee, and it must not selectively retain ordering assumptions that depend on removed barriers.

## Success criteria

- A table-driven protocol test records every required file synchronization, directory barrier, rename/replace, removal, and marker publication in deterministic order.
- Injected interruption immediately before and after each recorded operation always leaves modeled durable evidence containing a validated canonical, rollback, or protective source.
- At every modeled interruption, durable marker state is absent, equal to, or behind the completed durable database transition and never ahead of it.
- Tests prove `Prepared` is durable only after candidate/protective file durability and before the live connection is dropped.
- Tests prove `LiveMoved` advances only after the canonical-to-rollback namespace barrier completes.
- Tests prove `CandidateInstalled` advances only after both staging and app-data directory barriers complete.
- Tests prove marker removal follows durable canonical installation, successful open and validation, and ticket 01 readiness settlement.
- Recovery-copy interruption tests prove rollback/protective source evidence remains until canonical is durable and validated.
- Failed and unsupported required barriers retain marker/recovery evidence and produce only ticket 01's bounded `restore_failed` or `database_unavailable` outcomes.
- Existing integration coverage confirms canonical → rollback → protective recovery, validated `Ready`, command gating, and no stale-connection use remain unchanged.
- Existing marker JSON and all three marker states remain byte/schema compatible without a version migration.
- Focused and full repository Rust tests pass on the configured host without product-code scope expansion beyond the approved restore areas.
- Real Windows runtime verification on local NTFS satisfies every item in the Windows evidence gate; without that evidence, the change is not accepted as providing the stated durability guarantee.
