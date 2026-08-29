# Proposal: Sales History

## Intent

Let the store operator browse persisted sales and inspect one sale without changing historical records. The history must remain useful after catalog edits by showing sale-time line snapshots and persisted payment facts.

## Scope

### In Scope
- List bounded sale summaries in reverse chronological order.
- Filter by operator-facing calendar dates mapped to explicit half-open timestamp bounds.
- Load one sale's lines, total, and payments on demand.
- Provide navigation, loading, empty, and error states in the existing Sales vertical.

### Out of Scope
- Returns, cancellations, or any historical mutation.
- Aggregate reports, analytics, exports, or dashboards.
- Catalog repricing or relabeling of historical lines.

## Capabilities

### New Capabilities
- `sales-history`: Read-only, bounded sale browsing with date filtering and on-demand persisted detail.

### Modified Capabilities
- None.

## Approach

Add separate read-specific application interfaces for a lightweight summary list and an on-demand detail. Expose them through read-only Tauri commands backed by parameterized SQLite queries. Derive calendar-date boundaries explicitly, order summaries deterministically, and read stored line snapshots rather than current catalog data. Add an index aligned with chronological filtering; pagination or a hard limit remains a design choice.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/application/sales/` | Modified | Read models and narrow query interfaces |
| `src-tauri/src/infrastructure/sqlite/` | Modified | Summary/detail queries and supporting index migration |
| `src-tauri/src/commands/`, `src-tauri/src/lib.rs` | Modified | Read-only IPC commands and registration |
| `src/commands/`, `src/ui/sales/`, `src/ui/app.ts` | Modified | Adapter, navigation, list/filter, and detail UI |
| `src-tauri/tests/`, `src/ui/sales/*.test.ts` | Modified | Boundary, ordering, snapshot, payment, and state coverage |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Local calendar dates exclude edge sales | Medium | Specify and test half-open timestamp conversion |
| Large ranges create excessive work | Medium | Enforce bounded results and deterministic ordering |
| Catalog changes corrupt displayed history | Low | Query persisted snapshot columns only |

## Rollback Plan

Remove the history UI, commands, and read interfaces. The additive index may remain harmlessly or be removed by a forward migration; no persisted sale data is rewritten.

## Dependencies

- Existing persisted `sales`, `sale_lines`, and `sale_payments` records and snapshot fields.
- Existing React-to-Tauri-to-Rust-to-SQLite dependency direction.

## Success Criteria

- [ ] Operators can browse a bounded, newest-first sale list for selected calendar dates.
- [ ] Selecting a sale shows persisted lines, total, and payments without catalog reinterpretation.
- [ ] Empty, loading, and failure states are observable and no history action mutates data.
