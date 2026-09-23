# Catalog Redesign Implementation

## Objective
Implement the approved Catalog redesign on `feat/catalog-redesign`: persistent Table/Gallery presentation, Category Management subview with archive confirmation, and exactly one optional local product image stored safely in SQLite and included in backup/restore.

## Product decisions
- Products is the default Catalog surface.
- Table/Gallery preference persists across application restarts; toolbar controls change presentation only.
- Gallery has five normal-desktop columns and two compact columns; cards have explicit Edit actions.
- Table retains dense operational browsing with an optional thumbnail.
- Category Management is a Catalog subview with search, category name, active-product count, state, and Edit/Archive/Reactivate actions; category creation remains onboarding-only.
- Archive requires confirmation; reactivation is direct; category archive remains blocked while active products exist.
- One optional product image is persisted as SQLite BLOB data in a one-to-one record, with MIME type; accept PNG/JPEG/WebP, max 2 MiB and 1600px on either side. Backup/restore includes it automatically.

## Tasks
- [x] T1 Map the exact persistence, backup, IPC, UI, and test seams; split safely.
- [x] T2A Add product-image schema, schema validation, and backup/restore coverage.
- [x] T2B Add bounded image repository/application persistence and focused storage tests.
- [x] T2C Store a bounded internal thumbnail derivative with each image and prove backup/restore retention.
- [ ] T3A Add revision-checked native picker/image IPC and Rust command coverage without path exposure.
- [x] T3B Add TypeScript image decoder/adapters and Catalog image edit/thumbnail display integration.
- [x] T4A Add Catalog-scoped persistent Table/Gallery presentation and responsive Gallery.
- [x] T4B1 Add authoritative active-product counts to category metadata contracts.
- [ ] T4B2 Add dedicated Category Management subview and preserve lifecycle/recovery behavior (mounted and flow tests pass; the required runner's test typecheck is currently blocked by an existing T4B1 type error in `src/commands/catalog.ts:78`).
- [x] T4C Add archive confirmation before mutation; preserve direct reactivation.
- [ ] T5 Verify behavior, accessibility, migrations, backup, and visual contracts.

## Evidence
- Next schema version is 16; migration and current-schema validation live under `src-tauri/src/infrastructure/sqlite/`.
- Product browse/detail/edit span application catalog models, SQLite repository, Rust command payloads, `src/commands/catalog.ts`, and their focused tests.
- Backup/restore uses SQLite online backup and staged validation; a new image table is included automatically in snapshots but must be current-schema validated.
- Existing `dialog:allow-open` capability can support bounded native selection without granting frontend filesystem access.
- User approved internal thumbnail derivatives: retain original bytes for backup/editing and persist a bounded display thumbnail to avoid sending originals for every browse row.
- Image replace/remove is a revision-checked product edit that advances the catalog record revision; stale outcomes reuse existing recovery semantics.
- Table/Gallery preference is Catalog-scoped local UI state with defensive storage fallback to Table; it must not alter shared Sales/Inventory ProductBrowser presentation.
- Category Management uses backend-authoritative active-product counts, never counts inferred from the loaded browser page.

## T2A Evidence
- Strict TDD RED: `cargo test --manifest-path src-tauri/Cargo.toml --test sqlite_migrations upgrades_v15_with_one_to_one_product_images_and_cascade_delete` failed because the migration left `user_version` at 15 instead of advancing it to 16.
- GREEN: `cargo test --manifest-path src-tauri/Cargo.toml --test sqlite_migrations upgrades_v15_with_one_to_one_product_images_and_cascade_delete` — passed; migration, one-image uniqueness, FK rejection, MIME/size/nonempty constraints, and cascade deletion verified.
- TRIANGULATE: `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore product_image` — 3 passed; snapshot/stage retained exact MIME and BLOB bytes; current-schema validation rejected a missing and unconstrained image table.
- Initial verification passed (24 sqlite_migrations, 26 backup_restore), but independent verification found schema validation could accept ineffective image CHECK constraints through substring matching.
- Strict TDD RED: added `rejects_v16_backup_with_product_image_constraints_only_in_comments`; `cargo test --manifest-path src-tauri/Cargo.toml --test sqlite_migrations --test backup_restore` failed as intended because staged validation accepted the malformed v16 database.
- Strict TDD GREEN: replaced substring checks with comment-stripped structural comparison of the complete product-image table DDL; retained the existing schema and staged-restore behavior.
- Focused/required verification: `cargo test --manifest-path src-tauri/Cargo.toml --test sqlite_migrations --test backup_restore` — passed (24 sqlite_migrations, 27 backup_restore), including the regression and valid image backup/restore cases.

## T2B Evidence
- Strict TDD RED: `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_maintenance_sqlite product_images_validate_format_content_and_dimensions_before_storage` failed as intended because the image decoder dependency and `ProductImage` type were not implemented.
- GREEN: the focused `product_image` test run passed all 4 tests covering PNG/JPEG/WebP decode, invalid and mismatched content, exact/over byte limits, dimension limits, repository replace/read/remove, missing products, and rollback on failed replacement.
- TRIANGULATE: the same focused tests verified PNG/JPEG/WebP alternatives, over-limit and mismatched/invalid negatives, exact byte/dimension maxima, no image for nonexistent products, and preservation of the prior record after a failed update.
- Required verification: `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_maintenance_sqlite --test sqlite_migrations --test backup_restore` passed (9 catalog maintenance, 24 migration, 27 backup/restore tests).

## T2C Evidence
- Strict TDD RED: extended the storage test to require `ProductImage::thumbnail`; `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_maintenance_sqlite product_image_repository_replaces_reads_removes_and_rejects_missing_products` failed to compile because that behavior was not implemented.
- GREEN: added forward-only v17 thumbnail columns and validation, deterministic JPEG derivatives capped at 256px, same-transaction original/thumbnail replacement, paired removal/readback, and v16 compatibility.
- TRIANGULATE: storage tests cover absent v16-era thumbnail then replacement, dimensions, MIME/original retention, paired-column constraints, replacement rollback, and deletion; backup tests cover snapshot retention and staging a v16 image into v17 with both thumbnail fields absent.
- Required verification: `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_maintenance_sqlite --test sqlite_migrations --test backup_restore` — passed (10 catalog maintenance, 24 migrations, 29 backup/restore tests).
- Confirmed T2C defect: persisted-thumbnail decoding guessed the byte format but trusted the `image/jpeg` label, so PNG/WebP bytes were accepted as JPEG.
- Strict TDD RED: added `persisted_jpeg_thumbnail_rejects_png_or_webp_bytes`; `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_maintenance_sqlite persisted_jpeg_thumbnail_rejects_png_or_webp_bytes` failed because the mislabeled bytes were returned as a valid persisted image.
- Strict TDD GREEN: persisted thumbnails now verify the detected format is JPEG before decoding and reject over-limit thumbnail byte payloads before decode; both mislabeled PNG and WebP cases return the bounded persistence failure while absent v16-era thumbnails remain readable.
- TRIANGULATE: valid thumbnail behavior and legacy null-thumbnail compatibility remain covered by the existing repository and migration/backup tests.
- Required strict verification: `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_maintenance_sqlite --test sqlite_migrations --test backup_restore` — passed (11 catalog maintenance, 24 migrations, 29 backup/restore tests).

## T3A Evidence
- Strict TDD RED: the focused Rust runner failed because the new image request, thumbnail response, revision-aware persistence API, and thumbnail-read seam were not implemented.
- GREEN: `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_maintenance_commands --test catalog_maintenance_sqlite` — passed (4 command tests, 14 SQLite tests), including strict request validation, no path in response JSON, base64 JPEG thumbnail identity, revision advance, stale conflict, corrupt-thumbnail rejection, and atomic replacement rollback.
- TRIANGULATE: focused tests cover stale thumbnail reads, thumbnail identity/format, removal stale conflicts, and failed replacement preservation. Desktop command-surface cancellation/registration tests were added but could not execute because system `glib-2.0`/`gio-2.0` development packages are unavailable in this environment.
- The native picker command receives only product ID and expected revision, reads selected files inside Rust with a bounded byte limit, validates image contents before storage, and returns typed cancellation/errors. Existing `dialog:allow-open` is reused; no filesystem capability was added.

## T3B Evidence
- Strict TDD RED: the exact TypeScript runner failed because `createCatalogProductImageCommands` was not yet exported; mounted Catalog and thumbnail presentation tests also failed before UI integration.
- GREEN: `npx tsx --test --import ./test/react-dom.ts src/commands/catalog.test.ts src/ui/catalog/catalog-maintenance-screen.mounted.test.ts src/ui/catalog/product-browser.test.ts && npm run typecheck:tests` — passed (39 tests and test typecheck).
- TRIANGULATE: covered picker cancellation, path-bearing/malformed native payload rejection, oversized/noncanonical/non-JPEG thumbnail rejection, image replace/remove, duplicate-action pending lock, stale authoritative-detail recovery, and browse/detail thumbnail absent/present states. Browse payload projections do not expose image source fields.
- Desktop runtime verification was not attempted, as requested.
- Windows desktop compile fix: `choose_product_image_command` now returns `Result<ProductImageResponse, String>`, retains its typed safe outcomes inside `Ok`, and moves an owned `AppHandle` into the picker setup after releasing the `WebviewWindow`. The adjacent unnecessary `mut` on the selected image file was removed. Focused non-desktop command tests pass; desktop-feature compilation still requires Windows recheck.
- T3B contract correction: Rust now returns `image_unavailable` for a product at the requested revision that has no thumbnail, keeps stale revisions as `stale_catalog_record`, and keeps a missing product as `catalog_unavailable`; mounted tests verify absence renders `Sin imagen` without preview-load feedback and real unavailable outcomes still show feedback. Strict TDD RED: the Rust distinction test failed because absence returned `catalog_unavailable`, then exposed stale absence incorrectly mapping to `image_unavailable`; GREEN: `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_maintenance_commands` passed (5 tests). Required TypeScript verification: `npx tsx --test --import ./test/react-dom.ts src/commands/catalog.test.ts src/ui/catalog/catalog-maintenance-screen.mounted.test.ts src/ui/catalog/product-browser.test.ts && npm run typecheck:tests` — passed (40 tests and typecheck).

## T4A Evidence
- Strict TDD RED: the focused TypeScript runner failed as intended because Catalog view controls, Gallery rendering, and preference persistence were not implemented; the mounted remount test also exposed the test runtime's unavailable localStorage and was adjusted to use an isolated storage stub.
- GREEN: `npx tsx --test --import ./test/react-dom.ts src/ui/catalog/product-browser.test.ts src/ui/catalog/catalog-maintenance-screen.mounted.test.ts src/ui/app-shell.mounted.test.ts && npm run typecheck:tests` — passed (36 tests and test typecheck).
- TRIANGULATE: verified default Table, persisted Gallery across remount, malformed and throwing storage fallback, toggle accessible selected state and 44px CSS target, unchanged browse request count/query during switching, matching Table/Gallery product status/price/SKU/category/thumbnail and explicit Edit controls, 5/2 responsive grid rules, and no view controls for shared non-Catalog presentations.
- Gallery mode is owned by CatalogMaintenanceScreen and defensive localStorage helpers; ProductBrowser browse state and shared Sales/Inventory presentation paths are unchanged.

## T4B1 Evidence
- Strict TDD RED: the exact focused runner failed to compile because the application-level category metadata query contract had not been implemented.
- GREEN: `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_maintenance_application --test catalog_maintenance_sqlite --test catalog_maintenance_commands && npx tsx --test src/commands/catalog.test.ts` — passed (3 application, 15 SQLite, 6 command, and 15 TypeScript tests).
- TRIANGULATE: verified authoritative counts include active products only, categories with zero active products remain present, archived products do not count, Rust command JSON includes the count, and TypeScript rejects missing, negative, fractional, or unsafe counts.
- No Category Management UI or archive confirmation work was included.

## T4B2 Evidence
- Strict TDD RED: mounted tests failed because Category Management navigation, category rows/actions, and preserved subview context were not implemented; the category-search flow test failed because the filter selector was absent.
- GREEN/TRIANGULATE: the mounted and flow portions of the exact runner passed (29 tests), covering navigation and preserved browse state, search, authoritative count/state, edit entry, direct archive/reactivate, contextual archive-block feedback, unavailable/list retry, and absence of unsupported controls.
- Required exact runner: `npx tsx --test --import ./test/react-dom.ts src/ui/catalog/catalog-maintenance-screen.mounted.test.ts src/ui/catalog/catalog-maintenance-flow.test.ts && npm run typecheck:tests` — tests passed; `npm run typecheck:tests` failed at existing `src/commands/catalog.ts:78` (TS2322: `maintenanceRecord` yields a broad category-or-product target where `CatalogCategoryMaintenanceRecord` requires `category`). That file is outside T4B2 edit surfaces and was not changed. Keep T4B2 unchecked until the exact required runner passes.
- Follow-up: Strict TDD RED reproduced TS2322 with `npm run typecheck:tests`; GREEN explicitly set the decoded record's target to `category` after the existing runtime category check. `npm run typecheck:tests` — passed. No UI behavior changed; the T4B2 checkbox remains unchanged.
- Category archive confirmation remains unimplemented here; it is T4C.

## T4C Evidence
- Strict TDD RED: the exact focused runner failed in the mounted category and product flows because archive actions invoked lifecycle IPC immediately instead of opening a confirmation.
- GREEN: `npx tsx --test --import ./test/react-dom.ts src/ui/catalog/catalog-maintenance-screen.mounted.test.ts src/ui/catalog/catalog-maintenance-flow.test.ts src/ui/visual-system/confirmation-dialog.mounted.test.ts && npm run typecheck:tests` — passed (43 tests and test typecheck).
- TRIANGULATE: category and product archive dialogs make no IPC request before confirmation; Cancel and Escape make no mutation and restore focus to the opener, including when the confirmation is stacked over the product edit dialog; confirmation sends the captured entity ID/revision exactly once; pending disables confirmation and dismissal; blocked/stale outcomes retain existing feedback and recovery; reactivation remains direct; successful mutations retain existing refresh.

## Blockers
- T3A focused command/storage tests pass. The Windows desktop command compile issue in `choose_product_image_command` has been corrected, but desktop-feature compilation and command registration still require a Windows recheck before marking T3A complete.

## Constraints
- Preserve all Catalog domain validation, stale async handling, request sequencing, and recovery behavior.
- Do not expose filesystem paths or raw storage errors over IPC.
- No commit, push, PR, or merge without separate explicit authorization.
