# Apply Progress: Frontend UI/UX Visual System

## Current status

- Authoritative hybrid/OpenSpec status: `define-frontend-ui-ux-visual-system`, apply ready, **4/12** implementation tasks complete after formal W2C verification failed.
- Completed in order: W0 mounted harness, W1 base visual layer, W2A controls/state communication, and replanned W2B Panel + AlignedData.
- The separated W2C candidate is implemented within its assigned files but remains unverified: formal diagnostics reproduced a stale deferred-restoration race across dialog replacement and external focus escape.
- W2C is corrected to unchecked; W3–W9 remain unchecked and were not started. W3 stays blocked until W2C passes.
- The next delivery boundary remains stacked-to-main PR2C remediation only. Action context is repo-local at `/home/luis/velay/repuestos_autos`; warnings: none.

## Historical failed combined W2B candidate

- `structure.ts`: `Panel` names a semantic section; `AlignedData` renders captioned table structure, caller values, alignment/kind metadata, and fails fast when a row does not match its columns.
- `confirmation-dialog.ts`: controlled `restore | cancellation` dialog with accessible name/description, ordinary `Volver` or optional correction focus, ordinary two-button Tab/Shift+Tab wrapping, idle Escape/cancel, pending action lock, and invoker restoration after cancellation, parent-driven close, or dialog unmount when the opener remains. Formal verification found that StrictMode effect replay leaves focus outside, hidden controls are treated as focusable and can break reverse containment, and an initially pending dialog has no valid built-in initial focus target.
- `structure-dialog.mounted.test.ts`: covers product/SKU/stock/sale/date/amount/lifecycle/correction facts, exact value/order preservation, mismatch failure, both permitted purposes, dialog semantics, focus paths, and pending duplicate prevention.
- `styles.css`: adds only W2B panel/table/dialog selectors, responsive labeled-row source rules, and forced-colors boundaries. No rendered 960px viewport was observed or claimed; exact/native geometry remains W9.

## TDD Cycle Evidence

| Stage | Evidence |
|---|---|
| RED | Original public-seam run failed because W2B modules were absent; recovery also exposed a test-fixture opener replacement before the fixture was corrected. |
| GREEN | The implemented suite passes all five authored W2B mounted behaviors, including ordinary parent close/unmount restoration; formal verification found missing focus-lifecycle cases, so this is not a W2B PASS. |
| TRIANGULATE | Controls 3/3, base styles 1/1, mounted Inventory 3/3, full frontend 74/74, TypeScript, and build passed; read-only diagnostics reproduced StrictMode, hidden-focusable, and initially-pending focus failures. |
| REFACTOR | Preserved the partial candidate and extended its public-interface assertions without changing production interfaces or screen behavior. |

## Observed commands

- W2B focused before recovery edit: PASS 4/4; the next focused run exposed the fixture issue at 4/5, then the final `npm test` passed W2B 5/5 within 74/74.
- `npm run test:mounted`: PASS 3/3. Controls mounted: PASS 3/3. Base styles mounted: PASS 1/1.
- `./node_modules/.bin/tsc --noEmit`: PASS. `npm run build`: PASS, 51 modules; existing mixed Tauri import warning only.
- `git diff --check`: PASS after the reconciled artifact write.

## Historical boundary, files, and task state

- The failed combined boundary was **349/400 changed lines** = structure 48 + dialog 96 + mounted test 142 + CSS 33 + checkbox 2 + evidence 28.
- Its W2B checkbox was corrected to `[ ]` after formal verification failed; that combined candidate is no longer present in the repository worktree.
- Formal `verify-report.md` remains read-only historical failure evidence and was not changed during recovery.

## Replanned W2B recovery

- Before removal, exact copies of the dialog implementation, combined mounted test, and mixed production stylesheet were saved under `/tmp/visual-system-w2c-candidate-snapshot/` with `SHA256SUMS`.
- `structure.ts` is the current Panel and AlignedData implementation.
- `structure.mounted.test.ts` is the only current structural mounted suite and loads production CSS.
- `confirmation-dialog.ts` and `structure-dialog.mounted.test.ts` are absent from the repository candidate.
- Dialog-specific backdrop, modal, action-layout, and forced-colors CSS was removed; Panel, table, responsive-label, and structural forced-colors rules remain.
- W2C was not implemented or remediated, and no W3 work began.

## Replanned TDD Cycle Evidence

| Stage | Evidence |
|---|---|
| RED | The first separated mounted run failed because jsdom CSSOM omitted the `content` declaration from serialized media-rule text, exposing an invalid test observation. |
| GREEN | The assertion was narrowed to the loaded production stylesheet source; all three public structural behaviors then passed. |
| TRIANGULATE | The separated suite passed twice; controls, base styles, mounted Inventory, full frontend, TypeScript, build, scope, and line checks passed. |
| REFACTOR | Dialog imports/assertions/selectors were removed while caller values, logical order, native table semantics, and the small structural interface remained unchanged. |

## Replanned verification and boundary

- Focused structure: PASS 3/3 twice after RED correction.
- Controls: PASS 3/3. Base styles: PASS 1/1. Mounted Inventory: PASS 3/3.
- Full frontend: PASS 72/72. TypeScript: PASS. Build: PASS with the pre-existing mixed Tauri import warning.
- `git diff --check`: PASS. Snapshot hash verification, dialog-absence grep, allowed-surface scope check, and line-budget check: PASS.
- Responsive evidence proves source `data-label` metadata and the loaded 960px media rule only; it does not prove native 960px geometry.
- Exact PR2B boundary: **190/400 changed lines** = structure 48 + mounted test 79 + CSS 28 + checkbox 2 + incremental evidence 33. Headroom: **210**.
- No screen, control module, flow, command, IPC, Rust, persistence, package, lockfile, asset, W2C implementation, or W3 implementation file changed.
- Persisted task state: W2B `[x]`; W2C and W3–W9 `[ ]`.

## Remaining implementation tasks

- [ ] Implement and verify the type-restricted ConfirmationDialog after W2B. <!-- sdd-owner: implementation -->
- [ ] Implement and verify the persistent shell, navigation presentation, active-location cue, and stock-exception cue after W2. <!-- sdd-owner: implementation -->
- [ ] Implement and verify Sales/POS and confirmed-sale summary compositions through existing sale flow state after W3. <!-- sdd-owner: implementation -->
- [ ] Implement and verify Inventory composition, whole-unit operation states, projections, and alert priority after W4. <!-- sdd-owner: implementation -->
- [ ] Implement and verify Catalog maintenance master-detail, lifecycle, dynamic fields, and recovery states after W5. <!-- sdd-owner: implementation -->
- [ ] Implement and verify History list, persisted detail, inline return, and destructive cancellation modal after W6. <!-- sdd-owner: implementation -->
- [ ] Implement and verify shared-shell continuity for Product onboarding and Backup/Restore after W7. <!-- sdd-owner: implementation -->
- [ ] Produce and verify the final exact-size evidence bundle and invariant/accessibility audit after W8. <!-- sdd-owner: implementation -->

## W2C ConfirmationDialog increment

- Status consumed/produced: authoritative OpenSpec change `define-frontend-ui-ux-visual-system`; proposal/spec/design/tasks present; apply ready at 4/12 before this slice. Repo-local action context is `/home/luis/velay/repuestos_autos`, restricted to the authorized W2C edit surfaces; warnings: none.
- Delivery boundary: stacked-to-main PR2C after verified W2B commit `40f81d361e03f4856f96b5c31b942cdd83075d85`; W3 was not started.
- Completed persisted task: `[x] Implement and verify the type-restricted ConfirmationDialog after W2B.`
- Files changed: `confirmation-dialog.ts`, `confirmation-dialog.mounted.test.ts`, dialog-only selectors in `styles.css`, this incremental record, and the W2C task checkbox.
- Public interface: exact `restore | cancellation` purpose restriction and controlled open/pending/workflow/content callbacks; `Volver` is the safe action.
- Focus strategy: real closed→open invoker capture, live eligibility filtering, dialog-container fallback, and deferred cleanup canceled by StrictMode replay before connected-only restoration.

### W2C TDD Cycle Evidence

| Stage | Evidence |
|---|---|
| RED | Three independent snapshot-candidate diagnostics failed for StrictMode replay, a hidden trap endpoint, and initially pending focus. |
| GREEN | The separated mounted suite passed 10/10 after lifecycle, eligibility, containment, pending lock, and restoration behavior was implemented. |
| TRIANGULATE | Dialog passed 10/10 twice; structure 3/3, controls 3/3, base styles 1/1, Inventory mounted 3/3, and full frontend 82/82 passed. TypeScript, build, diff, scope, and snapshot-integrity checks passed. |
| REFACTOR | Focus eligibility and restoration were centralized behind the unchanged small dialog interface; no screen adopted the module. |

- Commands: both required focused dialog runs; focused structure/controls/base-style runs; `npm run test:mounted`; `npm test`; `./node_modules/.bin/tsc --noEmit`; `npm run build`; and `git diff --check` all passed.
- Deviation from design: none. The caller continues to own reason, acknowledgement, open, pending, and workflow state.
- Exact PR2C boundary: **316/400 changed lines** = source 123 + mounted tests 151 + dialog CSS 8 + checkbox 2 + incremental evidence 32. Headroom: **84**.
- Persisted task state after reconciliation: 5/12 complete; W2C `[x]`; W3–W9 remain `[ ]`.

### Remaining implementation tasks

- [ ] Implement and verify the persistent shell, navigation presentation, active-location cue, and stock-exception cue after W2C. <!-- sdd-owner: implementation -->
- [ ] Implement and verify Sales/POS and confirmed-sale summary compositions through existing sale flow state after W3. <!-- sdd-owner: implementation -->
- [ ] Implement and verify Inventory composition, whole-unit operation states, projections, and alert priority after W4. <!-- sdd-owner: implementation -->
- [ ] Implement and verify Catalog maintenance master-detail, lifecycle, dynamic fields, and recovery states after W5. <!-- sdd-owner: implementation -->
- [ ] Implement and verify History list, persisted detail, inline return, and destructive cancellation modal after W6. <!-- sdd-owner: implementation -->
- [ ] Implement and verify shared-shell continuity for Product onboarding and Backup/Restore after W7. <!-- sdd-owner: implementation -->
- [ ] Produce and verify the final exact-size evidence bundle and invariant/accessibility audit after W8. <!-- sdd-owner: implementation -->

## W2C formal verification correction

