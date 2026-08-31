# Apply Progress: Post-sale Lifecycle

## Delivery context

- Strategy: `ask-on-risk` resolved to `feature-branch-chain`.
- Current: **PR6C/task 6.4 complete**; PR7 UI is next. No delivery actions were created.
- Follow-up: PR7 UI; PR8 backup/integrity.

## Completed tasks

- [x] 0.1 Delivery gate
- [x] 1.1 RED — migration contract test written and observed failing at schema version 9.
- [x] 1.2 GREEN — added additive v10 migration and migration registration/structural validation.
- [x] 1.3 TRIANGULATE — added focused correction fact immutability and zero-cancellation movement constraint evidence.
- [x] 1.4 REFACTOR — formatted Rust and retained migration-local validation.
- [x] 3.2 GREEN — SQLite return adapter/use case is complete across PR3A1/PR3A2A/PR3A2B/PR3B/PR3C evidence.
- [x] 3.1 RED — return transaction cases are complete across PR3A2B/PR3B/PR3C/PR3D evidence.
- [x] 3.3 TRIANGULATE — return failure and concurrency evidence is complete across PR3B/PR3C/PR3D.
- [x] 3.4 REFACTOR — eligibility remains in the domain and Phase 3 traceability is complete.
- [x] 4.1 RED — cancellation transaction evidence is complete across PR4A/PR4B.
- [x] 4.2 GREEN — cancellation orchestration is complete across PR4A.
- [x] 4.3 TRIANGULATE — cancellation conflict, rollback, and serialization evidence is complete across PR4B.
- [x] 4.4 REFACTOR — shared replay/rollback plumbing is localized without merging business plans; Phase 4 traceability is complete.
- [x] 5.1 RED — Rust command contracts.
- [x] 5.2 GREEN — Rust commands.
- [x] 5.3 RED/GREEN — TypeScript adapter.
- [x] 5.4 TRIANGULATE/REFACTOR — TypeScript/Rust command seam parity.
- [x] 6.1 RED — cumulative SQLite history evidence is complete across PR6A1/PR6A2.
- [x] 6.2 GREEN — cumulative Rust history read model is complete across PR6A1/PR6A2.
- [x] 6.3 RED/GREEN — cumulative Rust/TypeScript history command contracts are complete across PR6B1/PR6B2A1/A2/B.
- [x] 6.4 TRIANGULATE/REFACTOR — immutable originals and legacy confirmed history evidence complete.

## Files changed

- Phase 1–4 schema, post-sale Rust domain/application/SQLite backend, and migration/lifecycle integration files — additive facts and atomic correction behavior.
- Phase 5 Rust/TypeScript post-sale command files — typed transport, runtime guards, and parity evidence.
- Phase 6 Rust history read model/command files and `src/commands/sales-history.ts` — lifecycle/correction projection and guarded history transport.
- `src/commands/sales-history.test.ts` — PR6B2 malformed, ordering, hostile-extra, and current-shape evidence.
- `openspec/changes/post-sale-lifecycle/tasks.md` and `apply-progress.md` — synchronized task/progress artifacts.

## Verification evidence

| Check | Cumulative result through Phase 4 |
| --- | --- |
| Migration suite | `cargo test --manifest-path src-tauri/Cargo.toml --test sqlite_migrations` passed: 16 tests. |
| Return lifecycle | `cargo test --manifest-path src-tauri/Cargo.toml --test post_sale_lifecycle return` passed: 9 tests, including return/cancellation serialization and the fully-returned cancellation case. |
| Cancellation lifecycle | `cargo test --manifest-path src-tauri/Cargo.toml --test post_sale_lifecycle cancellation` passed: 5 tests. |
| Formatting and diff validation | `cargo fmt --manifest-path src-tauri/Cargo.toml --check` and `git diff --check` passed. |
| Runtime harness | Focused migrated SQLite integration scenarios passed; PR4A happy/replay and PR4B rollback/serialization are covered. |
| PR5A2 command seam | Complete Rust contract matrix passed: DTO strictness, result/error serialization, mapping/leakage, and mock registration. |
| PR5B TypeScript seam | `npm test -- src/commands/post-sale.test.ts` passed: 49 tests; allowlisted success projection, malformed-line rejection, exact payloads, retry identity, conflict and malformed/rejected mapping passed. `cargo test --manifest-path src-tauri/Cargo.toml --test post_sale_commands` passed: 3 tests. |
| PR6A1 history seam | `cargo test --manifest-path src-tauri/Cargo.toml --test sales_history` passed: 3 tests, including a real migrated SQLite return/cancellation lifecycle projection. |
| PR6A2 history detail | `cargo test --manifest-path src-tauri/Cargo.toml --test sales_history` passed: 4 tests, including ordered headers/lines, zero-restored cancellation detail, and complete immutable-original snapshots. |
| PR6B1 Rust command seam | `cargo test --manifest-path src-tauri/Cargo.toml --test sales_history_commands` passed: 3 tests; `sales_history` regression passed 4 tests. |
| PR6B2B TypeScript matrix | `npm test -- src/commands/sales-history.test.ts` passed: 52 discovered tests; isolated/project TypeScript, test Prettier, Rust command 3, cargo fmt, and diff checks passed. |

## Scope, size, and rollback

- Cumulative implemented scope: PR1 schema-v10 facts, PR2 plans/contracts, complete Phases 3–4, and complete PR5 Rust/Tauri and TypeScript command transport/evidence.
- Authoritative PR4A delta against PR3E baseline tree `0892a8d3fb980a37eeff3a8736bf2abfc7ea707d`: repository `+97/-7`, integration test `+110/-2`, total `+207/-9 = 216` changed lines. Authoritative settled-tree PR4B delta from PR4A tree `f0bbf2593a633c42e90113729dd5749df4c22db2` to final PR4B tree `3cc55a94c0a7bbbc6ef79e9e038dafd1271c6aac`: `src-tauri/tests/post_sale_lifecycle.rs` `+373/-1 = 374` changed lines; production/export `+0/-0`.
- Rollback: PR4B removes only its cancellation test helpers/assertions; PR4A can restore cancellation repository stubs and remove its focused integration scenario. Neither affects committed schema, plans/contracts, or return behavior. Detailed boundary-specific rollback evidence remains in the historical sections below.

## Remaining tasks

- Phases 3–6 are complete. Task 7+ remains pending.

## PR7A1 — return-intent state seam

- Completed sub-boundary only: `history-flow` now opens a caller-ID-bearing return intent only for persisted confirmed details with a positive `remaining_returnable_quantity`; selections key on exact `sale_line_id`, not product ID.
- Local submission accepts only selected positive safe whole numbers within each persisted remaining quantity. A pending submission ignores duplicate starts; application errors retain all input values and the original request ID for retry.
- Successful submission changes only a `pending` intent to `reload_requested`; stale success while an intent is in `error` is ignored. It never changes detail quantities or status optimistically. The later presentation adapter must issue the authoritative history reload.
- Initial RED: `npm test -- src/ui/sales/history-flow.test.ts` failed with two missing return-intent reducer paths. Initial GREEN: the same command passed 54 tests after implementation. Gate-correction RED then proved stale success from `error` was incorrectly accepted; final GREEN passed all 54 discovered tests after an explicit retry returned the intent to `pending`.
- Verification: `npx tsc --noEmit` and `git diff --check` passed. `npx prettier --check src/ui/sales/history-flow.ts` passed. The complete test file retains only its pre-existing formatter delta: a reconstructed PR7A1 begin tree and the final tree both differ from their Prettier projections by `+155/-34`; the entire PR7A1 test region exactly matches its Prettier projection.
- Runtime harness: N/A — this is an injected pure state seam with no presentation or Tauri runtime path. GUI/manual testing is intentionally deferred to task 7.4.
- Authoritative immutable begin-to-final cumulative PR7A1 footprint: source `+216/-7`, tests `+95/-0`, tasks `+1/-0`, progress `+17/-0`, total `+329/-7 = 336` changed lines. The authoritative native correction-attempt delta was `+88/-60 = 148` changed lines; it is distinct from the final cumulative footprint. The final footprint remains within the 400-line budget; rollback removes the return-intent reducer/types/tests and this PR7A1 record, leaving existing history flow untouched.
- Task accounting: 7.1 remains pending (cancellation and presentation/accessibility evidence absent); 7.2–7.4 remain pending.

