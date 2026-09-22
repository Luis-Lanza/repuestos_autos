# Define Dashboard redesign handoff

## Goal

Create a repository-backed designer handoff for improving the Dashboard through the same evidence-first mockup workflow used for Sales.

## Direction

Use an operationally balanced hierarchy:

1. Today and month sales metrics provide immediate orientation.
2. Stock alerts receive prominent operational treatment and retain the existing Inventory action.
3. Top products, payment distribution, and recent sales provide supporting context.

## Constraints

- Design and documentation only; do not change production code.
- Preserve the current atomic Dashboard request, persisted facts, list limits, loading/error/empty states, retry behavior, stale-response guards, navigation, and stock-alert action.
- Keep the existing 961px breakpoint and 208px/176px persistent sidebar.
- Do not invent filters, trends, percentages, profit, customers, operators, configurable thresholds, refresh controls, drilldowns, exports, or unsupported actions.
- Prototype dependencies and scripts must never enter production.

## Tasks

- [x] Write the Dashboard Figma handoff and frame requirements.
- [x] Verify data authority, state coverage, responsive behavior, accessibility, and designer boundaries.

## Evidence

- Existing Dashboard implementation and tests under `src/ui/dashboard/` and `src/commands/dashboard.ts`.
- Existing shell and visual rules in `src/ui/app-shell.ts` and `src/ui/styles.css`.
- Established handoff pattern in `docs/design/sales-confirmed-figma-handoff.md`.
- Independent repository-backed handoff verification: passed.
- `git diff --check`: passed.
- Commit: pending explicit user authorization.