- Formal verdict: **FAIL**. The required focused suites passed 10/10 twice and the complete required command set stayed green, but independent lifecycle diagnostics found two critical focus-containment defects.
- Async cleanup race: replacing an open dialog instance synchronously leaves the old instance's zero-delay cleanup live; after the replacement focuses itself, the old timer restores its connected invoker and moves focus outside the still-open replacement.
- External focus escape: the trap is attached only through the dialog's bubbling `onKeyDown`; programmatically moving focus to an outside control leaves focus outside, and the next Tab is not intercepted by the dialog.
- StrictMode replay itself remained contained after a deferred tick. Eligibility filtering, safe initial fallback, idle/pending action locks, ordinary connected-only restoration, accessible semantics for valid content, caller-owned state, Action reuse, production CSS loading, and assigned-scope isolation passed inspection.
- Candidate boundary independently reconstructed at **316/400 changed lines** = source 123 + mounted tests 151 + dialog CSS 8 + candidate checkbox transition 2 + incremental apply evidence 32; headroom **84**. Verification bookkeeping does not expand the implementation candidate boundary.
- Task correction: W2C is `[ ]`; current task state is **4/12** complete. W3–W9 remain `[ ]`; no W3 work began.
- Delivery correction: PR2C is **not ready**. The next action is W2C-only remediation and re-verification; sync/archive remain out of scope and not ready.

## W2C critical-focus remediation
- RED reproduced 10/13 passing: replacement cleanup theft, external Tab escape, and invalid content values failed.
- GREEN/TRIANGULATE: dialog 13/13 passed twice fresh; structure 3/3, controls 3/3, base styles 1/1, mounted Inventory 3/3, and full frontend 85/85 passed.
- TypeScript, build, diff, scope, ownership, and boundary checks passed; only the existing mixed Tauri-import build warning remains.
- Per-ownerDocument connected-dialog ownership suppresses stale restoration; a cleaned-up capture listener contains internal and externally displaced Tab focus.
- The seam requires a nonblank string title and nonblank string or React element description without deep element-text introspection.
- Exact PR2C boundary: **362/400** = source 133 + mounted tests 179 + dialog CSS 8 + checkbox 2 + prior evidence 32 + remediation evidence 8; headroom **38**.
- Persisted state: W2C `[x]`, 5/12 complete; W3–W9 remain `[ ]`, and W3 was not started.

## W3 persistent shell increment
- Status consumed: authoritative hybrid/OpenSpec apply-ready state at 5/12; repo-local action context restricted to the authorized W3 surfaces with no warnings. The resolved delivery path is stacked-to-main PR3 after merged/post-merge verified W2C `576867a5763af8c07b28c5c71ea197cbdc600022`.
- Completed persisted task: `[x] Implement and verify the persistent shell, navigation presentation, and active-location cue after W2C; defer the truthful stock-exception cue to W5.`
- Interface/refactor: new `AppShell` accepts only `screen`, `onNavigate`, and `children`; `App` now selects private screen content once and wraps it in the persistent shell. Four App-owned temporary Sales-return wrappers were removed; Onboarding's existing `onBack` remains intact.
- Files changed: `src/ui/app.ts`, new `src/ui/app-shell.ts`, new `src/ui/app-shell.mounted.test.ts`, shell-only rules in `src/ui/styles.css`, task checkbox, and this cumulative progress artifact.
- Invariants: `SCREEN`, `NAVIGATION_ACTION`, `screenAfter`, initial Sales, screen mount/unmount, child-owned `<main>`, and English internal screen copy remain behaviorally unchanged. Inventory navigation is neutral; no fetch, poll, cache, badge, count, helper, warning, or status was added.

### W3 TDD Cycle Evidence
| Stage | Evidence |
|---|---|
| RED | The shell public-seam command failed because `app-shell.ts` did not exist. |
| GREEN | The focused mounted suite passed 4/4 for identity/navigation, action emissions/current location, complete transitions, persistent App navigation/focus, neutral Inventory, and CSS source contracts. |
| TRIANGULATE | Dialog 13/13, structure 3/3, controls 3/3, base styles 1/1, mounted Inventory 3/3, and full frontend 89/89 passed. TypeScript initially exposed required-children typing at the React `createElement` seam; after correction, focused 4/4, TypeScript, build, and diff checks passed. |
| REFACTOR | A type-only app import avoids the runtime cycle, while navigation values remain type-checked against the preserved public constants. |

- Responsive evidence is source-contract only: 208px default sidebar and the 176px `max-width: 960px` override are asserted without claiming native 960px geometry.
- Exact PR3 boundary: **274/400 changed lines** = App 54 + shell 35 + mounted test 99 + shell CSS 56 + checkbox 2 + incremental progress 28. Headroom: **126**.
- Deviation from design: the W3-specific stock cue is intentionally neutral per the maintainer amendment; truthful Inventory-owned alert state remains deferred to W5.
- Persisted task state: 6/12 complete. W4–W9 remain unchecked; W4 was not started.

### Remaining implementation tasks
- [ ] Implement and verify Sales/POS and confirmed-sale summary compositions through existing sale flow state after W3. <!-- sdd-owner: implementation -->
- [ ] Implement and verify Inventory composition, whole-unit operation states, projections, alert priority, and the scoped truthful stock-exception cue after W4. <!-- sdd-owner: implementation -->
- [ ] Implement and verify Catalog maintenance master-detail, lifecycle, dynamic fields, and recovery states after W5. <!-- sdd-owner: implementation -->
- [ ] Implement and verify History list, persisted detail, inline return, and destructive cancellation modal after W6. <!-- sdd-owner: implementation -->
- [ ] Implement and verify shared-shell continuity for Product onboarding and Backup/Restore after W7. <!-- sdd-owner: implementation -->
- [ ] Produce and verify the final exact-size evidence bundle and invariant/accessibility audit after W8. <!-- sdd-owner: implementation -->

## W4A Stage 1 — pure Sales foundations

- Status consumed/produced: authoritative hybrid/OpenSpec change `define-frontend-ui-ux-visual-system`; proposal/spec/design/tasks/apply-progress present; apply ready at **6/13** implementation tasks. Repo-local action context is `/home/luis/velay/repuestos_autos`, restricted to the seven authorized Stage 1 surfaces; warnings: none. Delivery is stacked-to-main PR4A with the approved `size:exception` ceiling of 1,050 changed lines.
- Stage status: Stage 1 objectives are complete, but W4A is intentionally **unchecked and incomplete**. No persisted task checkbox changed; W4B/W5 were not started.
- Interfaces/transitions: deterministic `formatBs`, strict `parseOptionalBs`, checked effective-price/subtotal/total functions, typed catalog discovery with monotonic request identity and stale-completion rejection, stale-line removal cleanup, and structured product/immutable-price/stock/availability projection. Acknowledged price affects draft calculations only; captured price/revision remain unchanged.
- Files changed: `sale-flow.ts`, `sale-flow.test.ts`, `catalog-result.ts`, `catalog-result.test.ts`, minimal compile-only adaptation in `sale-screen.ts`, and this cumulative artifact. Persisted summary, styles, mounted Sales tests, commands/IPC/Rust/persistence, and tasks remain unchanged.

### Stage 1 TDD Cycle Evidence
| Stage | Evidence |
|---|---|
| RED | Focused failures independently exposed missing money formatting/parsing, checked arithmetic, catalog race/state transitions, stale-line cleanup, and structured catalog projection. |
| GREEN | `catalog-result.test.ts` passed 3/3 and `sale-flow.test.ts` passed 12/12; combined focused execution passed 15/15. |
| TRIANGULATE | `npm test` passed 99/99; TypeScript, production build, and diff check passed. Build retained the existing mixed static/dynamic Tauri import warning. |
| REFACTOR | Pure arithmetic and presentation behavior stays behind small interfaces; command and persisted-summary semantics remain unchanged. |

- Exact cumulative W4A Stage 1 boundary: **440/1,050 changed lines** = Sales source/tests **413** (394 additions + 19 deletions) + incremental progress evidence **27** additions. Remaining headroom: **610**.
- Deviation from design: mounted composition, Spanish payment/search orchestration, CSS/reflow, and confirmation lifecycle wiring are intentionally deferred to Stage 2; existing screen output changes only enough to consume the structured projection.

### Remaining implementation tasks (13-task plan reconciled)
- [ ] Implement and verify the Sales draft and confirmation lifecycle after W3 within the approved bounded size exception. <!-- sdd-owner: implementation -->
- [ ] Implement and verify the persisted confirmed-sale summary after W4A, with no size exception. <!-- sdd-owner: implementation -->
- [ ] Implement and verify Inventory composition, whole-unit operation states, projections, alert priority, and the scoped truthful stock-exception cue after W4B. <!-- sdd-owner: implementation -->
- [ ] Implement and verify Catalog maintenance master-detail, lifecycle, dynamic fields, and recovery states after W5. <!-- sdd-owner: implementation -->
- [ ] Implement and verify History list, persisted detail, inline return, and destructive cancellation modal after W6. <!-- sdd-owner: implementation -->
- [ ] Implement and verify shared-shell continuity for Product onboarding and Backup/Restore after W7. <!-- sdd-owner: implementation -->
- [ ] Produce and verify the final exact-size evidence bundle and invariant/accessibility audit after W8. <!-- sdd-owner: implementation -->

## W4A Stage 2 — mounted Sales adoption and completion

- Status consumed/produced: authoritative OpenSpec change `define-frontend-ui-ux-visual-system`; proposal/spec/design/tasks/apply-progress were present, apply was ready at 6/13, and repo-local action context was restricted to the authorized W4A surfaces with no warnings. The resolved path is stacked-to-main PR4A with the approved `size:exception` ceiling of 1,050 lines.
- Completed persisted task: `[x] Implement and verify the Sales draft and confirmation lifecycle after W3 within the approved bounded size exception.` W4B and W5–W9 remain unchecked and were not started.
- Files changed across W4A: `sale-screen.ts`, new `sale-screen.mounted.test.ts`, `sale-flow.ts`/test, `catalog-result.ts`/test, Sales-only rules in `styles.css`, this cumulative progress artifact, and the W4A checkbox.
- Behavior: Spanish discovery/cart/payment/summary regions use shared Action, Field, Feedback, Badge, and Panel interfaces; exact product/SKU/category/stock/Bs facts, whole quantities, checked subtotals/total, immediate idle discard, and cash/QR parsing retain integer-centavo payloads and immutable captured facts.
- Race guards: monotonic search attempts reject reverse-order and post-unmount completion; a synchronous ref blocks same-turn duplicate confirmation; confirmation attempt identity and mounted state reject obsolete/post-unmount completion. Pending disables confirmation and discard, so the mounted suite asserts that explicit safeguard instead of inventing a pending-discard race.
- Stale handling shows old/current Bs values, blocks confirmation, requires exact `Aceptar precio actual`, stores current price/revision only in acknowledgement fields, and retries with the same request UUID. Native messages are never rendered; public response codes select bounded Spanish feedback.
- Responsive evidence is source-contract only: default four-region grid and `max-width: 960px` single-column DOM order use `minmax(0, 1fr)`/`min-inline-size: 0`; no native viewport geometry is claimed. The existing confirmed-result branch/handoff remains legacy for W4B.

