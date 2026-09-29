# Configurable Product Low-Stock Threshold

## Objective
Allow each product to define its own low-stock threshold while preserving the current threshold of one unit when a value is omitted.

## Problem and Rationale
The fixed quantity `1` is duplicated across inventory movement alerts, Catalog stock filters, and Dashboard alerts. It cannot reflect products that should be replenished earlier. A product-specific threshold must preserve the distinct out-of-stock condition at quantity zero.

## Scope and Constraints
- Store one mandatory per-product integer threshold with a forward-only SQLite migration and a default of `1` for existing products.
- Accept only whole thresholds greater than or equal to `1`; zero remains exclusively `out_of_stock`.
- Offer an optional field on product onboarding and Catalog product editing; an empty field resolves to `1`.
- Use the persisted threshold in Dashboard alerts, Catalog stock filters, and inventory-operation alert classification.
- Do not create category-level threshold settings or change stock movement semantics.

## Delivery Forecast
- Estimated authored change: 300–400 lines across migration, Rust contracts, TypeScript adapters/UI, and focused tests.
- Strategy: ask-on-risk. Do not commit, push, or open a PR without explicit user authorization.

## TDD and Checks
- Mode: standard; strict TDD disabled by `openspec/config.yaml`.
- Runners: `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run typecheck:tests`.

## Tasks
- [x] T1 Persist and validate the per-product threshold. Route: delegated direct (multi-file write). Scope: migration, catalog domain/application/repository contracts, inventory alert classification, and focused Rust tests. Checks: migration preserves existing products as 1; creation/edit reject 0 and invalid values; empty input resolves to 1; an out-of-stock item remains distinct. Evidence: `cargo test --manifest-path src-tauri/Cargo.toml` passed (all Rust tests, including threshold migration and inventory-alert coverage).
- [x] T2 Apply thresholds to read models and transport. Route: delegated direct (multi-file write). Scope: Dashboard query, Catalog stock filters, metadata/create/edit commands and TypeScript command decoders, with focused Rust/TypeScript tests. Checks: threshold-driven low/available/alerts results, Dashboard ordering/limit unchanged, and malformed IPC data is rejected. Evidence: Rust suite and `npm run typecheck:tests` passed; `npm test` passed (343 tests) after T2a admitted the exact audited feature paths.
- [x] T2a Admit the current feature paths to the W9 evidence-audit allowlist. Route: delegated direct. Scope: the exact feature task document, migration, and inventory paths already changed, plus only the three authorized `src-tauri/tests/command_seam.rs` additions. Checks: current feature passes W9; unrelated untracked paths remain rejected. Evidence: `npm test` passed (343 tests); W9 accepts exactly the two `low_stock_threshold: None` fixture lines and the corrected ordered `primary_location_code` projection, while rejecting adjacent-line and unrelated-path drift.
- [x] T3 Expose the optional field in product onboarding and Catalog editing. Route: delegated direct (multi-file write). Scope: flows, screens/dialog, accessibility, and mounted tests. Checks: blank defaults to 1; valid whole numbers submit; zero/invalid input shows field feedback; saved value reopens for editing. Evidence: `npm test` passed (345 tests); `npm run typecheck:tests` passed.
- [x] T3a Close verification coverage gaps. Route: delegated direct. Scope: inventory alert threshold-above-one assertion and explicit blank Catalog-edit submission default. Checks: focused Rust/frontend tests prove both behaviors. Evidence: focused Rust alert test passed (2 tests in `inventory_sale_alerts`); focused Catalog mounted suite passed (37 tests); `cargo test --manifest-path src-tauri/Cargo.toml` passed; `npm run typecheck:tests` passed. `npm test` ran 345 tests but one unrelated W9 protected-path audit failed because `src-tauri/tests/inventory_sale_alerts.rs` is not in its allowlist; do not expand T3a scope.
- [x] T3b Admit the authorized inventory alert test to W9. Route: delegated direct. Scope: only `src-tauri/tests/inventory_sale_alerts.rs`. Checks: W9 accepts that exact path and rejects near matches. Evidence: `npm test` passed (345 tests); W9 accepts only the exact inventory alert test path among the additions and rejects near-match/unrelated paths.
- [x] T4 Independently verify the feature. Route: delegated direct (verification trigger). Evidence: independent verification passed `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test` (345 tests), `npm run typecheck:tests`, and `git diff --check`; status was stable before/after checks. Acceptance criteria were confirmed across migration, Dashboard/Catalog/inventory classification, transport, UI behavior, and W9 boundaries.

