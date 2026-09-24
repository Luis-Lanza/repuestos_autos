# Global Product Pricing

## Goal
Replace customer-facing list-price terminology with sale-price terminology and add a global current purchase price, updated atomically by product creation, catalog edit, and stock entry while retaining stock-entry purchase price as immutable audit evidence.

## Decisions
- Product current prices are global: purchase, sale, and minimum sale.
- Stock entry requires purchase price and may optionally update sale and minimum sale prices.
- Stock count adjusts quantity only; it never changes prices.
- Sale/minimum updates affect all current stock immediately; confirmed sales remain immutable.
- Stock-entry purchase price is retained as audit evidence; historical products/movements remain unknown rather than fabricated.
- Minimum sale price must be positive and no greater than sale price; money remains integer centavos.
- Visible terminology changes globally; persisted legacy list-price names remain compatible aliases initially.

## Tasks
- [x] T1 Map and specify pricing contracts, invariants, migration and test slices. Evidence: current v17 stores sale/list and minimum prices; stock movements lack immutable purchase-cost evidence; inventory idempotency v1 omits all prices.
- [x] T2 Add SQLite migration, schema validation, backup/restore and storage coverage. Implemented and independently verified locally. Added v18 nullable immutable `unit_purchase_price_centavos` on stock-entry movements (historical movements remain unknown) and nullable global `purchase_price_centavos` on products. New writes will require a positive purchase cost; physical counts leave every price unchanged.
- [x] T3 Extend catalog/onboarding domain, application, IPC and TypeScript pricing contracts. Implemented, independently verified and committed locally. Purchase, sale and minimum pricing now flow through product create/edit and Catalog/onboarding contracts; canonical sale pricing retains compatibility aliases.
- [ ] T4 Extend stock-entry transaction, idempotency identity and audit cost persistence.
- [ ] T5 Update pricing UI and terminology across onboarding, Catalog, Inventory, Sales and checkout.
- [ ] T6 Validate cross-layer behavior, migrations, backup/restore and desktop UI flows.

## TDD and delivery
- Delivery strategy: stacked to main, selected by the user after T2 exceeded the ~400-line review budget. T2 is the first local work-unit commit; no push or PR has been authorized.
- TDD mode: standard (not strict), from `openspec/config.yaml` (`sdd.strict_tdd: false`; its testing policy selects standard mode unless strict TDD is explicitly enabled).
- Rust runner: `cargo test --manifest-path src-tauri/Cargo.toml`.
- T2 focused checks: `cargo test --manifest-path src-tauri/Cargo.toml --test sqlite_migrations` and `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore`.

## Evidence
- T1 contract map complete.
- T2: Added `0018_global_product_purchase_price.sql`, schema sequencing and restored-database validation. V17 upgrades preserve NULL historical costs; restored v18 databases must retain both positive safe-integer CHECK constraints. Focused migration tests pass (26/26); backup/restore tests pass (33/33); `git diff --check` passes. Independent verification passed. Committed locally; no push was authorized.
- T3: Focused Rust checks pass (48/48) and focused frontend checks pass (63/63). Independent verification passed. `tsc --noEmit` remains a documented, clean-HEAD-confirmed baseline failure: exactly 32 unrelated diagnostics; it is not a passing gate. Committed locally; no push was authorized.