### W4A Stage 2 TDD Cycle Evidence
| Stage | Evidence |
|---|---|
| RED | The new production-CSS mounted suite first failed 8/8 against English draft markup and missing discovery, Bs, lifecycle, validation, stale, and race behavior. |
| GREEN | Eight dense public-behavior scenarios pass for discovery states/races, cart, payment envelopes, malformed focus, duplicate/pending, localized failure, stale retry, unmount safety, and the existing result handoff. |
| TRIANGULATE | Mounted Sales passed 8/8 twice after final strengthening; catalog 3/3, flow 12/12, shell 4/4, dialog 13/13, structure 3/3, controls 3/3, base styles 1/1, mounted Inventory 3/3, and full frontend 107/107 passed. TypeScript, build, and diff checks passed. |
| REFACTOR | Presentation parsing, checked arithmetic, reducers, shared visual interfaces, and async guards remain at their existing seams; no command adapter, persisted summary, backend, or other screen changed. |

- Cumulative W4A boundary: **902/1,050 changed lines** = Stage 1 **440** + Stage 2 **462** (Sales screen delta 302, flow/tests delta 8, mounted suite 111, Sales CSS 14, task checkbox 2, progress evidence 25). Remaining headroom: **148**.
- Remaining implementation tasks:
  - [ ] Implement and verify the persisted confirmed-sale summary after W4A, with no size exception. <!-- sdd-owner: implementation -->
  - [ ] Implement and verify Inventory composition, whole-unit operation states, projections, alert priority, and the scoped truthful stock-exception cue after W4B. <!-- sdd-owner: implementation -->
  - [ ] Implement and verify Catalog maintenance master-detail, lifecycle, dynamic fields, and recovery states after W5. <!-- sdd-owner: implementation -->
  - [ ] Implement and verify History list, persisted detail, inline return, and destructive cancellation modal after W6. <!-- sdd-owner: implementation -->
  - [ ] Implement and verify shared-shell continuity for Product onboarding and Backup/Restore after W7. <!-- sdd-owner: implementation -->
  - [ ] Produce and verify the final exact-size evidence bundle and invariant/accessibility audit after W8. <!-- sdd-owner: implementation -->

## W4A formal verification correction

- Formal verdict: **FAIL** despite every prescribed repository command passing. Independent rendered diagnostics found three critical lifecycle/accessibility defects not covered by the green W4A suite.
- Pending draft lock is incomplete: only `Confirmar venta` and `Descartar borrador` are disabled. Search, `Agregar`, quantity, `Quitar`, cash, and QR remain enabled, and the affected Sales region has no busy state. Removing the submitted line before a stale response leaves an empty cart, disabled confirmation, and no acknowledgement action; other edits can change the intent associated with the retained UUID.
- The visible Spanish `Ventas` H1 is overridden with accessible name `Confirm sale` solely to satisfy the merged shell test's legacy English heading query. This mismatched heading is rejected; assistive technology does not receive the visible Spanish title.
- A stale response leaves focus on the now-disabled `Confirmar venta` button instead of moving it to the stale status or exact `Aceptar precio actual` recovery action.
- Task correction: W4A is `[ ]`; current implementation task state is **6/13**. W4B–W9 remain `[ ]` and deferred. W4A requires remediation and re-verification before W4B.
- The submitted candidate boundary remains independently reconstructed at **902/1,050 changed lines** = 848 implementation/test/CSS lines + 2 checkbox-transition lines + 52 apply-evidence lines; verification and correction bookkeeping do not expand that submitted implementation boundary.

## W4A critical-defect remediation
- Status: authoritative hybrid/OpenSpec apply-ready remediation at 6/13; repo-local allowed surfaces were supplied with no warnings; stacked-to-main and the ≤1,050 exception remain resolved.
- Completed persisted task: `[x] Implement and verify the Sales draft and confirmation lifecycle after W3 within the approved bounded size exception.`
- Remediation: confirmation synchronously locks search/add/quantity/remove/payment/discard/confirm mutation, marks the draft main busy, restores exact `Ventas` heading semantics, and focuses the connected current stale acknowledgement after render.
- Evidence: mounted Sales passed 8/8 twice after RED; catalog 3/3, flow 12/12, shell 4/4, dialog 13/13, structure 3/3, controls 3/3, base styles 1/1, mounted Inventory 3/3, full frontend 107/107, TypeScript, build, and diff check passed.
- Exact cumulative W4A boundary: **931/1,050 changed lines** = prior 902 + 14 implementation/test assertion lines + 15 remediation-evidence lines. Headroom: **119**.
- Deviations: none; payload, UUID retry, navigation, commands, IPC, Rust, persistence, W4B, and W5 remain unchanged.
- Remaining implementation tasks:
  - [ ] Implement and verify the persisted confirmed-sale summary after W4A, with no size exception. <!-- sdd-owner: implementation -->
  - [ ] Implement and verify Inventory composition, whole-unit operation states, projections, alert priority, and the scoped truthful stock-exception cue after W4B. <!-- sdd-owner: implementation -->
  - [ ] Implement and verify Catalog maintenance master-detail, lifecycle, dynamic fields, and recovery states after W5. <!-- sdd-owner: implementation -->
  - [ ] Implement and verify History list, persisted detail, inline return, and destructive cancellation modal after W6. <!-- sdd-owner: implementation -->
  - [ ] Implement and verify shared-shell continuity for Product onboarding and Backup/Restore after W7. <!-- sdd-owner: implementation -->
  - [ ] Produce and verify the final exact-size evidence bundle and invariant/accessibility audit after W8. <!-- sdd-owner: implementation -->

## W4A delivery and post-merge evidence

- The formal W4A PASS remains the pre-delivery verification recorded in `verify-report.md`; its bounded accounting remains **931/1,050 changed lines** with 119 lines headroom.
- Issue **#140** was approved and closed. PR **#141** merged to `master` as `d37a6c99072becf5bc60415e7ecec98ec7391867` at `2026-09-02T12:49:30Z`.
- The merged implementation scope is exactly eight files with **623 insertions and 239 deletions** (862 product/test/CSS lines); delivery bookkeeping does not change the formal 931-line W4A boundary.
- Post-merge verification passed: Sales mounted **8/8 twice**, catalog **3/3**, flow **12/12**, `test:mounted` **3/3**, and full frontend **107/107**, totaling **141/141 command-reported test executions**. TypeScript, build, diff, and status checks passed; the existing Vite static/dynamic import warning was the only warning. GitHub reported **0 checks**.
- `master`, local `origin/master`, and live `origin/master` all resolve to `d37a6c99072becf5bc60415e7ecec98ec7391867`. The only remaining worktree entry is the untracked eight-file OpenSpec change directory.
- W4A is **integrated**. Persisted task state remains **7/13 complete**; W4B is unblocked but was not authorized or applied, and W5–W9 remain deferred.
- Status consumed/produced: authoritative hybrid/OpenSpec, apply ready at 7/13, repo-local workspace `/home/luis/velay/repuestos_autos`, allowed edit root the repository, warnings none. Next authorization gate: explicit W4B apply authorization under its normal 400-line cap.

## W4B persisted confirmed-sale summary
- Status consumed: authoritative hybrid/OpenSpec, apply ready at 7/13; repo-local action context restricted to the five product/test surfaces plus W4B checkbox/progress, warnings none. Resolved delivery path: stacked-to-main PR4B under the normal 400-line cap after integrated W4A `d37a6c9`.
- Completed persisted task: `[x] Implement and verify the persisted confirmed-sale summary after W4A, with no size exception.` State is now **8/13**; W5–W9 remain unchecked and untouched.
- Files changed: `persisted-summary.ts`/test, minimum summary handoff in `sale-screen.ts`, summary cases in `sale-screen.mounted.test.ts`, summary-only `styles.css`, W4B checkbox, and this cumulative evidence.
- Behavior: confirmation snapshots command-result primitives before rendering; Spanish read-only tables preserve persisted IDs/names/SKUs/whole quantities/unit prices/subtotals, cash/QR facts, timestamp, and total. Current catalog/draft/result-fixture mutation cannot change the rendered snapshot. `Nueva venta` is the sole control and resets through the existing discard handoff.
- RED: focused public and mounted seams failed for missing semantic summary export, English/edit-like legacy output, decimal-point money, missing subtotals, and absent persistence isolation.
- GREEN: persisted summary passed 2/2 and mounted Sales passed 9/9.
- TRIANGULATE: both focused suites passed twice; catalog 3/3, flow 12/12, shell 4/4, mounted Inventory 3/3, and full frontend 108/108 passed. TypeScript, build, and diff checks passed.
- REFACTOR: projection owns a deep primitive snapshot and rendering reuses `AlignedData`/`Action`; command payload, UUID, lifecycle reducer, navigation, IPC, Rust, and persistence remain unchanged.
- Exact W4B boundary: **256/400 changed lines** = product/tests/CSS 237 + task checkbox 2 + progress evidence 17. Headroom: **144**. Responsive evidence is the authored 1200/default and 960 labeled-row source contract; no native geometry is claimed.
- Deviation: none. Warning: the pre-existing Vite mixed static/dynamic Tauri import warning remains.
- Remaining exact tasks; next formal gate is `sdd-verify` for W4B before W5 apply:
  - [ ] Implement and verify Inventory composition, whole-unit operation states, projections, alert priority, and the scoped truthful stock-exception cue after W4B. <!-- sdd-owner: implementation -->
  - [ ] Implement and verify Catalog maintenance master-detail, lifecycle, dynamic fields, and recovery states after W5. <!-- sdd-owner: implementation -->
  - [ ] Implement and verify History list, persisted detail, inline return, and destructive cancellation modal after W6. <!-- sdd-owner: implementation -->
  - [ ] Implement and verify shared-shell continuity for Product onboarding and Backup/Restore after W7. <!-- sdd-owner: implementation -->
  - [ ] Produce and verify the final exact-size evidence bundle and invariant/accessibility audit after W8. <!-- sdd-owner: implementation -->
## W4B formal verification correction
- Formal verdict: **FAIL**. Every prescribed repository command passed, but an independent production-contract diagnostic proved that the authoritative persisted timestamp is copied and rendered verbatim (`2026-08-14 10:42:00` from the SQLite-shaped contract) rather than being presented in the required stable Spanish-facing form such as `14/08/2026, 10:42`.
- Production otherwise passed direct inspection and diagnostics for a deep primitive snapshot, deliberately non-derived persisted subtotal/total/payment facts, authoritative zero and absent-payment handling, unavailable wording, sole `Nueva venta` action, clean reset, semantics, source reflow, and unchanged W4A/command/backend scope.
- Test sensitivity warning: existing W4B tests assert the raw timestamp, count only buttons for the sole-action check, omit several forbidden-action terms/link roles, and do not mutate every rendered nested field or exercise zero/absent persisted payments through the mounted summary.
- Task correction: W4B is `[ ]`; current implementation task state is **7/13**. W5–W9 remain `[ ]` and deferred. W4B needs timestamp remediation and strengthened focused evidence before re-verification; W5, delivery, sync, and archive are not ready.
- Exact submitted candidate boundary remains **256/400 changed lines** = independently measured product/tests/CSS **237** + W4B checkbox transition **2** + W4B apply evidence **17**; verification/correction bookkeeping does not expand the submitted candidate. Candidate diff SHA-256: `ea8161dc143dbc63c28d0c8f1edea9867a4ae1a067fc1fbf5684af5bf210c162`.
## W4B verification remediation
- Status: authoritative hybrid/OpenSpec W4B-only remediation at 7/13; repo-local allowed surfaces and stacked-to-main normal-cap path were resolved with no action-context warning.
- Completed persisted task: W4B restored to `[x]`; state is 8/13, while W5–W9 remain unchecked and untouched.
- Critical fixed: SQLite and RFC3339 timestamps preserve source wall-clock fields and render deterministically as `DD/MM/YYYY, HH:mm`; exact SQLite example is `2026-08-14 10:42:00` → `14/08/2026, 10:42`.
- Warnings strengthened: mounted role-wide checks allow only `Nueva venta` and reject edit/print/share/refund/receipt controls; nested line/payment mutation, authoritative zeros, and empty payments are covered.
- RED/GREEN: focused projection tests failed 3/4 before the formatter, then passed 4/4; mounted Sales passed 9/9 twice after strengthening.
- Commands: all prescribed focused, mounted, full 110/110, TypeScript, build, and diff checks passed; only the existing Vite mixed-import warning remains.
- Exact cumulative W4B boundary: **287/400** = prior 256 + remediation product/tests 23 + evidence 8; headroom **113**. No W4A command/payload/UUID/lifecycle, backend, verify-report, W5, or delivery surface changed.

