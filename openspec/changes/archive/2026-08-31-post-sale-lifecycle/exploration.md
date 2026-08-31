# Exploration: Post-sale lifecycle

## Scope

Define the missing post-sale lifecycle for the single-store, offline auto-parts POS: whole-sale cancellation and item returns, while preserving immutable sale facts, payment facts, stock movements, and future reporting semantics. Sales History is a delivered read-only capability. Reports are explicitly outside this change.

## Repository and product baseline

- `docs/PRD.md` is authoritative for the target behavior. It states that returns and cancellations are planned, not implemented; reports remain planned and out of scope here.
- `docs/ARCHITECTURE.md` confirms the React -> Tauri -> Rust application/domain -> SQLite direction. Rust owns business rules and the application use case owns the complete SQLite transaction. SQLite is a second line of defense.
- No root `CONTEXT.md` or ADRs were present; `docs/agents/domain.md` confirms the single-context documentation convention. Existing terminology is therefore drawn from the PRD and architecture documents.
- The repository has delivered Sales History artifacts and an active `local-backup-and-restore` change. The archived Sales History material establishes persisted sale-time snapshots and read-only historical access; backup design explicitly preserves sales, balances, and immutable movements. Those capabilities must remain compatible.
- The configured schema is version 9. The current source has no cancellation/return application module, Tauri command, or React UI.

## Existing implementation surfaces

### Persisted sale facts

- `sales` currently permits only `pending` and `confirmed` statuses (`0001_confirm_sale.sql`). It stores `request_id`, total centavos, and confirmation timestamp.
- `sale_lines` stores product identity, quantity, sale-time SKU/name snapshots, unit price, minimum-price snapshot, and line total. The snapshot is the historical source of truth; current catalog data must not replace it.
- `sale_payments` stores applied amounts; cash additionally stores tendered and change; QR stores only applied amount. There is no refund/payment-reversal table or payment gateway integration.
- `SqliteSaleRepository` and the delivered Sales History reader load confirmed sales, persisted lines, and persisted payments. History currently lists only `status = 'confirmed'` rows and detail also requires `status = 'confirmed'`; a lifecycle change must decide how cancelled sales remain visible without weakening historical immutability.

### Stock and movement facts

- `stock_balances` is a transactionally maintained non-negative read model.
- `inventory_movements` became polymorphic in migration 0006. It already allows `return` and `cancellation` movement types with positive deltas, requires sale and sale-line links, and has immutable update/delete triggers.
- The database has a composite foreign-key link from movement `(sale_line_id, sale_id, product_id)` to the original line. A partial index on `inventory_movements.request_id` is unique when present, suitable for operation idempotency, but the schema does not yet model return identity or return lines.
- The existing movement check requires a non-empty reason for cancellation, but does not require one for returns. It does not itself enforce eligibility, remaining quantities, or the no-double-restoration rule; those are cross-record application/domain invariants.
- Existing movement types also include `opening_stock`, `stock_entry`, and `adjustment`; this change must not change their semantics.

### Transaction and domain seams

- Confirm-sale persistence is application-owned and atomic: insert lines/payments, decrement balances with a guarded update, append one negative sale movement per line, then confirm the sale.
- `domain::sales` has `Quantity`, integer-centavo money, sale lines, payment derivation, and payment integrity rules, but no lifecycle state, return quantity, cancellation reason, or refund concept.
- The likely deep module seam is a Rust post-sale application module with narrow cancellation and return interfaces. It should load original persisted sale lines and prior returns, validate eligibility, mutate all related records in one transaction, and return persisted operation facts. React should remain a presentation adapter and never compute returnable quantities or decide stock restoration.
- Tauri commands should expose typed owned request/response contracts and stable error codes. The command registry in `src-tauri/src/lib.rs` and TypeScript command adapters/UI are absent for this capability.

## Canonical terminology and invariants

Use these terms unless product decisions revise them:

- **Confirmed sale**: the original sale record whose lines, sale-time snapshots, total, and payment facts are immutable.
- **Sale cancellation**: a whole-sale, one-time correction that transitions a confirmed sale to a cancelled status, requires a non-blank reason, and creates compensating positive stock movements for only units not already returned.
- **Return**: a partial or full post-sale operation against one or more original sale lines. It accepts positive whole-unit quantities and creates positive stock movements; it does not edit/delete the original sale.
- **Returned quantity**: the sum of accepted return-line quantities for one original sale line.
- **Remaining returnable quantity**: original sold quantity minus returned quantity.
- **Restorable-on-cancellation quantity**: original sold quantity minus returned quantity, evaluated atomically when cancellation is accepted.
- **Immutable sale facts**: original status transition aside, original line quantities, prices/snapshots, totals, and payment rows are never rewritten or deleted. A cancellation/return is an additive correction record.
- **No double restoration**: for every original sale line, `sale quantity = returned quantity + cancellation-restored quantity` at most; return and cancellation must never restore the same sold unit.
- **Stock integrity**: every accepted return/cancellation line increases `stock_balances` by exactly its accepted/restorable quantity and appends exactly one immutable positive movement linked to the original sale line; all related writes commit or roll back together.
- **Offline idempotency**: repeated submission of one operator intent must not duplicate a correction. A stable request ID and unique persistence binding are needed for both operation types, with same-request replay returning the persisted result.