### PR7A1 automatic gate correction

- Corrected `return_submit_succeeded` to accept only `pending`, preventing a stale completion after `return_submit_failed` from requesting a history reload. Direct reducer evidence now covers `error → explicit retry start → pending → success`, ignored stale success, duplicate submit while pending, and empty selection/value, zero, negative, fractional, unsafe-integer, and over-remaining quantities.
- Correction rollback: restore the prior success guard and remove only the new reducer assertions/validation cases plus this record; retain the PR7A1 return-intent seam and every earlier Phase 6 change. No task checkbox changed.
- Runtime harness: N/A — the injected pure reducer is the public state seam; presentation and GUI remain deferred to tasks 7.3–7.4.


## PR 2 blocked attempt — domain/application contracts

- At the time of this blocked attempt, tasks 2.1–2.4 remained unchecked and no PR 2 source or test files were retained.
- RED evidence: `cargo test --manifest-path src-tauri/Cargo.toml post_sale` failed as expected before implementation because the planned public post-sale types and functions did not exist.
- A coherent RED/GREEN implementation was exercised with the same focused command: exit 0, four post-sale library tests passed (plus the existing matching migration test). It was then reverted because the required two pure modules, their focused tests, and exports totaled 486 authored additions, exceeding the hard 400-line work-unit limit.
- Runtime harness: N/A — this slice contains only pure domain/application contracts.
- Rollback boundary: complete; the temporary PR 2 modules and exports were removed. PR 1 schema files remain untouched.
- Required decision: split PR 2 at a new verification/rollback boundary before retrying; do not widen this work unit.


## PR 2A — pure domain plans

- Rescope: the previous PR 2 budget blocker was resolved by isolating tasks 2.1–2.2 as the autonomous domain-only slice. PR 2B retains tasks 2.3–2.4 (application contracts/identity).
- [x] 2.1 RED — added focused in-module behavior tests and observed the planned public types/functions fail to compile before implementation.
- [x] 2.2 GREEN — added pure return and cancellation planners with typed validation errors and exported their public domain interface.

### Verification evidence

| Check | Result |
| --- | --- |
| RED: `cargo test --manifest-path src-tauri/Cargo.toml post_sale` | Failed as expected: `RequestedReturnLine`, `SaleCorrectionState`, and `plan_return` did not exist. |
| GREEN/focused: `cargo test --manifest-path src-tauri/Cargo.toml post_sale` | Passed: 2 post-sale domain tests and the existing matching migration test; all other test targets filtered to zero. |
| Formatting: `cargo fmt --manifest-path src-tauri/Cargo.toml` | Passed. |
| Diff validation: `git diff --check` | Passed. |
| Runtime harness | N/A — PR 2A has no runtime boundary; it is pure domain logic. |

### Work-unit boundary

- Authored size: 271 additions (265-line corrected domain module/tests plus 6 export lines); under the 400-line budget.
- Start/end: public pure planners and their tests only; no request identity, application repository, SQLite, command, UI, delivery, or backup work.
- Rollback: remove `src-tauri/src/domain/sales/post_sale.rs` and its exports from `src-tauri/src/domain/sales/mod.rs`; no persisted facts or schema need reversal.
- Remaining: PR 2B tasks 2.3–2.4 remain unchecked.


### PR 2A gate correction

- Fixed malformed original facts: `returned_quantity > sold_quantity` now follows checked subtraction and is rejected before cancellation can emit a negative residual.
- Added direct negative-request and over-returned-original tests; the latter failed RED with `restored_quantity: -1` before the fix.
- GREEN: `cargo test --manifest-path src-tauri/Cargo.toml post_sale` passed after the correction; formatting and `git diff --check` passed.
- Correction scope/rollback: only `post_sale.rs` changed; removing the module and its exports remains the full PR 2A rollback boundary.


## PR 2B — application contracts and identity

- [x] 2.3 RED/GREEN — added application request/result DTOs, stable typed errors, persisted request identity, a narrow repository seam, and the two-method lifecycle interface.
- [x] 2.4 TRIANGULATE/REFACTOR — proved canonical return ordering and that changed quantity, sale, reason, operation, or SHA-256 value does not match a persisted request.

### Verification evidence

| Check | Result |
| --- | --- |
| RED: `cargo test --manifest-path src-tauri/Cargo.toml post_sale` | Failed as expected: `CreateReturnRequest` and its required application dependencies did not exist. |
| GREEN/focused: `cargo test --manifest-path src-tauri/Cargo.toml post_sale` | Passed: 4 post-sale library tests and the existing matching migration test; all other targets were filtered to zero. |
| Formatting: `cargo fmt --manifest-path src-tauri/Cargo.toml` | Passed. |
| Diff validation: `git diff --check` | Passed. |
| Runtime harness | N/A — PR 2B defines pure application contracts and has no runtime boundary. |

### Work-unit boundary

- Authored size: 275 additions (269-line application module/tests plus 6 export lines); under the 400-line budget.
- Start/end: application contracts and canonical identity only; no concrete repository, transaction orchestration, SQLite, command, UI, history, backup, or delivery work.
- Rollback: remove `src-tauri/src/application/sales/post_sale.rs` and its exports from `src-tauri/src/application/sales/mod.rs`; PR 2A domain plans remain intact.
- Remaining: PRs 3–8 and final traceability/regression checks.

## PR 3 blocked pre-implementation — atomic return backend

- Tasks 3.1–3.4 remain unchecked; no PR 3 source, test, or registration change was created.
- Measured scope evidence: the existing single-sale SQLite repository is 272 lines and its transaction integration suite is 931 lines. PR 3 additionally requires multi-line request lookup/replay/conflict, exact line fact loading, one `IMMEDIATE` transaction, checked balance updates, movement and correction writes, persisted-result reconstruction, injected rollback, and competing-writer coverage.
- The 272-line repository reference alone leaves only 128 lines of the hard 400-line budget for all PR 3-specific transaction behavior and its mandatory integration tests. This cannot fit coherently without dropping required behavior or evidence.
- Required rescope: split PR 3 into a smaller atomic happy-path/replay slice and a separately authorized failure/concurrency slice, each with its own test and rollback boundary. Do not check partial tasks 3.1–3.4 against the current plan.
- Runtime harness: not run because no PR 3 candidate was created. PR 1 and PR2 artifacts remain untouched.


## PR 3A history

- The original 336-addition happy-path/replay candidate passed its temporary SQLite test but incorrectly owned `IMMEDIATE` in the adapter. A readable corrected combined candidate measured 475 additions and was reverted for the native budget breach.
- Two subsequent PR3A1 rescope confirmations retained no code because their remaining native capacity (32, then 28 lines) was below the minimum transaction seam before interface/test evidence.

## PR 3A1 — application-owned transaction seam

- Completed sub-boundary: `PostSaleUseCase` implements the two-method lifecycle interface and owns the transaction lifecycle through `PostSaleTransactionFactory::begin_immediate`. The factory supplies the repository transaction; the use case performs identity lookup, fact loading, domain planning, replay loading, and persistence only through `PostSaleRepository`.
- `finish` commits only success. It explicitly attempts rollback after every operation/domain/replay error and also after a commit error; any rollback failure deterministically maps to `PostSaleError::PersistenceFailure`. The abstraction deliberately does not assert an impossible rollback over a consumed concrete `rusqlite::Transaction`; PR3A2 must provide the concrete factory/transaction implementation.
- Original tasks 3.1–3.4 remain unchecked because this is only the application transaction seam; PR3A2 owns the SQLite return adapter and integration behavior.

### Verification evidence

