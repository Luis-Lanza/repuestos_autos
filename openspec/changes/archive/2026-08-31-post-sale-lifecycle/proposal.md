# Proposal: Post-sale lifecycle

## Intent

Add the first post-sale correction lifecycle to the offline point-of-sale application: multi-line item returns and whole-sale cancellation. Corrections must preserve the original sale, sale-line snapshots, totals, and payment records as immutable historical facts while restoring stock through additive, auditable records and inventory movements.

This change closes the operational gap between confirmed sales and Sales History without introducing refunds, accounting settlement, reports, or exchange workflows.

## Problem

Confirmed sales are currently final from the application's perspective. Operators can inspect them in Sales History, but cannot record returned items or cancel an entire sale. Although the database already recognizes positive `return` and `cancellation` inventory movement types, it does not model correction operations, enforce remaining quantities across records, or expose application and UI flows for them.

Without an explicit lifecycle, stock cannot be corrected safely after a sale, retries could restore units more than once, and cancelled sales could disappear from history because current readers only include confirmed records.

## Proposed scope

### Return operation

- Start a return from a selected Sales History detail.
- Accept one or more original sale lines in one request, each with a positive whole-unit quantity.
- Permit partial or full quantities up to each line's remaining returnable quantity.
- Treat the complete multi-line return as one atomic operation: all return records, stock balance increases, and immutable positive movements commit together or none do.
- Link every returned quantity to its exact original sale line, preserving correct behavior when a product appears on multiple lines.
- Do not require or store a return reason in v1.

### Whole-sale cancellation

- Allow a confirmed sale to be cancelled once, including after accepted returns.
- Require a non-blank cancellation reason.
- Restore only residual units not already returned, calculated atomically from persisted sale and correction facts.
- Allow a fully returned sale to be cancelled as a status and audit event with zero stock change.
- Preserve the original lines, snapshots, totals, and payments unchanged.

### Idempotency and concurrency

- Require a stable request ID for each return or cancellation intent.
- Replay the persisted outcome when the same request ID is submitted with the same canonical payload.
- Return a stable conflict and write nothing when the same request ID is reused with a different payload.
- Evaluate eligibility, prior returns, cancellation state, and restorable quantities inside the same SQLite write transaction so overlapping commands cannot over-return or restore a sold unit twice.

### Sales History

- Keep cancelled sales visible in list and detail views.
- Show correction status and full accepted correction detail, including returned quantities and cancellation-restored quantities.
- Continue showing the original sale-time line snapshots and original payment facts without mutation.
- Ensure the UI describes inventory corrections only and does not imply a refund, reimbursement, payment reversal, credit, or settlement.

### Persistence and application behavior

- Add additive cancellation and return records, including return lines and request identity needed for deterministic replay.
- Extend sale lifecycle state only as required to distinguish confirmed and cancelled sales.
- Produce exactly one immutable positive inventory movement for each correction line with a positive restored quantity, linked to the original sale and sale line.
- Increase `stock_balances` by exactly the accepted or residual quantity in the same transaction as the correction records and movements.
- Keep business-rule authority and transaction orchestration in Rust; SQLite constraints remain a second line of defense.
- Expose typed Tauri command contracts and thin TypeScript adapters; React remains a presentation adapter and does not calculate eligibility or restorable quantities.

## Core invariants

1. **Original facts remain immutable:** original sale-line quantities, product snapshots, prices, totals, and payment rows are never rewritten or deleted.
2. **No monetary implication:** payments remain informational facts. This slice records no refund or payment reversal, and user-facing language must not imply reimbursement.
3. **No double restoration:** for each original sale line, cumulative returned quantity plus cancellation-restored quantity never exceeds the original sold quantity.
4. **Atomic stock integrity:** every accepted positive correction quantity updates the balance and appends its immutable movement in the same transaction; failures roll back all effects.
5. **Line identity:** correction eligibility is based on the original sale-line identity, not product identity alone.
6. **Deterministic replay:** request identity and payload identity jointly distinguish valid replay from conflicting reuse.
7. **Historical visibility:** cancellation changes lifecycle status but never removes the sale or its correction history from Sales History.