## W4B remediation formal verification correction
- Formal verdict: **FAIL**. All prescribed commands passed, and valid SQLite/RFC3339 wall-clock examples are deterministic across host timezones, but the formatter accepts impossible calendar/clock components and reformats them as date-looking output (`2026-02-30 10:42:00` → `30/02/2026, 10:42`; invalid month/hour/minute/offset values behave similarly).
- Unrecognized input is preserved verbatim, including an independently observed 10,000-character string. Because `confirm-sale.ts` trusts the native response without runtime decoding, this unbounded fallback is not an acceptable validation or presentation guarantee; it avoids current-time substitution but can label arbitrary native text as `Fecha y hora`.
- The prior interactive-role, nested mutation, non-derived arithmetic, authoritative-zero, and empty-payment warnings are remediated and pass direct inspection, focused/mounted evidence, and independent diagnostics.
- Task correction: W4B is `[ ]`; state is **7/13**. W5–W9 remain unchecked and deferred. W4B needs bounded calendar/clock validation plus a non-fabricating bounded fallback and focused malformed/fraction/offset evidence before re-verification.
- Exact implementation candidate remains **287/400 changed lines** = product/tests/CSS 260 + W4B checkbox transition 2 + original W4B evidence 17 + remediation evidence 8; formal verification bookkeeping is excluded. W5, delivery, sync, and archive are not ready.
## W4B second verification remediation
- Status: authoritative OpenSpec W4B-only apply remediation at 7/13; supplied repo-local edit roots and stacked normal-cap path were safe and resolved.
- Completed persisted task: W4B restored to `[x]` (8/13); W5–W9 remain unchecked and untouched.
- Gregorian validation rejects impossible month/day/leap-year and clock components; RFC3339 numeric offsets require hour ≤23 and minute ≤59.
- Every malformed, unrecognized, or oversized timestamp maps to fixed `Fecha no disponible`; valid SQLite/ISO inputs preserve wall-clock fields without conversion.
- RED failed 3/4 at Feb 30; GREEN passed 4/4 twice with compact malformed, leap-day, SQLite, Z/offset, and 10,000-character cases.
- TRIANGULATE passed Sales 9/9 twice, catalog 3/3, flow 12/12, shell 4/4, mounted 3/3, full frontend 110/110, TypeScript, build, diff, scope, status, and 36 independent timezone diagnostics.
- Exact cumulative W4B boundary: **326/400** = prior 287 + implementation/test correction 31 + evidence 8; headroom **74**. No design deviation, W4A regression, verify-report edit, W5 work, or delivery action.

## W4B second-remediation formal verification correction
- Formal verdict: **FAIL**. Every prescribed command passed, Gregorian date/month/leap and clock validation passed, fixed fallback containment passed, and valid wall-clock output was timezone-independent; however, independent diagnostics proved impossible numeric offsets are still accepted and formatted as plausible dates.
- Exact failures: `+14:01`, `-14:01`, `+15:00`, and `-23:59` all produced `14/08/2026, 10:42` instead of exact `Fecha no disponible`. The valid boundary `+14:00` and `-14:00` passed.
- Test-sensitivity gap: the focused suite rejects only `+24:00` and invalid offset minutes, so it cannot detect values above the real `±14:00` bound or the rule that offset hour 14 requires minute 00.
- Task correction: W4B is `[ ]`; state is **7/13**. W5–W9 remain unchecked and deferred. W4B needs offset-bound remediation and focused edge tests before another formal verification.
- Exact candidate boundary remains **326/400** with **74** lines headroom. No code/test file was edited by verification; W5, delivery, sync, and archive are not ready.

## W4B numeric-offset boundary correction
- Status: authoritative OpenSpec W4B-only apply remediation at 7/13; supplied repo-local edit roots and stacked normal-cap path were safe, with no action-context warning.
- Completed persisted task: W4B restored to `[x]` (8/13); W5–W9 remain unchecked and untouched.
- RED: focused summary passed 3/4 and failed exactly at `+14:01`; GREEN: focused summary passed 4/4 twice.
- Numeric RFC3339 offsets now accept `Z`, offsets below 14 with minutes 00–59, and exact `+14:00`/`-14:00`; `±14:01`, `+15:00`, and `-23:59` use exact `Fecha no disponible`.
- Checks: Sales mounted 9/9 twice, catalog 3/3, sale-flow 12/12, shell 4/4, mounted Inventory 3/3, full frontend 110/110, TypeScript, build, diff, direct offset diagnostics, scope, and boundary passed.
- Exact correction delta: **34 lines** = source/test 17 + checkbox 2 + evidence 15; cumulative W4B boundary: **360/400**, headroom **40**.
- Deviation: none. No calendar/clock/fallback, W5, verify-report, or delivery surface changed.
- Remaining tasks:
  - [ ] Implement and verify Inventory composition, whole-unit operation states, projections, alert priority, and the scoped truthful stock-exception cue after W4B. <!-- sdd-owner: implementation -->
  - [ ] Implement and verify Catalog maintenance master-detail, lifecycle, dynamic fields, and recovery states after W5. <!-- sdd-owner: implementation -->
  - [ ] Implement and verify History list, persisted detail, inline return, and destructive cancellation modal after W6. <!-- sdd-owner: implementation -->
  - [ ] Implement and verify shared-shell continuity for Product onboarding and Backup/Restore after W7. <!-- sdd-owner: implementation -->
  - [ ] Produce and verify the final exact-size evidence bundle and invariant/accessibility audit after W8. <!-- sdd-owner: implementation -->

## W4B delivery and post-merge evidence

- Issue **#142** was approved and closed. PR **#143** merged to `master` as `d66d6135c8940cb9909fa75b3471b4412d797f29` at `2026-09-02T14:27:36Z`.
- The merged W4B scope is exactly five files with **188 additions and 98 deletions** (**286 product/test/CSS lines**). The formal cumulative boundary remains **360/400**, with **40 lines headroom**.
- Post-merge verification passed: persisted-summary **4/4 twice**, Sales mounted **9/9 twice**, `test:mounted` **3/3**, and full frontend **110/110**. These listed runs report **139/139 test executions**. TypeScript, build, diff, and status checks also passed.
- Observed output contained only the existing Vite mixed static/dynamic import warning and intentional negative ConfirmationDialog logs. GitHub reported **0 check-runs/status entries**, so this records local/post-merge evidence rather than CI approval.
- `master`, local `origin/master`, and live `origin/master` all resolve to `d66d6135c8940cb9909fa75b3471b4412d797f29`. The only worktree entry is the untracked OpenSpec change directory.
- W4B is **integrated**. Persisted task state remains **8/13 complete**; W5 is dependency-unblocked but was not authorized or applied. W6–W9 also remain deferred.
- Status consumed/produced: authoritative hybrid/OpenSpec, apply ready at 8/13, repo-local workspace `/home/luis/velay/repuestos_autos`, edits restricted to the two authorized evidence artifacts, warnings none. No tests, code, GitHub action, W5 work, sync, or archive operation was performed by this evidence-only update.
- Next authorization gate: explicit W5 apply authorization under its ordered stacked-to-main boundary and normal 400-line cap.

## W5 Inventory operation and scoped stock cue
- Status consumed: authoritative hybrid/OpenSpec apply-ready at 8/13; repo-local allowed roots and stacked-to-main W5 boundary were resolved with no action-context warning.
- Completed persisted task: W5 `[x]`; state is 9/13. W6–W9 remain unchecked and untouched.
- Files changed: `inventory-screen.ts`, its mounted/public tests, `app.ts`, `app-shell.ts`, its mounted test, Inventory-only `styles.css`, task checkbox/summary, and this evidence.
- Behavior: Spanish search/selection, entry/count fields, whole-unit validation, current/projected stock, pending/success/stale/failure/retry, and prioritized exact `Sin stock: 0` / `Stock bajo: 1` cues render in distinct operation/alert regions.
- Safety: synchronous confirmation lock prevents duplicate commands; request identity and mounted guards reject stale search and post-unmount completion; command payloads and recovery remain unchanged.
- Shell handoff: Inventory alone publishes the available alert count; loading, unavailable, empty, navigation replacement, and unmount clear it. No polling, store, cache, or extra fetch was added.

### W5 TDD Cycle Evidence
| Stage | Evidence |
|---|---|
| RED | The new mounted Inventory suite failed 4/4 against English copy and absent loading, validation, priority, retry, and cue behavior. |
| GREEN | Inventory mounted passed 4/4 and shell mounted passed 5/5 after the narrow presentation and cue handoff. |
| TRIANGULATE | Inventory 4/4 and shell 5/5 passed twice; public Inventory/flow passed 3/3; full frontend passed 112/112; TypeScript, build, and diff checks passed. |
| REFACTOR | Shared Panel/Field/Feedback/Badge/Action interfaces localize presentation while Inventory retains alert and async ownership. |

- Responsive evidence is source-contract only: 600px/324px regions at default and one-column operation-before-alerts at `max-width: 960px`; native geometry remains W9.
- Exact W5 boundary: **372/400 changed lines** = product/tests/CSS **343** + task checkbox/summary **4** + progress evidence **25**. Headroom: **28**.
- Deviation: none. Existing Vite mixed-import warning and intentional negative dialog logs remain; no Rust, IPC, command, business-rule, W6, review, delivery, sync, or archive work occurred.
- Remaining exact tasks; next phase is W5 verification, not W6 apply:
  - [ ] Implement and verify Catalog maintenance master-detail, lifecycle, dynamic fields, and recovery states after W5. <!-- sdd-owner: implementation -->
  - [ ] Implement and verify History list, persisted detail, inline return, and destructive cancellation modal after W6. <!-- sdd-owner: implementation -->
  - [ ] Implement and verify shared-shell continuity for Product onboarding and Backup/Restore after W7. <!-- sdd-owner: implementation -->
  - [ ] Produce and verify the final exact-size evidence bundle and invariant/accessibility audit after W8. <!-- sdd-owner: implementation -->

