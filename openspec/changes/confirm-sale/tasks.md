# Confirm Sale Implementation Tasks

## Review Workload Forecast

| Field | Value |
| ------- | ------- |
| Estimated changed lines | 1,900–2,700 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 bootstrap + catalog → PR 2 domain rules → PR 3 atomic persistence → PR 4 typed command seam → PR 5 React flow + integration |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: pending
400-line budget risk: High

The forecast exceeds both the configured 350-line review budget and the 400-line guard. Apply must stop for a delivery decision and chain-strategy selection before starting the first work unit.

## 1. Work Unit 1 — Bootstrap and Searchable Seeded Catalog

Start: documentation-only repository. Finish: runnable React/Tauri/Rust test scaffolds can open a disposable migrated SQLite database and search seeded active products. Rollback: remove the newly introduced scaffold, migration, and catalog files; recreate the disposable database.

- [x] 1.1 Add the minimal Tauri 2 + React/TypeScript scaffold in `package.json`, `tsconfig*.json`, `vite.config.*`, `src/main.*`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, and `src-tauri/src/lib.rs`; add only test scripts and dependencies that are actually runnable, then record their exact commands in the relevant manifests.
- [x] 1.2 **RED/bootstrap:** configure focused frontend and Rust test entry points under `src/**/*.test.*` and `src-tauri/tests/`; prove the harness fails on a missing disposable-database bootstrap/search capability before production implementation exists.
- [x] 1.3 **GREEN:** create `src-tauri/src/infrastructure/sqlite/` connection and migration modules plus `src-tauri/src/infrastructure/sqlite/migrations/0001_confirm_sale.sql`; enable `PRAGMA foreign_keys = ON` on every connection and create catalog, searchable-field, product, and stock tables with integer/check constraints and indexes.
- [x] 1.4 **TRIANGULATE:** extend disposable-database tests in `src-tauri/tests/catalog_search.rs` to read seeded active products by name, SKU, category, and configured searchable category field, while proving inactive products are excluded and stock/minimum-price centavos are returned.
- [x] 1.5 **GREEN:** add migration seed rows and implement the read-only catalog module in `src-tauri/src/application/catalog/` and `src-tauri/src/infrastructure/sqlite/catalog_repository.rs` until the search matrix passes without hard-coded product control flow.
- [x] 1.6 **REFACTOR:** consolidate migration/bootstrap helpers inside `src-tauri/src/infrastructure/sqlite/` without widening the catalog interface; run the newly added Rust and frontend commands and document any unavailable Windows packaging check in the apply evidence.

Verification boundary: a fresh disposable database migrates successfully, foreign keys report enabled, seed readback is deterministic, and the catalog search matrix passes. No sale write path exists yet.

## 2. Work Unit 2 — Sale Domain Rules

Start: Work Unit 1 catalog/bootstrap is green. Finish: framework-independent Rust domain rules validate sale lines and cash/QR/mixed payments using checked integer arithmetic. Rollback: remove only `src-tauri/src/domain/` sale modules and their tests.

- [x] 2.1 **RED:** add public-interface tests in `src-tauri/src/domain/` for `MoneyCentavos`, `Quantity`, `RequestId`, sale-line totals, current-minimum snapshots, and cash/QR payment values; cover positive examples and rejected fractional/zero/negative or overflow-prone inputs.
- [x] 2.2 **GREEN:** implement the smallest explicit value types and tagged payment interface in `src-tauri/src/domain/` using UUID parsing, checked `i64` arithmetic, positive whole quantities, and non-negative centavos.
- [x] 2.3 **TRIANGULATE:** add tests for below-current-minimum rejection, exact applied-payment equality, negative/inconsistent cash values, QR-only, cash-only, mixed payment, empty lines, and multi-line totals.
- [x] 2.4 **GREEN:** implement sale construction/validation in `src-tauri/src/domain/sales/` so authoritative rules are independent of React, Tauri, and SQLite.
- [x] 2.5 **REFACTOR:** reduce the domain interface to the values and operations required by `ConfirmSaleUseCase`, remove primitive mixing/duplicated calculations, and rerun the focused Rust domain suite.

Verification boundary: domain tests prove quantity, price-floor, total, and payment invariants without a database or command adapter.

## 3. Work Unit 3 — Atomic and Idempotent SQLite Confirmation

Start: Work Units 1–2 are green. Finish: `ConfirmSaleUseCase` owns one transaction that atomically persists or rolls back a complete sale and reconstructs persisted summaries. Rollback: remove sale migrations/repositories/use case and recreate the disposable database; do not rewrite any non-disposable real sale database.

