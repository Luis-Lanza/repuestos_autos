# Reports Movement Ledger

## Objective
Add a primary-sidebar Reports workspace that opens to a read-only inventory movement ledger and exports the currently filtered result as a PDF.

## Approved Product Decisions
- Reports is a primary sidebar destination, not an Inventory subview.
- The first release contains only the Movement Ledger; do not show placeholders for future report types.
- Default to the current calendar month and support date-range, product, and movement-type filters.
- Show immutable movement facts: timestamp, product/SKU, movement type, signed quantity delta, resulting quantity, and the persisted entry note or adjustment/cancellation reason when present.
- Export the current filtered result to a PDF.
- Reuse the existing desktop design system, navigation, accessibility, feedback, responsive, and guarded-async patterns.

## Constraints
- Do not edit or reinterpret historical movements from Reports.
- Do not invent a reason for returns, because none is persisted.
- Preserve movement immutability, sales/correction links, and stock integrity.
- Use Spanish UI copy and existing app visual primitives.
- No commit, push, or pull request is authorized.

## TDD and Checks
- Mode: standard; strict TDD disabled by `openspec/config.yaml`.
- Determine the exact focused Rust/frontend runners during T1 and record them here before implementation.

## Tasks
- [x] T1 Define and test the movement-ledger read model, filtering, bounded pagination, and command contract. Route: delegated exploration/design plus bounded implementation. Evidence: focused Rust and command-contract tests passed; independent re-verification passed.
- [x] T2a Implement the bounded PDF export backend and native save interaction. Route: delegated multi-file writer. Evidence: focused Rust export tests and independent re-verification passed.
- [x] T2b Implement the Reports sidebar route, guarded filter flow, and read-only ledger presentation using shared visual-system components. Route: delegated multi-file writer. Evidence: focused mounted UI, adapter/flow, typecheck, Rust contract checks, and independent re-verification passed.
- [x] T3 Independently verify the approved behavior, design-system conformance, and focused/full required checks. Route: delegated verification. Evidence: full Rust/frontend/typecheck/diff checks passed; desktop runtime limitation recorded.
- [x] T4 Repair the Windows desktop Tauri export-command contract. Route: delegated bug fix. Scope: return a Tauri-compatible async result without borrowed input lifetimes, preserve the bounded export response semantics, and add/adjust regression coverage. Evidence: focused export/frontend/typecheck checks passed and Windows desktop compile passed.
- [x] T5 Correct the Reports product filter after Windows smoke feedback. Route: delegated UI correction. Scope: reuse the established submitted-search/result-selection interaction (not the active-category Catalog data contract); add bounded ledger-owned product search that includes archived products and archived-category products; preserve stale-response/accessibility behavior; and correct the desktop filter-grid overlap with a deliberate two-row layout. Evidence: focused Rust/frontend flow/mounted tests and full frontend audit passed; subsequent Windows feedback isolated vertical alignment follow-ups handled by T6/T7.
- [x] T6 Correct Product/Movement Type vertical alignment after Windows smoke feedback. Route: delegated UI correction. Scope: render product search feedback/results outside the Product/Type/Apply control row so those controls stay top-aligned; preserve accessible feedback, selection, keyboard behavior, and responsive layout. Evidence: focused mounted test passed; subsequent Windows feedback isolated the Apply action alignment handled by T7.
- [ ] T7 Correct Apply Filters vertical alignment after Windows smoke feedback. Route: delegated UI correction. Scope: align the unlabelled apply action with the Type control input rather than its label by reserving equivalent label space; preserve form semantics and responsive behavior. Evidence: focused mounted test/typecheck/diff checks passed; final Windows visual confirmation remains pending.

## Acceptance Criteria
- Reports is a primary sidebar destination and initially shows only the Movement Ledger.
- The ledger defaults to the current month and filters by date range, product, and movement type.
- Every displayed row faithfully represents a persisted movement; notes/reasons appear only when stored.
- Rows are read-only and retain relevant sale/correction references.
- Repeated or stale filter responses cannot overwrite the current result.
- Loading, empty, error, retry, and ready states are accessible and visually consistent with the application.
- PDF export reflects the active filters, identifies the report and generated time, paginates its data, and communicates success/failure accessibly.
- Existing application behavior remains verified.