## W5 blank-count verification remediation
- Status: authoritative hybrid/OpenSpec apply-ready at 9/13; repo-local supplied edit surfaces and stacked W5 remediation path were safe, with no action-context warning. W5 remains `[x]`; W6–W9 remain unchecked.
- Fix/files: `inventory-screen.ts` and `inventory-flow.ts` reject blank/whitespace numeric intent; mounted/public flow tests prove blank has no projection and stays disabled while explicit `0` projects/submits as numeric zero.
- RED: public/flow failed 3/4 at blank projection (`0 !== null`); mounted failed because blank plus reason remained actionable.
- GREEN/TRIANGULATE: public/flow passed 4/4 twice, mounted passed 4/4 twice, full frontend 113/113, TypeScript, build, and diff check passed; the existing Vite mixed-import warning and intentional dialog logs remain.
- REFACTOR/deviation: no broader refactor and no design deviation; commands, Rust, IPC, styles, shell, W6+, `verify-report.md`, and delivery behavior were untouched.
- Boundary: remediation is **35 changed lines** (28 source/test + 7 evidence), below 60; tracked W5 diff is **363 lines**, and cumulative W5 boundary is **399/400**. Next phase is `sdd-verify` for W5.

## W5 delivery and post-merge evidence

- Issue **#144** was approved and closed. PR **#145** was squash-merged to `master` as `fb333ecfe94b3cc9f031e4255e0413da6fe44f71`.
- The merged W5 scope is nine tracked files with **231 additions and 132 deletions** (**363 product/test/CSS lines**). The cumulative W5 boundary remains **399/400**, with **1 line headroom**.
- Post-merge verification passed Inventory public/flow **4/4**, Inventory mounted **4/4 twice**, AppShell mounted **5/5 twice**, and full frontend **113/113**, totaling **135/135 listed test executions**. TypeScript, build, diff, and status checks also passed.
- Only the existing Vite mixed static/dynamic import warning and intentional negative ConfirmationDialog logs were observed. GitHub reported **0 check-runs and 0 commit statuses**, so no CI approval is claimed.
- Local `master`, local `origin/master`, and live remote `master` all resolve to `fb333ecfe94b3cc9f031e4255e0413da6fe44f71`. The only worktree entry remains the untracked OpenSpec change directory.
- W5 is integrated and independently verified. Persisted task state remains **9/13 complete**; W6 is dependency-unblocked but not yet applied. W7–W9 remain deferred.

## W6 Catalog maintenance master-detail
- Status consumed: authoritative hybrid/OpenSpec apply-ready at 9/13; repo-local allowed roots and assigned stacked-to-main W6 boundary were resolved with no action-context warning.
- Completed persisted task: W6 `[x]`; state is 10/13. W7–W9 remain unchecked and untouched.
- Files changed: Catalog screen, flow and focused tests; new mounted Catalog test; Catalog-only CSS; W6 checkbox; and this cumulative evidence.
- Behavior: Spanish Panel-based master/detail keeps selected identity, active/archived badges, category/product facts, dynamic fields, immediate `Archivar`/`Reactivar`, explicit loading/empty/unavailable/stale/success states, and shared Action/Field/Feedback/Badge treatment.
- Contracts: `Bs` text parses deterministically to positive integer centavos; expected revisions, inputs, command names, historical-price helper, lifecycle policy, and reload behavior remain intact. Request identity, mutation lock, and mounted guards reject stale/duplicate/post-unmount completion.

### W6 TDD Cycle Evidence
| Stage | Evidence |
|---|---|
| RED | The new mounted suite failed 3/3 against English prose and missing Spanish hierarchy, Bs editor, dynamic validation/focus, recovery, and responsive contracts. |
| GREEN | Catalog flow and mounted suites passed 7/7 after the presentation, form mapping, focus, lifecycle, and recovery adoption. |
| TRIANGULATE | Focused Catalog passed 7/7 twice; full frontend passed 116/116; TypeScript, build, and diff checks passed. |
| REFACTOR | Shared visual interfaces localize rendering while flow helpers retain deterministic Bs mapping and existing command contracts. |

- Responsive evidence is source-contract only: 336px/588px columns at default and list-before-detail single-column stacking with a 208px master scroll region at `max-width: 960px`; native geometry remains W9.
- Exact W6 boundary: **336/400 changed lines** = product/tests/CSS **315** + task checkbox **2** + progress evidence **19**. Headroom: **64**.
- Deviation: none. Existing Vite mixed-import warning and intentional negative dialog logs remain; no command, Rust, IPC, persistence, W7+, review, delivery, sync, or archive work occurred.
- Remaining exact tasks: W7 History, W8 continuity, and W9 evidence remain unchecked; next phase is `sdd-verify` for W6.

## W6 state remediation
- Status: authoritative hybrid/OpenSpec apply-ready at 10/13; repo-local supplied edit surfaces and stacked W6 remediation path were safe, with no action-context warning.
- Fix/files: Catalog screen suppresses empty during initial unavailability and exposes selected-detail reload; the flow synchronizes matching master, selection, detail activity, and revision. Changed `catalog-maintenance-screen.ts`, `catalog-maintenance-flow.ts`, both focused Catalog tests, and this evidence only.
- RED: focused Catalog failed 2/7 before production correction: lifecycle state was `active/archived/archived`, and the mounted unavailable recovery scenario failed.
- GREEN: focused Catalog passed 8/8 after the three bounded corrections.
- TRIANGULATE: focused Catalog passed 8/8 twice; full frontend passed 117/117; TypeScript, build, and diff checks passed.
- REFACTOR: no broader refactor or design deviation; commands, payloads, revisions, lifecycle authority, async guards, CSS, W7+, `verify-report.md`, and delivery remained untouched.
- Boundary: remediation is **37 changed lines** = 28 source/test + 9 evidence; cumulative W6 is **373/400**, leaving 27 lines. W6 remains `[x]`; exact remaining tasks are W7, W8, and W9 as unchecked below.
## W6 reload-chain race remediation
- Status: authoritative OpenSpec apply-ready at 10/13; supplied repo-local surfaces and stacked W6 path were safe, with no warning.
- RED: mounted adversarial A-reload/B-selection interleaving failed with detail calls `[1, 2, 1]`.
- GREEN/REFACTOR: reload continues only while its request identity is current; the mounted assertion and focused Catalog passed 9/9 twice.
- TRIANGULATE: full frontend 118/118, TypeScript, build, and diff check passed; existing Vite warning only.
- Boundary: exact remediation **27 lines**; cumulative W6 **400/400**. W6 stays `[x]`; W7–W9 stay unchecked.

## W6 delivery and post-merge evidence

- Issue **#146** was approved and closed. PR **#147** was squash-merged to `master` as `aeab6f09b4e7c0214d6308538240c7010ae6527b`.
- The merged W6 scope is five tracked files with **315 additions and 41 deletions** (**356 net product/test/CSS lines**). Cumulative W6 review accounting remains **400/400**.
- Post-merge verification passed focused Catalog **9/9 twice** and full frontend **118/118**, totaling **136/136 listed test executions**. TypeScript, build, diff, and status checks also passed.
- Only the existing Vite mixed static/dynamic import warning and intentional negative ConfirmationDialog logs were observed. GitHub reported **0 check-runs and 0 commit statuses**, so no CI approval is claimed.
- Local `master`, local `origin/master`, and live remote `master` all resolve to `aeab6f09b4e7c0214d6308538240c7010ae6527b`. The only worktree entry remains the seven-file untracked OpenSpec change directory.
- W6 is integrated and independently verified. Persisted task state remains **10/13 complete**; W7 is dependency-unblocked but not authorized or applied. W8–W9 remain deferred.

## W7A Sales History retrieval lifecycle safety

- Status consumed: authoritative hybrid/OpenSpec apply-ready at 10/13; repo-local action context was restricted to the five supplied W7A surfaces with no warning. Delivery is the approved first `stacked-to-main` slice in W7A→W7F under the normal 400-line cap.
- Bounded progress only: W7A is complete, but the single persisted W7 task intentionally remains `[ ]`; W7B–W7F composition, persisted-detail, correction presentation, return redesign, and cancellation modal work were not started.
- Deep seam: `createSalesHistoryInteraction` owns one monotonic retrieval/correction intent identity plus `invalidate`, StrictMode-safe `activate`, and `dispose`. List, detail, and correction-triggered reload completions dispatch only while their identity is current.
- Lifecycle wiring: a newer list/detail/reload intent supersedes every older completion; Back invalidates before returning to retained list state; unmount invalidates; and late correction success cannot dispatch success or reload an obsolete sale.
- Preserved contracts: request UUIDs, correction payloads, inclusive date-range adapter semantics, eligibility, reducer transitions, persisted facts, rendering/focus behavior, command adapters, IPC/Rust/persistence, and retry/reload semantics are unchanged.
- Files changed: `history-screen.ts`, `history-flow.test.ts`, new `history-screen.mounted.test.ts`, and this cumulative progress artifact. `history-flow.ts` required no production change.

### W7A TDD Cycle Evidence

| Stage | Evidence |
|---|---|
| RED | Two new public interaction scenarios failed: older list completion replaced the newer list (`71 !== 72`), and late correction success reloaded/replaced the newer selected sale (`71 !== 72`). |
| GREEN | A single monotonic intent seam guarded list/detail/correction reload completion; focused public History passed 18/18 and mounted History passed 3/3. |
| TRIANGULATE | Public coverage exercises older-list-after-newer-list, detail A after B, and late list after detail; mounted coverage exercises reverse list completion, post-unmount completion, and obsolete-sale correction success. Both focused suites passed twice; full frontend passed 123/123; TypeScript, build, and diff checks passed. |
| REFACTOR | Request identity, mounted lifecycle, and correction reload ownership are centralized behind the existing public interaction interface rather than distributed booleans. |

- Exact W7A boundary: **270/400 changed lines** = product/tests **249** + progress evidence **21**. Headroom: **130**. No task checkbox changed because W7 as a whole remains incomplete.
- Warning: the existing Vite mixed static/dynamic Tauri import warning and intentional negative ConfirmationDialog logs remain. No design deviation or unauthorized delivery/backend mutation occurred.
- Remaining exact implementation tasks stay unchecked: W7 full History redesign, W8 continuity, and W9 evidence. Current PR boundary ends at W7A retrieval safety; next chain slice is W7B only after formal verification and parent-owned delivery.

## W7A delivery and post-merge evidence

- Issue **#148** was approved and closed. PR **#149** was squash-merged to `master` as `e726debef6061d8153b19d9a6a05146ad723ac24`.
- The merged W7A scope is three tracked files with **245 additions and 4 deletions** (**249 product/test lines**). The cumulative W7A boundary remains **270/400**, with **130 lines headroom**.
- Post-merge verification passed History flow **18/18 twice**, History mounted **3/3 twice**, and full frontend **123/123**. TypeScript, build, diff, and status checks also passed.
- Only the existing Vite mixed static/dynamic import warning and intentional negative ConfirmationDialog logs were observed. GitHub reported **0 check-runs and 0 commit statuses**, so no CI approval is claimed.
- Local `master`, local `origin/master`, and live remote `master` all resolve to `e726debef6061d8153b19d9a6a05146ad723ac24`. The only worktree entry remains the untracked OpenSpec change directory.
- W7A is integrated and independently verified. W7 remains incomplete at **10/13**; W7B is dependency-unblocked, while W7C–W7F remain ordered behind it.

