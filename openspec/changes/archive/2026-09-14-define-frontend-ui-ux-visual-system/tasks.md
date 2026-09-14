# Implementation Tasks: Frontend UI/UX Visual System

## Status and delivery boundary

Native roadmap status supplied for this delegation is **Ready for task planning only**. This document is the implementation plan; it is not implementation authorization. The existing proposal/design stop gate is superseded only for creating this plan. Apply remains stopped until ticket 03 is complete and a separate explicit user authorization is given.

- `delivery_strategy: exception-ok`
- `chain_strategy: stacked-to-main`
- Review budget: **400 changed lines** (`additions + deletions`) for every normal review slice. PR4A has the explicit bounded `size:exception` below; PR4B and every other slice remain subject to the normal cap.
- Strict TDD is **false** in `openspec/config.yaml`; the plan still records exhaustive RED/GREEN/TRIANGULATE/REFACTOR evidence for each behavior slice.
- There are **13 tasks in 13 work units**. W0 is the mandatory prerequisite; W1 → W2A → W2B → W2C → W3 → W4A → W4B → W5–W9 are strictly ordered visual implementation/evidence slices.
- W0, W1, W2A, W2B, W2C, W3, W4A, W4B, and W5 are checked. W6–W9 remain unchecked.
- The failed combined candidate has been separated: PR2B retains only Panel and AlignedData, while the unverified dialog candidate is preserved outside the worktree for a separately authorized PR2C. The prior 349/400 combined boundary is failed evidence, not verification of PR2C.
- Maintainer decision: PR4A is one independently usable and revertible PR with a bounded `size:exception` approved up to **1,050 changed lines** for the listed W4A scope only. Stop and ask before apply if PR4A would exceed 1,050; PR4B receives no exception.

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 2,700–4,500 total; PR4A is bounded at roughly 700–1,050, PR4B is forecast at 260–350, and normal slices remain roughly 100–395 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | Ordered stacked-to-main PRs: PR 0 ticket 03 harness → PR 1 tokens/assets → PR 2A controls/state → PR2B Panel + AlignedData → PR2C ConfirmationDialog → PR 3 shell → PR4A Sales draft/confirmation lifecycle (`size:exception` ≤1,050) → PR4B persisted confirmed summary (260–350, normal cap) → PR 5 Inventory → PR 6 Catalog → PR 7 History → PR 8 continuity → PR 9 evidence/audit |
| Delivery strategy | exception-ok |
| Chain strategy | stacked-to-main |

Decision needed before apply: No
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

The maintainer selected ordered stacked-to-main delivery and explicitly approved the bounded PR4A exception. W0→W1→W2A→W2B→W2C→W3→W4A→W4B→W5→W6→W7→W8→W9 will be delivered as independently reviewable, test-green, and revertible PRs. PR4A is the only over-budget allowance, capped at 1,050 changed lines and limited to its listed Sales draft scope; ask for a new decision if it would exceed that bound. PR4B is a separate normal-cap PR forecast at 260–350 changed lines and has no exception. PR2B and PR2C must not be recombined, and the failed combined candidate must not be treated as verified.

## Source-authority checklist

1. **Current product authority:** preserve the existing React state navigation and screen/flow behavior in `src/ui/app.ts`, `src/ui/**/**-screen.ts`, `src/ui/**/**-flow.ts`, and the command adapters under `src/commands/**`; Rust/IPC/persistence remain authoritative and unchanged.
2. **OpenSpec authority:** use `proposal.md`, `specs/frontend-visual-system/spec.md`, and `design.md` in this change for requirements, scenario mapping, tokens, Spanish copy, `Bs` presentation, responsive sizes, accessibility, and non-goals.
3. **Testing authority:** use `openspec/config.yaml` and the mounted-harness contract in `.scratch/product-roadmap/issues/03-establish-mounted-react-test-harness.md`; do not claim mounted evidence before W0 is complete.
4. **Mockup authority:** Engram V3 review decisions are visual references only. They may inform warm-industrial composition, never behavior, facts, routes, or assets.
5. **Explicit exclusions:** do not add customers, tax, discounts, printing, receipts, accounts, roles, refunds, payment gateways, remote/cloud services, barcode workflows, global search, pagination, sorting, reports, bulk actions, or invented confirmation behavior. Do not copy generated HTML or use CDN dependencies.
6. **Presentation boundary:** use Spanish UI and `Bs` display/input while leaving integer-centavo command/Rust contracts, whole-unit inventory, persisted sale-time facts, request IDs, stale-price acknowledgement, restore acknowledgement, correction eligibility, and retry/reload behavior intact.

## Dependency graph and task rules

```text
W0 ticket 03 mounted harness
  └─> W1 tokens/assets/reset
       └─> W2A controls and state communication
            └─> W2B Panel + AlignedData only
                 └─> W2C type-restricted ConfirmationDialog only
                      └─> W3 persistent shell
                           └─> W4A Sales draft, discovery, cart, payment, and confirmation lifecycle
                                └─> W4B persisted confirmed-sale summary
                                     └─> W5 Inventory
                                          └─> W6 Catalog
                                               └─> W7 History/detail/return/cancellation
                                                    └─> W8 Onboarding + Backup/Restore continuity
                                                         └─> W9 exact-size evidence and invariant audit
```

Delivery order is W0→W1→W2A→W2B→W2C→W3→W4A→W4B→W5→W6→W7→W8→W9 as ordered stacked-to-main PRs. Every W1–W9 task is blocked unless W0 is complete; W3 is additionally blocked until W2C is verified, W4A is blocked until W3 is verified, W4B is blocked until W4A is verified, and W5 is blocked until W4B is verified. Keep each screen's behavior tests beside the screen/primitive changes it verifies. Use rendered output, semantic queries, callbacks, and state transitions rather than implementation-detail selectors.

## Work units

### W0 — Parent prerequisite: complete roadmap ticket 03 mounted React test harness

