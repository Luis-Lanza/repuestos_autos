# Product Browser Initial Load and Catalog Layout

## Objective
Fix the released browse UX regressions: Sales and Inventory must load their first active-product page automatically, and Catalog must remain readable and non-overlapping at supported desktop widths.

## Problem
Sales and Inventory leave the product browser in its initial empty state until the operator submits a blank search. Catalog places the complete ProductBrowser inside a narrow master pane beside category maintenance, causing filters, rows, and labels to compress or overlap. The published screenshot confirms the defect.

## Visual direction
Follow the saved Warm Industrial visual system and sibling mockups: persistent mineral-plum shell; ivory canvas; warm-white panels; copper active/focus accents; labels above controls; balanced operational density; stable master-detail hierarchy. Mockups are references only: preserve actual approved behavior and Spanish UI copy.

## Scope
- Autoload the first active-product page in Sales and Inventory exactly once on mount.
- Preserve alert-filter entry behavior in Inventory without duplicate browse requests.
- Recompose Catalog so category maintenance remains a compact master pane while the product browser/detail uses a readable wide work area.
- Add intermediate responsive handling before the fixed desktop layout overflows.
- Add mounted tests for automatic browse and CSS/layout contracts.

## Constraints
- Reuse the existing paged browse contract; no IPC, domain, or persistence change.
- Preserve request identity, unmount, pending, keyboard, feedback, and accessible-label behavior.
- Keep the warm-industrial tokens and avoid adding unsupported mockup behavior.
- No commit or push is authorized for this fix yet.
- TDD mode is not explicitly configured; use focused behavior-first tests.

## Acceptance criteria
- Sales loads the first page with blank query, active activity, page 1, and page size 20 without pressing Search.
- Inventory does the same; opening from the sidebar alert cue uses `stock_state: alerts` and triggers only one initial browse.
- Existing text searches and stale-result protections remain correct.
- Catalog product controls and rows do not live inside the narrow category master pane.
- At 1200px, Catalog uses a legible master-detail composition; before fixed widths overflow, it becomes a single-column layout.
- At 960px, filters and result rows stack without overlap or horizontal shell overflow.
- Focused mounted tests cover automatic loading and layout selectors/breakpoints.

## Tasks
- [x] T1 Add automatic first-page browsing to Sales and Inventory with no duplicate alert-route request.
- [x] T2 Recompose Catalog product browsing and responsive CSS using the visual references.
- [x] T3 Add focused tests and run the frontend verification suite.

## Progress
- 2026-09-16: User authorized this fix after testing `826dbc9` on Windows. Screenshot `/tmp/bug.png` shows the broken Catalog layout. Mapping confirmed initial browse is never triggered in Sales and only triggered for alert-entry Inventory.

## Verification evidence
- `npm run typecheck:tests` — passed (`tsc --project tsconfig.tests.json --noEmit`).
- `npm test` — passed (202 tests, 202 passed, 0 failed).
- Final independent verification found no Critical, High, or Medium issues. It confirmed one automatic Sales browse, one automatic alert-route Inventory browse, retained stale guards, a sibling Catalog product workspace, and the 1200px/961–1199px/960px layout contracts.
- Pixel-level browser geometry was not rendered in this Linux check; a 1200px and 960px Windows smoke test remains optional.

## Next step
User can test locally; commit and push require explicit authorization.
