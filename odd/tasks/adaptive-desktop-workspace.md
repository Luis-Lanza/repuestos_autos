# Adaptive Desktop Workspace

## Objective
Unify Sales, Inventory, and Catalog desktop canvas behavior so each screen uses available workspace width with proportionate operational columns while controls remain readable.

## Problem
Sales expands across the desktop with a catalog workspace plus summary rail. Inventory and Catalog retain fixed-width tracks, leaving a large unused right-hand area at Windows desktop widths.

## Decision
Use an adaptive operational canvas: the screen owns available width; grids use flexible proportional columns; form/search controls maintain practical readable bounds. Sales keeps its established summary rail. At compact widths existing stacked behavior remains.

## Scope
- Make Catalog category rail and product workspace proportionally fill the desktop content area.
- Make Inventory operation workspace and stock-alert rail proportionally fill desktop content area.
- Preserve Sales existing browse/cart layout and desktop summary rail.
- Preserve max/readability constraints for inline search and form controls.
- Retain <=960px stacking and local scrolling behavior.
- Add focused responsive/style contract coverage where established.

## Non-goals
- No domain/IPC/business-flow change.
- No global visual-system rewrite.
- No redesigned individual controls or content hierarchy.

## Acceptance criteria
- At wide desktop widths, Catalog and Inventory have no arbitrary blank canvas to the right of their primary cards.
- Rails remain legible and operation/product workspaces receive extra width.
- Controls do not become impractically wide.
- Sales keeps its current desktop functional layout.
- Compact breakpoint behavior is preserved.

## Tasks
- [x] T1 Map current desktop grid constraints and establish shared adaptive sizing rules.
- [x] T2 Apply Catalog and Inventory workspace grid changes with bounded controls.
- [x] T3 Verify wide/compact style contracts and regression behavior.

## Progress
- 2026-09-17: User reviewed Windows screenshots `/tmp/bug5.png` and `/tmp/bug6.png`, accepted adaptive operational canvas recommendation.

## Verification evidence
- Mapping found fixed CSS tracks only: Inventory `600px + 324px`, Catalog `336px + 588px`; Sales already uses flexible workspace plus `260px–320px` rail.
- Recommended: Catalog `minmax(280px, 4fr) minmax(0, 7fr)`, Inventory `minmax(0, 1.85fr) minmax(260px, 1fr)`, scoped bounded filter forms, and desktop two-panel Catalog from 961px upward.
- Focused mounted tests (`./node_modules/.bin/tsx --test --import ./test/react-dom.ts src/ui/app-shell.mounted.test.ts src/ui/catalog/catalog-maintenance-screen.mounted.test.ts src/ui/inventory/inventory-screen.mounted.test.ts src/ui/sales/sale-screen.mounted.test.ts`): 46 passed, 0 failed.
- `npm run typecheck:tests`: passed.
- `npm test`: 213 passed, 0 failed.

## Next step
No further implementation step remains; ready for parent review.