- [x] Complete and verify roadmap ticket 03 before any visual implementation task. <!-- sdd-owner: implementation -->

**Dependency:** none. This is the enforceable parent gate for W1–W9; its unchecked state blocks every child task.

**Start state:** `npm test` runs only existing headless/flow tests; no representative React screen is mounted with controlled command adapters, user events, cleanup, and semantic assertions.

**Likely edit/discovery surfaces for the ticket:** `package.json`, `tsconfig.json`, the test setup/configuration selected by ticket 03, and one representative mounted test under `src/ui/**`. The ticket itself is read-only input to this plan and must retain its non-goal of not introducing browser end-to-end infrastructure.

**Finish state:** complete via roadmap ticket 03 and PR #129, merged into `master` as `c8022ddba2d8c3dd02cbe3bb589508256da2e1a3`. One representative screen mounts under the project runner; semantic queries and keyboard/user events work; deterministic command mocks prove pending/success/failure; setup and cleanup isolate tests; post-merge mounted tests passed 3/3 twice, with the full suite at 65/65 and TypeScript/build/diff checks passing. GitHub reported zero checks, so this records local/post-merge evidence rather than CI approval; development evidence also includes a deliberately absent visible behavior causing a semantic assertion failure before the behavior was restored.

**Evidence sequence:** RED is the deliberate failing semantic assertion; GREEN is the smallest harness and representative screen behavior that passes; TRIANGULATE is the twice-run isolation and pending/success/failure check; REFACTOR is setup/cleanup simplification without weakening rendered-behavior assertions. Strict TDD remains disabled, so this is evidence required by ticket 03, not a project-wide strict-TDD switch.

**Acceptance/scenario mapping:** roadmap ticket 03 acceptance criteria; enables visual-system R1/R2/R8/R9/R11/R14 and mounted evidence for S03–S05, S20–S25, S29–S42.

**Rollback boundary:** remove only ticket 03 harness setup, representative mounted test, and test-only configuration additions.

**Non-goals:** no visual redesign, CSS/assets, command-contract changes, browser E2E stack, or screen-wide behavior change.

### W1 — Semantic tokens, offline assets/fallbacks, and global base layer

- [x] Implement and verify the light-first semantic token, asset/fallback, reset, and base-CSS layer after W0 is complete. <!-- sdd-owner: implementation -->

**Dependency:** W0 complete and twice-run mounted-harness evidence available.

**Start state:** no CSS files, token vocabulary, font declarations, UI icon source, or authored focus/reduced-motion rules; browser defaults provide layout and focus.

**Likely edit surfaces:** new `src/ui/styles.css` or equivalent global stylesheet; a narrowly scoped `src/ui/visual-system/tokens.ts` or token stylesheet if needed; `src/main.ts` for one stylesheet import; local asset discovery/additions under `src/assets/**` only after rights are verified; `package.json` only if an approved offline icon package is genuinely required. Existing `src/ui/**` remains behavior authority.

**Finish state:** semantic tokens cover canvas/surfaces/text/borders/action/focus/status/disabled, typography, spacing, geometry, elevation, and future-theme role separation; Barlow/IBM Plex Mono are packaged locally with Spanish glyphs and license metadata or documented Segoe UI/Cascadia Mono/Consolas fallbacks are used; Lucide is local/licensed if selected or accessible text/approved fallback icons are retained; reset/base styles provide 44px targets, 48px Sales controls, tabular numerals, authored focus, forced-colors-safe boundaries, reduced-motion behavior, and desktop reflow foundations without hard-coded screen names.

**Evidence sequence:** RED: mounted style/interaction checks expose missing semantic focus, target, Bs/numeric, or reduced-motion treatment; GREEN: token/base layer satisfies those rendered checks; TRIANGULATE: inspect representative Sales, Inventory, and modal states at both width contracts and run `npm test`; REFACTOR: remove duplicated raw values and retain semantic aliases with no behavior changes.

**Acceptance/scenario mapping:** R3–R7, R13–R14, R20–R21; S06–S19, S36–S42, S55–S59. Contrast ratios must be measured or explicitly reported as unresolved; no unmeasured color is claimed compliant.

**Rollback boundary:** revert only token stylesheet/module, base import, and newly added approved local asset/license files; do not revert W0 harness.

**Non-goals:** no dark-mode UI, brand rework beyond the simple wordmark direction, CDN/network font delivery, generated HTML, business formatting authority, or screen-specific layout.

### W2A — Controls and state communication

- [x] Implement and verify action, field, inline feedback, badge, and narrow Inventory pending-button adoption after W1. <!-- sdd-owner: implementation -->

**Dependency:** W1 complete; W2B is blocked until W2A is verified. **Chain slice:** PR2A, stacked-to-main after PR1 and before PR2B. **Forecast:** ~347 changed lines, below the 400-line review budget.

**Start state:** action, field, feedback, and badge markup is repeated native markup with inconsistent labels, severity, targets, and pending treatment; there is no shared control/state seam. The existing Inventory confirmation button remains the only permitted real-screen adoption target for this slice.

**Likely edit surfaces:** new `src/ui/visual-system/action.ts`, `src/ui/visual-system/field.ts`, `src/ui/visual-system/feedback.ts`, and `src/ui/visual-system/badge.ts`; colocated `src/ui/visual-system/action.test.ts`, `field.test.ts`, `feedback.test.ts`, and `badge.test.ts`; primitive selectors in `src/ui/styles.css`; and the existing confirmation-button render plus focused mounted assertions in `src/ui/inventory/inventory-screen.ts` and `src/ui/inventory/inventory-screen.mounted.test.ts` (with `src/ui/inventory/inventory-screen.test.ts` only if its public render contract needs coverage). Do not edit `src/ui/inventory/inventory-flow.ts`, `src/commands/**`, or other screens.

