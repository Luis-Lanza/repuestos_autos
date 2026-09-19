# Sales Dashboard V1

## Objective
Provide a first, operational Dashboard screen that summarizes sales today and in the current local calendar month, surfaces current stock alerts, and shows recent sales using the existing desktop visual system.

## Product decisions
- Dashboard is the first left-sidebar option and initial screen.
- Periods: Today and current local calendar month.
- Effective sales exclude cancelled sales.
- Effective total sums persisted sale totals; returns do not reduce money because no refund amount is persisted.
- Net units out = sold units minus accepted returned units for non-cancelled sales.
- Product ranking is by units sold.
- Payment distribution is by applied amount in Bs.
- Recent sales use a fixed backend limit.

## Scope
- One read-only dashboard query/IPC contract delivering a consistent snapshot.
- Today/month operational metrics, recent sales, stock alerts, top products by units, and payment amounts.
- Dashboard as initial sidebar screen using current panels, tables, tokens, feedback, and responsive behavior.
- Navigation from stock alerts to Inventory.
- Typed Rust/TypeScript decoding and focused tests.

## Non-goals
- No schema migration, user roles, remote API, charts library, refund allocation, profit/margin metric, category ranking, arbitrary report builder, or configurable date range.
- No changes to sale, return, cancellation, inventory, or backup behavior.

## Acceptance criteria
- Dashboard is first and active on launch.
- Today and calendar-month cards show effective sale count, total Bs, net units out, and cancelled sale count.
- Top products rank by net units sold and payment distribution uses applied Bs amounts.
- Recent sales and stock alerts are bounded, accessible, independently empty-safe, and actionable.
- Loading, failure/retry, stale response, and unmounted behavior are safe.
- All values remain consistent with persisted correction state and existing visual tokens.

## TDD and delivery
- TDD mode: not configured; use focused regression checks after each task.
- Delivery strategy: ask-on-risk; forecast under 400 authored changed lines per work unit.
- Commit/push: not authorized by the user.

## Tasks
- [x] T1 Add reporting application, SQLite reader, Tauri dashboard command, and runtime-decoded frontend adapter. Route: delegated writer (multi-file Rust/IPC change).
- [x] T2 Add Dashboard navigation, data loading, panels/tables, alert navigation, and responsive style contracts. Route: delegated writer (multi-file React change).
- [x] T3 Add and run backend, adapter, screen, shell, and integration verification. Route: delegated writer plus independent verification as required. W9 now accepts the exact authorized Dashboard `src-tauri/src/lib.rs` registration seam.

## Progress
- 2026-09-19: User chose Dashboard as the first sidebar option and initial screen.
- 2026-09-19: User chose top products by units and payment distribution by applied Bs amount.
- 2026-09-19: `master` fast-forward check found it current at `2d83cba`; feature branch `feat/sales-dashboard-v1` created.
- Read-only mapping found no migration/capability requirement; a single bounded read-only reporting command is the coherent contract.
- 2026-09-19: Corrected independently verified Dashboard defects: one explicit SQLite read transaction, identity-only top-product aggregation with deterministic nullable-snapshot fallback, section-specific UI feedback, and a distinct current-stock label.
- 2026-09-19: Final verification found the W9 protected-path audit excludes required Dashboard Rust registration surfaces. User explicitly authorized a narrow W9 allowlist update for the three module-registration files and the exact `src-tauri/src/lib.rs` command wrapper/registration surface only.
- 2026-09-19: W9 Dashboard policy now allows only the exact registration files `src-tauri/src/application/mod.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/infrastructure/sqlite/mod.rs`, and the exact Dashboard command registration/test seam in `src-tauri/src/lib.rs`; Dashboard implementation, test-module, other lib, and broad directory paths remain rejected.

## Verification evidence
- `./node_modules/.bin/tsx --test --import ./test/react-dom.ts src/commands/dashboard.test.ts src/ui/dashboard/dashboard-flow.test.ts src/ui/dashboard/dashboard-screen.mounted.test.ts`: 7 passed.
- `./node_modules/.bin/tsx --test --import ./test/react-dom.ts src/commands/dashboard.test.ts src/ui/dashboard/dashboard-flow.test.ts src/ui/dashboard/dashboard-screen.mounted.test.ts src/ui/app-shell.mounted.test.ts`: 13 passed.
- `npm run typecheck:tests`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml --test dashboard_reporting`: 3 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml --no-default-features`: passed; all existing Rust tests passed, including 3 Dashboard reporting tests.
- `./node_modules/.bin/tsx --test --import ./test/react-dom.ts src/ui/w9-evidence-audit.test.ts`: 10 passed.
- `npm test`: 228 passed; the W9 protected-path audit and Dashboard regression coverage pass.
- `cargo test --manifest-path src-tauri/Cargo.toml command_surface_tests --features desktop`: unavailable because the environment lacks the required `gio-2.0`, `gobject-2.0`, and `glib-2.0` system libraries.
- Final independent verifier: diff check passed; focused Dashboard suite 13/13; W9 audit 10/10; Dashboard Rust 3/3; no-default-features Rust suites 225 passed; typecheck passed; full frontend suite 228/228. No Dashboard findings.
- Native risk assessment was unavailable (empty native output), so the change was independently verified as a high-risk fallback.

## Commit evidence
- `feat(dashboard): add sales operations overview` (work-unit commit on `feat/sales-dashboard-v1`).

## Next step
Run Windows desktop smoke validation for initial Dashboard navigation and visual geometry. Commit/push only with explicit user authorization.
