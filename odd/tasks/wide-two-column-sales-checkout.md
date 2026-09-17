# Wide Two-Column Sales Checkout

## Objective
Make desktop checkout fit in one dialog viewport by widening it into a two-column composition: cart list on the left with independent scroll, payment/total/actions on the right in a stable rail.

## Problem
The current narrow checkout dialog has general modal scrolling plus cart scrolling. Payment and confirmation controls can leave the visible viewport while reviewing a long cart.

## Decision
At desktop widths above 960px, checkout uses a dialog up to 1040px wide. The cart is the only scrolling region; payment, current total, discard, Volver, and Confirmar venta remain in the right rail. At 960px and below, the dialog stacks and can scroll as required.

## Scope
- Add a checkout-specific right-rail layout seam while preserving shared dialog accessibility mechanics.
- Move/render current total and action footer in the desktop payment rail.
- Make desktop dialog viewport-contained without general overflow; retain cart-only scroll.
- Add narrow stacked fallback and focused mounted/CSS contract coverage.

## Constraints
- Preserve logical DOM order, accessible dialog naming, focus containment/restoration, Escape, pending locks, validations, feedback, and persisted-success handoff.
- No pricing, payment, domain, native contract, or browse behavior change.
- No commit or push authorized.
- TDD mode is not explicitly configured; use behavior-first tests.

## Acceptance criteria
- Desktop dialog is wide, two-column, and lacks general modal scrolling.
- Cart is left-side local scroll; payment/total/actions are visible on the right while cart scrolls.
- Total inside checkout updates with current draft.
- ≤960px stacks and allows modal scrolling as required.
- Existing focus, keyboard, pending, validation, failure, and success contracts remain correct.
- Mounted/CSS tests cover desktop/compact structure; Windows visual smoke is documented.

## Tasks
- [x] T1 Add checkout layout/action seam preserving shared dialog semantics.
- [x] T2 Implement desktop two-column and compact stacked styles.
- [x] T3 Add focused tests and verify checkout interaction contracts.

## Progress
- 2026-09-17: User authorized wider two-column checkout: cart left/scrolling, payment and final controls right/fixed, compact stack below 960px.

## Verification evidence
- `npm run typecheck:tests`: passed.
- `npm test`: passed, 208 tests passed and 0 failed.
- Mounted coverage observes the checkout layout seam, cart/rail DOM order, live current-total updates, focus restoration, pending locks, empty-cart disable, validation focus, failure feedback, and persisted success handoff.
- CSS contract coverage observes the desktop `min-width: 961px` 1040px/hidden-overflow two-column rules, cart-only desktop scrolling, and compact `max-width: 960px` stacked/scrollable fallback.
- Final independent verification found no code-level defects and confirmed all static/jsdom interaction contracts.
- Pixel-level browser geometry, scroll containment, and Windows visual smoke remain unverified because the authorized verification environment is jsdom-only.

## Next step
Run the Windows visual smoke at 1200×800 and 960×640 before delivery.