| Check | Result |
| --- | --- |
| RED: `cargo test --manifest-path src-tauri/Cargo.toml post_sale` | Failed as expected: retained focused tests referenced absent `PostSaleTransaction` and `PostSaleTransactionFactory` abstractions. |
| GREEN/focused: `cargo test --manifest-path src-tauri/Cargo.toml post_sale` | Passed: 8 post-sale unit tests, including fake transaction success→commit, operation failure→rollback, commit failure→rollback, and commit+rollback failure mapping; the matching migration test also passed. |
| Formatting: `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Passed. |
| Diff validation: `git diff --check` | Passed. |
| Runtime harness | N/A — PR3A1 provides a pure application seam and fake transaction/repository coverage; no production adapter exists. |

### Work-unit boundary

- Authoritative cumulative tree readback from baseline `f20b3e...` to corrected settled tree `cd75e8...`: `application/sales/mod.rs` is `+2/-1` and `application/sales/post_sale.rs` is `+360/-4`, for cumulative PR3A1 `+362/-5 = 367 changed lines`; within the 400-line limit. Native settlement `changed_lines: 290` was the correction-attempt delta, not the cumulative PR3A1 size.
- Rollback: remove `PostSaleUseCase`, `PostSaleTransaction`, `PostSaleTransactionFactory`, orchestration helpers, their focused fake transaction/repository tests, and the application exports; PR1/PR2 domain and contract types remain.
- Next: PR3A2 may add only the concrete SQLite return adapter, its transaction-factory implementation, and temporary-SQLite integration evidence against this seam.

## PR 3A2A — SQLite transaction factory and lifecycle

- Completed sub-boundary: added the concrete `SqlitePostSaleTransactionFactory` and supplied-transaction wrapper required by PR3A1. The factory begins exactly one `TransactionBehavior::Immediate`; its wrapper exposes the transaction only to the repository seam.
- Lifecycle truthfulness: fixed `COMMIT`/`ROLLBACK` statements run on the still-owned wrapper transaction. On successful completion, `DropBehavior::Ignore` prevents a second finalization; a failed commit leaves rollback capability with the wrapper for the application use case to attempt.
- No `PostSaleRepository` persistence, return/replay behavior, commands, or UI was added. Original tasks 3.1–3.4 remain unchecked; PR3A2B owns concrete return persistence and replay proof.

### Verification evidence

| Check | Result |
| --- | --- |
| RED: `cargo test --manifest-path src-tauri/Cargo.toml --test post_sale_lifecycle lifecycle` | Failed as expected: `SqlitePostSaleTransactionFactory` export did not exist. |
| GREEN/focused: same command | Passed: 1 migrated in-memory SQLite lifecycle test. It proves schema-v10 access, one committed transaction, a separate explicit rollback, and no rolled-back effect. |
| Formatting: `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Passed. |
| Diff validation: `git diff --check` | Passed. |
| Runtime harness | Passed — the focused integration test migrates a real in-memory SQLite database. |

### Work-unit boundary

- Authored source/test/export size: 99 additions (53-line lifecycle module, 44-line focused integration test, and 2 SQLite module/export lines); under the 400-line limit.
- Rollback: remove `src-tauri/src/infrastructure/sqlite/post_sale_transaction.rs`, `src-tauri/tests/post_sale_lifecycle.rs`, and its two `sqlite/mod.rs` registrations. PR3A1 application contracts remain.
- Next: PR3A2B may add only `PostSaleRepository` return persistence/replay behavior and integration assertions against this lifecycle seam.

## PR3A2B — SQLite return persistence and canonical replay

- Completed sub-boundary: `SqlitePostSaleRepository` persists return request/header/lines, exact sale-line facts, checked stock increments, and linked return movements only through the supplied PR3A1 transaction. Exact canonical replay reloads the persisted result without writes.
- RED: `cargo test --manifest-path src-tauri/Cargo.toml --test post_sale_lifecycle return` failed because `SqlitePostSaleRepository` was absent. GREEN: the same command passed 1 migrated in-memory SQLite return/replay test; `cargo fmt --check` and `git diff --check` passed.
- Runtime: passed. The focused integration test proves two-line original-line identity, request/header/line/movement facts, stock increments, reordered replay without duplicate effects, and compares one original line's quantity/line total, the sale total, and payment row count. Adapter inspection confirms its write set does not update original sale facts.
- Authoritative PR3A2B source/test/export delta from native begin/finish trees: repository `+215/-0`, test `+163/-4`, exports `+2/-0`, total `+380/-4 = 384 changed lines`; the native total including apply-progress is 398.
- Rollback: remove `post_sale_repository.rs`, the PR3A2B integration additions, and two SQLite registrations; retain PR3A1/PR3A2A. Original tasks 3.1–3.4 remain unchecked; PR3B owns failure/conflict/concurrency coverage.

## PR3B — return failure, conflict, rollback, and writer serialization evidence

- Completed sub-boundary: focused SQLite integration coverage proves a changed canonical payload under the same request ID returns `PostSaleError::RequestConflict` with request/header/line/movement and stock effects unchanged.
- An injected `BEFORE INSERT` SQLite trigger fails return-line persistence after the operation has begun. The use case maps the storage error to `PostSaleError::PersistenceFailure`, explicitly rolls back, and the test proves request/header/line/movement counts and both stock balances match their pre-attempt state.
- A migrated two-connection file-backed scenario snapshots the contender-visible balances, then holds one factory-owned `IMMEDIATE` transaction. A second writer with a one-millisecond busy timeout receives `PostSaleError::PersistenceFailure`; before lock release, the test proves both zero correction facts and unchanged contender-visible balances. After release, the same request succeeds once with the expected two-line facts. This exercises SQLite writer serialization without a partial outcome.
- Test-first result: the three focused cases were added before any production edit. Their first run was already GREEN because retained PR3A1/PR3A2A/PR3A2B behavior satisfied the new acceptance evidence; no artificial RED or production change was introduced. `cargo test --manifest-path src-tauri/Cargo.toml --test post_sale_lifecycle return` passed 4 tests; `cargo fmt --manifest-path src-tauri/Cargo.toml --check` and `git diff --check` passed.
- Concrete commit-failure injection remains intentionally unproven: `SqlitePostSaleTransaction` executes a fixed `COMMIT` while retaining its wrapper for PR3A1's explicit rollback attempt, but a deterministic commit fault would require a test-only lifecycle hook or storage corruption. Neither is an honest production-path test seam. PR3A1 fake-transaction coverage retains commit-failure→rollback and commit+rollback-failure→`PersistenceFailure` proof.
- Authoritative PR3B source/test/export delta from the PR3A2B settled tree/blob: `src-tauri/tests/post_sale_lifecycle.rs` `+141/-2 = 143` changed lines (net `+139`); no production or export changes. The retained slice is below the 400-line budget.
- Rollback: remove the three PR3B integration tests and their temporary-directory/import helper; retain PR3A1/PR3A2A/PR3A2B. At the PR3B boundary, original tasks 3.1–3.4 were still unchecked. PR3C owns the return edge matrix.

## PR3C — return edge matrix

