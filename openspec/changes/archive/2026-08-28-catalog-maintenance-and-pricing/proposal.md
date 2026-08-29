# Proposal: Catalog Maintenance and Pricing

## Intent

Let staff correct, retire, reactivate, and reprice catalog data without SQLite edits, stale search, or rewritten history.

## Users, Situations, and Outcome

On the shared computer, staff maintain data after onboarding. Stale drafts and competing edits surface instead of changing facts silently.

## Current-State Gap

Only creation exists. Categories lack lifecycle state; records lack revisions/audits; edits could desynchronize FTS; drafts lack price comparison.

## Scope

### In Scope
- Category list/detail, rename, archive, and reactivate; archival is blocked by active products.
- Product list/detail; edit SKU, name, typed values, and Current Catalog Price; archive/reactivate with stock preserved.
- Maintenance views can include archived entities; normal selling/search excludes them.
- Optimistic revisions, atomic search refresh, append-only audits, and stable opaque `stale_catalog_record` outcomes.
- Draft lines retain captured prices; changed prices require stale/repricing acknowledgement before confirmation.

### Non-Goals
- Returns, reports, bulk import, promotions, cost accounting, or historical rewriting.
- Hard deletion, identity reuse, schema evolution, category reassignment, price scheduling, persisted carts, accounts, or synchronization.

## Capabilities

### New Capabilities
- `catalog-maintenance`: Metadata edits, lifecycle, future pricing, revisions, audit, and maintenance visibility.

### Modified Capabilities
- `sales`: Compare captured and authoritative prices, requiring acknowledgement while preserving confirmed facts.
- `operational-inventory-control`: Apply product-and-category availability while preserving balances and movements.

## Business Rules and Approach

- Confirmed sale-line prices and historical facts are immutable; repricing affects later confirmations only.
- Category lifecycle never cascades product state. Product reactivation requires an active category and valid values.
- An intent-based Rust module owns rules. Its SQLite adapter atomically checks `expected_revision`, updates data and FTS, appends audit, increments revision, and returns persisted state.
- An additive migration preserves IDs, stock, movements, attributes, and sale facts. Catalog work never mutates inventory history.

## Implications and Risks

| Risk | Mitigation |
|---|---|
| FTS or audit drift | Update rows, FTS, and audit transactionally. |
| Surprising repricing | Preserve captured price and require acknowledgement. |
| Large category rename | Performance-check refresh for 20,000 products. |
| Review overload | Keep autonomous slices; ask before stacked-to-main delivery above 400 lines. |

## Rollout and Rollback

Back up first; preflight and migrate transactionally, reopen, then verify constraints and preserved facts. Roll back by restoring the backup and prior application, never by rewriting history.

## Dependencies

- Existing Catalog, Sales, Inventory, and SQLite migration contracts.

## Success Criteria

- [ ] Staff can edit, archive, reactivate, and reprice with conflict/lifecycle feedback.
- [ ] Selling, inventory, alerts, and maintenance views apply lifecycle visibility consistently.
- [ ] Stale carts cannot confirm a changed price without acknowledgement.
- [ ] Confirmed sales, stock, movements, and audit evidence remain readable and unchanged.
