# Catalog access and profit reporting

## Goal
Add device-local catalog access control, a period-based gross-profit report, correct retired category-field details in sales, and rename Dashboard to Metrics.

## Decisions
- Catalog access is device-local configuration outside SQLite and outside ordinary database backups.
- Sales remains operational while Catalog is locked: it receives a separate safe category-list projection and safe product-search projection, neither exposing purchase cost or catalog-management metadata.
- Unlocking Catalog authorizes only the Catalog screen's sensitive management session until the user explicitly chooses "Bloquear catálogo" or the app restarts. Inventory, product onboarding, Sales, history, reports, backup, and every other application screen remain usable while Catalog is locked.
- Sales search, inventory alerts, and movement-ledger product options remain available while Catalog is locked; these read surfaces never disclose purchase cost.
- Password and recovery code are stored only as salted, memory-hard hashes; plaintext values are never persisted.
- A new licensed device restored from a backup configures its own catalog password and recovery code.
- A database backup remains a full SQLite data replacement; it never transfers license or catalog-access configuration.
- Historical gross profit uses the immutable sale-line unit-cost snapshot. Missing historical costs remain explicitly partial rather than inferred from current product costs.
- Returns affect gross profit in the date range containing the return event, not the original sale date.
- T3 first version presents one inclusive local-date period total plus partial/unknown-cost disclosure; it does not add daily or product breakdowns. It reuses the movement-ledger local-date-to-UTC boundary conversion and excludes cancelled sales under existing reporting semantics.
- Retired category fields must not appear in current product details shown during sales.

## Tasks
- [x] T1 Map catalog access, report, category-field, navigation, IPC, migration, and backup seams. Route: delegated exploration. Catalog access has no existing local-secret store; backup remains SQLite-only. Gross-profit source is immutable sale-line unit-cost snapshots; field-retirement correction is expected in the current sales projection; Metrics is presentation-only.
- [ ] T2 Add local catalog access setup, password verification, recovery-code rotation, and backend authorization boundaries. Route: delegated writer. Independent audit correction pending: sales product search remains available outside Catalog but must not expose purchase cost through direct IPC; password-change UI needs duplicate-submit/stale-response protection; catalog-access config replacement needs durable parent-directory handling and Windows desktop verification. Required Windows desktop/runtime command-surface and local-config verification: `cargo test --manifest-path src-tauri/Cargo.toml --features desktop --test catalog_access && cargo test --manifest-path src-tauri/Cargo.toml --features desktop --lib sale_search_and_inventory_read_surfaces_stay_available_while_catalog_is_locked_without_cost`.
- [x] T3 Add a date-range gross-profit report using captured sale-line costs and partial-cost disclosure. Route: delegated writer. Closed final audit corrections: Reports exposes one main landmark with gross profit as a labelled region; zero-cost snapshots fail as invalid persisted data in both report readers.
- [ ] T4 Hide retired category fields from sale product details without rewriting historical business data. Route: delegated writer.
- [ ] T5 Rename Dashboard presentation to Metrics and verify navigation accessibility. Route: delegated writer.
- [x] T6 Verify device-local access stays outside backups, old backups restore safely, and all affected flows pass Linux/Windows checks. Windows automated evidence passed on 2026-10-02: `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_access installing_a_restored_sqlite_snapshot_does_not_replace_device_local_access_configuration` — 1/1 passed. Manual fixed-NTFS backup/restore also succeeded and Catalog remained unlockable with the same password. Route: delegated verifier.
- [x] T7 Keep Sales usable while Catalog is locked through a cost- and metadata-free sale browse/category projection; expose an explicit accessible Catalog lock action while preserving password/recovery controls. Route: delegated writer.
- [x] T8 Keep Inventory and product onboarding operational while Catalog is locked using a separate safe Inventory browse projection and onboarding-specific location contracts; preserve Catalog gating and Sales projection behavior. Route: delegated writer. Verified with command-surface and mounted locked-session regressions.
- [x] T9 Make Catalog table columns responsive without removing horizontal scrolling for genuinely wide content. Route: delegated writer. Independent audit corrections keep primary-location wrapping table-only, keep price/stock labels unbroken within minimum-width columns, and bind this UI-only candidate's exact paths and unified diff in W9. Verified with focused mounted Catalog/base-style and W9 regressions, full frontend suite, typecheck, build, and diff checks.

## Acceptance criteria
- Catalog mutations and catalog screens require a local password after setup; reads outside Catalog remain available.
- Password reset requires a recovery code and rotates that code; plaintext secrets never persist or cross IPC.
- Restoring a database backup onto another licensed PC requires local catalog access setup on that PC and does not overwrite its existing local access configuration.
- Gross-profit reports accept a date range, use immutable sale-line cost snapshots, exclude canceled sales, account for returns, and disclose missing costs.
- Retired category fields are absent from current sale product details.
- Sidebar and page title say Metrics rather than Dashboard.
- Existing compatible database backups restore to the current schema without fabricating missing historical costs.