- [x] 3.1 Extend `src-tauri/src/infrastructure/sqlite/migrations/0001_confirm_sale.sql` (or add the next versioned migration discovered from the scaffold) with `sales`, `sale_lines`, `sale_payments`, and `inventory_movements`; enforce unique non-null `sales.request_id`, row-level money/quantity/payment checks, foreign keys/indexes, non-negative stock, and triggers rejecting movement update/delete.
- [x] 3.2 **RED:** add disposable-database tests in `src-tauri/tests/confirm_sale_use_case.rs` for a successful multi-line cash sale, persisted minimum-price snapshots, conditional stock decrements, one negative immutable movement per line, and a summary reloaded from stored rows.
- [x] 3.3 **GREEN:** define transaction-scoped repository interfaces and SQLite adapters under `src-tauri/src/application/sales/` and `src-tauri/src/infrastructure/sqlite/`; repository methods must receive the application-owned transaction context and must not commit independently.
- [x] 3.4 **GREEN:** implement `src-tauri/src/application/sales/confirm_sale.rs` to reserve the request ID, reload current product/minimum/stock data, invoke domain validation, persist lines/payments, conditionally decrement stock, append movements, mark confirmed, reconstruct the aggregate, and commit once.
- [x] 3.5 **TRIANGULATE:** extend the same seam tests with QR-only, mixed payment, inactive/missing product, stale below-minimum price, unequal payment, inconsistent cash, one-line insufficient stock, and a multi-line failure after earlier writes; assert zero partial sales, payments, movements, or stock deductions.
- [x] 3.6 **RED/GREEN:** add repeated-request-ID tests that submit a changed retry payload and assert the original sale identity/summary is returned with unchanged row counts and stock; implement conflict-aware reservation and complete aggregate reconstruction without revalidating or applying the retry payload.
- [x] 3.7 **TRIANGULATE:** add integrity tests for incomplete aggregate reconstruction, foreign-key enforcement, request-ID uniqueness, row checks, and movement update/delete rejection; map incomplete persisted state to a persistence-integrity failure rather than a partial success.
- [x] 3.8 **REFACTOR:** centralize transaction rollback/error conversion and persisted-summary mapping without moving business rules into repositories; rerun domain and disposable-database suites.

Verification boundary: the use-case seam proves success, rollback, stock integrity, immutable movements, idempotency, and persisted reconstruction against SQLite without React.

## 4. Work Unit 4 — Typed Tauri Command Seam

Start: Work Unit 3 use-case tests are green. Finish: typed `search_products` and `confirm_sale` commands expose stable request/response/error contracts and exercise the complete Rust-to-SQLite path. Rollback: unregister and remove command adapters/DTOs while retaining the independently tested application modules.

- [x] 4.1 **RED:** add command-seam integration tests in `src-tauri/tests/command_seam.rs` for search DTOs and the complete confirmation behavior matrix, including malformed UUIDs, unsafe/non-integer-compatible shapes, stable error codes, successful persisted summaries, and same-request retry.
- [x] 4.2 **GREEN:** implement DTOs and thin adapters in `src-tauri/src/commands/catalog.rs` and `src-tauri/src/commands/confirm_sale.rs`; perform shape parsing/error mapping only, delegate once to application modules, and expose integer centavo/quantity fields without floating-point conversion.
- [x] 4.3 **TRIANGULATE:** assert every specified rejection maps to a stable discriminated code and leaks no SQLite/internal branching details; verify successful results contain sale ID, request ID, status, timestamp, lines, prices/snapshots, payments, total, and outcome from persisted records.
- [x] 4.4 **REFACTOR:** share only contract/error-mapping helpers that reduce duplication, register both commands in `src-tauri/src/lib.rs`, and rerun all Rust suites plus a check/build command present in `src-tauri/Cargo.toml`.

Verification boundary: command tests prove the typed React-facing seam through real application and SQLite adapters; no UI behavior is required.

## 5. Work Unit 5 — React Sales Flow and Integrated Evidence

Start: Work Unit 4 command seam is stable. Finish: the operator can search, manage a draft, pay, retry with one retained UUID, and view the persisted summary through the local React → Tauri → Rust → SQLite flow. Rollback: remove/disable the sales route and `src/ui/sales/` while preserving persisted sales and inventory history.

- [x] 5.1 **RED:** add reducer/adapter tests under `src/ui/sales/**/*.test.*` and `src/commands/**/*.test.*` for active search results, add/remove/discard without persistence calls, quantity/centavo input feedback, minimum-price prefill, payment draft state, and integer-safe command payloads.
- [x] 5.2 **GREEN:** implement typed adapters in `src/commands/catalog.ts` and `src/commands/confirm-sale.ts` plus the sale reducer/modules in `src/ui/sales/`; keep draft/presentation state in React and authoritative rules in Rust.
- [x] 5.3 **TRIANGULATE:** add UI tests for cash-only, QR-only, and mixed payload construction; pending/error states must preserve the draft and one generated request ID, repeated confirmation must reuse it, and discard/new intent must replace it.
- [ ] 5.4 **GREEN:** implement keyboard-friendly search, cart quantity/negotiated-price editing, payment entry, confirmation feedback, and error-code presentation in `src/ui/sales/`; do not add product management, licensing, networking, or other excluded workflows.
- [ ] 5.5 **RED/GREEN:** add tests proving confirmed UI state renders only the returned `PersistedSaleSummary`, including identity, request ID, timestamp, products, whole quantities, negotiated prices, payment breakdown, outcome, and Bs formatting; implement the summary view without recalculating authoritative totals.
- [ ] 5.6 **REFACTOR:** keep the command adapter as the replaceable UI test seam, remove duplicated Rust business rules from React, and rerun the configured frontend suite.
- [ ] 5.7 Run every test/check command introduced in `package.json` and `src-tauri/Cargo.toml`, exercise a local React → Tauri → Rust → SQLite smoke sale and same-ID retry, and record observed row/stock/movement evidence; report Windows packaging or other unavailable checks explicitly rather than claiming they ran.
- [ ] 5.8 Review changed paths against `openspec/changes/confirm-sale/specs/sales/spec.md` and confirm excluded product-management, licensing, network, return, cancellation, reporting, backup, synchronization, and fractional-quantity behavior was not introduced.

Verification boundary: focused frontend and Rust suites pass, the local end-to-end smoke path confirms and retries one persisted sale without duplicate effects, and unavailable platform evidence is documented honestly.