**Finish state:** a small, deep public interface provides primary/secondary/tertiary/destructive actions and an `IconAction` whose accessible name is mandatory and non-empty; a native-control `Field` shell covers text, search, date, select, checkbox, integer quantity, `Bs` money, and SKU; `Feedback` keeps routine validation/pending/success/advisory/retry/failure inline with status, alert, busy, or field-error intent; and `Badge` requires explicit visible text such as `Stock bajo: 1`, `Sin stock: 0`, `Activo`, or `Desactualizado`. Every target remains at least 44×44 CSS px, pending blocks duplicate activation, and only Inventory's existing pending confirmation button is replaced through the shared action interface.

**TDD evidence:** RED: mounted primitive tests fail for unnamed `IconAction`, native role/name and error association, pending duplicate activation, inline live intent, or text-independent badge meaning; the Inventory mounted test fails when the existing confirmation button is not the shared pending-aware action. GREEN: rendered-output and user-event tests pass for every control family and the narrow Inventory button substitution preserves its existing idle, disabled, pending, success, and failure observations. TRIANGULATE: exercise each public interface through semantic queries at least once, run the focused primitive and Inventory mounted tests twice for isolation, and run the existing Inventory flow tests without changing transitions or command payloads. REFACTOR: remove duplicated primitive markup/raw values and keep control complexity behind the shared interfaces without exposing implementation details or changing screen layout.

**Acceptance/scenario mapping:** R4, R5, R7–R9, R11, R14, and R20–R21; S09–S25, S29–S32, and S39–S42; F15 action, field, feedback, badge, focus, and licensing/state-sheet variants. Routine feedback stays inline and no modal is introduced by W2A.

**Rollback boundary:** revert only the four W2A visual-system modules, their colocated tests/style rules, and the one Inventory confirmation-button import/render substitution; retain W0/W1 and all Inventory flow, command, and other screen behavior.

**Non-goals:** no screen redesign, Spanish rewrite, flow change, new business state machine, routing, global notification service, toast-first feedback, generic filesystem/IPC abstraction, additional Inventory adoption, modal confirmation, or invented action.

### W2B — Panel and AlignedData only

- [x] Implement and verify Panel and AlignedData structural data behavior after W2A. <!-- sdd-owner: implementation -->

**Dependency:** W2A complete; W2C is blocked until W2B is verified. **Chain slice:** PR2B, stacked-to-main after PR2A and before PR2C. **Forecast:** ~190–235 changed lines, below the 400-line review budget.

**Start state:** shared controls and state communication are available, but named structural regions and aligned data presentation are absent. The existing partial combined candidate contains structural and dialog work; during apply, separate it before implementation evidence is recorded. Panel and AlignedData are the only W2B behavior targets. No dialog is in W2B.

**Exact edit surfaces:** retain or isolate `src/ui/visual-system/structure.ts` for `Panel` and `AlignedData`; split structural cases into `src/ui/visual-system/structure.mounted.test.ts` (the existing `src/ui/visual-system/structure-dialog.mounted.test.ts` is a transition/discovery surface only and is not verification); add or colocate structural public-interface tests under `src/ui/visual-system/structure.test.ts` if needed; and edit only the Panel/AlignedData selectors and responsive labeled-row rules in `src/ui/styles.css`. `src/ui/visual-system/confirmation-dialog.ts`, dialog tests, and dialog selectors are PR2C surfaces, not W2B surfaces. `src/ui/sales/history-screen.ts` and `src/ui/backup/backup-screen.ts` remain read-only discovery/reference surfaces and unchanged until W7/W8.

**Finish state:** `Panel` renders a named semantic region (`section` with resolved `aria-labelledby` and heading) with restrained surface/border/spacing structure. `AlignedData` renders native table semantics with caption, `thead`/`tbody`, visible column headers and `scope=col`; preserves caller value strings and DOM/logical order; assigns start/end alignment and text/SKU/numeric/money/date/value-kind metadata; mirrors headers as responsive labels without mutating data/order; and fails fast before row creation when any row has the wrong column count. It implies no sorting, pagination, editing, bulk behavior, or dialog.

**TDD evidence (RED → GREEN → TRIANGULATE → REFACTOR):** RED: structural tests fail for the semantic region/name, table/caption/header scopes, logical order, alignment/value kinds, responsive labels, and column-count mismatch. GREEN: rendered-output and semantic-query tests pass for Panel and AlignedData, including product, SKU, stock, sale, date, amount, lifecycle, and correction fixtures. TRIANGULATE: run only the separated structural test suite twice, inspect 960×640-oriented wrapping and label fixtures, run `npm test`, `./node_modules/.bin/tsc --noEmit`, and `npm run build`, and confirm no dialog, History/Backup screen, flow, command, IPC, Rust, or persistence surface changed. REFACTOR: deepen the structural interfaces, centralize structural selectors, and remove duplicated data markup without changing caller values or order.

**Acceptance/scenario mapping:** R10 S26–S28; structural portions of R14 S39–S40 and R20 S55–S57; F15 Panel and aligned-data variants. This slice does not claim modal confirmation or native 960×640 geometry; those belong to W2C or W9 respectively.

**Rollback boundary:** revert only the W2B structural module(s), separated structural tests/fixtures, and Panel/AlignedData CSS rules; retain W0/W1/W2A and the pre-existing dialog candidate for the separately planned W2C remediation. If separation cannot be cleanly isolated, stop and ask rather than restoring the failed combined candidate as verified.

**Non-goals:** no ConfirmationDialog or modal behavior, no History/Backup adoption, no screen redesign, no flow/command/IPC/Rust/persistence change, no new business state, no sorting/pagination/editing/bulk behavior, no Spanish rewrite, and no native geometry or W9 claims.

### W2C — Type-restricted ConfirmationDialog only

- [x] Implement and verify the type-restricted ConfirmationDialog after W2B. <!-- sdd-owner: implementation -->

**Dependency:** W2B complete and separately verified; W3 is blocked until W2C is verified. **Chain slice:** PR2C, stacked-to-main after PR2B and before PR3. **Forecast:** ~245–305 changed lines, below the 400-line review budget.