## Candidate bounded scope

1. Add schema support for cancellation and return records/lines and any operation request identity needed for replay-safe writes.
2. Add domain validation for positive whole quantities, non-blank cancellation reason, confirmed/non-cancelled eligibility, per-line remaining quantities, and cancellation's residual quantity calculation.
3. Add transaction-owned Rust use cases and SQLite adapters for return and whole-sale cancellation, including balance updates and immutable movements.
4. Add Tauri command contracts, TypeScript adapters, and a focused Sales UI flow reachable from delivered history/detail (or another explicitly selected Sales entry point).
5. Extend history projection only enough to preserve visibility and distinguish active/cancelled sales, while retaining original sale lines/payments and avoiding report aggregation.
6. Add domain, SQLite transaction, command-seam, and UI tests for atomicity, idempotency, snapshots, payment preservation, and no-double-restoration.

## Explicit exclusions

- Reports, analytics, exports, and report totals or report UI.
- Editing/deleting original sale lines, prices, totals, payment rows, or historical snapshots.
- Payment gateway integration, invoicing, tax/accounting, or automatic financial settlement.
- Customer accounts, credits, store credit, cash-drawer reconciliation, or actor/role attribution.
- Partial sale cancellation as a separate operation; the current product language says cancellation is whole-sale. Partial correction is represented by returns.
- Return reasons unless product explicitly requires them; the PRD currently requires a reason for cancellations and not for returns.
- Exchange workflows, replacement sales, restocking fees, condition inspection, damaged-return disposition, or supplier returns.
- Multi-store/cloud synchronization, fractional quantities, and changes to catalog or backup behavior.

## Product decisions blocking a sound proposal

1. **Payment semantics:** Does a return or cancellation record a refund/payment reversal, or is payment history strictly informational and unchanged? If refunds are recorded, which methods are supported, is mixed-payment refund allocation required, and should a refund amount be based on persisted sale-line price? The PRD defines stock behavior but does not define refund behavior.
2. **Cancellation timing and scope:** Is cancellation allowed only for `confirmed` sales, and is it allowed after one or more returns? The PRD says yes after prior returns, restoring only residual units, but the proposal should confirm whether a fully returned sale may still be cancelled (likely status-only/audit operation with zero stock restoration).
3. **Return operation identity:** Is one return request allowed to include multiple lines, and should retries use a UUID `request_id` exactly like sale confirmation? The existing unique movement request ID is not sufficient for multi-line operation identity without a return header.
4. **History behavior:** Must Sales History list/detail show cancelled sales and accepted returns immediately, or only expose cancellation status while return/cancellation detail is deferred? The current reader filters out non-confirmed rows, while the requirement says cancellations remain visible in history.
5. **Return UX and source:** Should returns start from a selected Sales History detail, and may the operator return a subset of lines in one operation? This determines the command payload and whether the delivered history screen is modified or a new Sales workflow is introduced.
6. **Return reason and operational audit:** Is a return reason required for audit symmetry, or intentionally omitted as the PRD currently specifies? Also confirm whether `operator_id` remains nullable/unset in v1.
7. **Idempotency conflict behavior:** For a reused request ID with a different payload, should the result be a stable conflict error (recommended) rather than replaying or silently accepting a different correction?
8. **Concurrent correction behavior:** The single shared computer reduces concurrency, but the transaction must still define behavior if two reads/commands overlap. Proposal should require an atomic eligibility check and reject the second operation rather than permitting over-return.

## Risks and architectural implications

- Expanding `sales.status` affects the delivered history reader, backup schema validation, and any existing assumptions that only confirmed sales are durable display records.
- A return line must reference the original `sale_lines.id`, not merely product ID, because the same product may occur in multiple sale lines and eligibility is per original line.
- The no-double-restoration rule cannot be safely implemented by reading balances alone; it requires aggregate prior return quantities and cancellation state inside the same write transaction.
- Payment facts are immutable, but unresolved refund semantics could accidentally turn an inventory correction into an accounting model. Keep that decision explicit and avoid inventing payment mutations.
- Schema checks and migration compatibility must preserve old databases and backup/restore validation. No external research is needed.
- The expected scope crosses Rust domain/application/infrastructure, Tauri, TypeScript, React, migrations, and tests. It is likely to exceed a single 400-line review unit; task planning should forecast vertical slices and use the configured `ask-on-risk` delivery strategy.

## Recommendation / readiness

The implementation direction is clear: additive correction records plus immutable positive movements, with residual eligibility calculated from original line facts and prior corrections in one Rust-owned SQLite transaction. A proposal is **not yet ready for approval** until payment/refund semantics, history visibility/detail, return request shape/idempotency, and return-reason policy are resolved. Reports should remain untouched except for preserving a future-compatible status/event model.
