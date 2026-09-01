# Technical Design: Recover After Live Restore Failure

## Context

`DatabaseState::install_validated_stage` owns the database mutex throughout live replacement. It creates and validates a protective snapshot, writes `Prepared`, changes the in-memory status to `Restoring`, drops the active connection, and then performs the durable file/marker sequence. Every fallible step after the drop currently uses `?`, so a failure can return with `status = Restoring` and `connection = None`.

The existing command gate is already correct: `with_read` and `with_write` accept only `Ready` and require a present connection. Startup recovery also has the required source policy: validate canonical first, then rollback, then protective, copy a validated fallback to canonical, open and validate canonical, and clear the marker. The design therefore deepens the existing `DatabaseState` module instead of adding a storage adapter or changing the restore protocol.

## Goals and invariants

After every ordinary `Result` exit that occurs after the active connection is dropped:

- state is `Ready` with a newly opened connection that passed the existing restore validation; or
- state is `Unavailable` with `connection = None`.

Additional invariants are:

1. The pre-restore connection is dropped once and is never retained or reused.
2. The mutex remains held until one terminal state is published, so commands cannot observe an intermediate connection.
3. Recovery uses canonical, rollback, and protective evidence in that order.
4. No path capable of creating a new empty canonical database runs until an existing canonical or fallback source has been selected and validated.
5. Recovery of service does not change the failed restore result into success.
6. Marker states, file names, transition ordering, backup format, WAL behavior, and recovery-source retention remain unchanged.

Panics, process termination, and filesystem durability redesign are outside this ticket; restart recovery remains responsible for crash outcomes.

## Design decisions

### 1. Separate preparation, disrupted replacement, and settlement

Keep the pre-disruption work in `install_validated_stage` unchanged: validate state readiness, create and validate `pre-restore.sqlite3`, and write `Prepared`. Errors here return `restore_failed` while the original `Ready` connection remains owned.

Immediately after setting `Restoring` and taking/dropping the connection, stop using `?` from the outer method. Run the existing move/marker/install/open/validate sequence as one captured `Result<Connection, ()>`. This sequence returns a connection only after `open_database` and `validate_restored_database` both succeed; it does not publish state and does not clear the marker.

The outer method then settles exactly once:

- **Candidate validated:** install the returned connection, set `Ready`, then attempt marker cleanup.
- **Replacement failed:** retain the original bounded `restore_failed` outcome and invoke in-process marked recovery while still holding the mutex.

This places terminal-state ownership in one method and prevents a new post-drop `?` from bypassing settlement.

### 2. Extract one private validated-recovery seam

Refactor `recover_marked_database` into a private helper with the conceptual interface:

```rust
fn open_validated_recovery_database(
    config: &DatabaseConfig,
    store: &BackupStore,
) -> Result<rusqlite::Connection, ()>
```

Its implementation preserves current startup policy:

1. Probe canonical read-only with `is_valid_database`.
2. If canonical is not valid, probe rollback and then protective read-only.
3. Copy the first validated fallback to canonical through the existing `restore_canonical_from` operation.
4. Only after valid evidence has been selected, call `open_database` for canonical and run `validate_restored_database` again on the writable connection.
5. Return the connection without changing in-memory state and without clearing the restore marker.

This is the smallest deep seam: source selection, safe canonical establishment, writable open, and final validation stay behind one private interface shared by startup and live recovery. No trait or general-purpose filesystem abstraction is introduced because production behavior does not vary.

`recover_on_startup` remains behaviorally compatible by calling this helper and then requiring `clear_restore_state` to succeed before constructing `Ready`. If selection, canonical restoration, writable open, validation, or marker cleanup fails, startup constructs `Unavailable` as it does today.

### 3. Publish readiness before fallible final cleanup

For a normally installed and validated candidate, publish `connection = Some(connection)` and `status = Ready` before calling `clear_restore_state`, matching the current safety property. If marker cleanup fails, return `restore_failed` but preserve the newly validated connection and `Ready` status. The retained marker is restart-compatible and will cause the existing canonical-first recovery path on the next launch.

For recovery after a replacement failure:

- If `open_validated_recovery_database` returns a connection, publish it as `Ready`, attempt marker cleanup, and return the **original** `restore_failed` result regardless of cleanup success or failure.
- If recovery cannot return a validated connection, assign `connection = None`, assign `status = Unavailable`, and return `database_unavailable`.

The original error is retained as the bounded code captured before recovery; recovery diagnostics and cleanup diagnostics never replace it while service is `Ready`. Only failure to establish any validated service connection changes the command outcome to `database_unavailable`.

### 4. Preserve the existing command contract

`DatabaseState::install_validated_stage` continues returning `Result<(), String>` with stable internal codes. `confirm_restore` already maps `database_unavailable` distinctly and maps every other installation failure to `restore_failed`; no protocol or serialized response change is required.

The implementation must not concatenate filesystem paths, SQLite errors, marker states, or recovery-source details into these strings. Internal helper errors remain erased to `()` or a private non-displayable enum.

## Data flow

### Successful restore

1. `confirm_restore` consumes and rechecks the staged candidate.
2. `install_validated_stage` prepares the protective database and `Prepared` marker.
3. Under the mutex, status changes to `Restoring` and the old connection is dropped.
4. Existing durable move/install/marker operations run unchanged.
5. Canonical is newly opened and fully validated.
6. The connection is published and status becomes `Ready`.
7. Marker cleanup succeeds and the command returns the existing restored response.

