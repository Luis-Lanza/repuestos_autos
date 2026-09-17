# Dense Sales Checkout Rows

## Objective
Compact the checkout cart into dense POS-style table rows so several products are visible in the cart viewport while retaining all sale facts, editable controls, validation, and accessibility.

## Problem
Current checkout lines render as tall stacked form cards. Repeated labels and vertically separated list/minimum/quantity/price/subtotal facts make each product consume excessive vertical space.

## Decision
The six-column table was rejected after Windows visual testing because it overlaps at the actual cart-pane width. Replace it with a compact two-level row: product/SKU, list/minimum, and product-specific remove action on the first line; quantity, final price, and subtotal aligned on the second line. At ≤960px this row stacks naturally while preserving keyboard operation and input labels.

## Scope
- Replace Sales checkout cart list markup with compact two-level list rows.
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
- Pending state disables row controls and confirmation as before.
- Desktop rows are compact and cart scroll remains local.
- At ≤960px rows stack without core-task horizontal scroll.
- Mounted tests cover semantics, labels, errors, focus, pending, interactions, and responsive CSS.

## Tasks
- [x] T1 Replace the rejected six-column table with compact two-level checkout rows.
- [x] T2 Apply responsive row styling without overlaps and preserve local cart scroll.
- [x] T3 Add regression coverage and independently verify the corrected layout.

## Progress
- 2026-09-17: User approved dense POS-style checkout rows after reviewing the wide checkout screenshot.
- 2026-09-17: Windows screenshot `/tmp/bug2.png` showed the six-column table overlaps and wraps uncontrollably in the actual cart pane. User authorized replacement with compact two-level rows; prior completion was reopened.

## Verification evidence
- `npm run typecheck:tests`: passed.
- `npm test`: passed, 209 tests passed and 0 failed.
- Mounted coverage now rejects the checkout table pattern, asserts two-level list-item rows and their product-specific actions, and preserves Field IDs, errors, focus, pending controls, interactions, local scroll, and responsive CSS contracts.
- Final independent verification confirmed the rejected table is removed and the two-level row structure preserves interaction contracts. Browser geometry and Windows visual validation remain pending because the automated environment does not render the desktop cart pane.

## Next step
Run the Windows visual validation against the replacement two-level cart rows.