## W7B bounded Spanish Sales History list

- Status consumed: authoritative hybrid/OpenSpec apply-ready at 10/13; repo-local action context restricted to the supplied W7B surfaces with no warning. Delivery is the approved second `stacked-to-main` slice after integrated W7A `e726deb`.
- Bounded progress only: W7B is complete and independently usable/revertible, but the aggregate W7 checkbox intentionally remains `[ ]`; W7C–W7F, W8, W9, persisted detail, correction presentation, return, and cancellation were not changed.
- Files changed: `history-screen.ts`, new History-local `history-presentation.ts`, History flow/mounted tests, History-only `styles.css`, and this cumulative evidence.
- Behavior: Spanish `Historial de ventas`, inclusive `Desde`/`Hasta`, `Cargar historial`, loading, empty, ready, bounded overflow, bounded error/retry, semantic sale facts, explicit `Confirmada`/`Cancelada`, and preserved `Ver detalle` selection.
- Safety: native error text is discarded in favor of known bounded Spanish copy; malformed timestamps use `Fecha no disponible`; command date conversion, W7A request identity, unmount guards, and detail/correction behavior remain unchanged.
- Responsive source contract: filters wrap into two fields plus a full-row action at `max-width: 960px`; shared aligned data becomes labeled records in unchanged DOM order, and History overrides horizontal data scrolling.

### W7B TDD Cycle Evidence

| Stage | Evidence |
|---|---|
| RED | Mounted History passed 2/5 and failed the three new Spanish loading/list/state scenarios against the legacy English prose list. |
| GREEN | Focused History passed 23/23 after semantic table, bounded presentation mapping, shared controls, and History-only responsive rules were added. |
| TRIANGULATE | Focused History passed 23/23 twice, then 23/23 after type correction; full frontend passed 125/125, TypeScript, build, and diff checks passed. |
| REFACTOR | Date/error/fact mapping moved behind one History-local presentation interface while the existing selection and W7A interaction interfaces stayed unchanged. |

- Exact W7B boundary: **200/400 changed lines** = product/tests/CSS **179** + progress evidence **21**. Headroom: **200**.
- Deviation: none. Existing Vite mixed-import warning and intentional negative dialog logs remain; no commit, delivery, issue, PR, backend, verify-report, sync, archive, W7C+, or W8 mutation occurred.
- Remaining exact unchecked tasks: `- [ ] Implement and verify History list, persisted detail, inline return, and destructive cancellation modal after W6. <!-- sdd-owner: implementation -->`; `- [ ] Implement and verify shared-shell continuity for Product onboarding and Backup/Restore after W7. <!-- sdd-owner: implementation -->`; `- [ ] Produce and verify the final exact-size evidence bundle and invariant/accessibility audit after W8. <!-- sdd-owner: implementation -->`.

## W7B delivery and post-merge evidence

- Issue **#150** was approved and closed. PR **#151** was squash-merged to `master` as `a6bc6dd1044e0c68a78cd2fb9f23e10c6628f0bd`.
- The merged W7B scope is five tracked files with **159 additions and 20 deletions** (**179 product/test/CSS lines**). The cumulative W7B boundary remains **200/400**, with **200 lines headroom**.
- Post-merge verification passed focused History **23/23 twice** and full frontend **125/125**. TypeScript, build, diff, and status checks also passed.
- Only the existing Vite mixed static/dynamic import warning and intentional negative ConfirmationDialog logs were observed. GitHub reported **0 check-runs and 0 commit statuses**, so no CI approval is claimed.
- Local `master`, local `origin/master`, and live remote `master` all resolve to `a6bc6dd1044e0c68a78cd2fb9f23e10c6628f0bd`. The only worktree entry remains the untracked OpenSpec change directory.
- W7B is integrated and independently verified. W7 remains incomplete at **10/13**; W7C is dependency-unblocked, while W7D–W7F remain ordered behind it.

## W7C persisted original Sales History detail
- Status consumed: authoritative OpenSpec apply-ready at 10/13; repo-local action context restricted to the seven supplied W7C surfaces with no warning. Delivery is the approved third `stacked-to-main` slice after integrated W7B `a6bc6dd`.
- Bounded progress only: W7C is complete, but aggregate W7 remains `[ ]`; W7D–W7F, W8, W9, correction records/actions, return, cancellation, commands, IPC, Rust, persistence, and `verify-report.md` were untouched.
- Files changed: History screen/presentation and focused tests, History-only CSS, and this cumulative evidence. The tasks artifact was re-read and intentionally remains unchanged.
- Behavior: `Volver al historial`, Spanish persisted identity and lifecycle badge, validated wall-clock time, semantic original item/payment tables, explicit unavailable snapshots, and authoritative `Bs` total render in a visually separate read-only region.
- Persisted-fact safety: product/SKU snapshots, quantities, sale-time unit prices, subtotals, payments, and total project directly from decoded detail; source mutation cannot substitute current catalog or draft values, and no catalog lookup exists in the detail path.
- Responsive source contract: original tables use shared logical labeled-row order and remove History-detail horizontal overflow at `max-width: 960px`; native exact-size evidence remains W9.

### W7C TDD Cycle Evidence
| Stage | Evidence |
|---|---|
| RED | Focused execution failed on the absent detail projection export and legacy English detail heading/composition. |
| GREEN | History presentation, flow, and mounted suites passed 26/26 with semantic original facts and stable unavailable/date formatting. |
| TRIANGULATE | Focused History passed 26/26 twice; full frontend passed 128/128; TypeScript, build, and diff checks passed. |
| REFACTOR | Persisted formatting moved behind the existing History-local presentation interface and rendering reused AlignedData, Badge, Action, and Feedback without changing interaction state. |

- Exact W7C boundary: **301/400 changed lines** = product/tests/CSS **282** + progress evidence **19**. Headroom: **99**.
- Deviation: none. Existing Vite mixed-import warning and intentional negative ConfirmationDialog logs remain.
- Remaining exact unchecked tasks: `- [ ] Implement and verify History list, persisted detail, inline return, and destructive cancellation modal after W6. <!-- sdd-owner: implementation -->`; `- [ ] Implement and verify shared-shell continuity for Product onboarding and Backup/Restore after W7. <!-- sdd-owner: implementation -->`; `- [ ] Produce and verify the final exact-size evidence bundle and invariant/accessibility audit after W8. <!-- sdd-owner: implementation -->`.

## W7C delivery and post-merge evidence

- Issue **#152** was approved and closed. PR **#153** was squash-merged to `master` as `aa1d117993dc9f9cce56552771cd36fbb6f27aad`.
- The merged W7C scope is six tracked files with **191 additions and 91 deletions** (**282 product/test/CSS lines**). The cumulative W7C boundary remains **301/400**, with **99 lines headroom**.
- Post-merge verification passed focused History **26/26 twice** and full frontend **128/128**. TypeScript, build, diff, and status checks also passed.
- Only the existing Vite mixed static/dynamic import warning and intentional negative ConfirmationDialog logs were observed. GitHub reported **0 check-runs and 0 commit statuses**, so no CI approval is claimed.
- Local `master`, local `origin/master`, and live remote `master` all resolve to `aa1d117993dc9f9cce56552771cd36fbb6f27aad`. The only worktree entry remains the untracked OpenSpec change directory.
- W7C is integrated and independently verified. W7 remains incomplete at **10/13**; W7D is dependency-unblocked, while W7E–W7F remain ordered behind it.

## W7D persisted correction history and command safety
- Finalization status consumed: authoritative OpenSpec apply-ready at 10/13 on base `aa1d117`; the maintainer-authorized narrower W7D successor preserved the verified candidate within repo-local allowed surfaces, with no action-context warning.
- Bounded progress only: W7D is complete; aggregate W7, W8, and W9 remain unchecked. W7E return controls and W7F cancellation modal were not redesigned.
- Files changed: History screen/flow/presentation and focused tests, mounted History fixture, History-only CSS, and this cumulative artifact.
- Behavior: Spanish semantic correction records preserve request/record/line/product IDs, raw persisted timestamps/reasons, statuses, and returned/restored quantities including zero, visually separate from original sale facts.
- Safety: work areas are mutually exclusive; eligibility remains per persisted remaining sale-line quantity, fully returned confirmed sales remain cancellable, and cancelled sales expose neither action.
- Command lifecycle: request UUIDs survive explicit retry; request-keyed reducer completion and interaction identity reject stale/mismatched completion; command success enters reload-requested state, and only successful persisted-detail reload closes intent.
- Feedback is bounded Spanish inventory-correction language; command payloads, adapters, IPC/Rust/persistence, W7A guards, W7B list, W7C original detail, and focus recovery are unchanged.

### W7D TDD Cycle Evidence
| Stage | Evidence |
|---|---|
| RED | Focused public tests failed for absent correction projection and simultaneous work areas; mounted persisted correction assertions were then added at the approved screen seam. |
| GREEN | History presentation/flow passed 22/22 and mounted History passed 6/6 with semantic records, zeros, eligibility, request-keyed completion, and Spanish bounded errors. |
| TRIANGULATE | Preserved independent diagnosis evidence: both focused sets passed twice (History 28/28 each run); full frontend passed 130/130; TypeScript, build, and diff checks passed. |
| REFACTOR | Correction projection and rendering are localized behind the History presentation seam; duplicate quantity validation was collapsed without changing payload authority. |
- Exact W7D boundary: **385/400 changed lines** = product/tests/CSS **367** + progress evidence **18**; headroom **15**. No design deviation; existing Vite mixed-import warning and intentional dialog logs remain.
- Remaining exact tasks: `- [ ] Implement and verify History list, persisted detail, inline return, and destructive cancellation modal after W6. <!-- sdd-owner: implementation -->`; `- [ ] Implement and verify shared-shell continuity for Product onboarding and Backup/Restore after W7. <!-- sdd-owner: implementation -->`; `- [ ] Produce and verify the final exact-size evidence bundle and invariant/accessibility audit after W8. <!-- sdd-owner: implementation -->`.

## W7D delivery and post-merge evidence

- Issue **#154** was approved and closed. PR **#155** was squash-merged to `master` as `202b1b3ce7c80011e26de2bc60545b05bd0db469`.
- The merged W7D scope is seven tracked files with **205 additions and 162 deletions** (**367 product/test/CSS lines**). The cumulative W7D boundary remains **385/400**, with **15 lines headroom**.
- Post-merge verification passed focused History **28/28 twice** and full frontend **130/130**. TypeScript, build, diff, and status checks also passed.
- Only the existing Vite mixed static/dynamic import warning and intentional negative ConfirmationDialog logs were observed. GitHub reported **0 check-runs and 0 commit statuses**, so no CI approval is claimed.
- Local `master`, local `origin/master`, and live remote `master` all resolve to `202b1b3ce7c80011e26de2bc60545b05bd0db469`. The only worktree entry remains the untracked OpenSpec change directory.
- W7D is integrated and independently verified. W7 remains incomplete at **10/13**; W7E is dependency-unblocked and W7F remains ordered behind it.

