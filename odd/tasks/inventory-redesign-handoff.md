# Define Inventory redesign handoff

## Goal

Create a repository-backed designer handoff for improving Inventory through the evidence-first mockup workflow used for Sales and Dashboard.

## Direction

Use an operation-first hierarchy:

1. Product search, selection, and stock operation are the dominant workflow.
2. The read-only stock-alert panel remains visible as operational context; current reset behavior may misleadingly empty its rows until another alert load.
3. Preserve the single browser → selected product → operation flow.
4. Keep confirmation feedback minimal and alerts non-actionable.

## Constraints

- Design and documentation only; do not change production code.
- Preserve product browsing, filters, pagination, request identity, idempotency, pending locks, stale-response guards, alert refresh, retry, and navigation behavior.
- Keep the existing 961px breakpoint and 208px/176px persistent sidebar.
- Do not invent detail/history routes, alert actions, extra persisted facts, configurable thresholds, bulk operations, suppliers, costs, lots, barcodes, transfers, refresh/polling, or dialogs.
- Prototype dependencies and scripts must never enter production.

## Tasks

- [x] Write the Inventory Figma handoff and minimal frame requirements.
- [x] Verify data authority, interaction states, responsive behavior, accessibility, and designer boundaries.

## Evidence

- Existing Inventory screen, flow, commands, shared ProductBrowser, tests, and Rust/SQLite contracts.
- Existing shell and visual rules in `src/ui/styles.css`.
- Established handoff pattern in `docs/design/dashboard-figma-handoff.md`.
- Independent read-only verification passed after correcting sidebar cue navigation, browser retention on reset, independent cue/panel refresh, stale results during loading, and misleading alert-panel empty state after reset. Untracked handoff whitespace check passed; no tests were run for this documentation-only change.
- Commit: pending explicit user authorization.
