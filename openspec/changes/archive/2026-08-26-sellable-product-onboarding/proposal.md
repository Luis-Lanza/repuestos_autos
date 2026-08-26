# Proposal: Sellable Product Onboarding

## Intent

Enable staff to replace category-specific spreadsheets by creating categorized products that are immediately searchable and sellable through the completed fixed-price checkout. This proposal is retrospective: the dirty worktree already contains a direct-route implementation used only as verification evidence; SDD did not guide it.

## Scope

### In Scope
- Create and list categories with typed required or optional fields.
- Create an active product with SKU, name, integer-centavo catalog price, typed category values, and positive whole-unit opening stock.
- Persist the product, typed values, stock balance, and immutable positive `opening_stock` movement atomically.
- Make created products searchable and sellable through existing fixed-price checkout.

### Out of Scope
- Editing, archiving, Excel import, images, barcode hardware, suppliers, or purchase costs.
- Changes to checkout pricing, payment, idempotency, or sale-history rules.
- Fractional inventory, accounts/roles, cloud sync, or multi-store operation.

## Capabilities

### New Capabilities
- `catalog-onboarding`: Category setup and atomic creation of searchable, stocked, active products.

### Modified Capabilities
- `sales`: Replace the seeded-only assumption so operator-created active products are searchable and sellable under unchanged checkout rules.

## Approach

Keep category setup separate from one deep product-creation interface. React owns form state; Tauri carries typed intent; Rust validates and owns the transaction; SQLite enforces constraints. Reuse search and checkout seams without extending checkout authority.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/ui/onboarding/`, `src/commands/onboarding.ts` | New | Workflow and IPC client |
| `src-tauri/src/{commands,application,domain}/` | Modified | Validation and orchestration |
| `src-tauri/src/infrastructure/sqlite/` | Modified | Values and opening stock |
| `openspec/specs/sales/spec.md` | Modified | Remove seeded-only catalog assumption |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Search misses the 1-second/20,000-product target | Med | Benchmark; adopt FTS/indexing if needed |
| Typed-value integrity relies on Rust | Med | Verify validation and rollback |
| Migration harms legacy movements | Med | Test representative databases and backup/compatibility |
| Change exceeds the 400-line review budget | High | Require explicit acceptance or retrospective review slices |

## Rollback Plan

Back up the database, revert onboarding and its migration together, restore the compatible snapshot, and retain seeded-product checkout. Never delete confirmed-sale history.

## Dependencies

- Existing global search, fixed-price checkout, `docs/PRD.md`, and `docs/ARCHITECTURE.md`.

## Success Criteria

- [ ] Staff can create a valid product and immediately find and sell it at its backend-resolved catalog price.
- [ ] Invalid input or persistence failure leaves no partial product, values, balance, or opening movement.
- [ ] Opening stock is positive whole units and produces one immutable timestamped movement.
- [ ] Verification separates implementation evidence from unproven performance, migration, and UI claims.
