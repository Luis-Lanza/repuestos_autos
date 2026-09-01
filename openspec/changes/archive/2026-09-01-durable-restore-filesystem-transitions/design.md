# Design: Durable Restore Filesystem Transitions

## Status and decision

This design deepens the restore filesystem module so `DatabaseState` retains connection/state settlement while one private module owns all file and marker ordering. The bounded claim is limited to successful required operations on real Windows with local NTFS app-data. It is not an absolute power-loss guarantee.

**Apply is split and gated.** Repository evidence shows no direct Windows dependency and no proven native directory-flush or replacement implementation. Transitive `windows-sys` entries in `Cargo.lock` are not a supported direct interface. The protocol and deterministic test implementation may be implemented first, but the Windows adapter, any target-specific dependency, and acceptance claim are blocked by gate W0 below. No native API is silently selected by this design.

## Module and seam

Add `src-tauri/src/infrastructure/filesystem/restore_transitions.rs` as a private module, declared with `mod restore_transitions;` from `filesystem/mod.rs`. `BackupStore` remains the caller-facing module and delegates these restore-only operations to the private implementation:

```rust
impl BackupStore {
    pub fn prepare_durable_restore(
        &self,
        stage: &Path,
        protective: &Path,
    ) -> Result<(), StorageError>;

    pub fn install_durable_restore(
        &self,
        stage: &Path,
        canonical: &Path,
    ) -> Result<(), StorageError>;

    pub fn recover_canonical_durably(
        &self,
        source: &Path,
        canonical: &Path,
    ) -> Result<(), StorageError>;

    pub fn complete_durable_restore(&self) -> Result<(), StorageError>;
}
```

The interface intentionally does not expose `sync_file`, `sync_dir`, rename, replace, remove, or marker-write primitives. Deleting this module would force ordering back into `DatabaseState` and `BackupStore`, which is the deletion test for a deep module.

The existing marker parser and `RestoreState` stay unchanged. `write_marker`, `write_restore_state`, `move_live_to_rollback`, `install_stage`, `restore_canonical_from`, and `clear_restore_state` become private implementation details or are replaced; they must not remain independently callable ways to bypass ordering.

### Private platform seam

Inside `restore_transitions.rs`, use a private, statically dispatched seam (names illustrative but concrete):

```rust
enum VerificationPhase<'a> {
    Prepare { stage: &'a Path, protective: &'a Path, canonical: &'a Path },
    Recovery { marker: &'a Path, source: &'a Path, canonical: &'a Path },
    Completion { marker: &'a Path, canonical: &'a Path },
}

trait DurableFs {
    fn verify(&self, root: &Path, phase: VerificationPhase<'_>) -> io::Result<VerifiedLayout>;
    fn sync_file(&self, path: &Path) -> io::Result<()>;
    fn sync_directory(&self, path: &Path) -> io::Result<()>;
    fn rename_no_replace(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn replace_file(&self, from: &Path, to: &Path, preserved: &Path) -> io::Result<()>;
    fn remove_file(&self, path: &Path) -> io::Result<()>;
}
```

`VerifiedLayout` is a private phase-tagged proof containing only canonicalized expected paths and observed marker/sidecar state; it contains no open native or Rust file handles. Mutation methods require the matching proof, so Prepare proof cannot authorize Recovery or Completion work. Verification opens paths with reparse-point-aware handles, queries them, and closes every directory, file, marker, sidecar, and SQLite validation handle before returning. No verification handle may survive into remove, rename, replacement, or cleanup.

`RestoreTransitions<F: DurableFs>` is private and generic. Production construction selects `platform::WindowsDurableFs` under `cfg(windows)`. `cfg(not(windows))` production selects `UnsupportedDurableFs`, whose `verify` fails before any disruption. A non-Windows recorder/real-filesystem adapter exists only in the private unit-test module and is injected into `RestoreTransitions`; it is not a public or repository-wide filesystem abstraction. Integration tests that require successful durable replacement run on Windows; host-independent ordering/failpoint coverage runs through private unit tests.

