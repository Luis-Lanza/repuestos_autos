# History Density Regression Correction

## Objective
Remove the remaining desktop whitespace regression for populated Sales History and make simple sale detail fit without artificial page scroll, while preserving correction behavior and accessible controls.

## Problem
The prior density rule treats two History rows as dense, so a second sale reintroduces vertical stretching. The simple detail marker only changes alignment; intrinsic height from panel spacing, empty correction history, and separately stacked correction actions still causes a small page overflow.

## Decision
Top-align every ready History list because backend results are bounded and shell scrolling owns long content. Keep strict simple-detail eligibility, but compact only that state through smaller spacing and a wrapping action group. Do not hide facts, reduce target sizes, set fixed heights, or suppress scrolling for long/open correction content.

## Scope
- Compact all ready History lists, including two sales.
- Compact simple detail panel/empty correction spacing.
- Group initial return/cancellation actions without altering their flows.
- Add focused regression coverage for ready-list and simple-detail boundaries.

## Non-goals
- No flow, command, IPC, domain, persistence, or correction semantics changes.
- No hidden audit facts, disabled scrolling, or smaller than 44px controls.

## Acceptance criteria
- Adding a second sale does not create an artificial vertical gap in History.
- A simple detail with no corrections fits compactly at desktop height when its content otherwise fits.
- Both correction actions stay visible and keyboard-operable before activation.
- Open or content-heavy correction states remain scrollable.

## TDD and delivery
- TDD mode: not configured; use focused regression checks after each task.
- Delivery strategy: ask-on-risk; forecast under 400 authored changed lines.
- Commit/push: not authorized by the user.

## Tasks
- [x] T1 Correct ready-list density and group simple-detail actions. Route: delegated writer (multi-file UI change).
- [x] T2 Add scoped simple-detail spacing rules that preserve dense scrolling. Route: delegated writer (multi-file UI change).
- [x] T3 Add and run regression coverage for list/detail boundaries. Route: delegated writer plus independent verification as required.

## Progress
- 2026-09-19: User reported the prior fix still fails after a second sale and that simple detail still scrolls.
- 2026-09-19: Mapping confirmed the exact-one predicate causes the list regression. Detail overflow is intrinsic content height, not free-space alignment.
- 2026-09-19: Every ready list now uses the desktop top-aligned density marker; strict simple-detail eligibility remains unchanged.
- 2026-09-19: Simple-detail desktop spacing is compacted without fixed sizing or scroll suppression, and initial correction actions share a wrapping layout hook.
- 2026-09-19: Focused regression coverage now includes two-plus/larger ready lists, action visibility, open-correction density boundaries, and CSS contracts.

## Verification evidence
- `./node_modules/.bin/tsx --test --import ./test/react-dom.ts src/ui/sales/history-flow.test.ts src/ui/sales/history-screen.mounted.test.ts`: 28 tests passed, 0 failed.
- `npm run typecheck:tests`: passed with exit code 0.
- `npm test`: 215 tests passed, 0 failed; existing intentional dialog-validation error logs were emitted during passing tests.
- Independent verifier: `git diff --check` passed; focused History suite passed 28/28; `npm run typecheck:tests` passed. No regressions found. Windows geometry remains a manual validation.
- Native risk assessment was unavailable (empty native output), so the change was independently verified as a high-risk fallback.

## Commit evidence
- `fix(history): preserve compact populated layouts` (work-unit commit on `fix/sparse-content-vertical-density`).

## Next step
Validate on Windows with two-plus sales and a simple detail at 1200×800 and 960×640. Commit/push only with explicit user authorization.
