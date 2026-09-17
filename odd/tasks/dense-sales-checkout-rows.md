# Dense Sales Checkout Rows

## Objective
Compact the checkout cart into dense POS-style table rows so several products are visible in the cart viewport while retaining all sale facts, editable controls, validation, and accessibility.

## Problem
Current checkout lines render as tall stacked form cards. Repeated labels and vertically separated list/minimum/quantity/price/subtotal facts make each product consume excessive vertical space.

## Decision
Desktop checkout uses a semantic editable table: Product/SKU, List/Minimum, Quantity, Final price, Subtotal, and Remove. Secondary price facts are compact in one cell. At ≤960px the table transforms to labeled stacked cells while preserving keyboard operation and input labels.

## Scope
- Replace Sales checkout cart list markup with native table semantics.
- Reuse existing Field controls, reducer actions, price IDs, validation, and focus behavior.
- Add dense desktop and stacked compact CSS.
- Add mounted semantic/accessibility/interaction/responsive coverage.

## Constraints
- No change to Sale flow, pricing rules, payment, native contract, checkout dialog behavior, or async protections.
- Retain 44px input/button targets and current Spanish copy.
- No commit or push authorized.
- TDD mode is not explicitly configured; use behavior-first tests.

## Acceptance criteria
- Each cart row exposes product/SKU, list/minimum, quantity, final price, subtotal, and a uniquely named remove action.
- Quantity/price changes preserve validation, field associations, focus, subtotal, total, and summary-unit behavior.
- Pending state disables table controls and confirmation as before.
- Desktop rows are compact and cart scroll remains local.
- At ≤960px rows stack without core-task horizontal scroll.
- Mounted tests cover semantics, labels, errors, focus, pending, interactions, and responsive CSS.

## Tasks
- [x] T1 Implement semantic dense checkout table markup.
- [x] T2 Implement desktop/compact row styles preserving local cart scroll.
- [x] T3 Update mounted tests and verify behavior.

## Progress
- 2026-09-17: User approved dense POS-style checkout rows after reviewing the wide checkout screenshot.

## Verification evidence
- `npm run typecheck:tests`: passed.
- `npm test`: passed, 209 tests passed and 0 failed.
- Mounted coverage verifies native table headers/cells, per-product remove names, preserved field IDs and validation associations, pending controls, interactions, and desktop/compact CSS contracts.
- Final independent verification found no blockers or code-level defects; browser geometry, horizontal overflow, and actual table scroll remain pending visual validation because jsdom cannot render them.

## Next step
Visually test the dense checkout cart on Windows before commit/push.