The seam records primitive attempts, but protocol milestones are inferred only after successful barriers. Every adapter error maps internally to `StorageError::StorageUnavailable`; `DatabaseState` keeps mapping that to `restore_failed` or `database_unavailable`. Native error text and paths never cross IPC.

## Phase-aware supported-layout verification

All three phases require an absolute, lexically unambiguous app-data `root` on Windows, on a fixed local drive whose filesystem name is exactly NTFS. The root volume is the expected volume for every phase path. The root and every existing directory ancestor used by the phase must be a directory opened with reparse-point inspection and must not have `FILE_ATTRIBUTE_REPARSE_POINT`. Every required file must be a regular non-directory, non-reparse file. Inspection uses read/write/delete-compatible sharing, and every inspection handle closes before mutation. Missing, ambiguous, inaccessible, remote, removable, non-NTFS, cross-volume, wrong-type, or reparse results fail closed.

Expected names are constructed internally: canonical is exactly `root/repuestos-autos.sqlite3`, rollback exactly `root/restore-rollback.sqlite3`, protective exactly `root/pre-restore.sqlite3`, marker exactly `root/restore-state.json`, publication temporary exactly `root/restore-state.json.part`, recovery temporary exactly `root/restore-recovery.sqlite3.part`, and preserved marker sidecars are exactly `root/restore-state.json.previous-{0..7}`. A lookalike `restore-state.json.previous-*` outside those eight slots is ambiguous protocol data and is rejected. Stage must be a file strictly below `root/backup-restore/staging`, with no `.`/`..` ambiguity, and its directory chain through `staging`, `backup-restore`, and `root` must be non-reparse directories. Every phase path must be on the root volume; cross-volume installation is rejected rather than copied.

| Phase | Required verification before mutation |
| --- | --- |
| Prepare | Canonical, stage, and protective exist as exact safe regular files; rollback, marker temporary, and recovery temporary are safe if present. The active marker must be absent for a new prepare. Every occupied sidecar slot is a safe regular non-reparse file whose bytes are exactly one of the three accepted marker payloads. Only this phase may classify those sidecars as completed-cycle evidence and recycle them. |
| Recovery | The active marker exists at the exact marker path, is a safe regular non-reparse file, and contains exactly one accepted marker payload. The source is exactly rollback or protective, has already passed SQLite validation through a read-only connection, and that connection is closed. Canonical is the exact canonical path and is either absent or a safe regular non-reparse file; the recovery temporary is safe if present. Marker, source, canonical, temporary, and root are on the same supported volume. No mutation occurs if any precondition fails. |
| Completion | Called only after a newly opened canonical connection has passed ticket 01 validation and has been published as `Ready`. Canonical exists at its exact path as a safe regular non-reparse file; rollback and protective are checked at their exact paths and are safe regular non-reparse files if present; all are on the supported root volume. A present marker must be safe and contain exactly one accepted payload. An absent marker is allowed, but success is idempotent only after this complete supported-boundary check. Sidecars are never recycled by Completion. |

These checks reuse the W0/W2-proven fixed-drive, NTFS, volume, file-attribute, reparse, and share-mode mechanics. Any uncertainty or unsupported result fails before the phase's first mutation. The bounded claim remains conditioned on successful requested OS operations, not physical-power-loss immunity.

## Exact transition sequencing

Each arrow below is an operation boundary at which failure is returned immediately. “Barrier” means a successful required `sync_directory`; process-visible rename alone is not a durable milestone.

### 1. Prepare evidence and publish `Prepared`

Called only after SQLite writers and validation connections for stage and protective have closed:

1. Obtain Prepare verification proof, including active-marker absence and complete validation of all occupied sidecar slots.
2. Run the initial app-data root barrier. This retry fence settles, where the OS permits, a sidecar deletion observed as absent after an earlier remove whose root barrier failed.
3. Recycle each occupied valid sidecar in ascending slot order: remove one sidecar, immediately sync the app-data root, and stop on either failure. Do not batch removals behind one barrier.
4. `sync_file(stage)`.
5. Sync the stage parent chain bottom-up through `staging`, `backup-restore`, and app-data root, so newly created directory entries are covered.
6. `sync_file(protective)`.
7. Sync app-data root for the protective entry.
8. Publish marker payload exactly `{"state":"prepared"}` using the marker algorithm below.
9. Return success; only now may `DatabaseState` set `Restoring` and drop the live connection.

