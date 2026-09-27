# Category Field Management

## Objective
Allow Catalog users to add and retire category-defined product fields from **Manage Categories → Edit**, without deleting existing product attribute values.

## Problem
Category fields are created only during onboarding. The existing category editor changes its name only, so categories cannot evolve when the store needs a new product attribute or wants to stop collecting an obsolete one.

## Scope
- Add new category fields through the existing category edit path.
- Retire existing fields while preserving their definition and product attribute values.
- Exclude retired fields from current product creation/edit validation and entry forms.
- Preserve historical retired values in product detail.
- Detect a category schema change when a product draft was based on an older schema and return recoverable feedback.

## Constraints
- Do not physically delete attribute definitions or values.
- V1 does not modify the type, required flag, label, or option set of an existing field.
- Keep stable definition IDs for retained fields.
- A retired field label may be reused by a new active definition; SQLite uniqueness applies to active definitions only.
- Category schema mutation must be transactional, revision-checked, and audited.
- Existing unrelated untracked ODD task files remain untouched.
- TDD mode: standard; source: `openspec/config.yaml` (`sdd.strict_tdd: false`).
- Rust runner: `cargo test --manifest-path src-tauri/Cargo.toml`.
- Delivery strategy: ask-on-risk. Forecast: ~350 authored changed lines, excluding generated files.

## Tasks
- [x] T1 Define category-schema lifecycle contracts and canonical acceptance scenarios. Route: delegated direct; trigger: multi-file write. Checks: focused domain/application tests specify add, retire, retained values, invalid stable-ID edits, revision conflicts, and stale product schemas.
- [x] T2 Persist active/retired definitions and implement transactional category-schema diff. Route: delegated direct; trigger: multi-file write. Checks: migrations, SQLite/application/domain tests, audit/revision behavior, and rollback evidence.
- [x] T3 Expose schema editing through typed Tauri/TypeScript contracts and enforce stale product-schema handling. Route: delegated direct; trigger: multi-file write. Checks: Rust command registration/contracts, TypeScript runtime decoders, malformed payloads, and product mutation conflict tests.
- [x] T4 Add accessible Manage Categories editor controls for adding and retiring fields. Route: delegated direct; trigger: multi-file write. Checks: flow and mounted tests for drafts, confirmation, pending/error/stale states, keyboard accessibility, and existing product form/detail behavior.
- [x] T5 Run focused end-to-end seam validation and independent verification. Route: delegated direct; trigger: verification. Checks: full Rust suite, full frontend suite, test typecheck, and diff check all pass.

## Acceptance Criteria
- A category can gain a valid field through its edit dialog.
- A field can be retired with explicit confirmation; its existing values remain stored and historically visible.
- Retired fields are not requested or required for new/current product edits.
- A product save based on a stale category schema fails safely and prompts reload rather than silently losing or misvalidating values.
- Invalid/duplicate field definitions, stale category revisions, and persistence failure preserve existing data.

## Progress
- 2026-09-27: Authorized by user. Read-only exploration completed. User selected retirement with value preservation. Branch: `feat/category-field-management`.
- 2026-09-27: T1/T2 backend slice recovered after an interrupted worker. The partial diff was reconciled; no unrelated tracked files were changed.
- 2026-09-27: Independent verification previously failed T1/T2: replacement labels, v19 structural validation, and candidate Rust formatting required correction.
- 2026-09-27: Focused correction checks passed, but independent reassessment failed: the existing `UNIQUE(category_id, label)` constraint prevents reusing a retired field label, despite the domain planner allowing it. Candidate-added formatting findings also remained outside the last correction surface.
- 2026-09-27: T1/T2 corrected: schema v19 rebuilds the definitions table to replace label uniqueness with a partial active-only unique index; migration tests retain IDs, options, values, and FK integrity, and the SQLite use case retires then recreates the same label with a new ID. Focused T1/T2 suites pass; candidate Rust additions are formatted. Full cargo fmt remains blocked by unrelated baseline formatting debt.
- 2026-09-27: Closed the remaining v19 validation gap by rejecting any non-partial unique index on `(category_id, label)`, independent of index name, while preserving validation of the required active-only partial index. Added a differently named global-index regression case; focused Rust suites and diff check pass.
- 2026-09-27: Tightened v19 validation to require the active-label partial index predicate to normalize exactly to `active=1`; added a regression for `active=1 OR active=0`. Focused SQLite catalog tests pass; repository-wide formatting remains blocked by the documented unrelated baseline debt.