### Failed restore with service recovery

1. Steps through connection disruption are identical.
2. A later transition, open, or validation operation fails and is captured as `restore_failed`.
3. The shared helper validates canonical, otherwise rollback, otherwise protective; a fallback is copied to canonical when required.
4. Canonical is newly opened and validated.
5. The recovered connection is published and status becomes `Ready`.
6. Marker cleanup is attempted; its result does not overwrite the original failure.
7. The restore command returns `restore_failed`; later reads and writes use only the recovered connection.

### Failed restore without service recovery

1. A post-disruption operation fails.
2. No source can establish a newly opened validated canonical connection.
3. Settlement explicitly writes `connection = None` and `status = Unavailable` before returning.
4. The restore command returns `database_unavailable`; subsequent `with_read` and `with_write` calls reject access without invoking their closures.
5. The marker and recovery files are retained for a future startup attempt.

## File-level changes

### `src-tauri/src/lib.rs`

- Capture all post-drop replacement failures rather than returning through `?`.
- Add the private shared validated-recovery helper and make marker cleanup caller-owned.
- Add one settlement path that publishes either `Ready` with a validated connection or `Unavailable` with none.
- Preserve current locking, public method shape, status gate, file sequence, and marker values.
- Add small colocated unit coverage only if a private cleanup callback seam is needed to prove cleanup-after-validation behavior; do not expose a production fault-injection interface.

### `src-tauri/src/commands/backup.rs`

- No behavioral change is expected. Verify the existing mapping remains `database_unavailable` for that exact code and `restore_failed` for recovered installation failures.

### `src-tauri/tests/backup_restore.rs`

- Add focused live-failure and terminal-state coverage using real temporary SQLite files and `BackupStore`.
- Reuse existing startup recovery fixtures to prove shared source ordering remains unchanged.

No schema, migration, frontend, IPC shape, Cargo feature, or `BackupStore` protocol change is planned.

## Deterministic test seam

Use the existing `install_validated_stage` interface as the primary fault seam; do not add a broad production abstraction.

A deterministic post-disruption recoverable fault is produced by passing a malformed staged file. Preparation succeeds because this low-level method intentionally assumes command staging already validated the candidate. The live database is moved to rollback, the malformed stage is installed, and final candidate validation fails only after the original connection has been dropped. Recovery must reject canonical, restore the valid rollback, return `restore_failed`, and expose prior data through a new connection.

A deterministic unavailable outcome uses the same malformed stage plus a directory at `restore-recovery.sqlite3.part`. After canonical validation fails, the valid fallback is selected, but the existing fallback-copy operation cannot remove its temporary-path directory, so no source can establish canonical storage. Settlement must return `database_unavailable`, leave no connection, and make both read and write closures remain uncalled. This fixture exercises real production mechanics without hooks, traits, sleeps, races, permission assumptions, or platform-specific file locking.

For cleanup-after-validation, prefer a private function/closure seam local to `lib.rs` only if a portable filesystem fixture cannot target `clear_restore_state` without also breaking earlier marker writes. The private seam should vary only the final `clear_restore_state` result, be exercised by a colocated unit test, and never appear in the public `DatabaseState` interface or `BackupStore` interface.

## Test matrix

1. **Recover prior database after candidate validation failure:** malformed stage, valid rollback/protective; assert `restore_failed`, `Ready`, prior category visible, rejected candidate absent, and marker cleared when cleanup works.
2. **Canonical-first recovery:** post-disruption failure with a valid canonical candidate and distinct rollback/protective data; assert canonical data wins.
3. **Rollback then protective ordering:** preserve existing focused startup tests and, where practical, exercise the shared helper through live settlement to show invalid canonical falls to rollback and invalid rollback falls to protective.
4. **Explicit unavailable:** block fallback canonical establishment with the deterministic temporary-path fixture; assert `database_unavailable`, read and write both reject, and neither closure executes.
5. **Cleanup failure after validation:** inject only final cleanup failure through the private local seam; assert `Ready`, validated data remains usable, marker remains, and result is `restore_failed`.
6. **Bounded command mapping:** assert responses contain only `restore_failed` or `database_unavailable` and the existing fixed messages, with no temporary path, SQLite text, or marker detail.
7. **Regression:** existing successful live restore and startup recovery tests remain unchanged and pass.

Run focused tests first, then the relevant Rust suite:

```sh
cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore
cargo test --manifest-path src-tauri/Cargo.toml
```

## Rollout and rollback

This is an in-process orchestration change with no persistent-format migration. Deploy with the focused Rust coverage and ordinary application release. Existing markers and recovery files remain readable by both old and new versions.

Rollback reverts the `DatabaseState` orchestration/refactor and its tests together. Stored databases, backups, marker values, rollback files, and protective snapshots need no conversion. The known prior behavior—requiring restart after some live failures—would return.

## Residual risks

- External processes can still mutate recovery files between read-only validation and copying/opening; final validation prevents publishing invalid storage but cannot make external filesystem interference atomic.
- Marker cleanup failure intentionally leaves a valid running connection plus a marker; restart will revalidate canonical and may remain unavailable if storage changes before restart.
- The fallback-copy implementation stops when establishment from the first validated fallback fails rather than trying a later fallback; this preserves current startup behavior and ticket 02's protocol boundary.
- Process crashes can occur between any durable operations; this ticket preserves, rather than strengthens, existing filesystem durability guarantees.
- Mutex poisoning still maps through the existing persistence failure behavior and is not redesigned here.