Sidecar recycling is Prepare-only and entirely pre-disruption. A malformed payload, non-regular file, reparse point, unexpected sidecar name, failed initial retry barrier, failed removal, or failed per-sidecar root barrier retains the sidecar where the OS permits and returns `restore_failed` while the original live canonical connection remains usable. Recovery and Completion never recycle sidecars, and an active marker blocks new-prepare cleanup rather than allowing active evidence to be mistaken for completed history.

### Completed-sidecar crash and retry states

| Interruption/failure point | Observable safe state | Required retry behavior |
| --- | --- | --- |
| Before the initial retry root barrier | No cleanup mutation from this attempt; sidecars may include evidence from the prior completed cycle. | Re-run full Prepare verification, require marker absence, then run the initial root barrier before any remove. |
| After the initial barrier, before a sidecar remove | Earlier namespace uncertainty has been fenced; the selected valid sidecar still exists. | Re-verify and remove that slot, then barrier immediately. |
| Remove fails | The sidecar is retained where the OS permits; no later slot is touched and disruption has not occurred. | Fail closed. A later prepare re-verifies all slots and begins with the root barrier; it does not assume a failed remove always preserved the name. |
| Remove is process-visible, before its root barrier | The sidecar can appear absent while deletion durability is uncertain; all database evidence and the live connection remain intact. | Fail closed on barrier failure or crash. The next prepare accepts that slot as absent only after full validation and uses the initial root barrier to fence the prior deletion. |
| After the per-sidecar root barrier | That slot is durably reusable within the bounded Windows/NTFS contract. | Continue to the next occupied slot, or proceed to stage/protective synchronization when none remain. |

The eight slots remain a hard bound for one active restore/retry episode: each same-directory marker replacement reserves a previously unused slot, an ordinary `Prepared` → `LiveMoved` → `CandidateInstalled` episode uses two, and ambiguous replacement/compensation retries may consume more but can never overwrite one of the eight preserved values. Exhaustion therefore fails closed for that active episode. Once Completion has removed the active marker, the next Prepare validates and durably recycles completed-cycle sidecars before disruption, making the same eight names reusable. Space is bounded while the number of successfully completed cycles is unbounded; five sequential successful cycles must demonstrate this lifecycle rather than lifetime slot consumption.

Terminal invariant: canonical is untouched; stage and protective have successful file and namespace requests; durable marker is absent/older or `Prepared`, never later.

### Marker create or replacement

Persistent compatibility is unchanged and exact: `Prepared` is the bytes `{"state":"prepared"}`, `LiveMoved` is `{"state":"live_moved"}`, and `CandidateInstalled` is `{"state":"candidate_installed"}`. The active filename remains `restore-state.json`; sidecars contain byte-for-byte one of those same payloads. No field, version, state, alternate encoding, or recovery filename is added.

For every state, including retries:

1. Remove a stale fixed `.part` if present; do not alter the current marker. Failure aborts.
2. Create `restore-state.json.part` with exclusive ownership, write the unchanged JSON bytes, `sync_file(part)`, then close the handle.
3. If the marker is absent, rename without replacement; otherwise use the platform’s proven same-directory atomic replacement primitive. No delete-then-rename gap is allowed for marker replacement.
4. Sync the app-data root.
5. Only after step 4 is the new marker state a durable protocol milestone.

If publication or the barrier fails, do not advance in memory. The active marker or an exact preserved sidecar remains recovery evidence where the OS permits, and a stale temporary may remain for safe retry; malformed or temporary data is never parsed as the active marker. This is deliberately bounded language: a process-visible remove or replacement followed by a failed directory barrier creates namespace uncertainty, so the design does not promise that the active marker always survives.

### 2. Canonical to rollback, then `LiveMoved`