- Completed sub-boundary: return construction rejects zero and negative quantities with `PostSaleError::InvalidRequest` and duplicate sale-line identities with `PostSaleError::Domain(DuplicateSaleLine)` before persistence. The integration scenario asserts effects and balances remain unchanged.
- The single-transaction return seam rejects over-remaining quantities with `Domain(QuantityExceedsRemaining)`, both a globally unknown line and a known line belonging to a separate confirmed sale with `Domain(SaleLineNotFound)`, and an absent sale with the newly explicit `Domain(SaleNotFound)`. Every rejection is asserted against unchanged request/header/line/movement facts and all relevant stock balances.
- SQLite `load_facts` now maps an absent sale to `SaleNotFound` rather than conflating absence with `SaleNotConfirmed`; this supplies the stable public identity error required by the return contract while keeping business eligibility in the domain.
- Deleting one required stock row before a two-line request produces `PersistenceFailure` only after the earlier plan line has begun persistence. Explicit rollback restores the request/header/line/movement counts, the first product balance, and the missing-row state exactly to their pre-attempt values.
- RED: `cargo test --manifest-path src-tauri/Cargo.toml --test post_sale_lifecycle return` failed as expected because `PostSaleDomainError::SaleNotFound` did not exist. GREEN: the same command passed 6 tests after the minimal domain/adapter correction. The PR3C gate-correction known-line/wrong-sale test was immediately GREEN against retained behavior, so no production change was needed. `cargo fmt --manifest-path src-tauri/Cargo.toml --check` and `git diff --check` passed.
- Authoritative PR3C source/test/export delta from the PR3B settled tree/blob `a5730a46e72320d3184acc53e0879ba7e56f8980`: test `+187/-1`, domain `+1/-0`, repository `+9/-8`, total `+197/-9 = 206` changed lines. No exports changed.
- Task accounting: checked 3.2 because cumulative PR3A1 through PR3C now implements its complete adapter/use-case acceptance scope. Tasks 3.1 and 3.3 remain unchecked because the existing busy test proves a blocked contender then retry, not two independently completing overlapping returns against the same remaining quantity; 3.4 remains unchecked pending the final return-scope refactor/traceability review.
- Rollback: remove the PR3C test additions and revert `SaleNotFound` plus the absent-sale mapping in `load_facts`; retain PR3A1/PR3A2A/PR3A2B/PR3B. Next: PR3D overlapping-return serialization.

## PR3D — overlapping-return serialization

- Completed sub-boundary: a file-backed migrated SQLite scenario starts two independent threads at one barrier. Each opens its own connection, sets a one-second busy timeout, constructs its own transaction factory/use case, and submits a distinct request ID for the same two-unit remaining quantity on one original sale line.
- Both callers complete through SQLite serialization: exactly one persists a return and exactly one receives `Domain(QuantityExceedsRemaining)` after re-reading the committed aggregate. The test asserts one request/header/line/movement/link, aggregate returned quantity exactly two, and the exact first-product stock delta of two with the other balance unchanged.
- This is stronger than PR3B's blocked-contender/retry proof because both independent operations are launched concurrently and each receives its final outcome without a caller-managed retry.
- Complete immutable-original-fact proof now snapshots and compares after both first return and canonical replay: the full original `sales` row (`id`, `request_id`, `status`, `total_centavos`, `confirmed_at`), every original `sale_lines` row in ID order (`id`, sale/product identity, quantity, negotiated/minimum prices, line total, SKU/name snapshots), and every `sale_payments` row in ID order (IDs, sale identity, method, applied/tendered/change values).
- Test-first result: the concurrency test and this immutability expansion were immediately GREEN against retained behavior, so no production change was needed. `cargo test --manifest-path src-tauri/Cargo.toml --test post_sale_lifecycle return` passed 7 tests; `cargo fmt --manifest-path src-tauri/Cargo.toml --check` and `git diff --check` passed.
- Authoritative cumulative PR3D source/test/export delta from the PR3C settled tree/blob `9c2f9d7a0e50abe6e2586ec17503888371f167f8`: test `+128/-22 = 150` changed lines; no production or export changes. The immutability gate-correction delta from PR3D's settled candidate was `+58/-21 = 79` changed lines.
- Task accounting: checked 3.1 and 3.3 in OpenSpec and Engram because the cumulative evidence now covers every listed return transaction, failure, replay/conflict, identity, busy, and competing-writer scenario. Task 3.4 remains unchecked.
- Rollback: remove the PR3D overlapping-return test and its thread synchronization imports; retain PR3A1 through PR3C. Next: PR3E return refactor/traceability.

## PR4 pre-implementation blocker — historical

- No PR4 source, test, or export changes were retained; tasks 4.1–4.4 remain unchecked in OpenSpec and Engram.
- Preflight: `cargo test --manifest-path src-tauri/Cargo.toml --test post_sale_lifecycle cancellation` exited 0 with 0 cancellation tests run and 8 filtered out. This is coverage discovery only, not cancellation behavior evidence.
- Readable decomposition estimate: SQLite cancellation persistence/reload with per-original-line zero-residual handling requires at least 110–125 lines; focused migrated tests for happy/replay/immutability, prior-return and fully-returned residuals, request conflicts, injected rollback, and return/cancel serialization require at least 315 lines including shared support. The conservative complete total is at least 450 lines, exceeding the 400-line work-unit limit before artifact delta.
- Required split: PR4A implements application/SQLite cancellation happy path, replay, partial-return residuals, and fully-returned zero-movement facts; PR4B supplies invalid/conflict, rollback, and return/cancel concurrency evidence. A PR4C refactor/traceability slice is optional only if PR4B evidence cannot remain readable under budget.
- Rollback boundary: no PR4 implementation exists to revert; PR1–PR3 source, tests, tasks, and evidence remain unchanged.

## PR3E — return refactor and traceability

- Completed task 3.4 without production churn. Structural readback confirms `PostSaleUseCase` owns transaction orchestration and calls the pure domain planner; `SqlitePostSaleRepository` maps persisted facts, executes fixed parameterized SQL through the supplied transaction, validates exact stock row counts, and reloads persisted results. It contains no return eligibility, remaining-quantity, duplicate-line, or request-conflict decisions.
- The repository's long SQL strings are fixed statements with all values bound through `?N`/`params!`; splitting them would not deepen the module or reduce the caller-facing interface. The existing focused helpers keep SQLite setup, facts, stock effects, and immutable-original snapshots local to `post_sale_lifecycle.rs`, so no code refactor was necessary.

### Phase 3 traceability

| Task / requirement | Implementation seam | Focused evidence |
| --- | --- | --- |
| 3.1 atomic multi-line return, exact line identity, immutable originals, replay | `PostSaleUseCase` + `SqlitePostSaleRepository::persist_return` | `return_persists_by_original_line_and_replays_canonical_requests` |
| 3.2 one caller-owned `IMMEDIATE` transaction and persistence/reload | `PostSaleTransactionFactory`, `PostSaleTransaction`, supplied `&Transaction` repository seam | lifecycle and return integration tests |
| 3.3 conflict, injected rollback, identity/bounds, missing stock, busy, concurrent over-return | pure `plan_return` + SQLite transaction/repository | `conflicting_return_retry_*`, `injected_return_line_failure_*`, `return_edge_rejections_*`, `missing_stock_row_*`, `immediate_writer_busy_*`, `overlapping_returns_*` |
| 3.4 eligibility locality and persistence safeguards | `domain::sales::post_sale`, `SqlitePostSaleRepository` | focused return suite, structural readback, parameterized SQL/row-count inspection |

- Test-first/refactor baseline: `cargo test --manifest-path src-tauri/Cargo.toml --test post_sale_lifecycle return` passed 7 tests before documentation-only reconciliation; `cargo fmt --manifest-path src-tauri/Cargo.toml --check` and `git diff --check` passed. Runtime harness: focused migrated SQLite integration scenarios, including separate file-backed connections.
- Authoritative PR3E source/test/export delta from PR3D settled tree/blob `0892a8d3fb980a37eeff3a8736bf2abfc7ea707d`: `+0/-0 = 0` changed lines; only task/progress traceability artifacts changed.
- Rollback: revert the 3.4 checkbox and PR3E traceability/progress documentation only; retain the accepted Phase 3 backend and schema-v10 facts. Next: PR4 cancellation backend.

## PR4A — cancellation happy path and canonical replay

