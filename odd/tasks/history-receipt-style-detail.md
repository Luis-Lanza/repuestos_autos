# History Receipt-Style Detail

## Objective
Make Sales History detail use the compact, receipt-like composition of the confirmed-sale summary while retaining every historical fact and correction workflow.

## Problem
The current historical detail renders original sale, payments, total, empty correction history, and operational actions as a long vertical audit screen. Even simple sales can require page scrolling, unlike the compact confirmed-sale summary.

## Decision
Reuse the confirmed summary's composition, not its component or payload projection: a history-specific desktop wrapper presents original articles as the main area and original payments plus total in a right rail. Correction history and return/cancellation workflows remain expanded, separate siblings below. The layout stacks below 960px.

## Scope
- Receipt-style original history summary at desktop width.
- Main articles region plus payment/total rail.
- Preserve all current historical facts, semantic tables, status, and identity.
- Preserve correction history and correction workflows outside the read-only summary.
- Focused layout and behavior regression coverage.

## Non-goals
- No reuse of confirmation payload projection or sale-confirmed actions.
- No command, flow, IPC, domain, persistence, or correction semantics changes.
- No hidden audit facts, fixed-height clipping, or inaccessible disclosure.

## Acceptance criteria
- Simple historical details visually match the confirmed-sale summary's main-content/rail hierarchy and avoid unnecessary scroll.
- Original articles, payments, total, status, sale ID, and timestamp remain visible and semantically equivalent.
- Returns, cancellation history, and correction actions/forms remain functional and separate.
- The rail stacks below 960px without clipping or loss of table semantics.

## TDD and delivery
- TDD mode: not configured; use focused regression checks after each task.
- Delivery strategy: ask-on-risk; forecast under 400 authored changed lines.
- Commit/push: not authorized by the user.

## Tasks
- [x] T1 Restructure original history summary into receipt-style content and rail seams. Route: delegated writer (multi-file UI change).
- [x] T2 Add desktop/compact responsive layout styling without affecting correction flows. Route: delegated writer (multi-file UI change).
- [x] T3 Add and run focused semantic, layout, correction-flow regression checks. Route: delegated writer plus independent verification as required.

## Progress
- 2026-09-19: User requested that `Ver detalle` use a layout similar to confirmed sale to avoid scroll.
- 2026-09-19: Mapping found `PersistedSaleSummaryView` is composition evidence only; history requires its own projections. Existing `AlignedData`, actions, badges, and feedback are reusable seams.
- 2026-09-19: Added a history-specific original-summary wrapper with articles in the main region and payments/total in a right rail; correction history and workflows remain sibling regions.
- 2026-09-19: Added desktop receipt grid and ≤960px stacked styles with article overflow vocabulary while preserving existing facts and correction behavior.

## Verification evidence
- Focused History mounted/flow tests: passed; 28 tests passed, 0 failed.
- `npm run typecheck:tests`: passed.
- `npm test`: passed; 215 tests passed, 0 failed. The suite emits expected console diagnostics from intentional ConfirmationDialog validation tests.
- No command, flow, IPC, Rust, persistence, or correction semantics changed.
- Independent verifier: `git diff --check` passed; focused History suite passed 28/28; `npm run typecheck:tests` passed. No regression findings; Windows geometry remains a manual validation.
- Native risk assessment was unavailable (empty native output), so the change was independently verified as a high-risk fallback.

## Commit evidence
- `feat(history): present details as receipt` (work-unit commit on `fix/sparse-content-vertical-density`).

## Next step
Validate the receipt-style detail on Windows at 1200×800 and 960×640 with simple and corrected sales. Commit/push only with explicit user authorization.