## Evidence
- 2026-09-27: T3 IPC/application implementation is currently partial: focused Rust maintenance suites passed (31 tests), TypeScript catalog tests passed (17 tests), and `git diff --check` passed. Product schema revision is now carried in the Rust/TypeScript edit contract and stale saves return `stale_category_schema`; however, the existing product edit-flow producer under `src/ui/catalog/catalog-maintenance-flow.ts` is outside the delegated edit surfaces and does not yet supply that revision. Do not treat T3 as complete until the caller contract is resolved.
- `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_maintenance_domain --test catalog_maintenance_application --test catalog_maintenance_sqlite --test product_onboarding`: passed (40 tests), including retired-label recreation, v18→v19 migration preservation/structural validation, and rejection of a differently named global unique index.
- `git diff --check`: passed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: fails on existing formatting debt in unrelated baseline tests (`confirm_sale_use_case.rs`, `dashboard_reporting.rs`, `inventory_domain.rs`, `inventory_sqlite.rs`, `product_onboarding.rs`, `sales_history_commands.rs`, `sqlite_migrations.rs`). The formatter output contains no findings in `src-tauri/src/infrastructure/sqlite/mod.rs` or `src-tauri/tests/catalog_maintenance_sqlite.rs`; those candidate files are formatted. Baseline and candidate both retain the unrelated formatter debt.
- Independent verification: previous T1/T2 defects are covered by the passing schema replacement, lifecycle, migration structural-validation, rollback, and arbitrary-name global-index tests; ready for reassessment.
- Final v19 predicate-gap correction: `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_maintenance_sqlite` passed (20 tests), including rejection of `WHERE active = 1 OR active = 0`; `git diff --check` passed. `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` fails only on unrelated pre-existing formatting findings in tests outside the authorized edit surfaces; neither modified Rust file appears in formatter output. This matches the baseline formatting failures recorded above.

- T3 wiring completion: exposed `category_revision` in product metadata detail, decoded and validated it in TypeScript, sent it as `expected_category_revision` from product edits, and made `stale_category_schema` trigger recoverable edit handling while preserving stale product behavior. Verified by `npx tsx --test src/commands/catalog.test.ts src/ui/catalog/catalog-maintenance-flow.test.ts` (23 passed), `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_maintenance_application --test catalog_maintenance_commands --test catalog_maintenance_sqlite` (31 passed), and `git diff --check` (passed).

- 2026-09-27: T4 adds accessible category schema drafts, keeps existing definitions immutable, requires confirmation before retirement, omits retired definitions from the typed schema-edit request, and reloads authoritative detail after stale schema conflicts. The exact focused frontend test command passed (53 tests), and `git diff --check` passed.
- 2026-09-27: Fixed two independently verified T4 regressions: retired required product attributes are omitted from edit form state, field validation, and edit requests while backend detail continues to retain historical definitions/values; a stale category-schema product edit now reloads selected authoritative detail through the established stale-record recovery flow. Added unit and mounted regression coverage, including existing stale-product recovery behavior.
- T5 final stale current-schema assertion now compares against `CURRENT_SCHEMA_VERSION`; `cargo test --manifest-path src-tauri/Cargo.toml` passed, `npm test` passed (307 tests), `npm run typecheck:tests` passed, and `git diff --check` passed.

## Next Step
- T1–T5 are complete; full Rust/frontend validation, test typecheck, and diff check passed.
