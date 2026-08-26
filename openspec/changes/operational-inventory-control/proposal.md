# Proposal: Operational Inventory Control

## Intent

Give the operator an offline workflow to replenish, reconcile physical counts, and act on shortages. Inventory facts exist, but no operational workflow or shortage view does.

## Scope

### In Scope
- Use Catalog global search to select active products for one-product entries or physical-count adjustments.
- Show derived alerts for active products: `0` = **Out of stock**, `1` = **Low stock**; order out first, then low, then deterministic product order.
- Add movement invariants, atomic/idempotent confirmation, immutable history, and restart proof.

### Out of Scope
- Returns, cancellations, suppliers/costs, catalog editing, reports, backup/restore, roles, cloud, and multi-store.
- Checkout cart price/total display defect; track separately as a prerequisite/follow-up, not part of this vertical.

## Capabilities

### New Capabilities
- `operational-inventory-control`: Entry, physical-count adjustment, immutable confirmation, and derived alerts.

### Modified Capabilities
- None; Catalog search, onboarding, and Sales remain unchanged.

## Business Rules and Approach

- Inventory owns balance changes/history; onboarding still creates opening stock atomically.
- Entry quantity is a positive whole unit; its note is optional. Adjustment accepts an absolute non-negative count, requires a non-blank reason, and rejects unchanged counts with a stable error code.
- Confirmation retains a UUID request ID; one write transaction revalidates product/balance, derives delta, guards overflow/non-negative stock, and persists exactly one movement with a database timestamp. Retry returns that result.
- Alerts are read projections from current balances, never persisted.

## Affected Areas

| Area | Impact | Description |
|---|---|---|
| `src-tauri/src/`, `src-tauri/tests/` | New/Modified | Module, migration, IPC, proofs |
| `src/commands/inventory.ts`, `src/ui/` | New/Modified | Adapter, workflow, alerts, navigation |

## Compatibility

Rebuild movements forward-only, preserving IDs, timestamps, and facts. Preflight incompatible rows/foreign keys; fail without advancing schema version. Enforce per-type sign, linkage, reason, request-ID, and sale-line/product invariants.

## Risks/Implications

| Risk | Mitigation |
|---|---|
| Legacy history loss | Preflight, exact copy, reopen tests |
| Stale preview overwrites intervening activity | Recompute under authoritative transaction |
| Duplicate retry | Detect request ID before transactional mutation |
| Change exceeds 400 review lines | Ask before apply; plan chained review slices |

## Dependencies and Deferrals

- Refresh stale SDD testing claims before apply/verify; manifests and tests prove runners exist.
- Defer configurable thresholds, notifications, operator attribution, and movement-history reporting.

## Rollback Plan

Stop rollout and restore the pre-migration database/application pair; never reinterpret migrated facts in place.

## Success Criteria

- [ ] Valid entry/adjustment confirms once, survives restart, and retries without duplicate balance or movement effects.
- [ ] Invalid, inactive, overflow, no-op, or failed operations leave balance/history unchanged with stable errors.
- [ ] Alert count/list immediately reflect committed balances with required labels and ordering.