**Start state:** the partial combined candidate already has a `ConfirmationDialog` surface but formal verification failed. Its prior 349/400 combined boundary is not verification. During apply, move only dialog implementation, dialog CSS, and dialog assertions out of `src/ui/visual-system/structure-dialog.mounted.test.ts` into the PR2C surfaces; do not carry combined W2B/W2C evidence or mark either task checked from that candidate.

**Exact edit surfaces:** `src/ui/visual-system/confirmation-dialog.ts`; separated `src/ui/visual-system/confirmation-dialog.mounted.test.ts` and `src/ui/visual-system/confirmation-dialog.test.ts` for public behavior/type evidence; and only ConfirmationDialog selectors in `src/ui/styles.css`. `src/ui/visual-system/structure.ts`, structural tests, Panel/AlignedData CSS, all History/Backup screens, and all flow/command/IPC/Rust/persistence files are read-only or PR2B-owned and must not be changed by W2C.

**Finish state:** the public `ConfirmationDialog` interface accepts exactly `restore | cancellation`, preserves caller-owned open/pending/confirm/cancel state plus existing reason and acknowledgement content, and provides an accessible dialog name/description. Focus behavior must satisfy every formal defect case: React StrictMode effect replay leaves focus inside the open dialog; hidden, `aria-hidden`, inert, display-none, visibility-hidden, disconnected, or otherwise non-rendered descendants are excluded from the focusable set; an opened-pending dialog focuses a safe internal container/static target when both actions are disabled; valid default and explicit initial-focus targets are honored while invalid/hidden/disabled explicit targets fall back safely; Tab and Shift+Tab wrap over currently valid focusables; Escape and safe `Volver` cancel only while idle; pending locks cancellation/confirmation and prevents duplicate activation; and focus restores to the invoker after idle cancel, parent-driven close, and unmount when the invoker remains connected, without attempting disconnected restoration. Fresh rerenders and stale refs must not escape containment.

**TDD evidence (RED → GREEN → TRIANGULATE → REFACTOR):** RED: mounted tests reproduce and fail for StrictMode effect replay, hidden/non-rendered initial and reverse-trap endpoints, opened-pending focus with disabled actions, invalid explicit focus, default focus, Tab/Shift+Tab wrapping, idle Escape/`Volver`, pending duplicate/escape lock, and cancel/parent-close/unmount restoration. GREEN: rendered-output, semantic-query, keyboard/user-event, and type tests pass for both permitted purposes and every listed lifecycle case. TRIANGULATE: run the separated dialog suite twice under the real React DOM harness, including `React.StrictMode`, hidden descendants and hidden ancestors, `inert`/CSS-hidden cases, initially pending and no-valid-action cases, rerender/ref replacement, connected/disconnected invokers, then run `npm run test:mounted`, `npm test`, `./node_modules/.bin/tsc --noEmit`, `npm run build`, and `git diff --check`; confirm no W2B structural or screen/flow/command contract changed. REFACTOR: keep the dialog interface small and deep, centralize focus eligibility/lifecycle handling, remove duplicated selectors, and preserve caller-owned workflow semantics.

**Acceptance/scenario mapping:** R12 S33–S35; dialog portions of R14 S39–S40; R20 S55–S57 dialog/focus/pending variants; F15 modal, focus, lifecycle, pending, Escape, and restoration variants. Modal use remains restricted to restore and irreversible cancellation; routine feedback stays inline.

**Rollback boundary:** revert only `confirmation-dialog.ts`, separated dialog tests/fixtures, and ConfirmationDialog CSS rules, retaining the independently verified W2B structural slice and W0/W1/W2A. Do not roll back or rewrite screen adoption because no screen adopts this dialog until W7/W8.

**Non-goals:** no Panel or AlignedData changes, no History/Backup adoption, no new destructive workflow or eligibility rule, no modal for archive/reactivate, return, save, search, retry, cart removal, or draft discard, no flow/command/IPC/Rust/persistence change, no screen redesign, and no native geometry, zoom, contrast, or W9 completion claim.

### W3 — Persistent app shell without navigation behavior changes

- [x] Implement and verify the persistent shell, navigation presentation, and active-location cue after W2C; defer the truthful stock-exception cue to W5. <!-- sdd-owner: implementation -->

**Dependency:** W2C complete; W0 remains a hard prerequisite.

**Start state:** `src/ui/app.ts` renders the full navigation only on Sales and plain Sales return buttons elsewhere; screen state is React state with no URL routing, identity, persistent shell, active location, or consistent labels.

**Likely edit surfaces:** `src/ui/app.ts`; new shell composition module under `src/ui/visual-system/**` or `src/ui/shell/**`; mounted app/navigation tests under `src/ui/**.test.ts`; only the minimum existing screen props needed to remove duplicate return controls. Do not alter `SCREEN`, `NAVIGATION_ACTION`, `screenAfter`, command adapters, or flow semantics.

**Finish state:** one shell wraps Sales, Inventory, Catalog, Product onboarding, Sales History, and Backup/Restore; Spanish labels are exactly `Ventas`, `Inventario`, `Catálogo`, `Alta de productos`, `Historial de ventas`, and `Copia y restauración`; identity, active location, keyboard order, and focus treatment are consistent at 1200×800 and 960×640. The persistent shell shows the neutral exact Spanish `Inventario` navigation label only: W3 renders no count, dot, warning, helper, or other cue that implies live stock information. Navigation still switches the same React screen states, starts at Sales, and does not promise draft preservation.

**Evidence sequence:** RED: mounted app tests fail when each navigation item is absent, active state is ambiguous, or navigation changes the wrong screen; GREEN: semantic navigation tests pass for all six destinations and Sales default; TRIANGULATE: keyboard traversal, 44px targets, minimum-size rendering, and screen-specific smoke assertions; REFACTOR: keep shell interface small and remove duplicate return wrappers without changing transitions.