## Progress
- 2026-09-29: User authorized implementation after selecting a dedicated Reports sidebar workspace, a Movement Ledger-only initial release, current-month default, PDF export, and strict reuse of the existing design system.
- 2026-09-29: Delivered as size-exception work unit `3be60bf feat(reports): add inventory movement ledger`; pushed to `origin/feat/reports-movement-ledger`. Six unrelated untracked ODD task documents remain excluded.
- 2026-09-29: Windows desktop compilation exposed a Tauri async-command contract error in `export_movement_ledger_command`; user authorized T4 repair and push to the same branch.
- 2026-09-29: User authorized a verification checkpoint push of T4 before Windows confirmation. Local export tests (6), frontend tests (360), test typecheck, and diff checks passed.
- 2026-09-29: Windows confirmation passed: `cargo check --manifest-path src-tauri/Cargo.toml --features desktop` completed successfully in 1m 00s.
- 2026-09-29: Windows smoke feedback identified an unscalable all-products dropdown and Product/Movement Type overlap. User approved reuse of the established Sales/Catalog/Inventory searchable product interaction.
- 2026-09-29: User authorized adding `src-tauri/src/lib.rs` to T5 so the existing ledger product-options command can receive bounded search query/page inputs.
- 2026-09-29: User authorized a narrow `src/ui/w9-evidence-audit.test.ts` update so its Reports command-shape guard accepts only the bounded product-search wrapper signature and call.
- 2026-09-29: T5 automated verification passed: focused Rust (8), focused frontend (13), test typecheck, and full frontend audit (363) passed; Windows smoke found Product/Movement Type vertical misalignment when product feedback is visible.
- 2026-09-29: User authorized T6 to move product feedback/results outside the controls row.
- 2026-09-29: Windows smoke found Apply Filters aligned with the Type label rather than the Type input; user authorized T7 correction.
- 2026-09-29: User requested merge before a final post-T7 Windows screenshot. `feat/reports-movement-ledger` fast-forwarded to `master` and pushed at `58eec5e`; T7 final visual confirmation remains a follow-up.

## Evidence
- T1: `cargo test --manifest-path src-tauri/Cargo.toml --test movement_ledger --test movement_ledger_commands` passed (5 tests); `git diff --check` passed. Independent re-verification confirmed strict filter validation, archived-product inclusion, stable ordering/paging, nullable recorded balance, exact UTC half-open semantics, and rejection of fractional-second bounds. Desktop-feature compilation remains unavailable in this Linux environment because GTK/GIO development libraries are missing.
- T2a: `cargo test --manifest-path src-tauri/Cargo.toml --test movement_ledger_export` passed (5 tests); `git diff --check` and untracked-file whitespace checks passed. Independent re-verification confirmed the 2,000-row non-truncating cap, active-filter export, metadata, paginated readable layout, complete long-detail continuations, strict request validation, and generic cancellation/write-failure feedback. Native dialog/filesystem-save runtime remains unavailable for verification in this GTK/GIO-limited Linux environment.
- T2b: `./node_modules/.bin/tsx --test --import ./test/react-dom.ts src/commands/movement-ledger.test.ts src/ui/reports/movement-ledger-flow.test.ts src/ui/reports/movement-ledger-screen.mounted.test.ts src/ui/app-shell.mounted.test.ts` passed (18 tests, no React `act(...)` warnings); `npm run typecheck:tests` passed; `cargo test --manifest-path src-tauri/Cargo.toml --test movement_ledger --test movement_ledger_commands` passed (6 tests); `git diff --check` and untracked-file whitespace checks passed. Independent re-verification confirmed Reports navigation/focus, current-month filters, archived-product selection, truthful read-only facts, stale-response protection, filter retention through paging/retry, accessible states, and path-free export feedback.
- T3: `cargo test --manifest-path src-tauri/Cargo.toml` passed; `npm test` passed (360 tests); `npm run typecheck:tests` passed; `git diff --check` and untracked-file whitespace scans passed. Final independent verification confirmed all acceptance criteria, actual PDF content/pagination/long-detail continuations, the exact W9 Reports path and save-permission guards, and scope integrity.
- T4: `cargo test --manifest-path src-tauri/Cargo.toml --test movement_ledger_export` passed (6 tests); `npm test` passed (360 tests); `npm run typecheck:tests` and `git diff --check` passed. Windows `cargo check --manifest-path src-tauri/Cargo.toml --features desktop` passed. Native interactive save-dialog behavior remains a manual runtime check, but the feature-gated desktop command compiles.

## Next Step
- Await a pull-request or merge decision. Optionally exercise the native PDF save dialog manually on Windows before release.
