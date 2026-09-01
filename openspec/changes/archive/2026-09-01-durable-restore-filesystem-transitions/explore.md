# Exploration: Durable Restore Filesystem Transitions

## Scope and compatibility boundary

Ticket 02 owns interruption-safe restore replacement at filesystem transition points. It must retain ticket 01's canonical behavior: recovery validates sources in the order canonical, rollback, protective; `Ready` owns a newly opened and validated connection; failed replacement returns `restore_failed` if service recovers and `database_unavailable` otherwise; commands never use the dropped connection. Backup contents, schema, UI behavior, and a general filesystem abstraction remain excluded.

Repository context identifies Windows as the production target, while current backup/restore execution evidence is Fedora-only and the archived ticket 01 verification explicitly records that no Windows Rust target/runtime test was available.

## Current protocol and durability gaps

`DatabaseState::install_validated_stage` currently creates and validates `pre-restore.sqlite3`, writes `Prepared`, drops the live connection, then performs:

1. remove any old rollback and rename canonical to `restore-rollback.sqlite3`;
2. write `LiveMoved`;
3. rename the staged candidate to canonical;
4. write `CandidateInstalled`;
5. open and validate canonical, publish `Ready`, and remove the marker.

`BackupStore::write_marker` writes a `.part` file, calls `File::sync_all`, and renames it over the marker. `restore_canonical_from` similarly syncs a copied temporary file before removing canonical and renaming the temporary file. However:

- no marker creation, replacement, or removal syncs the containing directory;
- no canonical/rollback/stage rename or removal syncs either affected directory;
- the protective SQLite snapshot and staged SQLite candidate are validated but are not explicitly reopened and `sync_all`ed after their SQLite connections close;
- `move_live_to_rollback` removes the previous rollback before establishing the new one;
- `restore_canonical_from` removes canonical before installing its synced temporary copy;
- tests construct marker/file combinations but do not prove that every required durability barrier was requested in order.

A successful `rename` establishes the process-visible namespace transition; it does not by itself prove that the directory entry survives machine interruption. Consequently, marker progress and database filenames can become durably reordered even though ordinary process-failure handling is correct. The most serious shape is a canonical move that survives while the marker creation/update does not: startup without a marker takes the normal `open_database` path, which may create canonical storage rather than invoke marked recovery.

## Required invariants

1. Before the live connection is dropped, the validated candidate and protective database must have completed the protocol's supported file-durability operation, and `Prepared` must have completed its file and namespace durability operations.
2. At every later interruption boundary, at least one of canonical, rollback, or protective must remain a validated recovery source; destructive cleanup must never be the last copy's next operation.
3. Marker progress may lag a completed database transition, but it must not lead an uncompleted transition. Startup remains safe because marked recovery validates canonical first and then rollback and protective.
4. A marker must not be durably removed until canonical has been installed durably, opened, validated, and published according to ticket 01 settlement rules.
5. Recovery copying must retain its source until the copied canonical has completed supported file and namespace durability operations and validation.
6. Every marker value and source ordering must remain understandable to ticket 01 behavior. A new marker format is unnecessary unless a maintainer explicitly accepts an upgrade/rollback compatibility protocol.
7. Claims must be bounded to ordered filesystem durability operations supported by the target OS/filesystem; neither Rust `std` nor an OS flush call proves survival against all hardware, controller-cache, filesystem, or storage-device failures.

## Interruption matrix