**Acceptance/scenario mapping:** R1–R2 (shell/navigation presentation only; the stock-exception scenario is deferred to W5), R7, R13–R14; S01–S05, S17–S19, S36–S42. W3 accepts the shell and neutral `Inventario` navigation only; the live stock-exception cue is not a W3 acceptance claim.

**Rollback boundary:** revert `src/ui/app.ts`, shell module/styles, and shell tests; retain shared primitives and base tokens.

**Non-goals:** no URL routes, accounts, roles, permissions, breadcrumbs, global search, shortcuts, draft persistence, live stock count/dot/warning/helper in the shell, alert polling, or new navigation destinations.

### W4A — Sales draft, discovery/search, cart, payment, and confirmation lifecycle

- [x] Implement and verify the Sales draft and confirmation lifecycle after W3 within the approved bounded size exception. <!-- sdd-owner: implementation -->

**Dependency:** W3 complete and verified; W4B is blocked until W4A is verified. W0 mounted screen evidence remains satisfied. **Chain slice:** PR4A, stacked-to-main after PR3 and before PR4B. **Forecast:** roughly 700–1,050 changed lines, with a maintainer-approved `size:exception` hard ceiling of **1,050 changed lines** (`additions + deletions`). This is one PR, independently usable and independently revertible. Stop and ask before apply if the measured PR4A diff would exceed 1,050; this exception applies only to the listed W4A scope.

**Start state:** `src/ui/sales/sale-screen.ts`, `src/ui/sales/catalog-result.ts`, and `src/ui/sales/sale-flow.ts` expose mostly English prose lists, raw centavo-facing inputs, no draft total, incomplete discovery/search state treatment, and insufficient protection against duplicate or late asynchronous completion.

**Exact edit surfaces:** `src/ui/sales/sale-screen.ts`, `src/ui/sales/catalog-result.ts`, and `src/ui/sales/sale-flow.ts` for the Sales draft, presentation-safe parsing/state wiring, request identity, synchronous duplicate guard, and lifecycle cleanup; Sales-only selectors in `src/ui/styles.css`; existing `src/ui/sales/catalog-result.test.ts` and `src/ui/sales/sale-flow.test.ts`; and a mounted rendered-behavior test at `src/ui/sales/sale-screen.mounted.test.ts`. `src/ui/sales/persisted-summary.ts` and `src/ui/sales/persisted-summary.test.ts` are W4B surfaces, not W4A implementation surfaces. `src/commands/catalog.ts`, `src/commands/confirm-sale.ts`, all Tauri/IPC/Rust files, and persistence remain unchanged.

**Finish state:** Sales remains independently usable with the shared shell and Spanish labels. At 1200×800 it exposes catalog discovery/results and cart beside payment/summary; at 960×640 it reflows in logical order—search/results, cart, payment, total/actions—with no horizontal core-task scroll. The draft visibly and separately covers initial/search-loading/no-results/populated/failure states; product identity, SKU, category, stock, immutable price, whole-unit cart quantity, line subtotal, draft total, `Bs` cash and QR fields, presentation-only parsing to the existing integer-centavo payload, stale-price acknowledgement, localized field/form validation and command failures, immediate discard, and confirmation idle/pending/success/failure lifecycle. Pending and duplicate confirmation are guarded synchronously. Late search/confirmation responses are ignored after a newer request, discard, or unmount, with no stale state restoration or unmounted update. W4A preserves the existing confirmation result handoff; W4B owns the persisted summary presentation.

**Exhaustive TDD evidence (RED → GREEN → TRIANGULATE → REFACTOR):** RED: mounted rendered-output and user-event tests fail for the initial search instruction, loading state, retained query/no-results state, populated result identity/SKU/stock/price and `Agregar`, cart line addition/removal, positive whole-unit quantity validation, line subtotal and draft total, Spanish `Bs` cash/QR labels and valid/invalid parsing, stale-price acknowledgement and disabled confirmation, localized validation and command-failure feedback, idle/pending/disabled/success/failure confirmation lifecycle, synchronous duplicate activation suppression, stale search/confirmation completion, late completion after discard, and late completion after unmount. GREEN: semantic queries, callbacks, state-transition assertions, and mounted interactions pass for every listed state and preserve existing command payload shapes and lifecycle results. TRIANGULATE: run the focused catalog-result, sale-flow, and mounted Sales suites twice in separate processes; exercise delayed search and confirmation promises with newer requests, discard, unmount, rerender, failure, and success; verify keyboard order, 1200×800 and 960×640 reflow, no horizontal core-task scroll, whole-unit quantities, integer-centavo adapter values, unchanged command names/contracts, and no state update after unmount; run `npm test`, `npm run build`, `./node_modules/.bin/tsc --noEmit`, and `git diff --check`. REFACTOR: keep parsing, request/lifecycle guards, and draft transitions behind small deep interfaces, centralize repeated Sales field/feedback composition, remove duplicate markup, and retain command/IPC/Rust authority without weakening the rendered evidence.

**Acceptance/scenario mapping:** W4A covers the draft portion of R6, R8–R15 and S14–S15, S20–S25, S29–S32, and S43–S44; include F01–F02 and the applicable Sales variants on F15. Together W4A and W4B preserve the prior W4 mapping of R6, R8–R15; S14–S15, S20–S25, S29–S45; and F01–F04. Do not add customer, tax, discount, barcode, print, receipt, refund, or extra confirmation behavior.

**Rollback boundary:** revert only PR4A's `src/ui/sales/sale-screen.ts`, `src/ui/sales/catalog-result.ts`, `src/ui/sales/sale-flow.ts` presentation/lifecycle changes, Sales-only CSS, and W4A tests. The Sales draft and confirmation lifecycle must remain coherent after reverting PR4A; retain W0–W3 and do not revert W4B or unrelated screens.

