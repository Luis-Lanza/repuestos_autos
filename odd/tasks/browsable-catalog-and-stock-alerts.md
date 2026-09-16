# Browsable Catalog and Stock Alerts

## Objective
Make Sales, Inventory, and Catalog usable with hundreds of products by replacing search-only or unbounded lists with a shared server-paged product browser. Surface low-stock and out-of-stock alerts persistently in the sidebar.

## Problem
Sales and Inventory start empty until a search is entered. Catalog loads all categories and products into one mixed list. Product search rejects blank queries and limits results to 20. The existing inventory cue is only populated while Inventory is mounted and is cleared on navigation, so alerts are not visible globally.

## Why
The expected catalog has 600+ products and categories. Operators must browse by category, find products quickly, edit catalog prices without scanning a long list, and notice stock risks from any screen.

## Scope
- Add a server-side paged, filterable browse contract for active catalog products.
- Reuse browse behavior in Sales and Inventory.
- Make Sales category-first with global search and stock-aware rows.
- Make Inventory browseable with stock-state filters.
- Keep a global inventory-alert count in the sidebar and navigate it to the Inventory alerts view.
- Replace Catalog's unbounded mixed maintenance list with a bounded, filterable product list while retaining separate category maintenance access.
- Add focused Rust and mounted UI tests for the observable flows.

## Constraints
- Use server-side paging; do not preload the full catalog or add virtualization unless evidence requires it.
- Preserve stale-response and pending-action protections.
- Keep Tauri responses runtime-decoded in `src/commands/`.
- Preserve catalog price validation and inventory integrity rules.
- The sidebar badge must not rely only on red color: expose a count and accessible alert text.
- No commit or push is authorized.
- TDD mode is not explicitly configured; use the repository's focused test suites as verification and preserve existing behavior-first test conventions.

## Acceptance criteria
- A blank Sales or Inventory search shows a bounded first page of active products.
- Users can filter product browsing by category; Inventory can additionally filter by stock state.
- Sales can add an available product from the browsed result list.
- Catalog provides bounded product browsing with search/category/activity filtering and preserves product price editing.
- The sidebar shows a live count for low-stock or out-of-stock products from every screen, hides it at zero, and opens Inventory alerts when activated.
- Stale browse and alert responses cannot overwrite newer UI state.
- Rendered-output and Rust tests cover success, empty, failure, and relevant accessibility states.

## Tasks
- [x] T1 Correct the paged catalog-browse contract integration so every browse and search path uses server paging.
- [x] T2 Correct reusable browser state: reset pagination when filters change and preserve valid category identities.
- [x] T3 Correct Sales integration so global search remains paged and refreshes the global inventory-alert summary after a sale.
- [x] T4 Correct Inventory/sidebar integration so alert counts are refreshed safely and remain accurate across screens.
- [x] T5 Correct Catalog integration: eliminate the unbounded mixed preload and allow editing an out-of-stock product.
- [ ] T6 Add mounted-flow coverage, resolve the W9 policy decision, and rerun all focused checks.

## Progress
- 2026-09-16: User authorized implementation after accepting category-first Sales browsing and a persistent sidebar inventory-alert badge. Current implementation has search-only Sales/Inventory, an unbounded mixed Catalog list, and an Inventory cue scoped to the mounted Inventory screen.
- 2026-09-16: Initial implementation added a browse contract, shared frontend browser, integrations, and a sidebar badge. Independent verification found that legacy search and catalog preload still bypass paging, category identity can drift, alert refresh is incomplete, catalog cannot edit unavailable products, pagination is not reset on query change, and mounted browse flows lack coverage. The initial checkmarks were reopened; the feature is not ready.
- 2026-09-16: User authorized a narrow W9 policy update for this feature's Catalog/IPC contract, together with the independently identified corrections.
- 2026-09-16: Corrected all Sales and Inventory browse requests to use the server-paged contract, added the bounded category-maintenance command, reset pagination for every browser filter, enabled Catalog administration for out-of-stock products, and wired stale-safe sidebar refreshes after mutations.
- 2026-09-16: Added mounted Sales, Inventory, Catalog, and sidebar coverage for paged requests, out-of-stock editing, mutation refresh, and failed alert refresh; W9 now allows only the authorized Catalog/IPC paths and registration additions.
- 2026-09-16: A second independent verification found three blocking correctness defects: App and Inventory own competing sidebar-alert refresh sequences, filter changes do not invalidate in-flight browse responses, and W9 splits a literal backslash-n rather than actual newlines. T1-T6 remain open until these are corrected and independently rechecked.
- 2026-09-16: Narrow correction completed: App now owns global sidebar-alert refresh sequencing; Inventory requests that owner after successful mutations and keeps its alert-panel refresh local; every browse filter increments the request identity to invalidate in-flight responses; W9 parses actual newline-separated diffs and regression coverage rejects unrelated Catalog/IPC drift.
- 2026-09-16: Required regression, typecheck, frontend, and Rust test commands all passed; T6 remains open because the desktop GTK compile check is still unavailable in this environment.

## Verification evidence
- `npm run typecheck:tests` passed.
- `npm test` passed: 199 tests, including the global-alert owner callback, four-filter stale-response invalidation, and W9 newline/drift regressions.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed: all tests passed.
- `npm run typecheck:tests` passed after the narrow correction.
- `cargo check --manifest-path src-tauri/Cargo.toml --features desktop` could not complete because the environment lacks the required system packages `glib-2.0 >= 2.70`, `gobject-2.0 >= 2.70`, `gio-2.0 >= 2.70`, and `gdk-3.0 >= 3.22`.
- Final independent verification passed `npm run typecheck:tests`, `npm test` (199/199), and `cargo test --manifest-path src-tauri/Cargo.toml`; it confirmed the stale global-alert race, stale browse-response race, and W9 newline parser bypass are resolved.
- The desktop-feature check fails in GTK/GIO dependency build scripts before project code compiles (`PKG_CONFIG_PATH` is unset); this is an environmental blocker, not a project compilation failure.

## Next step
Run `cargo check --manifest-path src-tauri/Cargo.toml --features desktop` in an environment with the required GTK/GIO development packages, then close T6. No system-package installation is authorized in this workspace.