- Completed sub-boundary: `SqlitePostSaleRepository` now persists and reloads cancellation requests through the existing application-owned single `IMMEDIATE` transaction. It writes one cancellation fact per original line, restores only the domain-planned residual, creates one linked `cancellation` movement only for a positive residual, and records a zero residual with `movement_id = NULL`.
- The adapter uses fixed parameterized SQL and an exact guarded stock-update row count; it does not add eligibility or residual arithmetic outside `plan_cancellation`. The cancellation header derives `SaleLifecycleStatus::Cancelled` while original sale, line, and payment snapshots remain unchanged.
- Focused migrated SQLite proof first failed with `PersistenceFailure` at the cancellation stub. GREEN proves a two-line sale after a two-line return: one original line has residual one and one is fully returned with a zero cancellation fact/no movement; it checks stock restoration, linked movement identity, normalized reason, immutable originals, and identical normalized-request replay with no duplicate facts or effects.
- Verification: `cargo test --manifest-path src-tauri/Cargo.toml --test post_sale_lifecycle cancellation` passed 1 test; return regression filter passed 7 tests. `cargo fmt --manifest-path src-tauri/Cargo.toml --check` and `git diff --check` passed. Runtime harness passed: focused migrated seeded SQLite integration scenario.
- Authoritative PR4A source/test delta reconstructed from the retained pre-PR4A stubs: repository `+97/-7`, integration test `+110/-2`, total `+207/-9 = 216` changed lines. With the inherited native consumption of 14, the implementation subtotal is 230 before progress reconciliation, below 400.
- Task accounting: 4.1–4.4 remain unchecked because their original scope also requires cancellation validation/conflict/rollback/concurrency and refactor evidence.
- Rollback: restore the two cancellation repository stubs and remove `cancellation_restores_residuals_and_replays_without_extra_effects`; retain the PR1 schema, PR2 application/domain contracts, and Phase 3 return backend. Next: PR4B failure, conflict, rollback, and return/cancellation serialization evidence.

## PR4B — cancellation invalid, rollback, and serialization evidence

- Completed sub-boundary: test-only evidence extends retained PR4A behavior; no production seam changed. It proves blank reason rejection, absent and pending sale errors, changed cancellation payload conflict, cancel-once rejection, and same request identity rejected across return/cancellation operation kinds. Every rejected case preserves request/header/line/movement effects and stock; original facts remain equal.
- A trigger injected before cancellation-line insertion fails after the header, stock update, and positive movement work has started. The application returns `PersistenceFailure`; exact correction counts/stocks and an explicit derived-status query prove rollback leaves the sale `confirmed` with no cancellation header.
- A migrated two-connection file-backed scenario synchronizes one return and one cancellation with independent factories and one-second busy timeouts. Cancellation always commits; the return either commits first or receives `Domain(SaleCancelled)`. The test verifies exact winner-dependent correction movement count, final stock `(8, 4)`, per-line aggregate restoration equal to sold quantities, and immutable originals.
- Gate correction: a focused fully-returned sale first accepts return quantities `(2, 1)` and then cancels. It proves one header, two zero/NULL-movement cancellation facts, zero cancellation movements, unchanged cancellation-time stock, derived `Cancelled` status, complete immutable originals, and exact replay. The cross-operation retry now compares deterministic full rows before/after `RequestConflict`: `post_sale_requests` (`id`, request/operation/sale identity, payload version/blob/hash, timestamp); return and cancellation headers/lines (all IDs, identities, quantities/restored quantities, reasons/timestamps, movement links); every inventory-movement column (`id`, product/sale/line, type, delta, timestamp, reason, operator/reference/request and counted/resulting fields); and all stock balances, each in stable primary-key order.
- Test-first result: after correcting test-only compile/SQL-literal mistakes, the intended new cases were immediately GREEN against retained PR4A/PR3 behavior, so no production correction was introduced. `cargo test --manifest-path src-tauri/Cargo.toml --test post_sale_lifecycle cancellation` passed 5 tests; the return filter passed 9 tests; formatting and diff checks passed. Runtime harness: migrated in-memory and file-backed SQLite integration scenarios passed.
- Concrete SQLite commit-failure injection remains intentionally absent: the truthful fixed `COMMIT`/`ROLLBACK` wrapper cannot safely fabricate a consumed-transaction fault. Retained PR3A1 fake-lifecycle tests prove application commit-failure→rollback and rollback-failure mapping.
- Authoritative settled-tree PR4B source/test/export delta: baseline `f0bbf2593a633c42e90113729dd5749df4c22db2` → final `3cc55a94c0a7bbbc6ef79e9e038dafd1271c6aac`; `src-tauri/tests/post_sale_lifecycle.rs` `+373/-1 = 374` changed lines and production/export `+0/-0`.
- Task accounting: checked 4.1–4.3 in OpenSpec and Engram because cumulative PR4A/PR4B evidence satisfies their complete original wording. Task 4.4 remains unchecked.
- Rollback: remove the PR4B cancellation helpers/assertions and four focused tests; retain PR4A persistence and all Phase 3 behavior. Next: PR4C refactor/traceability.

## PR4C — cancellation refactor and traceability

- Completed task 4.4. `PostSaleUseCase::finish` already owns the single transaction commit/explicit-rollback path. This slice extracted the duplicated request lookup, payload-match replay, and stable conflict handling into the generic `replay` helper. Return and cancellation retain separate fact loading, pure planning, and persistence calls, so their business plans are not merged.
- Structural readback confirms the domain owns residual/eligibility (`plan_cancellation`), the application owns the one `IMMEDIATE` transaction lifecycle, and `SqlitePostSaleRepository` receives only the supplied transaction for parameterized persistence/reload, checked stock rows, and zero-residual handling. No adapter refactor was needed: cancellation persistence/reload remains cohesive and does not encode domain eligibility.

### Phase 4 traceability

| Task / requirement | Implementation seam | Focused evidence |
| --- | --- | --- |
| 4.1 cancellation happy/partial/fully returned, replay, immutable originals, normalized reason | `plan_cancellation` + `SqlitePostSaleRepository::persist_cancellation` | `cancellation_restores_residuals_and_replays_without_extra_effects`, `fully_returned_sale_cancellation_records_zero_lines_and_replays` |
| 4.2 single `IMMEDIATE`, supplied transaction, facts/reload, zero lines | `PostSaleUseCase`, `SqlitePostSaleTransactionFactory`, `SqlitePostSaleRepository` | cancellation integration suite |
| 4.3 conflicts, rollback, and return/cancellation serialization | request identity/replay + SQLite transaction/repository | `cancellation_rejections_and_conflicts_leave_no_effects`, `injected_cancellation_failure_rolls_back_all_effects`, `overlapping_return_and_cancellation_serialize_without_double_restoration` |
| 4.4 shared plumbing and locality | `finish`, `replay`, domain planner, supplied-transaction repository | structural readback plus cancellation/return focused suites |

- Verification: `cargo test --manifest-path src-tauri/Cargo.toml --test post_sale_lifecycle cancellation` passed 5 tests; the return filter passed 9 tests; `cargo fmt --manifest-path src-tauri/Cargo.toml --check` and `git diff --check` passed. Runtime harness: migrated in-memory and file-backed SQLite scenarios.
- Authoritative PR4C-only source/test/export delta from final PR4B tree: `src-tauri/src/application/sales/post_sale.rs` `+28/-21 = 49` changed lines; no test/export changes.
- Rollback: restore the two operation-local replay matches and remove `replay`; revert the 4.4 checkbox and PR4C traceability documentation. Retain committed cancellation facts and all Phase 4 evidence. Next: PR5 command seams.

## PR5A — Rust/Tauri implementation seam

- Retained transport/wiring: strict DTOs, tagged outputs, stable inventory-only mapping, managed-state use-case calls, and both registrations. Initial RED observed the absent command module; the initial focused GREEN was incomplete.
- PR5A source/test/export delta remains `+373/-2 = 375`; rollback removes the command module, wrappers, registrations, and initial test. The command implementation required no PR5A2 production correction.

## PR5A2 — Rust command-contract evidence

- Test-first added evidence was immediately GREEN against retained behavior: top/nested return DTO strictness, cancellation DTO strictness and owned parsing, tagged success/error shapes, complete return fields, cancellation normalized reason/timestamp/two residual lines including zero, every stable application error code (including `request_conflict` and `persistence_failure`), and forbidden SQL/schema/driver/monetary vocabulary checks.
- Authentic Tauri mock coverage proves both post-sale commands are registered in the test handler; structural readback confirms both appear exactly once in each test and desktop handler and are absent from obsolete exclusions. Runtime: mock command seam, not a GUI shell.
- Authoritative settled-tree PR5A2 delta from retained PR5A: command module `+88/-0`, integration test `+174/-51`, total `+262/-51 = 313` changed lines (net `+211`), under its fresh 400-line budget. Gate correction added test-only reusable vocabulary assertions `+53/-46 = 99`; it changes neither production behavior nor task state. Rollback: remove only PR5A2 contract tests; retain PR5A transport. Tasks 5.1–5.2 are checked in both stores. Next: PR5B TypeScript adapter/parity (5.3–5.4).


