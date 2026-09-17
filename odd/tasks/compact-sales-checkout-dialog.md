# Compact Sales Checkout Dialog

## Objective
Reclaim Sales browsing space by replacing the large inline cart/payment/summary panels with a compact sticky summary and one accessible `Revisar y cobrar` dialog that owns cart review, editing, payment, and final confirmation.

## Problem
With a large catalog, the inline cart and payment panels consume the vertical space needed for product discovery. The operator must sacrifice browse visibility even when they only need to add products.

## Decision
The compact summary counts total units across draft lines and exposes the current total. `Revisar y cobrar` is the single checkout entry. The dialog preserves the existing sale state and async behavior; it is a routine checkout dialog, not a destructive-confirmation dialog.

## Scope
- Replace inline Sales cart, payment, and summary panels with a compact sticky summary trigger.
- Add a generic accessible checkout dialog seam or safely generalized dialog primitive.
- Render existing cart editing, price/quantity validation, payment inputs, feedback, and final confirmation inside the dialog.
- Preserve persisted summary behavior after success.
- Add mounted interaction/focus/pending/failure/responsive coverage.

## Constraints
- No native contract, domain, or pricing-rule change.
- Preserve stale/unmount/duplicate-confirmation guards and current Spanish messages.
- Dialog dismissal must preserve draft state and restore focus to its trigger.
- Keep 44px targets, focus containment, Escape behavior, and 1200×800/960×640 reflow.
- No commit or push is authorized.
- TDD mode is not explicitly configured; use behavior-first tests.

## Acceptance criteria
- Product browser becomes the dominant Sales work region.
- Summary shows total units, total amount, and `Revisar y cobrar`.
- Dialog has an accessible Spanish name/description, focus containment, Escape/Volver close behavior, and invoker focus restoration.
- Cart edits, payment fields, validation focus, failures, pending lock, and confirmation work inside the dialog.
- Successful confirmation shows the existing persisted summary; no draft state is lost on cancellation/failure.
- Mounted tests cover units count, dialog interaction, focus, pending/error states, stale/unmount safety, and responsive CSS contracts.

## Tasks
- [x] T1 Generalize/add the accessible routine checkout dialog seam.
- [x] T2 Prevent empty-cart confirmation after cart edits within the dialog.
- [x] T3 Add regression coverage and reverify the checkout flow.

## Progress
- 2026-09-17: User authorized compact checkout. They selected total units, not distinct products, for the summary counter.

## Verification evidence
- Focused Sales, flow, confirmation, and checkout mounted tests: 44 passed.
- `npm run typecheck:tests`: passed.
- `npm test`: 207 passed.
- Independent verification found an empty-cart confirmation defect: removing the final cart line in an open checkout dialog leaves `Confirmar venta` enabled and can invoke the native contract with no lines. T2/T3 were reopened; the feature is not ready.
- Mounted regression `disables empty-cart confirmation without invoking the native contract`: passed. The final-line removal leaves the dialog open, focuses `Volver`, disables `Confirmar venta`, and records zero native confirmation calls.
- `npm run typecheck:tests`: passed.
- `npm test`: passed, 208 tests passed and 0 failed.
- Final independent verification confirmed the empty-cart native disabled control, zero-line handler guard, zero-call regression, summary units, dialog focus/pending/failure/success behavior, and responsive static contracts.
- Pixel-level browser geometry remains unverified because jsdom cannot render the reference viewports.

## Next step
User can authorize commit/push and visually test the compact checkout at 1200×800 and 960×640.
