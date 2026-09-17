# Bootstrap Demo Catalog

## Objective
Provide a development-only command that turns a pristine bootstrap database into exactly 10 categories and 100 active automotive demo products with valid prices and opening stock.

## Problem
Manual onboarding of 100 products is impractical. Direct SQLite insertion would bypass product, stock movement, attribute, and FTS invariants.

## Decision
The loader is a development command, not a permanent UI/import feature. It operates only on a recognized pristine bootstrap catalog with no business activity, preserves the two shipped bootstrap categories/products, reactivates the shipped archived `BUJ-001` record, and adds eight categories plus 98 products to reach exactly 10 active categories/products. It refuses any database that is not the expected bootstrap state; it never deletes user data.

## Scope
- Add a development command that opens the same application database path used by the desktop app.
- Validate bootstrap-only eligibility before mutation.
- Create deterministic, varied automotive demo categories/products through the catalog/onboarding application path or equivalent invariant-preserving transactional seam.
- Make reruns idempotent: a complete recognized demo catalog reports success without duplicates; partial/unexpected data fails closed.
- Print a useful summary and run focused Rust tests.
- Document the exact Windows command and backup prerequisite.

## Constraints
- No direct ad-hoc SQL that bypasses catalog, stock, movement, and FTS consistency.
- Do not add a production UI control or Tauri IPC command.
- Never delete, truncate, or replace a database.
- Prices use integer centavos; opening stock is positive whole units.
- No commit or push is authorized.
- TDD mode is not explicitly configured; use behavior-first focused tests.

## Acceptance criteria
- On a fresh installed database, the command leaves exactly 10 active categories and 100 active products.
- Each new product has a unique SKU/name, valid list/minimum prices, positive opening stock, searchable catalog presence, and valid stock/movement facts.
- A complete recognized demo state is idempotent; any partial/unexpected/non-bootstrap state is refused without changes.
- The documented Windows invocation targets `%APPDATA%\com.repuestosautos.app\repuestos-autos.sqlite3` through the application's resolution logic.
- Focused Rust tests prove success, idempotency, refusal, and invariant preservation.

## Tasks
- [x] T1 Map/reuse the existing catalog bootstrap and application transaction seams for a development command.
- [x] T2 Strengthen bootstrap/complete-state recognition so tampered audit, FTS, stock, movement, revision, and content facts fail closed.
- [x] T3 Add adversarial recognition tests, rerun focused tests, and verify documentation.

## Progress
- 2026-09-16: User requested 10 total categories and 100 total products in the Windows local database, selected a development command, and accepted a bootstrap-only non-destructive loader.
- 2026-09-16: The shipped `BUJ-001` bootstrap product is archived; user explicitly selected its reactivation plus 98 new products so the final catalog contains 100 active products.
- Implemented the loader, transaction seam, explicit/default database path handling, CLI help, focused tests, and documentation.
- Independent verification found fail-closed recognition incomplete: it omits catalog audit facts and validates FTS only by row count/rowid, while complete-state checks omit stock, searchable values, revisions, movement timestamps, audit JSON, and full FTS content. T2/T3 were reopened; the loader is not ready.
- Recognition now compares complete ordered fingerprints for categories, products, stock balances, searchable values, FTS row IDs/product IDs/content, opening movements (including IDs, links, quantities, immutable timestamp, and nullable identity fields), and catalog audit records/content/timestamps. Demo opening movements and the reactivation audit use the deterministic timestamp `2025-01-01T00:00:00Z`.
- Adversarial tests cover pristine and complete states, tampering each meaningful catalog, searchable, FTS, stock, revision/activity, price, audit, and movement fact class; each refusal is checked for unchanged table counts.

## Verification evidence
- `cargo test --manifest-path src-tauri/Cargo.toml bootstrap_demo`: passed; 6 bootstrap-demo tests passed, including pristine and complete adversarial tamper matrices.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed; all unit, integration, and doc tests passed.
- `cargo run --manifest-path src-tauri/Cargo.toml --bin bootstrap-demo -- --help`: passed; printed command syntax, explicit database option, and the Windows `%APPDATA%\\com.repuestosautos.app\\repuestos-autos.sqlite3` path.
- Final independent verification found no blockers. It confirmed full ordered fingerprints, tamper refusal without mutation, exact 10/100 state, idempotency, transaction rollback, standalone CLI containment, and Windows backup documentation.

## Next step
User can authorize commit/push, then create an in-app backup and run the documented command against the Windows database.
