# Design: Sales History

## Technical Approach

Add a read-only Sales history slice following React → Tauri → Rust application → SQLite. A bounded summary query returns at most 100 confirmed sales plus `has_more`; selecting an ID performs a separate detail read. All money remains integer centavos, and detail reads persisted line/payment facts without joining catalog labels.

## Architecture Decisions

| Choice | Alternatives considered | Rationale |
|---|---|---|
| Hard cap of 100 summaries, implemented as `LIMIT 101`, discard row 101 and set `has_more` | Offset/cursor pagination | Meets the finite-bound and “more exists” requirements without adding pagination state; the UI tells operators to narrow the range. |
| Browser converts inclusive `YYYY-MM-DD` dates, using the desktop browser/system timezone, to local midnights and sends RFC3339 `from_utc` and next-day `to_exclusive_utc` | Fixed offset; database-local dates | Constructing each midnight independently respects DST. Rust parses with `time`, normalizes each bound using `to_offset(UtcOffset::UTC)`, validates `from < to`, then formats `YYYY-MM-DD HH:MM:SS` to match `CURRENT_TIMESTAMP` text. |
| Two deep read interfaces inside Sales | Expand `ConfirmSaleRepository`; one expanded query | Keeps confirmation/transaction authority unchanged and hides ordering, over-fetch, row validation, and assembly behind narrow seams. |
| Partial chronological index | No index; index all statuses | `sales(confirmed_at DESC, id DESC) WHERE status='confirmed'` matches the filter/order while excluding pending rows. |

## Data Flow

`SalesHistoryScreen → src/commands/sales-history.ts → list_sales_history_command / sale_history_detail_command → application read interface → SqliteSaleHistoryReader → SQLite`

The screen owns list/detail loading, empty, error, selection, and back states. Tauri commands use `DatabaseState::with_read`; no write connection or transaction is exposed.

## File Changes

| File | Action | Description |
|---|---|---|
| `src-tauri/Cargo.toml` | Modify | Add `time` parsing/formatting support. |
| `src-tauri/src/application/sales/history.rs` | Create | Read models, errors, and two interfaces. |
| `src-tauri/src/application/sales/mod.rs` | Modify | Declare and export the new `history.rs` module. |
| `src-tauri/src/infrastructure/sqlite/sale_history_repository.rs` | Create | Parameterized summary/detail reads and snapshot validation. |
| `src-tauri/src/infrastructure/sqlite/migrations/0009_sales_history_index.sql` | Create | Add the partial chronological index. |
| `src-tauri/src/infrastructure/sqlite/mod.rs` | Modify | Register v9 migration/adapter. |
| `src-tauri/src/commands/sales_history.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs` | Create/Modify | Map typed responses and register read commands. |
| `src/commands/sales-history.ts` | Create | Calendar conversion, IPC types, and invoke adapter. |
| `src/ui/sales/history-flow.ts`, `src/ui/sales/history-screen.ts`, `src/ui/app.ts` | Create/Modify | Reducer-driven retrieval states and navigation. |
| `src-tauri/tests/*sales_history*.rs`, `src/ui/sales/*history*.test.ts` | Create | Boundary, read, command, reducer, and navigation coverage. |

## Interfaces / Contracts

```rust
const SALES_HISTORY_LIMIT: usize = 100;
struct HistoryRange { from_utc: String, to_exclusive_utc: String }
struct SaleHistoryPage { /* private */ sales: Vec<SaleHistorySummary>, has_more: bool }
struct SaleHistorySummary { sale_id: i64, confirmed_at: String, status: String, total_centavos: i64, line_count: u32, payment_count: u32, payment_methods: Vec<PaymentMethod> }
struct SaleHistoryDetail { sale_id: i64, confirmed_at: String, status: String, total_centavos: i64, lines: Vec<HistoricalLine>, payments: Vec<HistoricalPayment> }
struct HistoricalLine { product_id: i64, sku: Option<String>, product_name: Option<String>, quantity: u32, unit_price_centavos: i64, line_total_centavos: i64 }
trait SaleHistorySummaryReader { fn list(&self, range: &HistoryRange) -> Result<SaleHistoryPage, HistoryError>; }
trait SaleHistoryDetailReader { fn detail(&self, sale_id: i64) -> Result<Option<SaleHistoryDetail>, HistoryError>; }
```

`SaleHistoryPage::from_overfetch` is the only constructor: it accepts at most the adapter's fixed 101-row fetch, exposes at most 100 rows, and derives `has_more`; callers and trait implementations cannot supply a limit or construct an oversized page. The list SQL hardcodes/binds the internal fetch size, filters `status='confirmed' AND confirmed_at >= ?1 AND confirmed_at < ?2`, and orders `confirmed_at DESC, id DESC`. Correlated counts plus cash/QR existence flags map methods in fixed Cash-then-QR order. Detail selects sale, lines ordered by `sale_lines.id`, and payments by `sale_payments.id`. It reads nullable snapshot columns directly: missing SKU/name remains `None` and the UI renders “Unavailable”; it never joins, backfills, or falls back to catalog labels. Invalid non-null quantities/money become `persisted_data_invalid`.

Command errors map to `invalid_range`, `sale_not_found`, or `persistence_failure`; state-level invoke failures remain retrieval errors. Tagged success/error payloads are mirrored in TypeScript.

## Testing Strategy

| Layer | Approach |
|---|---|
| Application | Fake each reader interface; prove validation, cap, not-found, and error mapping. |
| SQLite/migration | In-memory v9 data proves half-open edges, tie ordering, the unbreakable 100-row cap, index creation, deterministic payment methods, payment variants, and unchanged table snapshots after repeated reads. Rename catalog data after inserting legacy NULL snapshots and prove detail succeeds with both fields unavailable. |
| Command/UI | Inject invoke/calendar conversion; test desktop-timezone/DST bounds, non-zero RFC3339 offsets normalized to UTC before SQLite comparison, payloads, unavailable labels, loading/empty/error/detail/back states, navigation, and Tauri registration. |

## Threat Matrix

N/A — no routing, shell, subprocess, VCS/PR automation, executable-file classification, or process-integration boundary.

## Migration / Rollout

Forward-only v8→v9 index migration; no business rows are rewritten. Rollback removes the UI/commands/read modules; the index may remain harmlessly.

## Open Questions

None.
