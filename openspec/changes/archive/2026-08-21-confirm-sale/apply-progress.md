# Apply Progress: Confirm Sale

## Work Unit 1 — Bootstrap and Searchable Seeded Catalog

- Completed: 1.1–1.6.
- Added a minimal React/TypeScript and Tauri/Rust manifest/configuration scaffold, a disposable SQLite bootstrap, migration, seeded active/inactive catalog, indexed active search, and focused frontend/Rust test entry points. No sale write path was introduced.
- After Rust provisioning, the first Rust run exposed two scaffold defects: Tauri's desktop dependency attempted to compile host GUI dependencies, and the catalog query returned a statement-borrowed temporary. The manifest now keeps the unused desktop dependency behind an opt-in `desktop` feature for this headless catalog test slice, and the query collects into a local result before the statement is dropped.
- Added an executable foreign-key assertion to the disposable catalog test and configured TypeScript's bundler module resolution so the installed React type dependencies resolve.

## Work Unit 2 — Sale Domain Rules

- Completed: 2.1–2.5.
- Added framework-independent Rust domain value types for non-negative integer centavos, positive whole-unit quantities, and parsed UUID request IDs. `SaleLine` snapshots the minimum price, validates its price floor, and uses checked `i64` multiplication for totals.
- Added a tagged cash/QR payment model and a `Sale` constructor that rejects empty sales and requires applied payments to equal checked multi-line totals exactly. Cash construction rejects inconsistent tender/change values. Negative monetary inputs are rejected at `MoneyCentavos`; fractional values are unrepresentable by the `i64` quantity interface.
- The public domain interface exposes only typed values and the sale data required by the later use case. It has no React, Tauri, SQLite, or database dependency.

## TDD Cycle Evidence

| Phase | Evidence | Outcome |
| --- | --- | --- |
| RED | `src-tauri/tests/catalog_search.rs` was written before catalog implementation. The original `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_search` was blocked with `cargo: command not found` (exit 127). | Historical RED/bootstrap evidence retained. |
| GREEN | After provisioning, the same test command first failed to compile because the catalog query returned a temporary borrowing its statement. The minimal local-result correction compiled and passed. | PASS: 3 catalog tests. |
| TRIANGULATE | The disposable-database matrix searches name, SKU, category, and configured searchable field; asserts inactive exclusion, stock, minimum-price centavos, and foreign-key enablement. | PASS: 3 catalog tests. |
| REFACTOR | Bootstrap remains consolidated under `infrastructure/sqlite`; the catalog interface was not widened. The headless test profile keeps Tauri desktop dependencies opt-in until a command/desktop entry point exists. | PASS: `cargo check` without desktop feature. |
| RED (2.1) | Added `src-tauri/tests/sale_domain.rs` against the planned public domain seam before its production module existed. | Expected FAIL: `cargo test --manifest-path src-tauri/Cargo.toml --test sale_domain` exited 101 because `repuestos_autos::domain` was unresolved. |
| GREEN (2.2) | Added typed money, quantity, request-ID, sale-line, and payment implementations. | PASS: 2 domain tests. |
| TRIANGULATE (2.3) | Added price-floor, payment equality, inconsistent cash, QR-only, cash-only, mixed payment, empty-sale, and multi-line-total cases. | The first matrix run failed because its mixed-payment fixture applied 5,000 centavos to a 6,000-centavo sale; correcting the fixture to 6,000 made the intended rule pass. |
| GREEN (2.4) | Added `Sale::new`, which owns aggregate total and payment-equality validation without infrastructure dependencies. | PASS: 3 domain tests. |
| REFACTOR (2.5) | Kept primitive values encapsulated, exposed typed use-case accessors, and retained total/payment calculations inside domain types. | PASS: focused domain suite, existing catalog suite, `cargo check`, and format check. |

## Verification

