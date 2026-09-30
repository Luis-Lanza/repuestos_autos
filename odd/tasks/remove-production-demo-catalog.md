# Remove Production Demo Catalog Seed

## Objective
Ensure a new production database starts with no sample catalog data while preserving any customer data that could have evolved from the historical demo rows.

## Context
Windows fresh-install validation showed `Filtro de aceite` (`FLT-001`) and `Bujía archivada` (`BUJ-001`). They originate from `src-tauri/src/infrastructure/sqlite/migrations/0001_confirm_sale.sql`, not from residual data or licensing.

## Constraints
- Do not rewrite migration `0001_confirm_sale.sql`; it may already have been applied.
- Add a forward-only migration after schema version 22.
- Remove only exact known demo data and only when no customer-owned facts depend on it.
- Never delete a product, category, stock balance, movement, sale, return, cancellation, or other fact that could be customer data.
- Fresh databases and upgraded historical test databases must be covered.

## Tasks
- [x] T1 Map seeded records and every dependent table/foreign key; define a conservative deletion predicate and migration order. Route: delegated exploration. Delete only exact untouched `FLT-001`/`BUJ-001` rows with initial stock quantities and no sales, movements, attributes, images, audit, or location facts; preserve on any mismatch or dependency.
- [x] T2 Add a forward migration and migration tests proving clean seed removal plus preservation when dependent customer facts exist. Route: delegated writer. Evidence: schema 23 migration is transactional and guarded by exact seed values plus dependency/search/stock predicates; focused v23 migration tests cover fresh/v22 cleanup, preservation, FK integrity, and idempotent reopen. The complete Rust suite passes, including versioned backup fixtures through v23.
- [ ] T3 Independently verify migration safety and validate a fresh Windows install starts empty. Route: delegated verifier.

## Acceptance Criteria
- A fresh database contains no seeded demo categories/products after migration completion.
- Existing demo records with customer-owned dependent facts are preserved.
- Migration runs transactionally, advances schema safely, and preserves foreign-key integrity.
- Windows installation evidence confirms an empty Catalog before user onboarding.

## Progress
- 2026-09-30: Independent review recommendation addressed with focused schema-v23 preservation tests for a customer product attribute value and a customer-modified product name. The exact product fields, dependent attribute fact, and foreign-key integrity remain intact; targeted migration tests and the complete Rust suite pass.
- 2026-09-30: User approved correcting production demo catalog seed before merge.
- 2026-09-30: T2 implemented as schema 23; focused migration and catalog-search tests pass. The stale nullable-price migration test now creates its category explicitly for the empty fresh schema. The complete Rust test suite passes, including versioned backup/restore fixtures through v23; Windows fresh-install evidence remains pending T3.
- 2026-09-30: Replaced the v22 cleanup-test fixture's current-schema/user-version shortcut with the production migrations 0001–0022, stopping before v23. The fixture asserts its version and representative v22 columns plus historical stock/search rows before each cleanup scenario upgrades it; focused and full Rust tests and `git diff --check` pass.

## Next Step
- Independently review migration safety and validate a fresh Windows install.
