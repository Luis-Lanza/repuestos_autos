## Exploration: Sales History

### Current State
Sale confirmation is implemented end to end: React invokes `confirm_sale_command`, the Rust use case owns the transaction, and SQLite persists `sales`, `sale_lines`, `sale_payments`, and historical SKU/name/price snapshots. Confirmation can reload one sale only by `request_id`; there is no history navigation, date-range list command, sale-by-ID detail query, or frontend history adapter/screen. `confirmed_at` is stored as SQLite `CURRENT_TIMESTAMP` text, and the schema has no index for chronological filtering.

### Affected Areas
- `src-tauri/src/application/sales/` — add read models and a narrow history-query interface without expanding the confirmation interface.
- `src-tauri/src/infrastructure/sqlite/sale_repository.rs` — query persisted sale summaries by date and assemble one sale's lines and payments with deterministic ordering.
- `src-tauri/src/infrastructure/sqlite/migrations/` — add a forward-only index supporting confirmed-date ordering and filtering.
- `src-tauri/src/commands/` and `src-tauri/src/lib.rs` — expose and register read-only list/detail Tauri commands through `DatabaseState::with_read`.
- `src/commands/` and `src/ui/sales/` — add the TypeScript adapter, history flow, list/filter UI, and detail view.
- `src/ui/app.ts` — add navigation between the existing Sales screen and the Sales History screen.
- `src-tauri/tests/` and `src/ui/sales/*.test.ts` — prove filtering boundaries, ordering, historical snapshots, payments, empty/error states, navigation, and read-only behavior.

### Approaches
1. **Summary list plus on-demand detail** — return lightweight sale rows for a date interval, then load lines and payments for the selected sale ID.
   - Pros: Keeps the list query bounded, avoids repeating sale/payment rows, gives list and detail distinct deep interfaces, and scales better as history grows.
   - Cons: Selecting a sale requires a second local IPC/database query and separate loading/error state.
   - Effort: Medium

2. **Fully expanded history list** — return every matching sale with all lines and payments in one command.
   - Pros: One IPC call and simpler client-side selection after loading.
   - Cons: Unbounded payload growth, duplicated parent rows in SQL assembly, slower date-range browsing, and unnecessary detail work for unseen sales.
   - Effort: Medium

### Recommendation
Use a summary-list command plus an on-demand detail command inside the existing Sales vertical. Introduce read-specific application models/interface rather than widening `ConfirmSaleRepository`; implement parameterized SQLite queries ordered by `confirmed_at DESC, id DESC`, reuse the persisted snapshot columns, and keep commands on the read connection path. Add a migration index for the same filter/order shape. Define date filters as validated half-open timestamp bounds (`from` inclusive, `to` exclusive) so calendar-day conversion is explicit and boundary-safe.

### Risks
- `confirmed_at` is UTC-like text without an explicit offset; the proposal/spec must define how local calendar dates become UTC query bounds to avoid excluding edge-of-day sales.
- Existing legacy and current confirmation types coexist under `application/sales`; history must use the current persisted snapshot semantics and avoid deepening that duplication.
- Large or open-ended ranges can still produce large lists; the proposal should require a bounded result or pagination without adding analytics scope.
- Historical rows must remain readable after products are renamed or archived, using snapshots rather than current catalog labels.

### Ready for Proposal
Yes. The direction and non-goals are sufficiently fixed. The proposal should preserve read-only history, select the summary-plus-detail approach, specify date-boundary semantics and result bounding, and exclude returns, cancellations, analytics, and historical mutations.