- `npm test` — PASS: 1 Node frontend-harness test.
- `npx tsc --noEmit` — PASS.
- `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_search` — PASS: 3 passed, 0 failed.
- `cargo check --manifest-path src-tauri/Cargo.toml` — PASS.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check` — PASS after installing the stable `rustfmt` component.
- `cargo check --manifest-path src-tauri/Cargo.toml --features desktop` — UNAVAILABLE on this Linux host: Tauri desktop dependencies require missing GTK/GLib development packages (`gobject-2.0`, `glib-2.0`). Windows packaging was not run and remains unavailable for this Work Unit.
- `cargo fmt --manifest-path src-tauri/Cargo.toml && cargo fmt --manifest-path src-tauri/Cargo.toml --check` — PASS.
- `cargo test --manifest-path src-tauri/Cargo.toml --test sale_domain` — PASS: 3 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_search` — PASS: 3 passed, 0 failed.
- `cargo check --manifest-path src-tauri/Cargo.toml` — PASS.

## Files Changed

Work Unit 1: `.gitignore`, `package.json`, `package-lock.json`, `tsconfig.json`, `vite.config.ts`, `src/main.ts`, `src/catalog.test.js`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `src-tauri/src/lib.rs`, `src-tauri/src/application/**`, `src-tauri/src/infrastructure/**`, `src-tauri/tests/catalog_search.rs`, `openspec/changes/confirm-sale/tasks.md`, and this file.

Work Unit 2: `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/src/domain/**`, `src-tauri/tests/sale_domain.rs`, `openspec/changes/confirm-sale/tasks.md`, and this file.

## Delivery Boundary

Feature-branch chain Work Unit 2 only, based on committed Work Unit 1 (`cda366f`). This slice adds 321 source/test/config lines before OpenSpec progress evidence and remains within the 400-line budget. No commit, push, PR, runtime attempt settlement, or Work Unit 3 changes were made.

## Work Unit 3 — Atomic and Idempotent SQLite Confirmation (partial)

- Completed and verified: 3.1–3.2.
- Added SQLite sale, line, payment, and immutable inventory-movement tables with request-ID uniqueness, foreign keys, row checks, indexes, and triggers rejecting movement updates or deletes.
- Added the initial transaction-owned confirmation implementation required to turn the 3.2 RED test green: it reserves a request ID, reloads current product data, validates the domain sale, persists records, conditionally decrements stock, appends negative movements, marks the sale confirmed, and reloads a persisted summary.
- Stopped before 3.3 because completing the required transaction-scoped repository interfaces and SQLite adapters, plus their proof, would exceed this review slice's 400-line budget. Tasks 3.3–3.8 remain unchecked, including their required rollback, idempotency, and integrity matrices.

## TDD Cycle Evidence (Work Unit 3)

| Phase | Evidence | Outcome |
| --- | --- | --- |
| RED (3.2) | Added `confirm_sale_use_case.rs` before the application sales module existed. | Expected FAIL: unresolved `repuestos_autos::application::sales` import; `cargo test --manifest-path src-tauri/Cargo.toml --test confirm_sale_use_case` exited 101. |
| GREEN | Added the minimal transaction-owned SQLite confirmation flow and schema support. | PASS: 1 multi-line cash confirmation test, including persisted minimum-price snapshot, conditional decrement, movement count, and reloaded summary. |
| TRIANGULATE / REFACTOR | Deferred intentionally: these are tasks 3.3–3.8 and require the repository-adapter and failure/idempotency/integrity matrices. | Not run; budget guard. |

## Verification (Work Unit 3)

