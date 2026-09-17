# Bounded Confirmed Sale Receipt

## Objective
Make the persisted sale confirmation a compact desktop receipt: immutable sale header, locally scrollable articles left, stable payment/total/New Sale rail right, and no general page scrolling for normal sales.

## Problem
The current vertical persisted-summary composition makes the whole screen scroll even for ordinary confirmed sales, pushing the total and `Nueva venta` action away from view.

## Decision
Above 960px, confirmed sale uses a viewport-contained two-column receipt. Articles receive available left-side height and scroll only on overflow; payment, total, and `Nueva venta` remain visible in the right rail. At ≤960px sections stack and page scrolling is allowed.

## Scope
- Add semantic layout hooks to persisted summary without changing fact projection.
- Apply desktop receipt containment/local article scroll and compact stacked fallback.
- Add persisted-summary/mounted structure and CSS contract coverage.

## Constraints
- Preserve immutable persisted facts, logical DOM order, table semantics, only-one-action behavior, and current `Nueva venta` reset.
- No focus, sale-flow, payment, native, domain, or pricing behavior change.
- No arbitrary fixed row threshold; available viewport determines local article overflow.
- No commit/push authorized.
- TDD mode is not explicitly configured; use behavior-first tests.

## Acceptance criteria
- Desktop has full confirmation header, articles left, payment/total/action right.
- Normal articles do not cause page scrolling; long articles scroll locally only.
- Right rail remains visible as articles scroll.
- ≤960px stacks without core horizontal overflow and may use page scroll.
- Read-only persisted facts and sole `Nueva venta` behavior remain unchanged.
- Mounted tests cover layout hooks, semantics, reset/snapshot behavior, and CSS responsive/scroll contracts.

## Tasks
- [x] T1 Add persisted receipt layout hooks while preserving fact projection.
- [x] T2 Implement bounded desktop/stacked compact styles.
- [x] T3 Add tests and verify receipt interaction contracts.

## Progress
- 2026-09-17: User authorized compact confirmed-sale receipt after reporting excessive general scrolling.
- 2026-09-17: Added semantic receipt layout hooks, desktop containment with article-only scrolling, and stacked compact fallback without changing persisted projection or reset behavior.

## Verification evidence
- `npm run typecheck:tests`: passed.
- `npm test`: passed; 209 tests passed, 0 failed.
- Final independent verification found no blockers or code-level defects. It confirmed persisted fact projection/order, semantic tables, desktop article-only scroll contracts, compact fallback, and reset/sole-action behavior.
- Browser geometry, actual page overflow, and local scrolling at target viewports remain pending because jsdom cannot measure them.

## Next step
Run Windows visual validation at 1200×800 and 960×640 before commit/push.
