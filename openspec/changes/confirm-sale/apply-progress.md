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