- `cargo test --manifest-path src-tauri/Cargo.toml --test confirm_sale_use_case` — PASS: 1 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml --test sale_domain` — PASS: 3 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_search` — PASS: 3 passed, 0 failed.
- `cargo check --manifest-path src-tauri/Cargo.toml` — PASS.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check` — PASS.

## Files Changed (Work Unit 3)

`src-tauri/src/application/mod.rs`, `src-tauri/src/application/sales/mod.rs`, `src-tauri/src/application/sales/confirm_sale.rs`, `src-tauri/src/infrastructure/sqlite/migrations/0001_confirm_sale.sql`, `src-tauri/tests/confirm_sale_use_case.rs`, `openspec/changes/confirm-sale/tasks.md`, and this file.

## Delivery Boundary (Work Unit 3)

Feature-branch chain Work Unit 3 only, based on committed Work Unit 2 (`d94332e`). The source/test/schema portion is 239 additions and 0 deletions before OpenSpec evidence; no commit, push, PR, runtime attempt settlement, or Work Unit 4 work was performed. Rollback boundary: remove the new sales application module, confirmation test, and sale schema additions, then recreate only the disposable development database.

## Work Unit 3 — Feature-Chain Continuation

- Completed and verified: 3.3, 3.4, and 3.6.
- Added the `SaleRepository` application interface and `SqliteSaleRepository` adapter. Every repository operation receives the use-case-owned `rusqlite::Transaction`; no adapter opens or commits a transaction.
- `confirm_sale` still exclusively opens and commits the transaction. It delegates request-ID reservation, current product/minimum-price lookup, and persisted-summary reconstruction to the SQLite adapter.
- Added a changed-payload repeated-request test. The retry returns the original persisted summary and leaves sale, line, movement, and stock effects unchanged.
- Added a rollback test that exhausts a later line after prior writes. It proves that no sale, lines, payments, movements, or earlier stock decrement remains. Added incomplete-reservation evidence: a same-ID pending sale returns `persistence integrity failure` rather than a partial success.
- Stopped before tasks 3.5, 3.7, and 3.8 to keep this feature-chain slice below 400 changed lines. The QR/mixed/rejection matrix, full SQLite constraint/immutability matrix, and final rollback/error-mapping refactor remain for the next Work Unit 3 slice.

## TDD Cycle Evidence (Work Unit 3 Continuation)

| Phase | Evidence | Outcome |
| --- | --- | --- |
| RED | Added a public use-case test for a repeated ID pointing to an incomplete pending aggregate. | Expected FAIL: `Query returned no rows`; focused test command exited 101. |
| GREEN | Introduced transaction-scoped repository seam and SQLite adapter, mapping incomplete persisted aggregates to `persistence integrity failure`. | PASS: 4 confirmation use-case tests. |
| TRIANGULATE | Added changed-payload idempotency and later-line stock rollback cases alongside the existing successful multi-line cash case. | PASS: original summary/effects remain unchanged; rollback leaves zero persisted sale effects. |
| REFACTOR | Moved current-product lookup and persisted-summary mapping from the use case into `SqliteSaleRepository`; transaction ownership remains in `confirm_sale`. | PASS: focused Rust suites, `cargo check`, and format check. |

## Verification (Work Unit 3 Continuation)

- `cargo test --manifest-path src-tauri/Cargo.toml --test confirm_sale_use_case` — PASS: 4 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml --test sale_domain` — PASS: 3 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_search` — PASS: 3 passed, 0 failed.
- `cargo check --manifest-path src-tauri/Cargo.toml` — PASS.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check` — PASS.

## Files Changed (Work Unit 3 Continuation)

`src-tauri/src/application/sales/confirm_sale.rs`, `src-tauri/src/application/sales/mod.rs`, `src-tauri/src/application/sales/repository.rs`, `src-tauri/src/infrastructure/sqlite/mod.rs`, `src-tauri/src/infrastructure/sqlite/sale_repository.rs`, `src-tauri/tests/confirm_sale_use_case.rs`, `openspec/changes/confirm-sale/tasks.md`, and this file.

## Delivery Boundary (Work Unit 3 Continuation)

Feature-branch chain Work Unit 3 continuation, based on committed foundation `264650d`. Current slice is 334 source/test changes before OpenSpec evidence; the remaining required matrices would cross the 400-line boundary. No commit, push, PR, runtime-attempt acquire/settlement/reset, or Work Unit 4 work was performed. Rollback boundary: remove the new application repository interface, SQLite adapter, and continuation tests; restore the earlier use-case-local persistence helpers and recreate only the disposable development database.

## Work Unit 3 — Final Feature-Chain Slice