## W7E inline Sales History return workflow
- Status consumed: authoritative OpenSpec apply-ready at 10/13 on integrated W7D base `202b1b3`; repo-local edits were restricted to the eight supplied W7E surfaces, with no action-context warning. Delivery boundary is the fifth approved `stacked-to-main` slice in W7A→W7F.
- Bounded progress only: W7E is complete, but aggregate W7 remains `[ ]`; W7F cancellation, W8, W9, `verify-report.md`, commands, IPC, Rust, and persistence were untouched.
- Behavior: Spanish `Iniciar devolución de artículos` opens one inline sale-line-ID-keyed group per original line. Persisted remaining quantity is always visible, unavailable lines remain visible, and editable quantities stay strings until the existing command call converts them.
- Validation: selection and quantity errors are field-associated and focus the first invalid control in persisted line order. Pending/reload locks the full form and displays `Registrando devolución…`; the existing interaction guard prevents duplicate commands.
- Recovery/evidence: bounded Spanish failure preserves values and request UUID, exposes `Recargar detalle de venta`, and never renders native detail. Only a successful persisted-detail reload closes the form and displays the correction record.
- Responsive source contract: the return panel and groups use zero-minimum grid sizing, and the 960px rule preserves DOM order while preventing horizontal core-task overflow; native exact-size evidence remains W9.

### W7E TDD Cycle Evidence
| Stage | Evidence |
|---|---|
| RED | Focused History failed 4/19 against English return markup, numeric input, and sale-line-ID ordering before production changes. |
| GREEN | History flow passed 19/19 and mounted History passed 7/7 with Spanish inline behavior, association/focus, duplicate lock, retry UUID, exact payload, and persisted success evidence. |
| TRIANGULATE | Both focused suites passed twice; full frontend passed 131/131; TypeScript, production build, and diff checks passed. |
| REFACTOR | Return rendering moved behind one local deep form interface while the public reducer/interaction seams and command adapter remained unchanged. |

- Exact W7E boundary: **312/400 changed lines** = product/tests/CSS **293** + progress evidence **19**. Headroom: **88**; the working target of 360 is also satisfied.
- Deviation: none. Existing Vite mixed-import warning and intentional negative dialog logs remain.
- Remaining exact unchecked tasks: `- [ ] Implement and verify History list, persisted detail, inline return, and destructive cancellation modal after W6. <!-- sdd-owner: implementation -->`; `- [ ] Implement and verify shared-shell continuity for Product onboarding and Backup/Restore after W7. <!-- sdd-owner: implementation -->`; `- [ ] Produce and verify the final exact-size evidence bundle and invariant/accessibility audit after W8. <!-- sdd-owner: implementation -->`.

## W7E persisted-evidence remediation
- Status consumed: authoritative OpenSpec apply-ready at 10/13 on W7D base `202b1b3`; supplied repo-local edit surfaces and the parent-resolved stacked W7E remediation boundary were safe, with no action-context warning.
- Blocker fixed: a successful detail response containing no return with the intent's exact stable request UUID no longer closes that return intent.
- Reducer behavior: same-sale detail reload preserves the return intent; missing exact evidence yields a bounded recoverable error, while an exact persisted request-ID match alone closes it. Different-sale selection and all cancellation behavior remain unchanged.
- RED: the mounted public case passed 6/7 and failed because stale successful detail removed the form and reload action.
- GREEN: mounted History passed 7/7 and flow History passed 19/19 after the minimal reducer correction.
- TRIANGULATE: combined focused History passed 26/26 twice; full frontend passed 131/131; TypeScript, build, and diff checks passed.
- REFACTOR: exact persisted-evidence matching remains in the deterministic flow seam; payloads, adapters, IPC/Rust/persistence, UUID retry, interaction guards, selection identity, cancellation, and presentation are unchanged.
- Files changed by remediation: `history-flow.ts`, `history-screen.mounted.test.ts`, and this cumulative progress artifact.
- Boundary: remediation is **46 changed lines** = product/test **34** + evidence **12**; cumulative W7E is **358/400**, leaving **42** lines.
- Persisted tasks were re-read: aggregate W7 intentionally remains `[ ]`; W8 and W9 also remain unchecked, and `verify-report.md` remains untouched.

## W7E delivery and post-merge evidence

- Issue **#156** was approved and closed. PR **#157** was squash-merged to `master` as `63769caeaa879c623b97243f3883ab88dd9c4974`.
- The merged W7E scope is five tracked files with **179 additions and 148 deletions** (**327 product/test/CSS lines**). Cumulative W7E accounting remains **358/400**, with **42 lines headroom**.
- Post-merge verification passed focused History **29/29 twice** and full frontend **131/131**. TypeScript, build, diff, and status checks also passed.
- Only the existing Vite mixed static/dynamic import warning and intentional negative ConfirmationDialog logs were observed. GitHub reported **0 check-runs and 0 commit statuses**, so no CI approval is claimed.
- Local `master`, local `origin/master`, and live remote `master` all resolve to `63769caeaa879c623b97243f3883ab88dd9c4974`. The only worktree entry remains the untracked OpenSpec change directory.
- W7E is integrated and independently verified. W7 remains incomplete at **10/13**; W7F is dependency-unblocked and is the final ordered W7 slice.
## W7F cancellation preparation and destructive modal
- Status/action: authoritative OpenSpec apply-ready 10/13 on `63769ca`; supplied repo-local roots and final stacked W7F boundary were safe, warnings none.
- Completed W7 `[x]` and state 11/13; W8/W9 remain exact unchecked implementation tasks, with no verify-report/delivery/backend mutation.
- Files: History screen/flow and focused tests, History-only CSS, W7 checkbox, and this cumulative artifact; shared dialog interface stayed unchanged.
- RED/GREEN: focused History failed 5/27 for absent staging/modal/persisted evidence, then passed 27/27 twice with Spanish preparation, stable UUID, exact payload, zero residual, and matching-request closure.
- TRIANGULATE: shared dialog passed 13/13, full frontend 132/132, TypeScript, build, and diff checks passed; modal containment/focus/Escape/pending restoration remain shared evidence.
- REFACTOR/deviation: cancellation staging and persisted-evidence matching stay in existing public seams; no design deviation, return/backend/W8 behavior, payment reversal, or catalog lookup.
- Boundary: **357/400** changed lines including tests, checkbox, and this evidence; 960px cancellation/modal source reflow is covered, while native exact-size evidence remains W9.

## W8 delivery-risk forecast (no source changes)

- Status consumed: authoritative `gentle-ai.sdd-status` v2 selects `define-frontend-ui-ux-visual-system`; apply is ready at 11/13, W8 is the exact pending implementation-owned row, and repo-local `/home/luis/velay/repuestos_autos` is the only edit root. No action-context warnings were supplied.
- Delivery decision consumed: `ask-on-risk`, normal **400 changed-line** budget. The W8 task forecast itself declares high risk and chained delivery; the current session has not authorized `auto-chain`, `single-pr` with `size:exception`, or an exception.
- Measured baseline: onboarding source/form/test is 307 lines; backup source/flow/test is 62 lines; analogous mounted screen suites are 127 (Inventory), 146 (Catalog), and 355 (History) lines. Recent bounded visual slices required 315 product/test/CSS changed lines for Catalog alone and 327 for History returns alone.
- Forecast: one cohesive W8 candidate is **~700–920 changed lines** (onboarding composition/presentation/async safety and tests ~310–400; backup/restore state/dialog composition and tests ~300–400; continuity CSS plus artifact evidence ~90–120). This exceeds the 400-line budget before any production source edit.
- Proposed bounded split for maintainer approval: **W8A Onboarding continuity** (onboarding screen/form/flow tests and onboarding CSS, target 320–390) followed by **W8B Backup/Restore continuity** (backup screen/flow tests and backup CSS, target 330–395). Each preserves existing command/native-dialog boundaries; the W8 checkbox remains unchecked until both slices complete.
- No runtime command was launched, no acquire token was needed, no task checkbox changed, and no source/test/CSS file was edited.

## W8A Onboarding continuity recovery
- Status consumed: native `gentle-ai.sdd-status` v2 apply-ready for `define-frontend-ui-ux-visual-system`, repo-local workspace and allowed edit root `/home/luis/velay/repuestos_autos`, with no action-context warnings. The active attempt ledger was continued with token `sha256:78c9ae7436aa72828b0b594f2df1cc4ac4f7c8b569b66c382f88b7dcbc707d6b`; W8A is the selected split and W8B remains untouched.
- Preserved and repaired the partial onboarding candidate only: Spanish `Alta de productos`, shared `Action`/`Field`/`Feedback`/`Panel`, sequential category/product panels, category loading/empty/error/retry, dynamic field builder, exact empty copy, localized validation/pending/success/failure, Bs presentation parsing to integer centavos, positive whole-unit stock parsing, required/optional `attributeValuesFor` semantics, synchronous duplicate locks, request identity, stale completion rejection, and mounted cleanup guards.
- Files changed: `src/ui/onboarding/onboarding-screen.ts`, `onboarding-form.ts`, `onboarding-flow.ts`, `onboarding-form.test.ts`, `onboarding-flow.test.ts`, `onboarding-screen.mounted.test.ts`, and onboarding continuity rules in `src/ui/styles.css`. `src/commands/onboarding.ts`, `tasks.md`, backup files, shell, IPC/Rust/persistence, package files, and unrelated screens remain unchanged.
### W8A TDD Cycle Evidence
| Stage | Evidence |
|---|---|
| RED | Focused onboarding execution failed on the partial English screen, missing shared composition, incorrect parser escaping, and absent localized mounted states. |
| GREEN | Focused onboarding suites pass 13/13 with category loading/empty, shared panels, exact payload, localized failure, validation focus, duplicate protection, and post-unmount completion protection. |
| TRIANGULATE | Focused onboarding suites passed 13/13 twice; full frontend passed 143/143; `./node_modules/.bin/tsc --noEmit`, `npm run build`, and `git diff --check` passed. Build retains the existing mixed static/dynamic Tauri import warning; no GUI/manual evidence is claimed. |
| REFACTOR | Parsing, flow request identity, lifecycle guards, and shared control composition remain behind small onboarding seams; command names, payload shape, centavo price authority, whole-unit stock authority, and `attributeValuesFor` semantics were preserved. |
- Responsive evidence is source-contract only: onboarding panels use a two-column layout by default and one-column stacking at `max-width: 960px`; no native viewport/manual geometry claim is made. Reduced-motion behavior remains inherited from the existing global rules.
- W8 aggregate checkbox intentionally remains unchecked because W8B Backup/Restore is not implemented. W9 remains unchecked. No `tasks.md` edit or verification report mutation was made.
- Workload boundary: the runtime ledger measured this recovered W8A candidate at **525/400 changed lines**, so the normal cap is exceeded and no size exception was inferred. The attempt is recorded as requiring a maintainer reset/decision before delivery; W8B is not started. No commit, push, PR, merge, or delivery action occurred.