**Non-goals:** no persisted confirmed-summary redesign, no changes to pricing rules, payment arithmetic authority, integer-centavo contracts, request IDs, catalog search semantics, commands, IPC, Rust, persistence, or new sale actions; no modal for routine Sales work; no customer, tax, discount, barcode, print, receipt, refund, or other invented behavior. Existing IPC inbound-decoder gaps are residual risk outside W4A and are recorded, not fixed here.

### W4B — Persisted confirmed-sale summary only

- [x] Implement and verify the persisted confirmed-sale summary after W4A, with no size exception. <!-- sdd-owner: implementation -->

**Dependency:** W4A complete and verified; W5 is blocked until W4B is verified. **Chain slice:** PR4B, stacked-to-main after PR4A and before PR5. **Forecast:** **260–350 changed lines**, within the normal 400-line cap. No `size:exception` is approved or available for W4B. This is one independently reviewable and revertible PR.

**Start state:** the confirmed-sale result is available from W4A, but `src/ui/sales/persisted-summary.ts` and its composition/test surface do not yet present an authoritative, read-only persisted outcome distinct from the editable draft.

**Exact edit surfaces:** `src/ui/sales/persisted-summary.ts`; the minimum confirmed-summary handoff/rendering changes in `src/ui/sales/sale-screen.ts`; summary-only selectors in `src/ui/styles.css`; `src/ui/sales/persisted-summary.test.ts`; and the confirmed-summary cases in `src/ui/sales/sale-screen.mounted.test.ts`. `src/ui/sales/sale-flow.ts` is read-only for W4B except for an already-established result handoff that cannot change its transitions. `src/commands/confirm-sale.ts`, all other command adapters, Tauri/IPC/Rust files, and persistence remain unchanged.

**Finish state:** the editable draft is replaced by a persisted, read-only summary with authoritative sale identity, sale date/time, original item identity/SKU, quantities, sale-time unit prices, line subtotals, payment facts, and total. It uses `Bs` display formatting and aligned numeric data, preserves unavailable historical text, and offers exactly one action: `Nueva venta`. Summary values come only from the persisted confirmed-sale result; the implementation never recomputes prices, subtotals, payments, or total from draft state or current catalog data and never substitutes current catalog facts for sale-time facts.

**Exhaustive TDD evidence (RED → GREEN → TRIANGULATE → REFACTOR):** RED: mounted and public-interface tests fail when identity/time, item/SKU, quantity, sale-time price, subtotal, payment facts, total, read-only treatment, `Bs` formatting, unavailable historical text, or the sole `Nueva venta` action is missing; fail when edit/print/share/refund/extra actions appear; and fail when any summary field is recomputed from draft/current-catalog fixtures or changes after current-catalog mutation. GREEN: rendered semantic queries pass for authoritative persisted fixtures at 1200×800 and 960×640, including labeled-row reflow, read-only facts, exact `Nueva venta`, and unchanged result handoff. TRIANGULATE: run `persisted-summary.test.ts` and the mounted Sales summary cases twice in separate processes; mutate draft and current-catalog fixtures after confirmation, use unavailable historical snapshots, verify identity/time/items/sale-time prices/subtotals/payments/total remain unchanged, check keyboard order and no horizontal core-task scroll, then run `npm test`, `npm run build`, `./node_modules/.bin/tsc --noEmit`, and `git diff --check`; confirm no W4A flow transition, command payload, decoder, IPC, Rust, or persistence surface changed. REFACTOR: keep the summary interface small and deep, centralize authoritative read-only fact rendering, remove draft/catalog recomputation paths and duplicate selectors, and preserve the sole-action contract.

**Acceptance/scenario mapping:** W4B completes R6, R7, R10–R11, R15, and the persisted-summary portions of R20; specifically S14, S17, S26–S28, S30, and S45. Include F03–F04 and the confirmed-summary variants on F15. Together with W4A, this completes the full W4 mapping stated above.

**Rollback boundary:** revert only the summary module, summary-only Sales handoff/CSS, summary tests, and mounted summary cases from PR4B; retain the independently usable W4A draft/confirmation lifecycle and all prior slices. Reverting W4B must not alter persisted sale data or command contracts.

**Non-goals:** no W4A draft/search/cart/payment parsing or lifecycle changes; no recomputation from draft/current catalog; no editing persisted facts; no new action beyond `Nueva venta`; no print, receipt, share, refund, customer, tax, discount, barcode, report, route, command, IPC, Rust, persistence, or backend contract changes. No size exception applies. Existing IPC inbound-decoder gaps remain residual risk outside W4B and are not implementation scope.

### W5 — Inventory operation, stock-priority states, and scoped stock cue

- [x] Implement and verify Inventory composition, whole-unit operation states, projections, alert priority, and the scoped truthful stock-exception cue after W4B. <!-- sdd-owner: implementation -->

**Dependency:** W4B complete and verified; W0 mounted screen evidence is mandatory.

**Start state:** `src/ui/inventory/inventory-screen.ts` and `inventory-flow.ts` render English prose, unlabeled note/reason inputs, silent alert/search loading, and a single vertical operation/alert sequence with weak low/out-of-stock distinction.

**Likely edit surfaces:** `src/ui/inventory/inventory-screen.ts`, `src/ui/inventory/inventory-flow.ts` only for presentation-safe state wiring, existing `src/ui/inventory/inventory-screen.test.ts`, new mounted Inventory tests, and shared visual-system adapters. If the cue is surfaced on the persistent `Inventario` navigation item, the minimum W5-owned integration may touch `src/ui/app.ts`, the W3 shell module, and their mounted shell assertion; that handoff must consume Inventory-owned state only. Keep `src/commands/inventory.ts` and `src/commands/catalog.ts` unchanged.

**Finish state:** at 1200×800 operation and alerts form distinct regions; at 960×640 alerts stack below the operation. Spanish labels expose selected name/SKU/current stock, `Entrada de stock` or `Conteo físico`, whole-unit input, note/reason, projected balance, inline advisory/stale/error/retry, pending, success, and primary action. `Stock bajo: 1` and `Sin stock: 0` use text/icon/structure, never color alone; no supplier/cost or extra confirmation is implied. When Inventory owns and refreshes its existing alert state, W5 provides a truthful stock-exception cue on the `Inventario` navigation item using only the existing alert text/count/icon. The cue is shown only while that alert state is available, is hidden while loading/unavailable/cleared, and is removed when Inventory unmounts or another screen replaces it; it must not falsely persist as cached shell state.

