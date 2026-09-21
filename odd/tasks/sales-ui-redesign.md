# Sales UI redesign

## Objective
Apply the approved Sales/POS visual direction without changing business logic, IPC, persistence, or checkout behavior.

## Approved references
- `docs/design/sales-pos/compact-active-draft.png`
- `docs/design/sales-pos/compact-empty-draft.png`
- Background Sales screen in `docs/design/sales-pos/desktop-checkout.png`
- `docs/design/sales-pos-figma-handoff.md`
- Static source references under `docs/design/sales-pos/reference-source/`

## Scope
First implementation slice only:
- Light persistent shell treatment using existing navigation markup.
- Sales-specific catalog presentation.
- Read-only active and empty draft summaries, with a bounded remove-line action in the active summary.
- Focused mounted tests for observable output and CSS contracts.

## Non-goals
- No checkout markup or checkout selector changes.
- No business-flow, calculation, command, IPC, Rust, SQLite, or persistence changes.
- No external font/icon dependencies.
- No redesign of Inventory, Catalog maintenance, History, Dashboard, Onboarding, or Backup.
- No PR or merge without explicit user authorization.

## Invariants
- ProductBrowser default presentation remains unchanged outside Sales.
- Search sequencing, stale-response guards, pagination, validation, pending locks, and confirmation behavior remain unchanged.
- Summary remains non-editable except for removing a draft line; quantity and price editing remain inside checkout.
- One `h1`, semantic named panels, visible labels, live feedback, 44px targets, focus, reduced-motion, and forced-colors support remain intact.
- Desktop/compact breakpoint remains 961px; sidebar remains 208px/176px.

## Tasks
- [x] T1 Add focused mounted expectations for the approved shell, Sales catalog, and empty/active summaries.
- [x] T2 Implement the Sales-scoped shell/catalog/summary visual slice.
- [x] T3 Run focused and full frontend verification, typecheck, and build.
- [x] T4 Perform structural readback and review the final diff against scope.
- [ ] T5 Validate the rendered desktop and compact result against the approved PNGs in the native app.
- [x] T6 Add an accessible inline remove action to each active summary line and verify it.

## Acceptance criteria
- Sales matches the approved desktop and compact composition closely within the existing React architecture.
- Empty draft shows guidance, zero totals, and disabled checkout.
- Active draft shows non-editable line facts, units, subtotal/total, an accessible remove-line action, and enabled checkout.
- Products already in the draft show a disabled `Agregado` action in Sales.
- Other ProductBrowser callers preserve their current markup and behavior.
- Checkout behavior and markup remain unchanged.

## Progress
- 2026-09-21: User authorized beginning the redesign with the first Sales slice.
- 2026-09-21: User authorized commit and push to the feature branch so the native result can be validated on Windows.
- Read-only mapping identified five edit surfaces and estimated 300–465 changed lines, primarily CSS.
- T1 RED: focused mounted tests failed on the missing light-shell and Sales presentation expectations.
- T2 GREEN: delegated writer implemented the bounded slice in five files; focused mounted tests passed 26/26. Diff reported 140 additions and 33 deletions.
- T3 independent commands passed, but structural readback found that removing the generic late `[data-ui-sale-list] > li` override changed Catalog and Inventory desktop rows. T2 reopened for bounded CSS/test correction.
- T2 correction RED: focused suite failed 1/26 while the generic late override was absent. GREEN: restored the generic two-column override before the more-specific Sales rule; focused suite passed 26/26. Candidate is 174 changed lines.
- T3 re-verification passed every command but found a second locality issue: generic `[data-ui-sale-search]` was changed from two to three columns for Catalog and Inventory. T2 reopened for one more bounded CSS/test correction.
- T2 second correction RED: focused suite failed 1/26 while generic search locality was absent. GREEN: restored generic two-column search and scoped the three-column layout to Sales; focused suite passed 26/26. Candidate is 175 changed lines.

## Verification evidence
- Focused: 26 passed, 0 failed.
- Full frontend: 237 passed, 0 failed.
- Test typecheck: passed.
- Production build: passed with the existing mixed static/dynamic Tauri `core.js` import warning.
- `git diff --check`: passed.
- Final focused: 26 passed, 0 failed.
- Final full frontend: 237 passed, 0 failed.
- Final test typecheck: passed.
- Final production build: passed with the existing mixed static/dynamic Tauri `core.js` import warning.
- Final `git diff --check`: passed.
- Final structural readback: generic Catalog/Inventory search and rows preserved; Sales-only search/row rules are scoped; checkout, flows, IPC, and Rust untouched.
- Candidate: 5 tracked files, 144 insertions, 31 deletions (175 changed lines).
- Work-unit commit: `4ed3df2` (`feat(ui): redesign Sales catalog and summary`).
- Native rendered desktop comparison: substantially aligned; requested follow-up is direct line removal from the summary. Low-contrast inventory alert cue and the intentionally deferred checkout redesign were also identified.
- T6 RED: focused mounted suite failed 2 tests because summary remove controls were absent.
- T6 GREEN: added a summary-scoped 44px `×` control with a unique accessible name, existing pending guard, and existing `remove_product` transition; focused suite passed 21/21.
- T6 independent verification: focused 21/21, full frontend 238/238, test typecheck, and `git diff --check` passed.
- Compact native visual comparison remains pending on Windows.
