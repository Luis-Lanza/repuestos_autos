# History Correction Modals

## Objective
Keep the receipt-style Sales History detail compact by moving return and cancellation preparation from inline expansions into accessible modal workflows.

## Problem
The initial correction actions currently expand return/cancellation forms below the historical receipt, increasing page height and mixing read-only audit facts with operational edits.

## Decision
Return opens an accessible form modal. Cancellation opens an accessible preparation modal, then retains the existing destructive confirmation modal as a second step. Forms stay mutually exclusive and close only on persisted-evidence reload, not command success alone.

## Scope
- Modalize return preparation and submission.
- Modalize cancellation preparation while preserving two-step cancellation confirmation.
- Preserve validation, pending locks, stable request IDs, stale-result guards, recovery, focus trapping/restoration, and history reload evidence.
- Remove inline correction form expansion from detail.
- Add focused reducer, mounted interaction, focus, and style coverage.

## Non-goals
- No changes to return/cancellation command payloads, domain rules, persistence, or historical evidence.
- No one-step cancellation.
- No modal close on transient command success.

## Acceptance criteria
- Starting a return or cancellation does not expand the receipt detail inline.
- Return modal supports existing selection/quantity validation and submission behavior.
- Cancellation preparation modal leads to the existing destructive confirmation, and returning preserves entered state.
- Escape/close restore invoker focus unless pending; pending blocks duplicate submit and dismissal.
- Existing retry/reload/persisted-evidence behavior remains intact.

## TDD and delivery
- TDD mode: not configured; use focused regression checks after each task.
- Delivery strategy: ask-on-risk; forecast under 400 authored changed lines.
- Commit/push: not authorized by the user.

## Tasks
- [x] T1 Add modal state/transitions and form-dialog seam. Route: delegated writer (multi-file UI change).
- [x] T2 Move return/cancellation preparation UI into accessible modals. Route: delegated writer (multi-file UI change).
- [x] T3 Add and run focused async, accessibility, and layout regression coverage. Route: delegated writer plus independent verification as required.

## Progress
- 2026-09-19: User requested modals for return and cancellation so detail remains compact.
- 2026-09-19: User chose retained two-step cancellation: preparation modal, then destructive confirmation.
- 2026-09-19: Mapping found existing async guards and destructive dialog can be preserved; return needs close/modal reducer state and a generic form-dialog seam.
- Correction pass: detail reload failures now convert retained correction intents to recoverable modal errors without treating absent detail as pending; correction forms render independently of the historical-detail snapshot so direct reload failures retain actionable reload and close recovery; confirmation-to-preparation handoff keeps focus in the preparation form.

## Verification evidence
- `npx --no-install tsx --test --import ./test/react-dom.ts src/ui/sales/history-flow.test.ts src/ui/sales/history-screen.mounted.test.ts src/ui/visual-system/confirmation-dialog.mounted.test.ts` passed: 45 tests, 0 failures, including the mounted direct detail-reload failure recovery regression.
- `npm run typecheck:tests` passed.
- `npm test` passed: 219 tests, 0 failures.
- Coverage includes failed detail-reload recovery with an enabled retry, persisted-evidence-only closure, stale completion guards, pending dismissal locks, and focus remaining in cancellation preparation after confirmation reversal.
- Final independent verifier: `git diff --check` passed; focused suite passed 45/45; `npm run typecheck:tests` passed with no findings. Non-failing validation-path TypeError traces were emitted by tests.
- Native risk assessment was unavailable (empty native output), so the change was independently verified as a high-risk fallback.

## Commit evidence
- `feat(history): open corrections in modals` (work-unit commit on `fix/sparse-content-vertical-density`).

## Next step
Validate return and two-step cancellation modals on Windows, including detail-reload failure recovery. Commit/push only with explicit user authorization.