- Completed and verified: 3.5, 3.7, and 3.8. Work Unit 3 is now complete; Work Unit 4 was not started.
- Expanded the disposable SQLite use-case seam matrix for QR-only and mixed payments, plus inactive, missing-product, stale-below-minimum, and unequal-payment rejection. Each rejected request leaves sales, lines, payments, movements, and stock unchanged. Existing seam coverage proves insufficient stock and later-line rollback; existing domain coverage proves inconsistent-cash rejection before a request can be constructed.
- Added SQLite integrity evidence for negative-total row checks, request-ID uniqueness, foreign-key enforcement, and immutable movement update/delete triggers. The existing incomplete-reservation test proves incomplete aggregate reconstruction returns `persistence integrity failure` rather than a partial summary.
- Refactored `confirm_sale` through a single transaction helper that explicitly commits successful work and rolls back every operation error. Domain validation remains in `Sale`, and persisted-summary mapping remains in `SqliteSaleRepository`.

## TDD Cycle Evidence (Work Unit 3 Final Slice)

| Phase | Evidence | Outcome |
| --- | --- | --- |
| RED | Added the QR/mixed, rejection, and SQLite-integrity matrix at the public `confirm_sale` seam before refactoring production code. The already-implemented prior slice satisfied the new cases on the first run, so no failing production gap was observed. | PASS: 7 confirmation tests; no production behavior was added before the tests. |
| GREEN | No new behavior was needed: the existing use case, domain constructors, schema constraints, and movement triggers satisfied the expanded matrix. | PASS: 7 confirmation tests. |
| TRIANGULATE | Exercised QR-only/mixed success; inactive, missing, stale-price, and unequal-payment rollback; existing one-line/later-line insufficient-stock rollback; incomplete aggregate; foreign keys, unique IDs, row checks, and immutable movements. | PASS. |
| REFACTOR | Centralized explicit transaction commit/rollback and database-error conversion in `in_transaction`; retained business rules in the domain and persisted-summary mapping in the SQLite repository. | PASS: focused Rust suites, check, and format check. |

## Verification (Work Unit 3 Final Slice)

