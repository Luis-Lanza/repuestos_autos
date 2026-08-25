# Fixed-price checkout with backend-derived payment amounts

## Intent

Make sale confirmation authoritative and predictable: the backend resolves each product's current catalog price at confirmation, stores that value as the sale-time historical snapshot, and derives cash applied and change from the sale total, QR amount, and operator-entered tendered cash.

This removes price and cash-allocation authority from the cashier-facing UI while preserving cash-only, QR-only, and mixed-payment checkout. Future catalog-price updates affect only new sales; confirmed sales retain their original sale-time prices.

## Scope

### In scope

- Accept only product identity and positive whole-unit quantity for each checkout line; do not accept an operator-supplied sale-line price.
- Display the current catalog price in the draft for guidance, while treating the backend price resolved at confirmation as authoritative.
- Resolve catalog prices inside backend sale confirmation and persist each resolved price as the historical sale-line snapshot.
- Accept operator-entered tendered cash and an optional QR-applied amount.
- Derive cash applied and change in the backend after accounting for QR-applied value.
- Support cash-only, QR-only, and mixed payments.
- Reject a QR amount greater than the authoritative sale total.
- Reject insufficient tendered cash for the remaining amount after QR.
- Preserve atomic confirmation, stock integrity, immutable inventory movements, request-ID idempotency, and persisted sale summaries.
- Return persisted summaries containing the authoritative line prices and payment breakdown, including applied amounts and cash tendered/change where applicable.
- Define an explicit SQLite migration and backward-compatibility approach for existing sale and payment data during design.

### Out of scope

- Negotiated, discounted, or operator-editable sale-line prices.
- Retroactive repricing of confirmed sales after catalog updates.
- Product-management workflows for changing catalog prices.
- QR gateway or payment-terminal integration.
- Invoicing, accounting, credit, refunds, returns, or cancellation behavior beyond preserving compatibility with existing confirmed-sale records.
- Licensing, authentication, multi-store, or synchronization capabilities.

## Product and domain decisions

| Topic | Decision |
| --- | --- |
| Catalog price | A product has one current fixed catalog price. The backend resolves it when confirmation executes. |
| Historical price | Each confirmed sale line stores its resolved catalog price. Later catalog changes do not alter old sales. |
| Draft price | The UI may display a current catalog price, but it is informational and may be superseded by the authoritative value resolved at confirmation. |
| Cash input | The cashier enters tendered cash, not cash applied or change. |
| Mixed payment | QR applied is considered first; cash applied is the remaining total, and change is tendered cash minus that remainder. |
| Invalid payment | QR above the total or tendered cash below the remaining amount prevents confirmation. |
| Retry behavior | Reusing the same request ID returns the already-persisted sale and must not reprice, duplicate payment records, or deduct stock again. |

## Affected areas

| Area | Required change |
| --- | --- |
| React draft and checkout screen | Remove editable line-price and derived cash fields from cashier input; retain product, quantity, tendered cash, and QR input with clear persisted-summary display. |
| TypeScript command seam | Send product/quantity lines and payment inputs without client-authoritative prices, cash-applied values, or change. |
| Rust command adapter | Map the reduced confirmation request and expose authoritative persisted summary values. |
| Rust application and domain | Resolve catalog prices, calculate the total, validate QR, derive cash applied/change, and retain transaction and idempotency authority. |
| SQLite repository and schema | Persist sale-time line-price snapshots and complete payment facts while retaining request-ID uniqueness and transactional stock updates. |
| Database migration | Design must explicitly choose how existing databases and historical rows are migrated or interpreted, including defaults/nullability, constraint rollout, and rollback compatibility. No migration behavior may be assumed silently. |
| Tests | Cover authoritative repricing at confirmation, historical price retention, all payment modes, invalid QR, insufficient tender, idempotent retries, stock rollback, atomic persistence, and summary readback. |

## Design requirements

- Preserve the dependency direction: React → Tauri command seam → Rust application/domain → SQLite adapters.
- Keep catalog-price and payment-calculation rules in Rust rather than duplicating authority in React or TypeScript.
- Keep `ConfirmSaleUseCase` responsible for the complete transaction.
- Represent money as integer centavos of Bs through a domain money type; do not introduce floating-point monetary calculations.
- Preserve the unique `request_id` behavior so retries return the original persisted values rather than recomputing against a newer catalog price.
- Treat the SQLite migration and backward-compatibility policy as an explicit design decision. The design must identify the existing schema/data shapes, migration sequence, handling of legacy rows, constraint compatibility, and rollback behavior before implementation tasks are approved.

## Risks and mitigations

| Risk | Mitigation direction |
| --- | --- |
| A draft displays a price that changes before confirmation | Make the confirmation response authoritative and refresh the UI from the persisted summary. |
| A retry after a catalog update is accidentally repriced | Resolve idempotency before creating or recalculating sale details; return the existing persisted sale. |
| Client and backend payment calculations diverge | Keep the backend as the only authority for cash applied and change; client calculations are display-only if retained. |
| Mixed-payment edge cases create overpayment or underpayment | Validate QR against total, derive only the remaining cash amount, and reject insufficient tender. |
| Schema changes damage or misrepresent historical records | Require a documented migration/backward-compatibility decision and migration-focused verification before rollout. |
| Partial persistence corrupts stock or summaries | Keep lines, payments, stock balances, movements, and sale summary state in one application-owned SQLite transaction. |
| Contract changes break callers or fixtures | Update the React, TypeScript, Tauri, Rust, repository, and test contracts as one vertical slice. |

## Rollback

Rollback must restore the previous application contract and its compatible database schema without deleting or repricing confirmed-sale history. Before implementation, design must establish whether the schema migration is reversible directly or requires a forward-compatible rollback that leaves newly added columns/data in place while the prior application ignores them. Any rollback must preserve sale lines, payment records, request IDs, stock movements, and stock balances.

If runtime behavior must be disabled before a full rollback, checkout confirmation should be blocked rather than falling back to client-authoritative prices or payment amounts.

## Success criteria

- A cashier can confirm a sale by selecting products, entering positive whole-unit quantities, and providing only the payment inputs relevant to cash and/or QR.
- The backend uses the current catalog price at confirmation and stores it on each sale line.
- Updating a catalog price affects later confirmations but does not change any persisted sale summary or historical line price.
- Cash-only, QR-only, and mixed sales persist applied amounts that exactly equal the authoritative sale total.
- Cash confirmation persists tendered cash and backend-derived change; insufficient tender is rejected.
- A QR amount greater than the sale total is rejected without persisting the sale or changing stock.
- Repeating confirmation with the same request ID returns the original persisted summary without duplicate sale data or additional stock deduction.
- Any validation or persistence failure leaves sale data, payments, stock balances, and inventory movements unchanged.
- The persisted summary returned to the UI reflects the authoritative historical prices and payment values stored in SQLite.
- The approved design documents and verifies a concrete migration/backward-compatibility policy for existing databases and historical records.
