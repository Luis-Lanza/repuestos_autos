# Design: Operational Inventory Control

## Technical Approach

Add a deep Inventory module across React → Tauri → Rust → SQLite. Unchanged Catalog search supplies advisory selection; an Inventory port delegates authoritative confirmation to SQLite. Checkout pricing remains separate.

## Architecture Decisions

| Choice | Rejected alternative | Rationale |
|---|---|---|
| Persistence-independent `InventoryRepository` seam | Copy Catalog's `rusqlite::Transaction`-leaking trait | Fake and SQLite adapters share a small interface; transactions remain local. |
| Absolute physical count | Signed adjustment input | Captures what was observed and lets authoritative current stock determine the delta. |
| Request ID on immutable movements | `source_reference` or UI-only deduplication | A unique persisted operation identity survives restart without overloading external references. |
| Derived alert query | Persisted alert state | Balance remains the only truth; sales and inventory changes appear immediately. |
| Forward-only v6 rebuild | Alter weak v5 checks in place | SQLite table checks require rebuilding while preserving historical facts exactly. |

## Data Flow and Request Lifecycle

```text
Catalog search → Inventory form (advisory balance) → inventory.ts → Tauri command
 → use case → InventoryRepository → BEGIN IMMEDIATE → SQLite → persisted result
                                                  ↘ derived alert query
```

The UI creates one UUID v4 at first confirmation and retains it across validation, IPC, and persistence failures. Retries reuse it; changing product, operation kind, or quantity/count starts a new intent and ID. Success displays persisted old balance, delta, result, movement/request IDs, note/reason, and database timestamp; “New operation” resets it. If old balance differs from the projection, success explicitly reports intervening stock change. Alerts refresh after success and screen entry; sales need no coupling because alerts are queried on demand.

## Domain and Interfaces

`domain/inventory.rs` owns `StockEntryQuantity` (>0), `PhysicalCount` (≥0), optional entry note, trimmed `AdjustmentReason`, `AlertClassification`, operation/result types, and stable `InventoryError`. JavaScript safe-integer checks are convenience; Serde shape checks, domain construction, transactional product/activity checks, checked arithmetic, and SQLite constraints remain authoritative.

```rust
pub trait InventoryRepository {
    fn confirm(&mut self, operation: InventoryOperation)
        -> Result<PersistedInventoryOperation, InventoryError>;
    fn list_alerts(&self) -> Result<Vec<InventoryAlert>, InventoryError>;
}
```

Use cases are generic over this trait and never import rusqlite. Three commands—`confirm_stock_entry_command`, `confirm_physical_count_command`, `list_inventory_alerts_command`—return tagged envelopes. TypeScript uses flat contracts, const-derived kinds, and allowlisted IPC payloads.

Stable codes: `invalid_request`, `invalid_quantity`, `invalid_count`, `reason_required`, `missing_product`, `inactive_product`, `unchanged_count`, `quantity_overflow`, `persisted_data_invalid`, and `persistence_failure`; database details never cross IPC.

## Transaction and Schema

The adapter starts `BEGIN IMMEDIATE` and returns any existing request-ID result before examining changed retry payload. Otherwise it loads active product/balance, derives delta/result, reserves by inserting the movement, guardedly updates balance, reads persisted facts, and commits. Failure rolls back; a uniqueness race reloads the winner.

`0006_operational_inventory_control.sql` rebuilds `inventory_movements`, adding nullable `request_id`, `counted_quantity`, and `resulting_quantity`; a unique partial request-ID index; and a composite FK `(sale_line_id, sale_id, product_id)` backed by a unique `sale_lines(id, sale_id, product_id)` index. Checks enforce: positive/unlinked opening and entry; negative/linked sale; positive/linked return; nonzero/unlinked adjustment with reason; positive/linked cancellation with reason; request/result facts for entry/adjustment; counted=result for adjustment. Timestamps keep `DEFAULT CURRENT_TIMESTAMP`; immutable triggers are recreated.

Before DDL, v6 validates columns, foreign keys, all row rules, and sale-line/sale/product consistency. It copies IDs, timestamps, and legacy fields verbatim, runs `foreign_key_check`, then advances `user_version`. Failure rolls back. Rollout requires a backup; rollback restores it with the previous binary—no down migration.

## File Map

| Action | Paths |
|---|---|
| Create | `src-tauri/src/domain/inventory.rs`; `application/inventory/{mod.rs,repository.rs}`; `infrastructure/sqlite/inventory_repository.rs`; `commands/inventory.rs`; migration 0006; `src/commands/inventory.ts`; `src/ui/inventory/{inventory-flow.ts,inventory-screen.ts}`; focused tests |
| Modify | Rust module indexes, `src-tauri/src/lib.rs`, SQLite migrator/tests, `src/ui/app.ts` |
| Preserve | `src/commands/catalog.ts`, formatting-only Sales diffs, unrelated untracked paths |

## Testing, Operations, and Performance

Unit tests cover values, overflow, no-op, classification, reducer/request retention, stale projection, payload allowlisting, and errors. Fake-port tests prove orchestration. SQLite/command tests prove atomicity, changed-payload retries, intervening writes, alert ordering/filtering, constraints, rollback, restart, and v5→v6 preservation/rejection. Apply/verify run existing npm build/tests and both Rust suites.

Alerts join active products/balances at `quantity <= 1`, ordered by quantity, normalized name, then ID; count is the same result length. Verify `EXPLAIN QUERY PLAN` and <100 ms local reads. Request ID plus immutable facts provide recovery evidence; diagnostics retain internal context while IPC stays opaque.

## Threat Matrix

| Boundary | Applicability |
|---|---|
| Documentation-like paths | N/A — no executable classification |
| Git repository selection | N/A — no VCS execution |
| Commit state | N/A — no commit automation |
| Push state | N/A — no push automation |
| PR commands | N/A — no PR automation |

## Delivery and Risks

This vertical will exceed 400 changed lines. With `ask-on-risk`, tasks require a decision before apply and slice migration proofs, Rust module, IPC, and UI into verified chained work units. Primary risks are migration mistakes, retry corruption, and stale-balance confusion; preflight/restart tests and persisted confirmation mitigate them. No blocking questions remain.