`install_durable_restore` owns this sequence after the live SQLite connection is closed:

1. Assert no SQLite/file handle owned by the restore path remains open.
2. If old rollback exists, remove it, then sync app-data root before continuing. This is safe because canonical and the independently durable protective source remain.
3. Rename canonical to rollback without replacement.
4. Sync app-data root.
5. Publish unchanged `LiveMoved` marker using the marker algorithm.

Terminal invariant: after step 4, rollback and protective are recovery evidence; marker may safely lag at `Prepared`. `LiveMoved` is never durable before the root barrier.

### 3. Stage to canonical, then `CandidateInstalled`

1. Rename stage to canonical without replacement; cross-volume failure is not converted to copy.
2. Sync the app-data destination directory.
3. Sync the stage source directory.
4. Sync its parent chain through `backup-restore` to app-data root where distinct namespace entries were created by staging.
5. Publish unchanged `CandidateInstalled` marker.

The destination barrier is requested first to favor canonical publication, but this design does not infer undocumented cross-directory commit ordering from that order. Both barriers must succeed before marker advancement, while rollback/protective remain retained.

Terminal invariant: marker is `LiveMoved` or `CandidateInstalled`; if it is `CandidateInstalled`, both source and destination namespace barriers completed. Rollback and protective remain intact.

### 4. Fallback copy while retaining source

`recover_canonical_durably` never moves or deletes `source`:

1. Obtain Recovery verification proof: require the exact existing valid marker, exact closed-and-validated rollback or protective source, exact canonical path, supported fixed local NTFS root, same-volume relationship, and safe regular/non-reparse conditions.
2. Remove stale `restore-recovery.sqlite3.part`, then sync app-data root before reuse.
3. Copy source to a newly created temporary; sync and close the temporary file; sync app-data root for its entry.
4. If invalid canonical exists, remove it and sync app-data root. An absent canonical needs no removal. Source remains intact.
5. Rename temporary to canonical without replacement and sync app-data root.
6. Return for a new canonical open and full validation. Do not clear the marker, sidecars, rollback, or protective source.

At every failure, source and marker are retained where the OS permits; no success is returned before the canonical namespace barrier. Rollback and protective database files are never deleted by fallback or completion, including after a replacement canonical validates; only valid completed marker sidecars have the Prepare-only recycling lifecycle.

### 5. Readiness and durable marker removal

After canonical has completed installation barriers, `DatabaseState` opens and validates a new connection and publishes it as `Ready`. It then calls `complete_durable_restore`:

1. Obtain Completion verification proof for the supported fixed local NTFS boundary, exact canonical/rollback/protective paths, same-volume relationship, and safe regular/non-reparse files. Read and retain the exact bytes of a present valid marker, then close the marker handle.
2. If the marker was absent, return idempotent success now; absence never bypasses step 1 and is accepted only after `Ready`.
3. Remove `restore-state.json`.
4. Sync app-data root.
5. Return success only after the barrier; leave all sidecars and rollback/protective database evidence untouched.

If removal or its barrier fails, return failure and keep the validated connection `Ready`, yielding `restore_failed`. The implementation attempts to preserve or republish the exact pre-removal marker bytes—not an assumed `CandidateInstalled` value—from retained bytes or an exact sidecar where the OS permits. Any compensation uses the normal synchronized publication algorithm, closes handles before mutation, remains within the eight-slot active-episode bound, and cannot convert failure to success. If removal was process-visible and the root barrier or compensation failed, the marker may be present, sidecar-only, or absent after restart; durable validated canonical plus permanently retained rollback/protective databases keep recovery bounded. The design does not claim the marker always survives a failed remove or barrier.

## Windows handle and operation contract

The Windows adapter must satisfy all of the following; W0 must bind them to documented, compiled, runtime-tested native calls before implementation is accepted:

- Directory handles request only access needed for the selected flush operation, include the documented directory-open/backup-semantics flag if required, use sharing equivalent to read/write/delete sharing, and do not use delete-on-close.
- File handles used for sync or copying use sharing compatible with later rename/delete where safe. All SQLite validation, copy, marker-temp, and barrier handles are closed before namespace mutation involving that path.
- A successful flush of a file handle and a successful flush of each affected directory handle are separately required. “Write through” or an API name containing “replace” is not treated as a substitute without evidence.
- Marker replacement must preserve the old marker until one process-visible same-directory replacement operation succeeds. Database moves use no-replace semantics after explicit destination handling.
- Unsupported filesystem, remote/removable storage, reparse ambiguity, sharing violation, invalid-handle result, access denial, replacement failure, or flush failure is fail-closed.

Candidate assumptions to test—not accepted facts—are that a Windows directory open may require backup-semantics and that share-read/share-write/share-delete prevents the adapter’s own handles from blocking transitions. Exact access masks, flags, replacement primitive, flush primitive, filesystem query, crate, version, and crate feature list remain unresolved until W0. `Cargo.toml` receives a direct `[target.'cfg(windows)'.dependencies]` entry only after that selection; transitive lockfile packages must not be imported directly.

## `DatabaseState` settlement flow

`DatabaseState` continues to own SQLite and application state, not filesystem ordering:

1. Create and validate stage/protective using existing SQLite paths; close both validation/writer connections.
2. Call `prepare_durable_restore(stage, protective)` while the live connection remains usable. Phase verification and all completed-sidecar recycling finish before any `Restoring` transition.
3. On success set `Restoring`, take and drop the live connection.
4. Call the single `install_durable_restore(stage, canonical)` operation; it owns rollback, both marker advances, and all barriers.
5. Open and fully validate canonical; assign the new connection and set `Ready`.
6. Call `complete_durable_restore`; cleanup failure returns `restore_failed` but does not discard the validated connection.
7. On any post-disruption failure, retain marker evidence where the OS permits, validate canonical → rollback → protective, use `recover_canonical_durably` only for fallback copy after Recovery verification, then publish `Ready` or `Unavailable` exactly as ticket 01 requires.

Startup first reads a valid active marker. If canonical already validates, it opens and publishes that new connection as `Ready`, then runs Completion verification before idempotent absence handling or marker mutation. If fallback is needed, it selects only the exact validated rollback or protective path, closes its SQLite validation connection, obtains Recovery verification proof, copies durably, opens and validates canonical, publishes `Ready`, and only then invokes Completion. A missing or malformed marker, wrong source or canonical path, unsupported layout, cross-volume relation, unsafe file type, or reparse ambiguity fails before native mutation. Normal marker-free open remains unchanged and does not call recovery mutation; unsupported durability is encountered when a restore transition or completion requires this seam.

## Test design

### Deterministic protocol tests

Private unit tests use `RecordingFs`, a modeled namespace, and a failpoint keyed by `(operation_kind, ordinal)`. Table-drive failure immediately before and after every file sync, directory barrier, removal, rename/replacement, and marker publication for prepare, install, fallback, and completion.

Assert recorded exact sequences and modeled terminal invariants: marker never leads; `Prepared` precedes disruption; `LiveMoved` follows root barrier; `CandidateInstalled` follows both directory barriers; fallback source is never removed; cleanup follows readiness callback; every post-disruption state retains modeled canonical, rollback, or protective evidence. Add phase-verification assertions that no native, SQLite, or file handle remains open at the first mutation.

Required focused cases are: five sequential successful prepare/install/Ready/complete cycles proving sidecars are recycled and slots reused; failpoints before/after the initial retry barrier and each sidecar remove/root-barrier pair followed by successful retry; malformed JSON, non-regular, reparse, and out-of-range sidecars retained and rejected before disruption; Recovery rejection for missing/malformed marker, wrong rollback/protective source, wrong canonical path, cross-volume or unsupported storage, and unsafe/reparse files before any mutation; Completion rejection on an unsupported boundary before marker mutation; and absent-marker idempotent success only after `Ready` and supported-boundary verification. Existing stale `.part`, rollback, marker replacement, ordering, source-retention, and bounded-outcome tests remain.

### Real filesystem tests