| Interruption point | Required recoverable evidence | Marker allowed after restart |
|---|---|---|
| Before durable `Prepared` | canonical remains untouched; in-process failure retains ticket 01 behavior | absent or old completed state only |
| After durable `Prepared`, before live move | canonical plus durable protective | `Prepared` |
| During/after canonical-to-rollback rename, before directory barrier | durable protective is the independent source; namespace may reflect either name | `Prepared` |
| After live-move directory barrier, before durable `LiveMoved` | rollback plus protective | `Prepared` |
| After durable `LiveMoved`, before candidate install | rollback plus protective | `LiveMoved` |
| During/after stage-to-canonical rename, before both directory barriers | rollback and protective; candidate namespace may be indeterminate | `LiveMoved` |
| After candidate-install barriers, before durable `CandidateInstalled` | canonical candidate plus rollback/protective | `LiveMoved` |
| After durable `CandidateInstalled`, before validation | canonical candidate plus rollback/protective | `CandidateInstalled` |
| During marker removal | validated durable canonical remains; stale marker is safe | present or absent |
| During fallback copy/install | rollback or protective source is retained; incomplete temporary/canonical is never the only copy | existing marker retained |

Because the stage lives under `backup-restore/staging` while canonical lives in the app-data root, candidate installation changes two directories. Even on one filesystem, the protocol must account for the source-directory removal and destination-directory addition separately. If these paths can ever cross volumes, rename is not a usable primitive and restore must reject that arrangement or use a separately designed copy protocol; current repository construction keeps both under the same app-data root.

## Platform semantics and Rust `std` limits

- `File::sync_all` is the available standard-library request to synchronize file contents and metadata. It is appropriate after closing SQLite writers and reopening the resulting database file, but its success is still an OS/filesystem guarantee rather than an absolute power-loss guarantee.
- Rust `std` provides `fs::rename` and `fs::remove_file`, but no portable directory-sync interface and no portable primitive whose contract combines destination replacement with directory durability.
- On Unix-like systems, opening a containing directory and calling `sync_all` is a conventional way to request namespace durability, subject to filesystem support.
- Windows is the product constraint. Directory handles and namespace flushing require Windows-specific handle/open semantics that are not represented by a portable `std::fs::File::open(directory)` contract. A Windows adapter may need native APIs or a vetted crate, and its exact support must be compiled and exercised on the supported Windows filesystem before any durability claim is accepted.
- Windows also makes open-handle lifetime operationally important. Ticket 01 already had to add explicit connection drops before test cleanup. All rename/remove steps must continue to occur only after every SQLite/file handle that can block them is closed.
- Native replacement/flush APIs must not be assumed to provide power-loss atomicity merely because they are named “replace,” “write-through,” or “flush.” Their selected flags, filesystem support, and error behavior require explicit Windows evidence.

The defensible product language is therefore “requests ordered file and namespace durability on supported local filesystems and preserves a validated recovery source across each protocol step,” not “guarantees recovery after every power loss.”

## Recommended protocol direction

Keep the existing three marker meanings and canonical-first recovery policy. Strengthen the mechanics rather than adding states:

1. Close the SQLite writer for stage/protective files, reopen each file, request file durability, and request durability for newly created parent entries where supported.
2. Write each marker through a uniquely controlled temporary file, sync the file, atomically replace/publish the marker, then sync the marker directory where supported. A stale `.part` must be safely replaceable on retry.
3. Only after durable `Prepared`, move canonical to rollback and complete the app-data directory barrier before durably publishing `LiveMoved`. The protective source remains independent while an old rollback is removed/replaced.
4. Install the stage, then complete barriers for both staging and app-data directories before durably publishing `CandidateInstalled`.
5. Open and validate canonical and preserve ticket 01 settlement. Remove the marker only afterward, followed by a marker-directory barrier where supported.
6. For fallback recovery, copy to a temporary file, close and sync it, install/replace canonical without deleting the fallback source, complete the target-directory barrier, validate canonical, and only then clear the marker.
7. Any unsupported or failed required durability operation should fail closed into ticket 01's recovery/`Unavailable` outcomes; it must not be silently treated as durable success.

The old rollback is not the sole safety source because the newly durable protective snapshot exists before `Prepared`. Nevertheless, cleanup order should be explicit: never remove an old fallback until another independent validated source has completed its durability barrier.

## Smallest deep seam

