# Dashboard Realized Gross Profit

## Objective
Show realized gross profit for **Today** and **This Month** on the Dashboard without rewriting historical cost facts.

## Problem
Dashboard currently reports effective sales, persisted revenue, net units out, and cancelled sales. Product purchase cost is mutable and sale lines do not preserve the cost that applied at confirmation, so historical gross profit cannot be calculated reliably.

## Scope
- Capture an immutable, nullable unit-cost snapshot for every newly confirmed sale line.
- Keep legacy sale lines unknown; never backfill from current product purchase prices.
- Report a signed realized gross-profit metric for Today and This Month only when every effective sale line in that period has a known cost snapshot.
- Exclude cancelled sales and reverse profit for returned units.
- Render `Ganancia bruta` or `No disponible` in both Dashboard metric groups.
- Amend the Dashboard design/report contract and focused tests for the new fact.

## Constraints
- Gross profit is `(confirmed sale unit price - unit-cost snapshot) * net units`; losses are valid signed centavo values.
- Product cost changes after confirmation must not affect historical profit.
- Existing Dashboard persisted revenue semantics remain unchanged: returns reduce net units and profit, not `effective_total_centavos`.
- Use checked integer arithmetic; neither overflow nor missing cost may become a numeric profit.
- Preserve the atomic Dashboard snapshot: both periods and all existing sections load or fail together.
- Use a forward-only SQLite migration and validate compatibility on upgrades.
- TDD mode: standard; source: project configuration (`openspec/config.yaml`, strict TDD disabled).
- Rust runner: `cargo test --manifest-path src-tauri/Cargo.toml`.
- Delivery strategy: ask-on-risk. Forecast: approximately 300 authored changed lines, excluding generated files.

## Tasks
- [x] T1 Add immutable nullable sale-line cost snapshots. Route: delegated direct; trigger: multi-file write. Checks: migration upgrade/schema validation, transactional confirmation snapshot, cost-change immutability, nullable legacy records, checked arithmetic.
- [x] T2 Add realized-profit reporting and typed dashboard contract. Route: delegated direct; trigger: multi-file write. Checks: known/unknown costs, cancelled sales, returns, losses, signed runtime decoding, atomic report behavior.
- [x] T3 Render and document the Dashboard metric. Route: delegated direct; trigger: multi-file write. Checks: both period panels, known/unknown states, accessible labels, design-contract alignment, mounted UI coverage.
- [x] T3a Repair the W9 protected-tree audit for the authorized gross-profit paths. Route: delegated direct; trigger: multi-file write. Checks: audit allowlist remains exact, focused audit passes on the final intended tree, no unrelated path is permitted.
- [ ] T3b Include untracked paths in the W9 live protected-tree scan. Route: delegated direct; trigger: multi-file write. Checks: live audit rejects an unrelated untracked protected path and permits only the intended untracked migration/test paths; user authorized baseline exclusion for the pre-existing unrelated ODD task documents.
- [x] T4a Format candidate-added Rust paths. Route: delegated direct; trigger: multi-file write. Checks: only candidate formatting changes, no baseline-debt cleanup, focused formatter evidence.
- [x] T4 Independently verify the completed work unit. Route: delegated direct; trigger: verification. Checks: focused Rust/frontend suites, TypeScript test typecheck, formatting/diff evidence, and independent scope review.

## Acceptance Criteria
- Every newly confirmed sale line preserves the purchase cost that applied at confirmation, without changing after product-cost edits.
- Dashboard Today and This Month display signed gross profit only when all included effective sale lines have known snapshots.
- Any included effective legacy line without a snapshot yields `No disponible`, never a guessed or partial result.
- Cancellations contribute neither revenue metrics nor profit; returns reduce net units and gross profit.
- A loss is represented accurately as a negative centavo amount.
- Existing Dashboard state, scope, atomic load behavior, and non-profit facts remain intact.