## Acceptance Criteria
- Existing products behave as if their low-stock threshold were `1` after migration.
- A new or edited product may store only a whole threshold of `1` or greater; a blank UI field stores `1`.
- Quantity `0` is always classified as out of stock, never low stock.
- Quantities from `1` through the product threshold are low stock; larger quantities are available.
- Dashboard and Catalog filters use the same product-specific classification as inventory alerts.
- The threshold can be set during onboarding and modified from the Catalog product editor.

## Progress
- 2026-09-29: User confirmed the invariant: the minimum threshold is `1`, because quantity `0` must be reported directly as out of stock.
- 2026-09-29: User approved the T1 expansion required by the migration dispatcher and existing product-input test literals: `src-tauri/src/infrastructure/sqlite/mod.rs`, `src-tauri/tests/product_onboarding.rs`, `src-tauri/tests/catalog_browse.rs`, and `src-tauri/tests/command_seam.rs`.
- 2026-09-29: User approved the remaining T1 contract-test paths: `src-tauri/tests/catalog_maintenance_application.rs`, `src-tauri/tests/catalog_maintenance_sqlite.rs`, and `src-tauri/tests/catalog_maintenance_domain.rs`.
- 2026-09-29: User approved `src-tauri/tests/catalog_maintenance_commands.rs` to update the direct catalog-edit request fixture and unblock T1 compilation.
- 2026-09-29: User approved `src-tauri/tests/backup_restore.rs` to update schema-version assertions affected by migration 0022.
- 2026-09-29: User authorized correcting the pre-existing stale `primary_location_code` expectation in `src-tauri/tests/command_seam.rs`, which blocks T1's required Rust suite but is unrelated to threshold behavior.
- 2026-09-29: User authorized adding only the current feature paths to W9's exact allowlist after its protected-path audit blocked T2's frontend suite.
- 2026-09-29: User authorized W9's two exact `command_seam` audit lines: the threshold fixture field and the separately authorized stale `primary_location_code` expectation.
- 2026-09-29: User authorized the exact W9 audit additions for the two `low_stock_threshold: None` fixture lines and stale ordered `primary_location_code` projection; `npm test` passed with 343 tests, including adjacent-line and unrelated-path rejection checks.
- T3a: Added coverage for inventory alert classification at threshold 3 (quantity 2 is low stock; quantity 0 remains out of stock) and verified blank Catalog edit submission sends threshold 1. Focused suites and Rust full suite passed; the frontend suite initially exposed that W9 needed the explicitly authorized inventory regression path.
- T3b: W9 now admits exactly `src-tauri/tests/inventory_sale_alerts.rs`; rejection coverage protects near-match filenames and unrelated paths. `npm test` passed (345 tests).

## Evidence
- Read-only exploration found hard-coded threshold behavior in `src-tauri/src/domain/inventory.rs`, `src-tauri/src/application/catalog/mod.rs`, and `src-tauri/src/infrastructure/sqlite/dashboard_repository.rs`.
- T3 exposes the optional product-specific threshold in onboarding and Catalog editing; mounted tests verify defaulting, validation, submission, and persisted-value editing. `npm test` passed (345 tests); `npm run typecheck:tests` passed.

## Next Step
- Await explicit authorization to commit the verified feature. No commit or push has been made.
