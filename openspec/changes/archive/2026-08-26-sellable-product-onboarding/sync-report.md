# Sync Report: Sellable Product Onboarding

## Status

**synced** — The delta specifications were merged into the canonical OpenSpec specifications before the change was archived.

## Sync scope

| Domain | Change delta | Canonical target | Result |
|---|---|---|---|
| `catalog-onboarding` | `openspec/changes/sellable-product-onboarding/specs/catalog-onboarding/spec.md` | `openspec/specs/catalog-onboarding/spec.md` | Created |
| `sales` | `openspec/changes/sellable-product-onboarding/specs/sales/spec.md` | `openspec/specs/sales/spec.md` | Updated |

## Delta applied

| Domain | Operation | Result |
|---|---|---|
| `catalog-onboarding` | Full spec copy | Created five requirements and their seven scenarios as the new canonical domain spec |
| `sales` | MODIFIED `Active Product Search and Cart` | Included seeded and operator-created active products, added the onboarded-product scenario, and retained cart and fixed-price behavior |
| `sales` | MODIFIED `Confirm-Sale Scope Exclusions` | Replaced the seeded-only catalog assumption while retaining the existing checkout scope exclusions |
| both | REMOVED / RENAMED | None |

The canonical sales specification retains the unrelated fixed-price checkout requirements for whole-unit quantities, payment integrity, atomic confirmation and stock integrity, idempotency, persisted summaries, negotiated-value exclusion, and migration compatibility. No existing fixed-price checkout requirement was deleted.

## Checks performed

- Read proposal, both delta specs, design, tasks, apply progress, verification report, configuration, and the pre-existing canonical sales specification.
- Confirmed the persisted tasks artifact is complete: 10/10 implementation tasks checked; no unchecked task remains.
- Confirmed fresh verification is passing with 7/7 requirements, 13/13 scenarios, zero blockers, and zero critical findings.
- Applied the new `catalog-onboarding` spec as the missing canonical domain spec.
- Merged the two sales modifications by preserving all unrelated canonical requirements and existing fixed-price checkout detail.
- Ran `git diff --check -- openspec/specs/sales/spec.md`; it passed.
- Preserved unrelated dirty/untracked paths and the protected formatting-only sales UI changes.

## Archive destination

`openspec/changes/archive/2026-08-26-sellable-product-onboarding/`