## PR5B — TypeScript adapter and seam parity

- Completed tasks 5.3–5.4: `src/commands/post-sale.ts` is a thin Tauri IPC adapter with discriminated return/cancellation response unions, fixed snake_case `{ request }` payloads, runtime result guards, and stable `persistence_failure` normalization for rejected or malformed values. It contains no sale eligibility or residual arithmetic.
- One generated request UUID is captured by `beginReturn` or `beginCancellation` and reused by every `submit` retry; direct request functions remain available for callers that already own the ID. The focused tests prove exact Rust result-field casing, cancellation normalized reason/timestamp and zero-residual lines, `request_conflict`, and inventory-only rejection text.
- Initial RED: `npm test -- src/commands/post-sale.test.ts` failed as expected with `ERR_MODULE_NOT_FOUND` because `src/commands/post-sale.ts` did not exist (46 passed, 1 failed). Initial GREEN passed all 48 discovered TypeScript tests after adapter implementation.
- Gate correction RED: otherwise-valid return/cancellation response fixtures containing unknown top-level and nested `refund_amount`, `sql`, `schema`, and `driver` fields failed because raw spreads/line arrays leaked them (48 passed, 1 failed). The same focused test also covers malformed return `quantity` and cancellation `restored_quantity` types. GREEN: declared-field projection now discards extras and maps malformed required lines to stable `persistence_failure`; `npm test -- src/commands/post-sale.test.ts` passed all 49 tests. `npx tsc --noEmit`, `npx prettier --check src/commands/post-sale.ts src/commands/post-sale.test.ts`, `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, and `git diff --check` passed. Rust parity regression `cargo test --manifest-path src-tauri/Cargo.toml --test post_sale_commands` passed 3 tests.
- Runtime harness: N/A. A native development shell requires an interactive desktop environment unavailable to this noninteractive executor; the focused adapter uses an injected Tauri-invoke seam and the Rust command suite covers the managed command handler.
- Initial PR5B file addition was `+354/-0` (adapter `+229/-0`, test `+125/-0`). The independent gate-correction delta is adapter `+53/-24`, test `+111/-0`, total `+164/-24 = 188` changed lines, below its fresh 400-line boundary. Final PR5B retained file footprint is adapter `+258/-0`, test `+236/-0`, total `+494/-0`; this total spans the prior settled PR5B slice plus the separately authorized correction attempt.
- Rollback: remove only `src/commands/post-sale.ts` and `src/commands/post-sale.test.ts`; retain the PR5A/A2 Rust commands, registrations, backend facts, and all prior phases. Phase 5 is complete; next: PR6 Sales History lifecycle read model.


## PR6A preflight — combined history read model blocked

- The original combined 6.1–6.2 slice was estimated at a minimum 413 source/test lines and was correctly split before any candidate was retained. PR6A1 and PR6A2 preserve its list-versus-detail verification/rollback boundary; original tasks remain unchecked until cumulative evidence satisfies their full wording.

## PR6A1 — lifecycle/list and line aggregate projection

- Completed sub-boundary: public summaries now expose `has_corrections`; history status is derived from a cancellation header while the base sales filter remains `s.status = 'confirmed'`. `HistoricalLine` now exposes exact `sale_line_id`, accepted returned quantity, cancellation-restored quantity, and remaining returnable quantity from additive facts only.
- RED: `cargo test --manifest-path src-tauri/Cargo.toml --test sales_history` failed as expected because `has_corrections` and the four public line-aggregate fields did not exist. GREEN: the same command passed 3 tests after the application/read-model projection was added. It runs a real migrated SQLite scenario that confirms a sale, records a return, cancels residual inventory, then proves the persisted base status remains `confirmed`, the list/detail derived status is `cancelled`, corrections are visible, and the exact original-line aggregate is `(returned=1, cancelled=1, remaining=0)`.
- Reader safety: all dynamic filter values remain bound through `?1`–`?3` and `params!`; the new lifecycle/aggregate queries are fixed SQL. The reader validates non-negative correction quantities and checked remaining subtraction before exposing them.
- Verification: `cargo fmt --manifest-path src-tauri/Cargo.toml --check` and `git diff --check` passed. Runtime harness passed through the focused migrated SQLite test; no command, TypeScript, UI, or original-sale mutation was added.
- Authoritative PR6A1 source/test delta: application `+5/-0`, SQLite reader `+43/-7`, test `+106/-0`, total `+154/-7 = 161` changed lines. The successful implementation settlement also changed apply-progress `+15/-7 = 22`, so its native attempt total is 183. With the carried failed-preflight consumption of 14, authoritative cumulative native consumption is 197/400.
- Task accounting: 6.1 and 6.2 remain unchecked because ordered return/cancellation collections and the complete immutable-original snapshot matrix belong to PR6A2. Rollback: remove the five projected summary/line fields, corresponding fixed reader projections, and the PR6A1 test/helper; retain all accepted post-sale facts. Next: PR6A2 correction-detail collections and immutable-original evidence.


## PR6A2 — ordered correction details and immutable originals

- Completed sub-boundary: `SaleHistoryDetail` now includes ordered return headers with request identity, timestamps, header IDs, and exact line facts, plus optional cancellation header/detail with normalized reason and every original line, including zero-restored lines. All data is reconstructed from additive correction facts; PR6A1 lifecycle/list and aggregates are unchanged.
- RED: `cargo test --manifest-path src-tauri/Cargo.toml --test sales_history` failed as expected because the public `returns` and `cancellation` detail fields did not exist. GREEN: the same command passed 4 tests. The real migrated SQLite scenario confirms a two-unit sale, creates two ordered returns, then records a zero-restored cancellation. It asserts ordered request/header identities and timestamps, exact line identities/quantities, cancellation reason and zero line, and full equality of original sales, sale_lines, and sale_payments snapshots before and after both corrections.
- Reader safety: all dynamic IDs use bound `?1` parameters; correction query ordering is fixed (`occurred_at, id` for headers and `sale_line_id` for lines). The reader has no eligibility or persistence logic. `cargo fmt --manifest-path src-tauri/Cargo.toml --check` and `git diff --check` passed. Runtime harness passed through the focused migrated SQLite suite.
- Authoritative PR6A2 settled-tree delta `9e938a7..db19eca`: DTOs `+30/-0`, exports `+2/-1`, SQLite reader `+80/-1`, test `+103/-5`, source/test/export total `+215/-7 = 222`. Apply-progress changed `+15/-3` and tasks changed `+2/-2`, so artifacts are `+17/-5 = 22`; native total is `+232/-12 = 244`, matching settled `changed_lines` and remaining below the 400-line boundary.
- Task accounting: checked 6.1 and 6.2 in OpenSpec and Engram because PR6A1+PR6A2 cumulatively satisfies their complete listed lifecycle, discoverability, aggregate/detail, ordering, immutable-original, and parameterized-filter wording. Tasks 6.3–6.4 remain pending. Rollback: remove only PR6A2 detail DTOs/exports, detail queries, and the PR6A2 test extension while retaining PR6A1 list aggregates and all persisted correction facts. Next: PR6 command/TypeScript contracts.

## PR6B1 — Rust/Tauri history command transport

- Tagged Rust responses serialize PR6A lifecycle/detail DTOs without production changes; command 3/history 4 and checks pass. Final test `+190/-0`, correction `+20/-0`.

## PR6B2 — TypeScript history contracts blocked by review budget

- Full adapter/matrix candidate measured `+611/-20 = 631` and was reverted; 6.3 remains open.

## PR6B2A — DTO/success projection blocked by review budget

- Prior narrowed candidate was reverted at `+518/-34 = 552`; split into A1 declarations and A2 projection.

## PR6B2A1 — public TypeScript history DTO declarations

- Exported Rust-JSON lifecycle/correction/payment/return/cancellation DTOs; strict fixture RED then TypeScript GREEN. Delta `+108/-4 = 112`; runtime unchanged. 6.3/6.4 remain open.

## PR6B2A2 — allowlisted TypeScript success projection

- Allowlisted current decoder requires explicit no-correction fields and drops extras. Initial A2: `+60/-54 = 114`.
- Gate correction maps decoder throws through `.then(decoder).catch(failure)` to `persistence_failure`: source/test `+17/-1 = 18` (adapter `+1/-1`, test `+16/-0`), artifacts `+6/-5 = 11`, native `29`. Runtime injected invoke; GUI N/A; rollback retains A1 DTOs.

## PR6B2B — malformed/order/hostile TypeScript history matrix

- Completed 6.3: public seam covers malformed data, rejection/extras, order, and current uncorrected shapes; failures map to `persistence_failure`.
- Accounting: prior test `+62/-55`, progress `+2/-2`, native `121`, retained `+7/-0`; unit-price `+1/-0`; current `+325/-50` vs pre-B `+156/-50`, retained `+169/-0`. Checks pass.

## PR6C — historical immutability and legacy compatibility

- Completed 6.4 evidence-only: SQLite snapshots prove original sales/lines/payments unchanged after return+cancellation; command seam proves legacy confirmed rows have false corrections, zero aggregates, empty returns, and no cancellation. Fixed-SQL reader needs no refactor.
- Rust 4+3, TS 52, fmt/diff passed. Runtime: corrected/uncorrected migrated command details; GUI N/A. PR6C: test `+14/-0`, task `+1/-1`, progress `+7/-3` = `+22/-4` (26); rollback removes its assertions/traceability. Phase 6 complete; next PR7 UI.

## PR7A1 — return-intent state

- Retained state seam: confirmed details with positive persisted remaining quantity may open a caller-ID-bearing return intent; local whole-number/exact-line validation, retry identity retention, pending duplicate prevention, stale-success ignore, reload request, and no optimistic history mutation are covered. Task 7.1 remains pending because cancellation and presentation/accessibility evidence were absent.

## PR7A2 — cancellation-intent state

- Completed task 7.2 cumulatively with PR7A1: a confirmed detail may open one caller-ID-bearing cancellation intent even when every line is fully returned; cancelled details cannot open or submit. The reducer trims reason ends, requires a non-blank normalized reason and explicit confirmation, prevents pending duplicates, retains request ID/reason/confirmation on application failure, ignores stale completion outside pending, and emits only `reload_requested` on success without changing detail status or quantities.
- RED: `npm test -- src/ui/sales/history-flow.test.ts` failed with 54 passed / 1 failed because cancellation actions had no reducer state. GREEN: the same command passed 55 tests. `npx tsc --noEmit` and `git diff --check` passed. Source Prettier passed; the test-file Prettier delta is unchanged from the pre-existing baseline in the PR7A2-added region.
- Runtime harness: N/A — injected pure reducer state seam; no presentation/Tauri runtime path is part of this slice. Rollback: remove the cancellation intent types/actions/handlers and its focused reducer test, restore task 7.2 to pending, and retain PR7A1 return behavior. Tasks 7.1, 7.3, and 7.4 remain pending.

### PR7A2 automatic gate correction

- Test-only direct evidence confirms the opening UUID survives explicit retry and `reload_requested`; it rejects submit when an existing intent sees a persisted cancelled detail, and preserves the same persisted detail object in pending and error states.
- Verification: `npx tsx --test src/ui/sales/history-flow.test.ts` passed 9 tests; `npm test -- src/ui/sales/history-flow.test.ts` passed 55 tests; `npx tsc --noEmit` and `git diff --check` passed. The complete test-file Prettier check still fails at the inherited baseline; the new test regions match Prettier output exactly.
- Correction delta: production `+0/-0`; direct test evidence `+13/-0`; OpenSpec evidence `+6/-0`; total retained correction `+19/-0`. Rollback removes only these new assertions/setup and this correction record; no task checkbox changed.


## PR7B — lifecycle presentation adapter

- Completed task 7.3: Sales History detail now renders visible lifecycle status, exact `Original sale items`, `Original payment facts`, and `Inventory correction history` sections. Original snapshots and payment facts remain separate from ordered persisted return records and cancellation audit lines, including zero-restored quantities.
- Return presentation is keyed by exact `sale_line_id`; its selection and whole-number inputs display and bound only the persisted `remaining_returnable_quantity`, keeping repeated-product sale lines distinct. Cancellation presentation requires a reason and explicit inventory-correction confirmation. State-owned selectors hide unavailable actions: cancelled sales expose no correction actions, while fully returned confirmed sales still expose cancellation.
- RED: `npm test -- src/ui/sales/history-flow.test.ts` observed 55 passed / 1 failed because lifecycle/correction presentation was absent. GREEN: the same command passed 57 tests after the adapter change. `npx tsc --noEmit` passed.
- Runtime harness: N/A — this bounded unit renders through the pure `HistoryScreen` static-markup seam; Tauri desktop GUI execution, focus restoration, live alerts, and command invocation remain task 7.4.
- Rollback: remove the lifecycle/correction detail sections and correction form presentation from `src/ui/sales/history-screen.ts`, plus the focused static-markup test; retain persisted history facts, adapters, reducer state, and commands.
- Task accounting: checked 7.3 only. Tasks 7.1 and 7.4 remain pending.

### PR7B targeted formatter correction

- Immutable initial candidate accounting: screen `+53/-8`, focused test `+117/-0`, tasks `+1/-1`, progress `+10/-0`; total `+181/-9 = 190` changed lines.
- Applied only the Prettier projection for the PR7B lifecycle presentation region in `history-screen.ts`; legacy drift outside that region is byte-identical and behavior is unchanged.
- Final cumulative PR7B begin-to-final accounting: screen `+314/-11`, focused test `+117/-0`, tasks `+1/-1`, progress `+18/-0`; total `+450/-12 = 462` changed lines.
- Correction-attempt delta from the immutable initial candidate: screen `+310/-52`, focused test `+0/-0`, tasks `+0/-0`, progress `+8/-0`; total `+318/-52 = 370` changed lines, within the hard 400-line correction boundary.
- Verification: `npm test -- src/ui/sales/history-flow.test.ts` passed 57 tests; `npx tsc --noEmit`, the targeted Prettier projection comparison, and `git diff --check` passed.

- **PR7C — implementation prepared; tasks 7.1 and 7.4 remain pending:** typed return/cancellation command submission preserves reducer UUIDs and exact sale-line IDs, de-duplicates pending requests, retains conflict/malformed-result values with accessible alerts/reload, then reloads authoritative detail without optimism. Automated UI evidence is prepared, but task 7.4 requires display-capable manual return, retry, cancellation, keyboard, focus, and corrected-history scenarios. `npm run tauri:dev` compiled Vite/Rust but GUI is unavailable after Wayland Gdk Error 71. Native initial accounting was screen `+193/-18`, tests `+188/-0`, task `+1/-1`, progress `+2/-0`, total `+384/-19 = 403` (not 400). Bounded compaction final accounting is screen `+189/-18`, tests `+186/-0`, task `+1/-1`, progress `+2/-0`, total `+378/-19 = 397`; correction attempt delta is screen `+1/-5`, tests `+8/-10`, progress `+1/-1`, total `+10/-16 = 26` changed lines. Rollback only submission helpers/callbacks/tests, retain all prior state, presentation, adapters, and facts.

## PR7C accessibility/runtime-evidence correction

- Prepared production accessibility behavior without claiming unavailable GUI evidence: reducer-owned structured validation deterministically supplies the first invalid exact sale-line focus target (or the first selectable exact line when none is selected); invalid cancellation supplies the required reason or confirmation target. `HistoryScreen` consumes only that target through its focus effect, without duplicating return eligibility, format, integer, or remaining-quantity rules.
- Correction form buttons and inputs now use a scoped 44px minimum inline target style. No global stylesheet or unrelated UI was changed. Static markup tests cover both the focus-selection/management seam and the rendered 44px contract; typed command orchestration, UUID retry reuse, duplicate suppression, alerts, and authoritative reload behavior remain unchanged.
- Evidence: RED was 13 passed / 2 failed (missing focus helpers and target styles); GREEN is `npx tsx --test src/ui/sales/history-flow.test.ts` 15/15, `npx tsc --noEmit`, `npm test` 61/61 (the prior 59 plus two focused tests), targeted Prettier projection for touched/new regions, and `git diff --check`, all passing. The static-render effect-runner test proves `HistoryScreen` schedules and invokes the focus effect; whole-file Prettier remains intentionally unchanged because its drift predates this correction.
- Authoritative native begin-to-final correction accounting: `history-screen.ts` `+85/-4`, `history-flow.test.ts` `+123/-1`, `tasks.md` `+2/-1`, and this progress record `+10/-0`; total `+220/-6 = 226` changed lines, below the 400-line hard bound.
- Runtime evidence remains unavailable: `npm run tauri:dev` previously compiled Vite/Rust but the GUI stopped at Wayland Gdk Error 71. Do not substitute static/injected evidence for manual display-capable return, retry, cancellation, keyboard, focus, and corrected-history scenarios, and do not retry that GUI command in this environment.
- Task accounting: 7.1 remains pending. Task 7.4 is explicitly unchecked/pending until the required manual display-capable scenarios are recorded. Rollback removes only the scoped focus helpers/effect, target styles, focused tests, and this correction record; it retains the existing commands, reducer orchestration, persisted facts, and history presentation.


## PR7C gate 2 correction — reducer-owned focus seam

- Attempt 71 begins from immutable native tree `d5071edb575ae083a0f25ffd0094f2f95677ab76` (begin identity `sha256:8080cc488ef76d57f1c9a3149749613f95f47441453af28925b930e9a5c3782e`). The final native tree does not exist until settlement; this record intentionally reports only the exact begin-to-current snapshot and must be reconciled with native `changed_lines` and evidence after settlement.
- The reducer persists structured local validation (`message`, `focus_target`) and `HistoryScreen` consumes that target through its effect without reimplementing exact-line eligibility, positive-integer syntax, safe-integer, or remaining-quantity rules. A static-render effect-runner test proves the component schedules and executes that focus callback; removing the effect wiring makes the test fail.
- No task checkbox changed: tasks 7.1 and 7.4 remain pending. Display-capable manual return, retry, cancellation, keyboard, focus, and corrected-history scenarios are still required. Wayland Gdk Error 71 remains the truthful GUI limitation and the GUI command was not retried.
- Exact begin-to-current snapshot: `history-flow.ts` `+119/-22`, `history-screen.ts` `+29/-61`, `history-flow.test.ts` `+31/-24`, `tasks.md` `+0/-0`, and this progress record `+14/-4`; total `+193/-111 = 304` changed lines. These are computed from the immutable begin tree, not a guessed final tree.
- RED: `npx tsx --test src/ui/sales/history-flow.test.ts` failed because `correctionFocusTarget` was not exported. GREEN: the same focused command passed 15/15 after the reducer-owned validation result and HistoryScreen effect wiring were added. Rollback: revert only the reducer validation metadata, `HistoryScreen` focus-effect wiring/test seam, focused test assertions, and this correction entry; retain prior command, state, presentation, target-size, and history behavior.
- Verification: `npm test` passed 61/61, `npx tsc --noEmit` passed, `npx prettier --check src/ui/sales/history-flow.ts` passed, and `git diff --check` passed. Whole-file Prettier for `history-screen.ts` and `history-flow.test.ts` remains an inherited baseline failure; the immutable begin tree was restored byte-for-byte before this correction, and the touched correction hunks match Prettier's projection without rewriting unrelated regions.
- Mutation sensitivity: removing only the `HistoryScreen` effect invocation made the focused suite fail at `scheduledEffects` (`0 !== 1`); restoring the exact source object returned the focused suite to 15/15.

## PR7D — task 7.1 UI-flow evidence completion

- Completed task 7.1 from cumulative PR7A–PR7D evidence. Retained reducer, command-orchestration, presentation, alert, focus, target-size, and authoritative-reload tests already cover intent UUID retention, exact-line selection, whole-number/local validation, pending disablement, successful reload, stale/conflict value retention, reload/retry, cancellation reason/confirmation, and cancelled/fully-returned action rules without optimistic history mutation.
- New static `HistoryScreen` evidence verifies both correction paths use labelled native forms, checkbox/number/text inputs, and submit controls, preserving keyboard-operable browser semantics; it also rejects refund, reimbursement, payment-reversal, credit, and settlement wording. This is a production-adapter contract, not a substitute for the display-capable manual scenarios owned by task 7.4.
- Standard mode: no new RED was observed because this additional assertion suite passed against the retained implementation immediately. `npx tsx --test src/ui/sales/history-flow.test.ts` passed 16/16. The strongest non-GUI runtime seam remains the same production interaction test: typed return/cancellation adapters submit exactly once, retain values on failures, and reload authoritative persisted detail with no optimistic mutation.
- Verification: focused UI tests 16/16, `npx tsc --noEmit`, project `npm test` 62/62, targeted Prettier projection for this new test hunk, and `git diff --check` passed. Whole-file Prettier remains an inherited baseline drift outside this hunk.
- Task accounting: task 7.1 is checked in both hybrid stores. Task 7.4 remains unchecked: its display-capable manual return, retry, cancellation, keyboard, focus, and corrected-history evidence is still unavailable after the prior Wayland Gdk Error 71 and was not retried.
- Exact attempt-local begin-to-final accounting: `history-flow.test.ts` `+46/-0`, `tasks.md` `+3/-3`, and this progress record `+10/-0`; total `+59/-3 = 62` changed lines, below the 400-line hard boundary.
- Rollback: revert this static semantic/language test, the 7.1 checkbox/evidence note, and this progress entry. Retain the existing reducer, command orchestration, presentation, focus/target accessibility, persisted correction facts, and the separate 7.4 runtime blocker.

## PR7E — display-capable manual runtime evidence

- Launcher progression observed: direct Wayland launch failed with `Gdk Error 71`; `GDK_BACKEND=x11` opened a blank window with GBM buffer errors; `GDK_BACKEND=x11 WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run tauri:dev` opened the real application successfully.
- In the real Tauri window, the user manually completed a return and a cancellation, then confirmed that stock was restored and Sales History showed the sale as cancelled.
- On follow-up, the user confirmed keyboard-only operation, focus returning to the first invalid field, immutable original item/payment facts, and a visible zero-restored correction line.
- The user then explicitly confirmed the required retry: during the return, an invalid value was submitted, corrected in the same still-open form, and resubmitted successfully without closing or manually reloading the screen.
- Retained automated evidence covers typed commands, stable retry identity, duplicate suppression, authoritative reload, accessible alerts, reducer-owned focus, 44px targets, and 62 tests through PR7D.
- Task 7.4 is checked in both hybrid stores: the retry blocker is resolved and Phase 7 is complete. The automated state matrix covers repeated-product lines, fully returned confirmed sales, cancelled sales, command conflicts, and malformed command results; the human-observed runtime record supplies return, retry, cancellation, keyboard/focus, and corrected-history scenarios.
- Limitation: these results are human-observed; no screenshot artifact exists beyond the conversation image. No source, test, build, or runtime command was run in this evidence-only update.
- Exact PR7E evidence-only begin-to-final accounting: `tasks.md` `+3/-3` and this progress record `+5/-3`, total `+8/-6 = 14` changed lines, below the 400-line hard bound. Task accounting is 29/36 complete; Phase 8 remains pending.
- Rollback: restore the prior unchecked task 7.4 note and remove only the PR7E retry-confirmation/completion accounting; retain the existing commands, reducer orchestration, presentation, and persisted facts.