**Evidence sequence:** RED: mounted scenarios fail for no selection, search no-results/loading, stock entry, physical count reason validation, pending duplicate prevention, stale projection, success/failure, exact one/zero stock cues, and a shell cue that persists after Inventory unmount; GREEN: semantic rendered behavior passes and the cue appears only from available Inventory-owned alert state; TRIANGULATE: verify positive whole-unit controls, projection facts, keyboard order, 1200/960 reflow, mount/unmount and unavailable-state clearing, and existing flow tests; REFACTOR: keep operation and alert modules deep, keep the shell handoff narrow, and remove duplicate severity markup.

**Acceptance/scenario mapping:** stock-exception portion of R2, plus R6, R8–R14, R16; S16, S20–S25, S29–S42, S46–S47. Include F05–F06 evidence expectations. The cue must use existing Inventory state only, with no additional global fetch, polling, notification, or cross-screen cache.

**Rollback boundary:** revert Inventory composition, presentation-only flow wiring, Inventory tests, and the scoped `src/ui/app.ts`/W3 shell cue handoff and assertions; retain prior slices.

**Non-goals:** no inventory business-rule changes, fractional stock, supplier/cost fields, physical-count modal, additional global fetch/polling/notification, global alert store, cached stock cue, cue persistence after Inventory unmount, or new recovery command.

### W6 — Catalog maintenance master-detail

- [x] Implement and verify Catalog maintenance master-detail, lifecycle, dynamic fields, and recovery states after W5. <!-- sdd-owner: implementation -->

**Dependency:** W5 complete; W0 remains a hard prerequisite.

**Start state:** `src/ui/catalog/catalog-maintenance-screen.ts` and `catalog-maintenance-flow.ts` render English list/detail prose, centavo price editing, weak empty/loading presentation, and lifecycle actions without shared severity or selected-record hierarchy.

**Likely edit surfaces:** `src/ui/catalog/catalog-maintenance-screen.ts`, `src/ui/catalog/catalog-maintenance-flow.ts` only for presentation-safe state exposure, colocated flow tests, new mounted Catalog tests, and shared data/field/badge modules. Keep `src/commands/catalog.ts` payloads and revision semantics unchanged.

**Finish state:** 1200×800 presents a scannable master list beside detail/editor; 960×640 stacks the list above detail with selected identity retained. Active/archived, product/category facts, editable metadata, dynamic attributes, `Bs` catalog price presentation, read-only historical-price helper, field errors/focus, busy/pending, success, unavailable/retry, stale reload, immediate existing `Archivar`, and `Reactivar` remain distinct and non-color-coded. Empty records receive an explicit state.

**Evidence sequence:** RED: mounted tests fail for selection context, archived versus active cues, dynamic required/option/number errors, pending disabled controls, stale/unavailable recovery, and 960 stacking; GREEN: semantic rendered behavior passes; TRIANGULATE: assert command calls preserve expected revision/input contracts, run existing flow tests, and verify keyboard reading order; REFACTOR: keep master/detail and editor interfaces small while localizing form mapping.

**Acceptance/scenario mapping:** R4, R6–R14, R17, R20–R21; S09–S19, S23–S32, S36–S42, S48–S49. Include F07–F08 evidence expectations.

**Rollback boundary:** revert Catalog composition/presentation wiring and tests only; retain W1–W5 shared layers.

**Non-goals:** no filtering/sorting, pagination, bulk edit, lifecycle policy, archive modal, history mutation, or current-value substitution for persisted history.

### W7 — Sales History list/detail/return/cancellation

- [x] Implement and verify History list, persisted detail, inline return, and destructive cancellation modal after W6. <!-- sdd-owner: implementation -->

**Dependency:** W6 complete; this is the largest screen slice and must be split before apply if its measured diff approaches 400 lines.

**Start state:** `src/ui/sales/history-screen.ts` and `history-flow.ts` render English dense prose, distinct list/detail views without the shared shell, inline cancellation without a modal, and correction states that need shared feedback/data patterns while preserving existing focus recovery.

**Likely edit surfaces:** `src/ui/sales/history-screen.ts`, `src/ui/sales/history-flow.ts` only where presentation-safe state wiring is needed, existing `src/ui/sales/history-flow.test.ts`, new mounted History list/detail/correction tests, and shared modal/data/feedback modules. Keep `src/commands/sales-history.ts`, `src/commands/post-sale.ts`, request IDs, eligibility, persisted facts, and reload interaction unchanged.

**Finish state:** list supports bounded Spanish `Desde`/`Hasta`, loading/empty/ready/bounded/error/retry; detail keeps original sale identity, sale-time products/prices/payments, correction history, unavailable historical text, and eligible actions visually distinct. Return remains inline with whole-unit line selection/quantity, field errors/focus recovery, pending/failure/reload, and persisted evidence. Cancellation retains reason and acknowledgement, then uses a modal with `Volver`, consequence text, contained focus/Escape/return focus, pending, failure/reload, and refreshed `Cancelada` evidence. At 960×640 all content is readable in logical stacked order; no pagination/export/report/sorting is added.

**Evidence sequence:** RED: mounted tests fail for list states, detail persistence, unavailable snapshots, return selection/validation/focus/pending/reload, cancellation acknowledgement/modal focus/pending/failure/success; GREEN: semantic user-event scenarios pass; TRIANGULATE: run existing history flow tests, assert no current-catalog substitution, verify 1200/960 reflow and exact modal-only destructive behavior; REFACTOR: deepen detail/correction interfaces and eliminate repeated status/action markup without changing flow transitions.

