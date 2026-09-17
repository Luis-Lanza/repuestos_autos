# Catalog Edit Modal and Lifecycle

## Objective
Replace Catalog's inline detail/editor with an accessible edit modal, unify category/product archive-reactivate actions inside that modal, and make lifecycle feedback visible beside its action.

## Problem
Catalog mixes browsing, editing, and category lifecycle actions in one long screen. The editor overwhelms the list. Category archive errors are rendered far from their triggering action, and products expose no lifecycle action despite backend support.

## Decision
`Editar` opens a routine, accessible form dialog. The modal loads the authoritative metadata detail, edits/saves it, and exposes immediate reversible lifecycle action: `Archivar` for active records and `Reactivar` for archived records. No confirmation step is required. Categories with active products remain blocked by existing domain policy; feedback appears adjacent to the lifecycle action.

## Scope
- Remove inline catalog editor from normal browse layout.
- Add routine Catalog edit modal with focus/keyboard/pending/error behavior.
- Move category lifecycle action into modal and expose product lifecycle action from real detail activity.
- Style archive as outline garnet and reactivate as secondary.
- Refresh lists after lifecycle/save success and preserve authoritative expected revisions.
- Add focused mounted behavior/accessibility/CSS coverage.

## Constraints
- No browse IPC shape, native command, domain transition policy, or persistence change.
- Keep archive/reactivate immediate and reversible.
- Preserve existing edit validation, stale errors, and data decoding.
- No commit/push authorized.
- TDD mode is not explicitly configured; use behavior-first tests.

## Acceptance criteria
- Catalog browse layout shows categories/products without an inline editor.
- Edit opens named/accessible modal with authoritative loaded detail and focus containment/restoration.
- Both categories and products can archive/reactivate from real detail state.
- Lifecycle success refreshes relevant list; lifecycle failure stays visible beside action.
- Active category with active products remains blocked and explains failure in modal.
- Modal pending state locks editing/lifecycle, Escape, and duplicate actions.
- Archive uses outline garnet; reactivate uses secondary styling.
- Existing price/name/SKU/attribute validation and save behavior remain correct.

## Tasks
- [x] T1 Add Catalog routine edit modal seam and move inline editing into it.
- [x] T2 Implement immediate archive/reactivate action with near-action feedback for categories and products.
- [x] T3 Rehydrate authoritative detail before unlocking every stale-recovery path, then reverify.

## Progress
- 2026-09-17: User authorized Catalog modal redesign and chose immediate archive/reactivate. Windows screenshot `/tmp/bug3.png` showed the rejected inline editor and unclear lifecycle action.
- 2026-09-17: Full-refresh recovery now remains locked through delayed or failed product browse and retry does not repeat mutation. Final verification found stale mutation/edit failure recovery unlocks without reloading authoritative detail; T3 reopened.

## Verification evidence
- `npm run typecheck:tests` — passed.
- `npm test` — passed (206 tests).
- Focused Catalog flow and mounted tests — passed (12 tests).
- Independent verification found refresh-failure recovery incomplete: after successful save/lifecycle followed by failed list refresh, the modal remains with stale list context and no actionable retry. T3 was reopened; the feature is not ready.
- `npm run typecheck:tests` — passed.
- `npm test` — passed (209 tests).
- `npx tsx --test --import ./test/react-dom.ts src/ui/catalog/catalog-maintenance-flow.test.ts src/ui/catalog/catalog-maintenance-screen.mounted.test.ts` — passed (12 tests).
- Catalog dialog mounted recovery test — passed (1 test).
- Final independent verification found a remaining two-stage refresh race: recovery unlocks after maintenance-list refresh but before product browse completes. The modal must remain locked until both stages succeed; T3 reopened.
- `npm run typecheck:tests` — passed.
- `npm test` — passed (211 tests).
- `npx tsx --test --import ./test/react-dom.ts src/ui/catalog/catalog-maintenance-flow.test.ts src/ui/catalog/catalog-maintenance-screen.mounted.test.ts` — passed (14 tests).
- Final independent verification found stale mutation/edit failure recovery re-enables controls without rehydrating authoritative detail, permitting repeat stale writes. T3 reopened; feature not ready.
- 2026-09-17: stale mutation and edit recovery now keeps modal actions locked through list and product-browse refresh, rehydrates the selected authoritative detail before unlocking, and never repeats the failed mutation. Added mounted coverage for stale lifecycle/edit recovery, delayed detail loading, authoritative revision use, and recovery completion.
- `npm run typecheck:tests` — passed.
- `npm test` — passed (213 tests).
- `npx tsx --test --import ./test/react-dom.ts src/ui/catalog/catalog-maintenance-flow.test.ts src/ui/catalog/catalog-maintenance-screen.mounted.test.ts` — passed (16 tests).

## Next step
T3 implementation and verification complete.