## W8A final bounded evidence after authorized exception
- Native status consumed: active `W8A onboarding continuity` attempt, token retained by the parent; maintainer-authorized W8A-only exception is **600 changed lines**. No source edits were made during finalization.
- Focused onboarding flow/form plus mounted suites: **13/13 twice**, in separate processes. Full frontend: **143/143**.
- `./node_modules/.bin/tsc --noEmit`: PASS. `npm run build`: PASS with the pre-existing mixed static/dynamic Tauri import warning. `git diff --check`: PASS.
- Scope audit: only the W8A onboarding source/tests, onboarding selectors in `src/ui/styles.css`, and this artifact are in scope; W8B backup/restore and the aggregate W8 checkbox remain untouched.
- Evidence confirms preserved command names/payload envelope, integer-centavo price and positive whole-unit stock authority, Spanish two-panel composition, category loading/empty/error/retry, dynamic fields, localized validation/pending/success/failure, synchronous duplicate locks, stale/post-unmount guards, focus/live semantics, and responsive source contract. No GUI/manual evidence is claimed.
- Final workload boundary: **535/600 changed lines** (510 W8A source/test/CSS lines + 15 prior evidence lines + 10 final evidence lines), within the explicit exception; no commit, push, PR, merge, or settle was performed.

## W8B Backup/Restore continuity
- Status consumed: native `gentle-ai.sdd-status` v2 apply-ready for `define-frontend-ui-ux-visual-system`, W8B attempt ordinal 17 active with token `sha256:bd97b1e05410487bf3787bdcdc32cc9de4ade9e2a069e0f2d89e6dde4f6a1583`, repo-local workspace `/home/luis/velay/repuestos_autos`, and no native budget-risk or scope warning. The aggregate W8 checkbox remains intentionally unchecked because W8A/W8B are separately authorized slices.
- W8B continuity uses the existing AppShell through the unchanged `App` handoff. `BackupScreen` now renders Spanish `Copia y restauración` with bordered Backup and Restauración panels, persisted path/date/size/schema facts, candidate facts, bounded Spanish recovery, native picker cancellation, and the exact shared restore dialog purpose/title/consequence/actions. Native dialog appearance and `src/commands/backup.ts` were not changed.
- The backup flow now exposes idle/prepared/invalid/expired/unavailable/pending/success/failure/recovery meanings, maps known native error codes to bounded Spanish copy, and centralizes synchronous duplicate locks, attempt identity, disposal, and post-unmount/stale completion suppression. The restore token and acknowledgement remain caller-owned and the existing command payloads remain untouched.
- Files changed: `src/ui/backup/backup-screen.ts`, `src/ui/backup/backup-flow.ts`, `src/ui/backup/backup-flow.test.ts`, `src/ui/backup/backup-screen.mounted.test.ts`, Backup/Restore-only rules in `src/ui/styles.css`, and this cumulative artifact. No task checkbox was changed; `tasks.md` remains read-only and W8/W9 remain unchecked.

### W8B TDD Cycle Evidence
| Stage | Evidence |
|---|---|
| RED | The new mounted suite failed against the legacy English header/buttons and inline restore composition before Backup/Restore production adoption. |
| GREEN | Focused flow and mounted Backup/Restore suites pass 9/9 with Spanish panels, all requested state mappings, cancellation, acknowledgement gating, exact dialog semantics, pending locks, duplicate suppression, focus containment, and unmount safety. |
| TRIANGULATE | The focused suite passed 9/9 twice in separate processes; TypeScript, production build, and `git diff --check` passed. Full `npm test` reports 149/150 because the pre-existing AppShell mounted test still asks for the superseded English Backup heading; that test is outside the authorized W8B edit surfaces. No GUI/manual evidence is claimed. |
| REFACTOR | Backup async ownership moved behind `createBackupInteraction`; rendering reuses `Action`, `Field`, `Feedback`, `Panel`, and `ConfirmationDialog` while command names, payloads, native handoff, token, acknowledgement, and reducer intent remain preserved. |

- Exact W8B changed-line accounting at this phase: **259/400** = tracked Backup/Restore source, focused test, and Backup/Restore CSS delta **150** (118 additions + 32 deletions) + mounted test **92** + cumulative artifact **17**. The untracked onboarding/preflight files remain outside this W8B accounting. Headroom: **141**.
- Verification commands: focused Backup/Restore suites 9/9 twice; `npm test` 149/150 (one stale out-of-scope AppShell assertion); `./node_modules/.bin/tsc --noEmit` PASS; `npm run build` PASS with the existing mixed static/dynamic Tauri import warning; `git diff --check` PASS. Native `src/commands/backup.ts` diff is empty. No GUI/manual evidence, commit, push, PR, merge, verify-report, tasks, IPC, Rust, persistence, or unrelated-screen mutation occurred.
- Remaining exact unchecked tasks: `- [ ] Implement and verify shared-shell continuity for Product onboarding and Backup/Restore after W7. <!-- sdd-owner: implementation -->`; `- [ ] Produce and verify the final exact-size evidence bundle and invariant/accessibility audit after W8. <!-- sdd-owner: implementation -->`. W8B is complete as its assigned slice, but aggregate W8 remains unchecked pending the parent’s task reconciliation and final verification route.

## W8B Backup/Restore reconciliation
- Status consumed: native `gentle-ai.sdd-status` v2 was apply-ready for `define-frontend-ui-ux-visual-system`; active native attempt ordinal **18**, work unit `W8B Backup Restore reconciliation`, token `sha256:88c41e4b5c43c681597efc54d32f0a1d93fd6cca4cc5fdf299900dc1382892e2`, and objective generation 17. The native attempt allowed one bounded completion at 400 changed lines; no new acquire, reset, commit, push, PR, merge, or delivery action was performed.
- Authorized reconciliation only: changed the one stale assertion in `src/ui/app-shell.mounted.test.ts` from `Backup and restore` to the production heading `Copia y restauración`. No production file, Backup/Restore implementation, command, IPC, Rust, persistence, shared visual module, onboarding file, tasks, verify report, package/lock, mockup, or unrelated test was changed by this reconciliation.
- Focused Backup/Restore suites passed **9/9 twice** in separate processes using `backup-flow.test.ts` and `backup-screen.mounted.test.ts`. Full `npm test` passed **150/150**. `./node_modules/.bin/tsc --noEmit` passed; `npm run build` passed with the pre-existing mixed static/dynamic Tauri import warning; `git diff --check` passed. No GUI or manual evidence is claimed.
- Native changed-line accounting is authoritative and is intentionally not replaced with local arithmetic; the terminal native settle result records the exact reconciliation accounting for this attempt. The prior failed attempt’s native accounting was **256/400** with evidence revision `sha256:793d198243f16989e7d3ed0b4c1190b70d678cde7632b0c55252182e4f7f1440`; this run addressed only its stale heading failure.
- Aggregate W8 and W9 checkboxes remain unchecked. `tasks.md` and `verify-report.md` were not edited. The W8B slice is reconciled and complete as an assigned slice; whole-change verification remains blocked on the unchecked W9 task and the aggregate W8 task.

## W8 aggregate task reconciliation
- Parent reconciliation after both authorized slices: W8A onboarding and W8B Backup/Restore are complete and individually verified; W8 is now checked in `tasks.md`. W9 remains unchecked and intentionally unstarted. The W8B native objective remains complete at 9/400 for the final reconciliation attempt, with full frontend 150/150, TypeScript, build, and diff checks passing; no GUI/manual evidence or delivery action is claimed.

## W9 exact-size evidence and final invariant/accessibility audit
- Status consumed: native `gentle-ai.sdd-status` v2, change `define-frontend-ui-ux-visual-system`, apply `ready`, task state 12/13 before this work, repo-local workspace `/home/luis/velay/repuestos_autos`, allowed edit roots limited to `src/ui/**` and the W9 OpenSpec artifacts. Runtime attempt acquired with token `sha256:a06ec4096bd386fb8b5bd68791a3145fdd0e4c9a7666fda636aaddfdbc69aada`; no delivery action was authorized or performed.
- Completed persisted task: W9 is checked in `tasks.md`. The new `src/ui/w9-evidence-audit.test.ts` is the narrowly scoped audit surface; it records the exact reference pairs 1200×800 and 960×640, verifies mounted-suite presence, responsive source contracts, shared accessibility seams, Spanish/`Bs`/whole-unit cues, and protected native/delivery paths.
- Mounted evidence: the existing mounted suites plus the W9 audit suite ran through the configured frontend runner. `npm test` passed **154/154** twice in separate processes. This is jsdom/rendered-behavior evidence and authored CSS/source-contract evidence; it is not a native window measurement or screenshot claim.
- Exact-size record: 1200×800 is covered by the authored desktop composition contracts (208px shell, screen layouts, 44/48px controls); 960×640 is covered by the authored compact contracts (176px shell, stacked primary layouts, labeled data rows, no core-task horizontal overflow). No native geometry was inferred from jsdom.
- Accessibility/invariant audit: mounted behavior and source checks cover keyboard/focus paths, semantic names, live status/alert/busy intent, field associations, modal containment/return, reduced motion, forced colors, target sizing, grayscale/non-color text cues, Spanish copy, `Bs` presentation, positive whole-unit stock, persisted historical facts, and no command/IPC/Rust/persistence/package changes.
- RED/GREEN/TRIANGULATE/REFACTOR: RED was the preflight finding that no W9 exact-size evidence record existed; GREEN was the four-case audit suite passing; TRIANGULATE was two full `npm test` runs plus TypeScript, build, and diff checks; REFACTOR kept the audit as one small evidence seam without changing product behavior.
- Verification commands: `./node_modules/.bin/tsx --test src/ui/w9-evidence-audit.test.ts` PASS 4/4; `npm test` PASS 154/154 twice; `./node_modules/.bin/tsc --noEmit` PASS; `npm run build` PASS with the pre-existing mixed static/dynamic Tauri import warning; `git diff --check` PASS. Intentional negative ConfirmationDialog logs remain expected test output.
- Manual/screenshot availability: **UNRESOLVED and release-blocking for native-desktop claims.** The current environment is Linux `tty` with no `DISPLAY` or `WAYLAND_DISPLAY`, and no supported Windows desktop/manual capture was available; therefore no screenshot, native exact-size geometry, 200% zoom, or manual grayscale/contrast claim is made. Contrast measurements remain unresolved where the design brief marks candidates for validation.
- Deviations: none to product behavior or architecture. No browser E2E framework, generated HTML/assets, CDN dependency, mockup behavior, command/IPC/Rust/persistence mutation, or commit/push/PR/delivery was added.
- Remaining tasks: none in `tasks.md`; W9 completion is persisted, but the native-desktop evidence blocker above must be resolved by a supported environment before the corresponding release claims can pass verification.