## Progress
- 2026-09-28: User authorized implementation after selecting unavailable (not estimated or partial) profit for periods containing historical lines without cost snapshots, and profit reversal for returned units.
- Branch: `feat/dashboard-realized-gross-profit` from `master` at `dd65863`.
- T1 work-unit commit: `675da30 feat(sales): snapshot cost at confirmation`.
- T1 completed by a delegated writer: migration v21 adds a nullable sale-line unit-cost snapshot, confirmation captures current product cost inside the transaction, and confirmed-line immutability includes the snapshot. No Dashboard reporting/UI changes were made.
- The writer corrected one malformed source line found by its first migration-test attempt; the rerun passed. Independent verification found no blocking issue.
- T2 completed by a delegated writer: Dashboard metrics now serialize gross profit as `{"status":"known","amount_centavos":<signed>}` or `{"status":"unavailable"}`. The report excludes cancelled sales, uses returned-unit-adjusted net quantities, treats empty periods as known zero, and rejects per-line arithmetic overflow. Independent verification found no blocking issue.
- T3 completed: both Dashboard metric panels render signed known profit or `No disponible`, and the Dashboard handoff now permits only realized gross profit. T3a admitted the exact authorized paths to W9 and added unrelated-neighbor rejection coverage; no production change was made by the repair.
- T4 functional checks all passed, but its independent verifier found a medium verification gap: W9's live path scan uses `git diff --name-only HEAD` and omits untracked files. T3b is required before this work unit can close.

## Evidence
- Read-only map: sale prices are immutable snapshots, but `sale_lines` have no purchase-cost snapshot; `products.purchase_price_centavos` is mutable and unsuitable for historical reporting.
- Current Dashboard contract: `src-tauri/src/application/reporting/mod.rs`, `src-tauri/src/infrastructure/sqlite/dashboard_repository.rs`, `src/commands/dashboard.ts`, and `src/ui/dashboard/dashboard-screen.ts`.
- Existing Dashboard design handoff prohibits profit and must be revised within this task.
- Writer verification: `cargo test --manifest-path src-tauri/Cargo.toml --test sqlite_migrations` passed (27); `cargo test --manifest-path src-tauri/Cargo.toml --test sale_cost_snapshot` passed (3); `git diff --check` passed.
- Native assessment was unavailable (empty native output), so the current candidate was treated as high risk and independently verified.
- Independent verification: migration/legacy-null behavior, both confirmation paths, transaction-time capture, and post-confirmation immutability passed. Residual low-risk gap: startup validation asserts the capture trigger exists but does not compare its definition; do not broaden this feature for that follow-up without separate authorization.
- T2 writer verification: `cargo test --manifest-path src-tauri/Cargo.toml --test dashboard_reporting` passed (7); `./node_modules/.bin/tsx --test src/commands/dashboard.test.ts` passed (4); `npm run typecheck:tests` passed; `git diff --check` passed. Native assessment was again unavailable, so the candidate was independently verified with no findings. The T1-focused suites were not rerun by that verifier, and `git diff --check` excludes untracked files; T4 must cover the full final tree.
- T3 writer verification: Dashboard mounted test passed (5/5), `npm run typecheck:tests` passed, and `git diff --check` passed. The original `npm test` W9 failure was corrected by T3a: focused audit passed (13/13), then `npm test` passed (338/338), and `git diff --check` passed. Expected React `act(...)` warnings and dialog-validation logs remain non-failing test output.
- T4 verification: all focused Rust/frontend checks and the full frontend suite passed; W9 now audits untracked candidate paths while excluding only the six user-authorized baseline task documents. The gate remains incomplete because `cargo fmt --check` found candidate-added formatting in four Rust paths; T4a must format only those paths. A separate low-risk trigger-definition validation gap remains out of scope.

## Next Step
- Continue T3b: distinguish the exact pre-existing ODD baseline documents from this candidate, preserve untracked scanning for every other path, then rerun T4.
