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

## Evidence
- T1: `cargo test --manifest-path src-tauri/Cargo.toml --test movement_ledger --test movement_ledger_commands` passed (5 tests); `git diff --check` passed. Independent re-verification confirmed strict filter validation, archived-product inclusion, stable ordering/paging, nullable recorded balance, exact UTC half-open semantics, and rejection of fractional-second bounds. Desktop-feature compilation remains unavailable in this Linux environment because GTK/GIO development libraries are missing.
- T2a: `cargo test --manifest-path src-tauri/Cargo.toml --test movement_ledger_export` passed (5 tests); `git diff --check` and untracked-file whitespace checks passed. Independent re-verification confirmed the 2,000-row non-truncating cap, active-filter export, metadata, paginated readable layout, complete long-detail continuations, strict request validation, and generic cancellation/write-failure feedback. Native dialog/filesystem-save runtime remains unavailable for verification in this GTK/GIO-limited Linux environment.
- T2b: `./node_modules/.bin/tsx --test --import ./test/react-dom.ts src/commands/movement-ledger.test.ts src/ui/reports/movement-ledger-flow.test.ts src/ui/reports/movement-ledger-screen.mounted.test.ts src/ui/app-shell.mounted.test.ts` passed (18 tests, no React `act(...)` warnings); `npm run typecheck:tests` passed; `cargo test --manifest-path src-tauri/Cargo.toml --test movement_ledger --test movement_ledger_commands` passed (6 tests); `git diff --check` and untracked-file whitespace checks passed. Independent re-verification confirmed Reports navigation/focus, current-month filters, archived-product selection, truthful read-only facts, stale-response protection, filter retention through paging/retry, accessible states, and path-free export feedback.
- T3: `cargo test --manifest-path src-tauri/Cargo.toml` passed; `npm test` passed (360 tests); `npm run typecheck:tests` passed; `git diff --check` and untracked-file whitespace scans passed. Final independent verification confirmed all acceptance criteria, actual PDF content/pagination/long-detail continuations, the exact W9 Reports path and save-permission guards, and scope integrity. The desktop-feature check could not compile because this Linux host lacks GLib/GIO/GDK/ATK development libraries; native save-dialog/filesystem runtime remains unverified.

## Next Step
- Await explicit authorization to create a reviewable work-unit commit. Before delivery, validate native save behavior on a desktop-capable host with the required runtime/development libraries.
