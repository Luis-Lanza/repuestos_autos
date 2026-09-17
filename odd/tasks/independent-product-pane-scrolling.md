# Independent Product Pane Scrolling

## Objective
Keep product discovery usable with the 100-product demo catalog by giving product lists independent vertical scroll viewports in Sales, Inventory, and Catalog while retaining search, filters, actions, and pagination in stable positions.

## Problem
The shell content is the only current scroll container. Product lists grow naturally, so navigating many rows moves the entire screen and hides search controls, actions, and adjacent operational panels.

## Visual direction
Follow the Warm Industrial references: viewport-contained panels, warm-white surfaces, visible panel boundaries, balanced operational density, and stable master-detail hierarchy. Use panel-local list scrolling, not fixed arbitrary heights or an additional whole-screen scroll owner.

## Scope
- Establish flexible/min-size containment from each screen layout and product panel to the shared ProductBrowser list.
- Make only the product result list independently scrollable; keep form and pagination outside it.
- Apply the behavior to Sales, Inventory, and Catalog.
- Replace Catalog master-list arbitrary height limits with available-space containment where needed.
- Add mounted CSS/DOM contract coverage and retain responsive behavior.

## Constraints
- The shell remains the sole page-level scroll owner.
- Do not change browse contracts, flows, pagination, async protections, or accessibility semantics.
- Preserve 1200px desktop composition, 961–1199px Catalog reflow, and ≤960px stacked controls/rows.
- No commit or push is authorized.
- TDD mode is not explicitly configured; use focused behavior-first tests plus a manual Windows visual smoke recommendation.

## Acceptance criteria
- At 100 rendered products, each product list has its own vertical scroll viewport.
- Search/filter form, row actions, and pagination remain outside that list viewport.
- Sales cart/payment, Inventory operation/alerts, and Catalog editor remain reachable while list rows scroll.
- No arbitrary 520px/208px product-list limits remain in the affected Catalog containment path.
- Existing loading, empty, error, disabled, keyboard, and responsive states remain intact.
- Mounted tests assert structural/CSS containment; a 1200×800 and 960×640 Windows visual smoke is documented.

## Tasks
- [x] T1 Establish shared ProductBrowser and screen/panel scroll containment.
- [x] T2 Apply responsive Sales, Inventory, and Catalog layout rules without nested page scroll.
- [x] T3 Add mounted coverage and run frontend checks.

## Progress
- 2026-09-17: User authorized independent product-list scrolling after loading 100 demo products. Mapping confirmed ProductBrowser lists have no overflow containment and shell content owns the only scroll.
- 2026-09-17: Added flexible shell-to-screen-to-panel containment, independent ProductBrowser list scrolling, stable pagination placement, responsive grid rows, and available-space Catalog master-list containment.
- 2026-09-17: Mounted contracts cover 100 rendered products in Sales, Inventory, and Catalog, with search and pagination remaining outside the result list and operational/editor panels remaining present.

## Verification evidence
- `npm run typecheck:tests`: passed.
- `npm test`: passed; 205 tests, 0 failures.
- Final independent verification confirmed the shell remains the page-scroll owner, ProductBrowser form/results/pagination are sibling regions, result-list overflow is local, Catalog arbitrary height limits are removed, and 100-product mounted contracts pass.
- Catalog master scrolling remains intentional available-space local containment; other pre-existing local overlays are not part of the product-result viewport contract.
- Windows visual smoke remains required at 1200×800 and 960×640 because jsdom cannot validate computed geometry.

## Next step
Run the documented Windows visual smoke at 1200×800 and 960×640 before commit/push or release review.