Keep/update `src-tauri/tests/backup_restore.rs` for unchanged JSON bytes/states, canonical → rollback → protective selection, successful install, permanent rollback/protective retention, stale temporary retry, cleanup failure with `Ready`, and `database_unavailable` when no source validates. Add the five-cycle, cleanup retry, malformed-sidecar, recovery precondition, unsupported Completion, and absent-marker-after-Ready cases at the narrowest appropriate seam. Non-Windows tests verify unsupported production selection fails before disruption; successful durable-transition integration tests are `cfg(windows)` and require NTFS.

### Mandatory Windows local-NTFS evidence

Acceptance requires a recorded run on real Windows with app-data on local NTFS, not Wine, a cross-compile, API documentation, a recorder, or only process-kill tests. Evidence must show:

1. selected dependency/features compile and exact native calls execute;
2. filesystem/local-volume checks accept NTFS and reject tested unsupported storage;
3. marker create, replace, remove, Prepare-only sidecar recycling, and their barriers work with retries across five sequential cycles;
4. Prepare, Recovery, and Completion verification reject unsupported, wrong-path, and reparse layouts before mutation, while canonical→rollback and stage→canonical work only after all handles close;
5. both stage-source and app-data-destination barriers are observed;
6. injected sharing/flush/replacement failures return bounded failure and retain recoverable evidence;
7. restart recovery preserves canonical → rollback → protective and `Ready`/`Unavailable` outcomes;
8. the Windows native focused suite is rerun after these lifecycle and phase-verification changes, including cleanup failpoints and absent-marker Completion.

Process-kill cases may prove restart protocol behavior but are explicitly not physical power-loss evidence.

## File-change plan

- `src-tauri/src/infrastructure/filesystem/restore_transitions.rs`: new private deep implementation, platform modules, recorder/failpoint unit tests.
- `src-tauri/src/infrastructure/filesystem/backup_store.rs`: delegate narrow restore interface; preserve unrelated backup publication.
- `src-tauri/src/infrastructure/filesystem/mod.rs`: private module declaration; no platform primitives exported.
- `src-tauri/src/lib.rs`: settlement calls only; no file ordering.
- `src-tauri/tests/backup_restore.rs`: compatibility, real filesystem, bounded outcome, and Windows gate coverage.
- `src-tauri/Cargo.toml`/`Cargo.lock`: target-specific direct dependency only after W0 proves exact selection.

## Feasibility and rollout gates

**W0 — blocks Windows adapter/Cargo selection:** on real Windows local NTFS, build a minimal repository-local feasibility spike that proves documented access/share/flag values, directory flushing, file flushing, marker replacement, no-replace rename, volume/filesystem detection, closure behavior, and error reporting. Record exact crate/version/features and native contracts. If any required operation is unsupported, stop apply; do not ship a reduced guarantee.

**W1 — protocol implementation:** implement the private seam, recorder, failpoints, unsupported non-Windows adapter, and settlement call shape. This may merge only if it cannot be represented as completed Windows durability support.

**W2 — Windows implementation and acceptance:** implement the W0-proven adapter and target dependency, run focused/full Rust tests and the NTFS runtime matrix, then enable the bounded claim. Missing W2 evidence blocks acceptance/release of this change.

Rollback reverts the private module, settlement calls, tests, and target dependency together. Marker filename, JSON, states, rollback/protective filenames, and source order are unchanged, so no data or marker migration is needed; rollback merely restores the prior weaker durability behavior.

## Residual risks

- Successful OS barriers still depend on filesystem, controller, device, and hardware behavior; arbitrary power-loss survival is not promised.
- A directory-flush failure after process-visible removal creates unavoidable namespace uncertainty; fail-closed reporting and durable canonical evidence bound, but cannot erase, that uncertainty.
- Antivirus, indexing, backup software, or another process may hold incompatible Windows handles and cause a safe failure.
- Stage directory ancestry may evolve; layout validation must reject unexpected paths rather than omit an affected directory barrier.
- No Windows runtime evidence exists in the current environment, so W0 and W2 remain open and the durability claim is not yet acceptable.
