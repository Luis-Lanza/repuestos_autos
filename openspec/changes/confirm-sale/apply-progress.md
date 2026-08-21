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

## Remaining

Work Units 1–2 are complete. Work Units 3–5 remain out of scope for this slice. Desktop GTK validation remains unavailable on this Linux host because required GTK/GLib development packages are missing; Windows packaging was not run. Rust formatting is installed and passes its final check.
