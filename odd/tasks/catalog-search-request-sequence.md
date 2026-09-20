# Catalog Search Request Sequence

## Objective
Make Catalog search reliably apply the user’s submitted query and prevent the initial background browse from overwriting it.

## Problem
`ProductBrowser` increments reducer request IDs on each query edit, while `CatalogMaintenanceScreen` uses an independent lower `browseAttempt` sequence. A submitted multi-character query is rejected as stale. The initial post-category-load browse can also use a stale browser closure and supersede a user search.

## Decision
Generate every Catalog browse request ID strictly above the current reducer request ID, following the established Sales pattern. Capture and submit an explicit browser snapshot for each request so background initialization cannot overwrite newer user intent.

## Scope
- Align Catalog browse request identity with `ProductBrowser` state.
- Guard initial-load browse against newer user search intent.
- Add mounted regression coverage for multi-character submit and delayed initial-load race.

## Non-goals
- No catalog query, Rust, IPC, FTS, decoder, payload, accessibility-label, or visual redesign changes.

## Acceptance criteria
- Typing a multi-character Catalog query and pressing Buscar replaces results with the matching response.
- Initial automatic browse cannot replace a newer submitted query.
- Existing stale-response guards continue rejecting genuinely old requests.

## TDD and delivery
- TDD mode: not configured; use focused regression checks after each task.
- Delivery strategy: ask-on-risk; forecast under 400 authored changed lines.
- Commit/push: not authorized by the user.

## Tasks
- [x] T1 Align Catalog request IDs and snapshot browsing intent. Route: delegated writer (multi-file React change).
- [x] T2 Add mounted regression coverage for submitted multi-character and load-race paths. Route: delegated writer (multi-file React change).
- [x] T3 Run focused Catalog and frontend regression checks. Route: delegated writer plus independent verification as required.

## Progress
- 2026-09-19: User reported Catalog Buscar appeared inert.
- 2026-09-19: Mapping found independent request-ID domains; a post-initial-load browse can also overwrite user intent.

## Verification evidence
- `./node_modules/.bin/tsx --test --import ./test/react-dom.ts src/ui/catalog/catalog-maintenance-screen.mounted.test.ts src/ui/catalog/product-browser.test.ts`: passed, 17 tests.
- `npm run typecheck:tests`: passed.
- `npm test`: passed, 230 tests; existing `ConfirmationDialog` validation tests log expected TypeErrors while passing.
- Final independent verifier: diff check passed; focused Catalog suite 17/17; typecheck and full frontend suite 230/230 passed. No regression findings.
- Native risk assessment was unavailable (empty native output), so the change was independently verified as a high-risk fallback.

## Commit evidence
- `fix(catalog): apply submitted searches` (work-unit commit on `fix/catalog-search-request-sequence`).

## Next step
Validate Catalog search on Windows with multi-character terms; commit/push only with explicit user authorization.
