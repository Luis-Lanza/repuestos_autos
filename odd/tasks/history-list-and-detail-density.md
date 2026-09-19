# History List and Detail Density

## Objective
Keep a small populated Sales History list and a simple sale detail compact on desktop, while making the return action visibly discoverable and preserving scrolling for genuinely dense audit/correction content.

## Problem
Windows screenshots show that one sale returns History to its stretched dense layout. A simple detail view overflows the viewport by only a small amount, producing an unnecessary page scroll, and `Volver al historial` looks like unstyled text rather than a clear navigation action.

## Decision
Extend sparse density from empty History to exactly one sale. Mark only a provably simple ready detail as sparse. Use alignment-only desktop CSS so detailed/correction states keep normal scrolling. Place the existing return button beside the heading using the existing secondary action style.

## Scope
- One-sale History list compact layout.
- Simple sale detail compact layout.
- Prominent accessible return action in the detail header.
- Focused mounted/style regression coverage for sparse and dense states.

## Non-goals
- No history flow, command, IPC, domain, persistence, or correction behavior changes.
- No hidden/collapsed audit facts.
- No global scrolling change; detailed and open correction states remain scrollable.

## Acceptance criteria
- Exactly one sale keeps filters and row visually contiguous below the heading.
- A simple one-line, one-payment detail without corrections or open actions does not add artificial page height.
- Dense details and active correction forms retain their current scrolling behavior.
- `Volver al historial` remains keyboard-operable, keeps its accessible name, and is visibly a secondary button beside the heading.

## TDD and delivery
- TDD mode: not configured; use focused regression checks after each task.
- Delivery strategy: ask-on-risk; forecast under 400 authored changed lines.
- Commit/push: not authorized by the user.

## Tasks
- [x] T1 Add state-aware sparse markers and detail header action seam. Route: delegated writer (multi-file UI change).
- [x] T2 Add scoped desktop CSS for compact list/detail and visible action alignment. Route: delegated writer (multi-file UI change).
- [x] T3 Add and run focused regression coverage for sparse/dense History states. Route: delegated writer plus independent verification as required.

## Progress
- 2026-09-19: User supplied Windows screenshots showing one populated History sale reintroducing large vertical whitespace and a simple detail with an unnecessary small overflow. User authorized implementation.
- 2026-09-19: Read-only mapping found the list marker only handles `empty`; the simple detail has no density marker; `Volver al historial` is an existing tertiary action before the heading.

## Verification evidence
- `./node_modules/.bin/tsx --test --import ./test/react-dom.ts src/ui/sales/history-screen.mounted.test.ts`: 8 tests passed.
- `npm run typecheck:tests`: passed with no diagnostics.
- `./node_modules/.bin/tsx --test src/ui/sales/history-flow.test.ts`: 20 tests passed.
- `npm test`: 215 tests passed.
- Independent verifier: `git diff --check` passed; mounted History tests passed 8/8; History flow tests passed 20/20; `npm run typecheck:tests` passed. No behavioral regression found. Browser/Tauri visual geometry remains a manual Windows check.
- Native risk assessment was unavailable (empty native output), so the change was independently verified as a high-risk fallback.

## Commit evidence
- `fix(history): compact sparse sales details` (work-unit commit on `fix/sparse-content-vertical-density`).

## Next step
Validate the populated one-sale list and simple detail on Windows at 1200×800 and 960×640. Commit/push only with explicit user authorization.