Keep `DatabaseState` responsible for connection ownership, validation, `Ready`/`Restoring`/`Unavailable`, and bounded outcomes. Deepen `BackupStore` around the restore protocol instead of introducing a repository-wide filesystem trait:

- private platform helpers for durable file close/sync, durable rename/replace, durable removal, and containing-directory barriers;
- one narrow restore-transition implementation that owns file/marker ordering and exposes only the transition operations needed by `DatabaseState`;
- one private test-only operation recorder/failpoint seam inside that implementation, not a public filesystem adapter.

This concentrates platform mechanics and ordering in the filesystem module while leaving ticket 01 settlement unchanged. A broad filesystem abstraction would expose too many shallow operations and make callers responsible for durability ordering again.

## Testing strategy

1. Add a table-driven protocol test around the private operation recorder. Inject interruption immediately before and after every file sync, directory barrier, rename/replace, removal, and marker publication; assert the modeled durable namespace always contains canonical or a retained recovery source and that marker state never leads the durable database transition.
2. Add real temporary-filesystem integration tests for each completed transition and startup recovery combination, preserving canonical → rollback → protective ordering and bounded ticket 01 results.
3. Add stale `.part`, existing rollback, existing canonical, marker-replacement, marker-removal, and failed-directory-barrier cases. Assert failures retain the marker and recovery sources.
4. Add recovery-copy interruption cases proving rollback/protective is never moved or deleted before the new canonical is durable and validated.
5. Keep process-kill tests separate from durability claims: killing a child process can validate restart semantics and operation ordering, but cannot emulate sudden power loss or prove cache persistence.
6. Run focused and full Rust tests on the current host, then compile and execute the filesystem matrix on a real supported Windows target. Windows evidence must include handle-sharing behavior, marker overwrite/removal, directory barrier support or documented fallback, and cleanup after all handles close.
7. If local NTFS is selected as the support boundary, test there specifically. Removable FAT/exFAT, network shares, and unusual filesystems must not inherit an untested guarantee.

## Blast radius and non-goals

Likely implementation/test files for later phases are `src-tauri/src/infrastructure/filesystem/backup_store.rs`, `src-tauri/src/lib.rs`, `src-tauri/tests/backup_restore.rs`, and dependency/target configuration only if a Windows native helper is selected. `commands/backup.rs`, backup contents, schema/migrations, backup destination behavior, unrelated filesystem publishing, and UI/IPC outcomes should remain unchanged.

## Explicit unresolved product/protocol decisions

1. **Supported storage boundary:** Must the durable contract cover only the local Windows app-data filesystem (preferably a named filesystem such as NTFS), or also FAT/exFAT, removable media, and network-backed app-data? The latter materially weakens what can be promised and tested.
2. **Windows implementation policy:** May the change add a small Windows-native dependency/`cfg(windows)` adapter for directory and replacement operations, or must it remain `std`-only and explicitly report namespace durability as unsupported on Windows? `std`-only cannot honestly satisfy a portable directory-barrier requirement.
3. **Failure policy for unsupported barriers:** Should startup/live restore fail closed to `database_unavailable` when the target filesystem rejects a required directory flush, or continue with a documented reduced guarantee? The safety-oriented recommendation is fail closed for the declared supported filesystem.
4. **Compatibility policy:** Should the exact existing marker JSON and three states remain unchanged for downgrade/restart compatibility? Repository evidence indicates they are already sufficient when writes are durably ordered; changing them would require a versioning and rollback decision.
5. **Acceptance wording:** Confirm that the outcome is ordered durability with retained recovery evidence on supported filesystems, not an absolute power-loss guarantee across arbitrary hardware and filesystems.

Decisions 1–3 genuinely block a truthful Windows design. Decisions 4–5 should be confirmed before proposal/spec language is finalized.

## Evidence limits

This exploration used repository evidence and standard-library/platform constraints only; no research lane or external platform validation was available. It therefore recommends a Windows feasibility/verification gate and does not claim that any particular native Windows directory-flush sequence has already been proven in this repository.
