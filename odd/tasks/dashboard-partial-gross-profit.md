# Dashboard Partial Gross Profit

## Objective
Show operationally useful gross profit for Today and This Month even when some effective sale lines lack historical cost snapshots.

## Decision
Display the signed sum from known-cost effective sale lines and a clear incomplete-data indicator. The indicator counts effective sale lines missing a cost snapshot, including lines fully returned. Unknown-cost lines contribute neither gain nor loss; they are never estimated from current product cost.

## Scope
- Replace known/unavailable profit with a typed amount plus missing-cost-line count.
- Preserve cancellation exclusion, return-adjusted known-line profit, signed losses, empty-period zero, checked arithmetic, and atomic report loading.
- Render complete or partial gross profit in both Dashboard metric panels.
- Update the Dashboard design handoff and focused tests.

## Tasks
- [x] T1 Change reporting and IPC contract to partial-profit metrics. Route: delegated direct. Checks: known sum, line count, cancellations, returns, losses, empty periods, overflow, decoder validation.
- [x] T2 Render complete/partial profit and revise design contract. Route: delegated direct. Checks: both periods, exact Spanish warning, accessible semantics, mounted tests, no state-machine regression.
- [x] T3a Admit the current feature task document to W9's exact allowlist. Route: delegated direct. Checks: current task permitted, unrelated untracked paths still rejected.
- [x] T3 Independently verify final tree. Route: delegated direct. Checks: focused Rust/frontend suites, typecheck, W9, diff and untracked integrity.

## Acceptance Criteria
- A period with missing-cost lines shows known-line gross profit and `Faltan costos en N líneas de venta.`
- A period without missing-cost lines shows complete gross profit without that warning.
- Unknown costs never become estimated values; cancellations do not count; returned unknown-cost lines still count.

## Progress
- 2026-09-28: User authorized the change after finding a test sale with one product missing a purchase price. User chose counting fully returned unknown-cost lines as incomplete.

## Evidence
- Final verification: Rust reporting 7/7; focused frontend/W9 24/24; full frontend 340/340; typecheck and diff checks passed. W9 audits the current untracked task and rejects arbitrary untracked paths.

## Next Step
- Await explicit commit/push authorization.