**Acceptance/scenario mapping:** R4, R6, R8–R14, R18, R20–R21; S09–S19, S20–S42, S50–S52. Include F09–F14 evidence expectations and full F15 correction variants.

**Rollback boundary:** revert History composition, presentation-only flow wiring, modal/data adapters, and tests as one unit; retain prior shared layers and screens.

**Non-goals:** no edit of persisted history, refund behavior, return modal, cancellation rule changes, report/export/pagination, or invented product/customer data.

### W8 — Onboarding and Backup/Restore continuity

- [x] Implement and verify shared-shell continuity for Product onboarding and Backup/Restore after W7. <!-- sdd-owner: implementation -->

**Dependency:** W7 complete; W0 mounted evidence remains required even though these are continuity slices rather than hero frames.

**Start state:** `src/ui/onboarding/onboarding-screen.ts`, `onboarding-form.ts`, `src/ui/backup/backup-screen.ts`, and `backup-flow.ts` use long English forms, raw centavo-facing onboarding price, silent category loading, unguarded duplicate submissions, and inline restore confirmation.

**Likely edit surfaces:** onboarding and backup screen/flow/form files named above; new mounted continuity tests under their existing directories; shared fields, feedback, modal, and shell modules. Preserve `src/commands/onboarding.ts`, `src/commands/backup.ts`, native file-dialog calls, restore token/acknowledgement, and all existing state transitions.

**Finish state:** Onboarding uses the shell and two clear panels, shared Spanish fields, dynamic category attributes, `Precio de catálogo (Bs)`, positive whole-unit opening stock, empty-category/loading/validation/pending/success/failure feedback, and no invented wizard/draft. Backup/Restore uses separate panels, native dialog handoff, idle/prepared/invalid/expired/unavailable/pending/success/failure states, existing replacement acknowledgement, and the shared destructive modal titled `Restaurar datos locales`; no cloud/scheduling behavior appears.

**Evidence sequence:** RED: mounted tests fail for category loading/empty, dynamic fields, Bs/whole-unit labels, command failure/pending, native picker cancellation, prepared restore acknowledgement, and modal focus; GREEN: rendered continuity scenarios pass; TRIANGULATE: verify command payloads and flow tests remain unchanged, keyboard/live intent, reduced motion, and no duplicate activation; REFACTOR: consolidate shared form/recovery usage and keep screen-specific composition local.

**Acceptance/scenario mapping:** R8–R14, R19–R21; S20–S42, S53–S54, plus F15 onboarding/backup variants. Restore is the only continuity destructive modal; native dialogs are not restyled.

**Rollback boundary:** revert onboarding/backup composition and tests together; retain shared primitives and primary workflow slices.

**Non-goals:** no onboarding wizard, draft preservation, cloud backup, scheduling, new restore command, or backend contract change.

### W9 — Exact-size mounted/screenshot/manual evidence and final invariant audit

- [x] Produce and verify the final exact-size evidence bundle and invariant/accessibility audit after W8. <!-- sdd-owner: implementation -->

**Dependency:** W0–W8 complete, each prior slice verified, and any review-chain decision resolved before apply.

**Start state:** all primary and continuity screens have shared implementation and focused tests, but there is no consolidated proof at the 1200×800 starting size and 960×640 minimum, no complete mounted/screenshot/manual review record, and no final cross-screen invariant audit.

**Likely edit/verification surfaces:** mounted tests and test fixtures under `src/ui/**`; existing `package.json` commands only; any narrowly scoped test-harness viewport helper from W0. Do not add a browser E2E framework or generated HTML. Screenshot/manual evidence may be captured by the supported desktop/manual process rather than committed as product assets; record exact viewport, platform, and command provenance in the task evidence.

**Finish state:** evidence covers exact 1200×800 and 960×640 mounted or desktop views for shell, Sales/summary, Inventory, Catalog, History/detail/corrections, with continuity states represented through tests/state variants. Reviewers can verify keyboard traversal, visible focus, semantic names, live status/alert/busy/error associations, modal focus containment/return, contrast measurements or unresolved labels, grayscale/non-color cues, reduced motion, 200% zoom/reflow, no horizontal core-task scroll, Spanish copy, `Bs` formatting, and whole-unit stock. A final audit confirms no unsupported mockup behavior, CDN dependency, changed IPC/Rust/persistence authority, historical-fact substitution, or lost safeguard.

**Evidence sequence:** RED: audit assertions/checklist fail for any missing size, state, accessibility, licensing, or invariant evidence; GREEN: all required mounted/manual evidence is present and `npm test` passes; TRIANGULATE: run the configured frontend runner twice, inspect exact-size screenshots/manual views, and compare every required scenario to R1–R22; REFACTOR: remove redundant evidence helpers or selectors without lowering observable coverage.

**Acceptance/scenario mapping:** R1–R22 and S01–S61; exact F01–F15 contract, especially S36–S42 and S55–S61. Any unavailable Windows/desktop evidence must be reported honestly and remains a release blocker for the corresponding claim.

**Rollback boundary:** remove only W9 evidence helpers/fixtures and audit material; never roll back verified screen behavior to hide a failed check.

**Non-goals:** no mockup generation, production image/logo creation, CDN/network asset, browser E2E infrastructure, business-rule change, or apply authorization.

## Apply-stop gate

No W1–W9 task may begin until W0/ticket 03 is marked complete with its mounted representative screen evidence and isolation proof. After W0, implementation still requires **explicit user authorization**; this task plan does not grant it. PR4A may use only its approved bounded `size:exception` and must stop and ask if its measured diff would exceed 1,050 changed lines; PR4B and every other slice remain under the normal 400-line cap. If ticket 03 is incomplete, if a slice exceeds its applicable budget, if contrast/licensing evidence is unresolved, or if any source-authority invariant fails, stop and request a decision. The final visual system must not be declared complete merely because CSS renders: behavior, state, accessibility, responsive, offline-asset, and invariant evidence must all pass.
