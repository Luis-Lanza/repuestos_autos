# Tasks: Catalog Maintenance and Pricing

## Review Workload Forecast

| Field | Value |
|---|---|
| Estimated authored changed lines | Historical plan superseded; PR3 357 actual, PR4 399 actual, PR5 400 actual; PR6–PR8 pending estimates |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR1 → PR2 → PR3 → PR4 → PR5 → PR6 → PR7 → PR8 |
| Delivery strategy | ask-on-risk |
| Chain strategy | stacked-to-main |

Decision needed before apply: No (resolved to chained PRs)
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

### Suggested Work Units

| Unit | Goal (start → finish) | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|---|---|---|---|---|---|
| 1 | v6 → v7 schema and domain transitions | PR1 | `cargo test --manifest-path src-tauri/Cargo.toml --test sqlite_migrations` | N/A—in-process seam | Revert migration, version, domain |
| 2 | PR1 → use case and SQLite adapter | PR2 | `cargo test --manifest-path src-tauri/Cargo.toml catalog_maintenance` | N/A—SQLite integration seam | Revert catalog application/adapter |
| 3 | PR2 → lifecycle IPC and maintenance UI | PR3 | `npx tsx --test src/commands/catalog.test.ts src/ui/catalog/catalog-maintenance-flow.test.ts` | N/A—protected pre-existing `dist/` cannot be cleaned safely | Revert catalog command/UI and command/reducer tests |
| 4 | PR3 → domain/application metadata intent and validation | PR4 | `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_maintenance_domain --test catalog_maintenance_application` | N/A—in-process domain/application seam | Revert metadata intent, repository seam, and focused tests |
| 5 | PR4 → guarded SQLite persistence and v7→v8 forward migration | PR5 | `cargo test --manifest-path src-tauri/Cargo.toml --test sqlite_migrations --test catalog_maintenance_sqlite` | N/A—in-process SQLite seams | Revert SQLite adapter, v8 migration/registry, and focused tests |
| 6 | PR5 → typed Rust/Tauri metadata edit IPC | PR6 | `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_maintenance_commands` | N/A—in-process command seam | Revert typed command DTOs, registration, and tests |
| 7 | PR6 → accessible metadata adapter and edit UI | PR7 | focused catalog TypeScript tests | N/A—protected `dist/` cannot be cleaned safely | Revert catalog adapter, edit UI, and tests |
| 8 | PR7 → stale-cart and inventory behavior | PR8 | `cargo test --manifest-path src-tauri/Cargo.toml confirm_sale_application` | `npm run tauri:dev`—reprice/reconfirm | Revert sales/inventory integration |

## Phase 1: Migration and Domain Foundation

- [x] 1.1 RED: extend `src-tauri/tests/sqlite_migrations.rs` for v6→v7 preservation, incompatible preflight, reopen, and constraints.
- [x] 1.2 GREEN: create `0007_catalog_maintenance.sql`; update `src-tauri/src/infrastructure/sqlite/mod.rs` with backfill, revisions, immutable audit, and transaction.
- [x] 1.3 RED: add `src-tauri/tests/catalog_maintenance_domain.rs` for normalized uniqueness, typed values, centavos, and independent lifecycle.
- [x] 1.4 GREEN: modify `src-tauri/src/domain/catalog.rs` with states, intents, snapshots, transition plans, and validation errors.

## Phase 2: Application and SQLite Core

- [x] 2.1 RED: add `src-tauri/tests/catalog_maintenance_application.rs` cases for ordering, expected revisions, stable outcomes, and rollback.
- [x] 2.2 GREEN: modify `src-tauri/src/application/catalog/{mod.rs,repository.rs}` with `MaintainCatalogUseCase::execute(intent)` and a narrow load/apply seam.
- [x] 2.3 RED: add `src-tauri/tests/catalog_maintenance_sqlite.rs` for races, immutable audit, FTS rollback, visibility, dual filters, 20,000-product rename, and plans.
- [x] 2.4 GREEN: modify `src-tauri/src/infrastructure/sqlite/{catalog_repository.rs,sale_repository.rs,inventory_repository.rs}` for immediate transactions, FTS/audit atomicity, and filters.

## Phase 3: Lifecycle IPC and Catalog UI (PR3)

- [x] 3.1 RED: add command-seam tests and `src/commands/catalog.test.ts` for deny-unknown payloads, opaque outcomes, and SQL-detail hiding.
- [x] 3.2 GREEN: modify `src-tauri/src/commands/catalog.rs` and `src-tauri/src/lib.rs`; add allowlisted `src/commands/catalog.ts` adapters and registration.
- [x] 3.3 RED: add `src/ui/catalog/catalog-maintenance-flow.test.ts` for loading, unavailable, validation, conflict, failure, recovery, and archived records.
- [x] 3.4 GREEN: create lifecycle `src/ui/catalog/{catalog-maintenance-flow.ts,catalog-maintenance-screen.ts}` and modify `src/ui/app.ts` for accessible navigation/state.

## Phase 4: Metadata Intent and Validation (PR4)

- [x] 4.1 RED: extend catalog domain/application tests for metadata validation, duplicate normalization, price/typed-value validation, and stale revisions.
- [x] 4.2 GREEN: add category/product metadata edit intents and a narrow application repository seam with expected revisions and stable typed outcomes.

## Phase 5: SQLite Persistence and Forward Migration (PR5)

- [x] 5.1 RED: extend migration/SQLite tests for v7→v8 compatibility, guarded metadata writes, typed-value replacement, audit/FTS rollback, and category rename refresh.
- [x] 5.2 GREEN: add v8 normalized-name migration/preflight and transactional metadata persistence with category/product FTS refresh and audit.

## Phase 6: Typed Rust/Tauri Edit IPC (PR6)

- [x] 6.1 RED: add command-seam coverage for typed edit requests, opaque outcomes, and deny-unknown payloads.
- [x] 6.2 GREEN: add allowlisted Rust/Tauri edit DTOs, response projection, unavailable mapping, and registration.

## Phase 7: Accessible Metadata Editing (PR7)

- [x] 7.1 RED: extend catalog adapter/reducer tests for metadata requests, success projection, validation, pending, and conflict reload.
- [x] 7.2 GREEN: extend catalog TypeScript adapter and accessible edit UI with category/product detail, labels, feedback, and retry flows.

## Phase 8: Stale Cart, Inventory, and Verification (PR8)

- [x] 8.1 RED: extend `src-tauri/tests/confirm_sale_application.rs`, `src/commands/confirm-sale.test.ts`, and `src/ui/sales/sale-flow.test.ts` for captured price/revision, stale blocking, acknowledgement, second-change rejection, and immutable lines.
- [x] 8.2 GREEN: modify `src-tauri/src/application/sales/{mod.rs,confirm_sale.rs}`, `src-tauri/src/commands/confirm_sale.rs`, `src/commands/confirm-sale.ts`, and `src/ui/sales/{sale-flow.ts,sale-screen.ts}`.
- [x] 8.3 RED: add archived-operation cases to `src-tauri/tests/inventory_sqlite.rs` proving balances, movements, alerts, and sale facts remain unchanged.
- [x] 8.4 GREEN: update `src-tauri/src/infrastructure/sqlite/inventory_repository.rs` for active-only operations and alerts.
- [x] 8.5 Verify Rust/frontend suites, MockRuntime, redirected production build, accessibility, maintenance, and stale-cart smoke; `tauri:dev` is N/A because protected pre-existing `dist/` cannot be safely mutated.