## Affected areas

- SQLite schema migrations, constraints, indexes, and backup/restore schema compatibility.
- Rust sales domain types and lifecycle validation.
- Rust application use cases and SQLite persistence adapters for returns and cancellation.
- Inventory balance and immutable movement persistence.
- Tauri command request, response, registration, and stable error contracts.
- TypeScript command adapters and Sales History list/detail presentation.
- Domain, transaction, command-seam, and UI tests covering lifecycle behavior.

## Out of scope

- Refunds, payment reversals, payment gateway operations, cash-drawer settlement, or mixed-payment refund allocation.
- Reports, analytics, exports, or changes to report totals.
- Exchanges, replacement sales, credits, customer accounts, restocking fees, or damaged-item disposition.
- Actor, operator, or role attribution.
- Return reasons.
- Partial cancellation as a separate operation; partial correction is represented by returns.
- Editing or deleting original sale lines, snapshots, totals, or payment records.
- Multi-store or cloud synchronization, fractional quantities, catalog changes, or new backup behavior beyond schema compatibility.

## Risks and mitigations

### Double stock restoration

Concurrent or repeated corrections could otherwise return more units than were sold. Mitigate by validating aggregate prior corrections and cancellation state inside one application-owned SQLite transaction, backed by database uniqueness and integrity constraints.

### Ambiguous idempotency

A request ID alone cannot safely distinguish a legitimate retry from accidental reuse. Persist enough canonical request identity to compare payloads, replay exact matches, and reject mismatches without writes.

### History regressions

Expanding sale status can cause cancelled sales to disappear from existing readers or weaken snapshot guarantees. Update list/detail projections deliberately and retain immutable original facts alongside additive correction detail.

### Accidental accounting semantics

Showing original payments beside corrections may be misread as evidence of reimbursement. Use explicit inventory-correction language and avoid refund amounts, reversal states, or settlement actions.

### Migration and backup compatibility

New tables, status rules, and indexes must upgrade existing schema-v9 databases without data loss and remain valid under backup/restore schema checks. Use forward migrations and compatibility tests with existing confirmed sales.

### Review workload

The change crosses persistence, Rust application/domain logic, Tauri, TypeScript, React, and tests and is likely to exceed the 400 changed-line review budget. Task planning must forecast review workload and propose bounded vertical slices; the configured `ask-on-risk` delivery strategy requires a decision before apply if the forecast exceeds the budget.

## Rollback

- Before release, roll back application changes and the unreleased migration together, restoring the prior read-only Sales History behavior.
- After any migrated database has accepted corrections, do not delete correction records or reverse immutable movements automatically. Roll back by disabling the new commands and UI while keeping the additive schema and historical data readable.
- Any later compensating stock action must use the existing explicit inventory correction model rather than mutating or deleting accepted lifecycle facts.
- Backup and restore validation must continue to preserve databases containing the additive lifecycle records even if the feature UI is temporarily disabled.

## Success criteria

- An operator can create one atomic return containing partial quantities from multiple original sale lines from Sales History detail.
- Returns reject zero, negative, fractional, unknown-line, and over-return quantities without partial writes.
- A confirmed sale can be cancelled with a non-blank reason before or after returns; only residual units are restored.
- A fully returned sale can be cancelled with zero stock change while retaining a visible cancellation audit event.
- Every accepted positive correction quantity produces the exact matching stock balance increase and one immutable positive movement linked to the original sale line.
- Original sale lines, snapshots, totals, and payment rows remain byte-for-byte logically unchanged after returns and cancellation.
- Same-ID/same-payload retries return the persisted result without duplicate records or stock changes; same-ID/different-payload requests return a stable conflict with no writes.
- Cancelled sales remain visible in Sales History with status, accepted returns, restored quantities, original snapshots, and original payment facts.
- No command, persistence record, or UI copy claims or implies a refund or payment reversal.
- Existing sale confirmation, inventory movement semantics, Sales History access, and backup/restore compatibility remain intact.
- Automated tests prove domain validation, transaction rollback, concurrency-safe eligibility, idempotency, immutable history, command contracts, and the focused UI flows.