- `cargo test --manifest-path src-tauri/Cargo.toml --test confirm_sale_use_case` — PASS: 7 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml --test sale_domain` — PASS: 3 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_search` — PASS: 3 passed, 0 failed.
- `cargo check --manifest-path src-tauri/Cargo.toml` — PASS.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check` — PASS.
- `npm test` — PASS: 1 passed, 0 failed.
- `npx tsc --noEmit` — PASS.
- `git diff --check` — PASS.

## Files Changed (Work Unit 3 Final Slice)

`src-tauri/src/application/sales/confirm_sale.rs`, `src-tauri/tests/confirm_sale_use_case.rs`, `openspec/changes/confirm-sale/tasks.md`, and this file.

## Delivery Boundary (Work Unit 3 Final Slice)

Feature-branch chain final Work Unit 3 slice, based on committed prior slices `264650d` and `99a4f73`. This uncommitted source/test refactor adds 219 and deletes 37 lines before OpenSpec evidence, remaining below the native 400-line limit. No commit, push, PR, runtime-attempt acquire/settlement/reset, or Work Unit 4 work was performed. Rollback boundary: restore `confirm_sale`'s prior transaction handling and remove only this final rejection/integrity test matrix; recreate only the disposable development database.

## Remaining

Work Units 1–3 are complete. Work Units 4–5 remain out of scope. Desktop GTK validation remains unavailable on this Linux host because required GTK/GLib development packages are missing; Windows packaging was not run. Rust formatting is installed and passes its final check.

## Work Unit 4 — Typed Tauri Command Seam

- Completed: 4.1–4.4. Added typed search/confirmation DTOs, stable discriminated command errors, persisted summary mapping (including timestamp/outcome), and desktop-feature command registration.
- Integer `i64` centavos and quantities cross the command seam unchanged. The adapter parses shapes and maps errors only; confirmation still delegates once to application/domain code.
- Files: `src-tauri/src/commands/**`, `src-tauri/src/lib.rs`, domain serialization support, persisted-summary timestamp mapping, `src-tauri/tests/command_seam.rs`, Cargo manifests, and `tasks.md`.

## TDD Cycle Evidence (Work Unit 4)

| Phase | Evidence | Outcome |
| --- | --- | --- |
| RED | `command_seam.rs` preceded the command modules. | Expected fail: unresolved `commands` module and missing `serde_json` (exit 101). |
| GREEN | Added typed DTO parsing, application delegation, summary/error mapping, and registration. | PASS: command seam, 4 tests. |
| TRIANGULATE | Covered search, persisted retry, malformed UUID, invalid quantity, below-floor price, and JSON fractional/out-of-range rejection. | PASS. |
| REFACTOR | Shared only command mapping helpers; rules remain application/domain-owned. | PASS: full Rust suite and format check. |

## Verification (Work Unit 4)

- `cargo test --manifest-path src-tauri/Cargo.toml --test command_seam` — PASS: 4 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` — PASS: 17 passed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check` — PASS.
- `cargo check --manifest-path src-tauri/Cargo.toml --features desktop` — UNAVAILABLE: Linux host lacks GTK/GLib development packages.
- `git diff --check` — PASS.

## Delivery Boundary (Work Unit 4)

Feature-branch-chain Work Unit 4 only, based on `4f936de`. Source/test/config changes remain below the 400-line budget before OpenSpec evidence. No commit, push, PR, runtime-attempt acquire/settlement/reset, or Work Unit 5 work was performed. Rollback: remove command DTO/adapters, command registration, seam test, and serialization-only support; retain the independently tested application/domain modules.

## Remaining

Work Unit 5 only. Desktop GTK and Windows packaging validation remain unavailable on this Linux host.

## Work Unit 5 — React State and Command-Seam Slice

- Completed and verified: 5.1–5.3. Added replaceable typed catalog/confirmation adapters plus a pure sale-flow reducer for search results, draft lines, centavo/quantity feedback, payment drafts, and retained request IDs.
- The reducer deliberately provides feedback only; it does not reproduce Rust price, payment, or stock authority. No persistence call occurs for remove/discard.
- Stopped before 5.4 to remain within the 400-line feature-chain boundary. No React screen, confirmation orchestration, summary view, smoke sale, or excluded workflow was added.

| TDD Cycle Evidence | Result |
| --- | --- |
| RED | `node --test src/ui/sales/sale-flow.test.ts` failed with missing `sale-flow.ts` (exit 1). |
| GREEN | Added adapters/reducer; focused tests passed after removing an action-field leak from cash draft state. |
| TRIANGULATE | Added mixed cash/QR payload and retained-request-ID retry/new-intent cases. |
| REFACTOR | Kept Tauri invocation at the typed adapter seam and React draft state pure. |

- Verification: `npm test` PASS (8); `npx tsc --noEmit` PASS; `cargo test --manifest-path src-tauri/Cargo.toml` PASS (17); `cargo fmt --manifest-path src-tauri/Cargo.toml --check` PASS; `cargo check --manifest-path src-tauri/Cargo.toml` PASS; `git diff --check` PASS.
- Files: `src/commands/{catalog,confirm-sale}.ts`, adapter/reducer tests, `src/ui/sales/sale-flow.ts`, `package.json`, `tsconfig.json`, `tasks.md`, and this progress file.
- Delivery boundary: feature-branch-chain Work Unit 5 first slice, based on `ac91191`; 396 authored additions/deletions including OpenSpec evidence. No commit, push, PR, runtime attempt acquire/settlement/reset, or final verification. Rollback: remove only the new UI state/adapter modules, their tests, and test-script/config support.
- Remaining: 5.4–5.8 (rendered keyboard flow, persisted-summary view, refactor, local smoke/retry, and scope review). Desktop GTK/Windows packaging remain unavailable on this Linux host.

## Work Unit 5 — Confirm Sale UI Completion Slice

- Completed and verified: 5.4–5.8. The React entry point now renders an accessible, keyboard-operable sales screen with labeled catalog search, cart quantity and negotiated-centavo edits, cash tender/change and QR entries, pending submit state, error-code feedback, discard, and new-sale actions.
- Confirmation retains the reducer's request ID through pending/error retries and calls the existing replaceable typed command adapter. React does not calculate price floors, payment equality, stock, or persistence rules.
- Added `persisted-summary.ts` and its focused test. The confirmed view is populated solely from `PersistedSaleSummary` fields and renders sale identity, retained request ID, status/outcome, timestamp, product SKU/name/whole quantity/negotiated price, payment details, and a display-only Bs total.
- Added `index.html` so the Vite production build exercises the React entry point.
- Smoke evidence: the Rust command-seam smoke (`confirms_a_persisted_sale_and_reuses_the_original_summary_for_a_retry`) confirms a seeded QR sale and retries the same request ID with a changed payload; it returns the original summary (total 2,500 centavos) without reapplying sale effects. The use-case suite separately proves stock decrement, one negative movement per line, and zero residual rows/stock changes on rollback. The full desktop-hosted React-to-Tauri process could not run because desktop-feature compilation requires unavailable Linux GTK/GLib packages; Windows packaging was not run.
- Exclusion review: changed paths are limited to the React entry point and sales UI state/presentation. No product management, licensing, network, returns, cancellation, reporting, backup/restore, synchronization, or fractional-quantity workflow was added.

## TDD Cycle Evidence (Work Unit 5 UI Completion)

| Phase | Evidence | Outcome |
| --- | --- | --- |
| RED (5.5) | Added `persisted-summary.test.ts` before `persisted-summary.ts`. | Expected failure: `ERR_MODULE_NOT_FOUND` for the missing summary module (exit 1). |
| GREEN | Added display-only Bs formatting and persisted-summary detail mapping, then wired the reducer/screen to the typed confirmation response. | PASS: focused summary test and all 9 frontend tests. |
| TRIANGULATE | The test asserts identity, UUID request ID, timestamp, product, whole quantity, negotiated price, cash tender/change, outcome, and total from a known persisted response. | PASS. |
| REFACTOR | Preserved the existing typed command adapter seam; React holds draft/presentation state only and sends all rule decisions to Rust. | PASS: frontend suite and TypeScript check. |

## Verification (Work Unit 5 UI Completion)

- `npm test` — PASS: 9 passed, 0 failed.
- `npx tsc --noEmit` — PASS.
- `npx vite build` — PASS after adding the missing Vite `index.html` entry point.
- `cargo test --manifest-path src-tauri/Cargo.toml` — PASS: 17 integration tests passed.
- `cargo check --manifest-path src-tauri/Cargo.toml` — PASS.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check` — PASS.
- `git diff --check` — PASS.
- `cargo check --manifest-path src-tauri/Cargo.toml --features desktop` — UNAVAILABLE: this Linux host lacks GTK/GLib development packages, so desktop React-to-Tauri runtime and Windows packaging were not exercised.

## Files Changed (Work Unit 5 UI Completion)

`index.html`, `src/main.ts`, `src/ui/sales/sale-flow.ts`, `src/ui/sales/sale-screen.ts`, `src/ui/sales/persisted-summary.ts`, `src/ui/sales/persisted-summary.test.ts`, `openspec/changes/confirm-sale/tasks.md`, and this file.

## Delivery Boundary (Work Unit 5 UI Completion)

Feature-branch-chain Work Unit 5 completion slice, based on committed Work Unit 5 state `0f82d01`. This slice adds approximately 221 authored implementation/test lines before OpenSpec evidence and remains within the 400-line budget. No commit, push, PR, runtime-attempt acquire/settlement/reset, or final SDD verification was performed. Rollback boundary: remove the React screen, summary presentation/test, entry HTML, and reducer success state; retain the independently tested Rust command/application/persistence path.

## Remaining

All Confirm Sale implementation tasks (1.1–5.8) are checked. Final independent SDD verification and platform desktop/Windows packaging evidence remain outside this apply slice.

## Work Unit 5 — Authorized TypeScript Correction

- Authorized scope: resolve the reported TypeScript module-resolution blocker at `src/ui/sales/persisted-summary.test.ts:5` only.
- The sibling module `src/ui/sales/persisted-summary.ts` is present at the imported path, and `tsconfig.json` enables `allowImportingTsExtensions` with bundler resolution. No source edit was necessary: the exact import resolves and the focused summary test passes as part of the frontend suite.
- Verification: `npm test` — PASS: 9 passed, 0 failed; `npx tsc --noEmit` — PASS; `cargo test --manifest-path src-tauri/Cargo.toml` — PASS: 17 integration tests passed; `cargo check --manifest-path src-tauri/Cargo.toml` — PASS; `cargo fmt --manifest-path src-tauri/Cargo.toml --check` — PASS; `git diff --check` — PASS.
- The root-level `cargo test`, `cargo check`, and `cargo fmt --check` commands were unavailable because the Rust manifest is intentionally located at `src-tauri/Cargo.toml`; the manifest-qualified equivalents above passed.
- Files changed by this correction: `openspec/changes/confirm-sale/apply-progress.md` only. Existing untracked summary source/test files were not modified.
- Delivery boundary: final Work Unit 5 correction only. No runtime attempt acquire, settlement, reset, commit, push, or PR operation was performed.

## Final-Verification Diagnostic Reconciliation

- Replaced the persisted-summary test and screen's explicit `.ts` local-module specifier with the normal extensionless TypeScript form. Added `tsx` as the test runner so Node executes those TypeScript imports using the same resolver convention as the TypeScript/Vite configuration.
- Verified evidence: `npm test` — PASS: 9 passed, 0 failed; `npx tsc --noEmit` — PASS; `cargo test --manifest-path src-tauri/Cargo.toml` — PASS: 17 integration tests passed; `cargo check --manifest-path src-tauri/Cargo.toml` — PASS; `cargo fmt --manifest-path src-tauri/Cargo.toml --check` — PASS; `git diff --check` — PASS.
- Files changed for this reconciliation: `package.json`, `package-lock.json`, `src/ui/sales/persisted-summary.test.ts`, `src/ui/sales/sale-screen.ts`, and this progress record. The separate SQLite/catalog/Tauri final-verification blockers were not inspected or changed.
- Delivery boundary: diagnostic reconciliation only. No runtime attempt acquire, settlement, reset, commit, push, or PR operation was performed.

## Persistent SQLite Remediation Slice

- Completed: added a production database configuration that resolves to `repuestos-autos.sqlite3` beneath the supplied application-data directory, creates its parent directory, enables foreign keys, and applies the initial migration once through SQLite `user_version` before returning a file-backed connection.
- Added reopen-survival coverage at the public confirmation seam: a confirmed sale is written through the production configuration, the connection is closed, the database is reopened, and a changed same-ID retry returns the original summary without a second sale or stock deduction.
- Deliberate scope: catalog metadata and Tauri target wiring were not changed. The persistent configuration is ready for the existing desktop composition layer to supply its application-data directory in its dedicated slice.

## Verification (Persistent SQLite Remediation)

- RED: `cargo test --manifest-path src-tauri/Cargo.toml --test confirm_sale_use_case` failed with unresolved `open_database` and `production_database_config` imports (exit 101) after the reopen-survival test was added.
- GREEN: the same focused test passed: 8 passed, 0 failed.
- `cargo check --manifest-path src-tauri/Cargo.toml` — PASS.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check` — PASS.
- `git diff --check` — PASS.

## Files Changed (Persistent SQLite Remediation)

`src-tauri/src/infrastructure/sqlite/mod.rs`, `src-tauri/tests/confirm_sale_use_case.rs`, and this progress record.

## Delivery Boundary (Persistent SQLite Remediation)

Approved feature-branch-chain remediation slice only. The implementation/test delta is 103 added lines before OpenSpec evidence, within the 400-line limit. No catalog metadata, Tauri target, runtime attempt acquire/settlement/reset, commit, push, or PR operation was performed. Rollback: remove the persistent configuration/open helper and reopen-survival test; delete any disposable local `repuestos-autos.sqlite3` created for manual testing.

## Authorized Node Type/Module Diagnostic Correction

- Added `@types/node` as a development dependency so the Node built-in `node:assert/strict` and `node:test` imports in `catalog-result.test.ts` resolve under TypeScript.
- Changed the test-runner import to the named Node ESM convention: `import { test } from "node:test"`.
- Focused type evidence: `npx tsc --noEmit --allowImportingTsExtensions --moduleResolution bundler --module esnext --target es2022 --strict --types node src/ui/sales/catalog-result.test.ts` — PASS.
- Verification: `npm test` — PASS: 10 passed; `npx tsc --noEmit` — PASS; `npx vite build` — PASS; `git diff --check` — PASS.
- Files changed: `package.json`, `package-lock.json`, `src/ui/sales/catalog-result.test.ts`, and this progress record. No lifecycle, runtime-attempt, commit, push, or PR action was performed.
- Workload / PR boundary: authorized diagnostic correction only; no task scope or delivery boundary changed.

## Final Verification — Catalog Metadata Remediation

- Completed the catalog-display portion of the final-verification remediation. The existing typed `ProductSearchResult` contract already carries `category_name` and `minimum_unit_price_centavos`; the sales screen now renders both values for every search result.
- Added a focused catalog presentation seam and test. It verifies the typed result's category and 2,500-centavo minimum price are displayed as `Brakes` and `Bs 25.00`, alongside the existing SKU, name, and stock information.
- TDD evidence: the new test was run before `catalog-result.ts` existed and failed with `ERR_MODULE_NOT_FOUND`; adding the minimal formatter made the focused test pass.
- Verification: `npx tsx --test src/ui/sales/catalog-result.test.ts` — PASS; `npx tsc --noEmit` — PASS; `npm test` — PASS: 10 passed; `npx vite build` — PASS; `git diff --check` — PASS.
- Files changed in this slice: `src/ui/sales/catalog-result.ts`, `src/ui/sales/catalog-result.test.ts`, `src/ui/sales/sale-screen.ts`, and this progress record.
- Delivery boundary: feature-branch-chain final-verification catalog metadata slice only; approximately 32 authored source/test lines, below the assigned 250-line boundary. No Tauri window, SQLite/Tauri target, runtime attempt acquire/settlement/reset, commit, push, or PR operation was performed.

## Final Verification — Desktop Target and Persistent Database Wiring

- Completed the remaining production-path remediation: added the feature-gated `repuestos-autos` binary target, Tauri build script, and `src-tauri/src/main.rs`, configured one initial Tauri window, and changed desktop startup to open the existing migration-backed SQLite database beneath Tauri's application-data directory instead of creating an in-memory catalog.
- The desktop setup obtains Tauri's platform-specific application-data directory, passes it to `production_database_config`, and manages the resulting file-backed connection for the existing command handlers. That configuration creates parent directories, enables foreign keys, and migrates once before the window can invoke commands.
- Feasible checks passed: full headless Rust suite (18 integration tests, including reopen-survival persistence), headless Cargo check, Rust format check, frontend suite (10 tests), TypeScript check, Vite build, JSON window-config assertion, Cargo metadata target discovery, and `git diff --check`.
- `cargo metadata` now reports both the library and the `repuestos-autos` binary target. The desktop-feature compile remains unavailable on this Linux host: missing Cairo/GDK/GTK/GLib/WebKit/libsoup development packages cause `cargo check --features desktop` to stop in native dependency build scripts before the Tauri runtime can compile. No desktop-hosted smoke run or Windows packaging was claimed.
- Files changed: `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`, `src-tauri/tauri.conf.json`, and this progress record.
- Workload / PR boundary: authorized final-verification remediation only; 55 authored implementation/config lines before OpenSpec evidence, within the 400-line limit. No runtime attempt acquire/settlement/reset, commit, push, or PR operation was performed.

## Final Remediation — Rust LSP cfg Visibility

- Scoped `src-tauri/src/main.rs` to call `repuestos_autos::run()` only when the `desktop` feature is enabled, matching the public function's existing cfg gate. A no-op non-desktop `main` keeps default-feature Rust analysis and checks free of an unresolved cfg-gated symbol while preserving desktop startup behavior.
- Verification: `cargo check --manifest-path src-tauri/Cargo.toml` — PASS; `cargo fmt --manifest-path src-tauri/Cargo.toml --check` — PASS; `git diff --check` — PASS.
- Files changed: `src-tauri/src/main.rs`, `openspec/changes/confirm-sale/apply-progress.md`.
- Workload / PR boundary: approved feature-branch-chain final remediation only; 7 source lines plus progress evidence, within the assigned 50-line limit. No runtime attempt acquire/settlement/reset, commit, push, or PR operation was performed.
