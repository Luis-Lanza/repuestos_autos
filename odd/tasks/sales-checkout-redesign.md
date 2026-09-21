# Sales checkout modal redesign

## Objective
Redesign the existing `Revisar y cobrar` modal to match the approved desktop and compact POS references without changing checkout business behavior.

## Approved references
- `docs/design/sales-pos/desktop-checkout.png`
- `docs/design/sales-pos/compact-checkout.png`
- `docs/design/sales-pos-figma-handoff.md`
- Static source references under `docs/design/sales-pos/reference-source/`

## Delivery strategy
- One bounded work unit and one PR.
- Target: remain below 400 changed lines.
- If the coherent implementation exceeds 400 lines after one honest pass, stop and request a size exception rather than compressing code or tests.

## Scope
- Sales-owned checkout markup and visual hooks.
- Sales-scoped desktop and compact checkout styling.
- Focused mounted tests for observable structure, interactions, accessibility, and breakpoint contracts.
- Native Windows validation at desktop, 961px, 960px, and narrower compact widths.

## Non-goals
- No generic `CheckoutDialog` or `ConfirmationDialog` redesign.
- No custom quantity steppers.
- No new change calculation.
- No business-flow, pricing, stock, payment, idempotency, command, IPC, Rust, SQLite, or persistence changes.
- No edits to `src/ui/sales/sale-flow.ts`.
- No external fonts, icon packages, prototype scripts, or runtime dependencies.

## Invariants
- Existing initial focus, focus trap, Escape handling, focus return, backdrop no-op, validation focus, pending locks, and stale-response guards remain intact.
- Existing request construction and confirmation response handling remain intact.
- Quantity remains a native spinbutton; final price, cash, and QR remain the existing controlled inputs.
- Logical DOM order remains stable across breakpoints.
- Desktop starts at 961px; compact ends at 960px.
- New CSS is rooted under a Sales checkout marker to prevent generic-dialog leakage.

## Tasks
- [x] T1 Add focused mounted expectations for the approved Sales checkout structure and preserved interaction contracts.
- [x] T2 Implement the Sales-owned checkout markup and scoped desktop/compact styling.
- [x] T3 Run independent focused/full verification, typecheck, build, diff check, and structural scope review.
- [ ] T4 Validate the rendered modal against desktop and compact references in the native Windows app.

## Acceptance criteria
- Desktop modal presents an independently scrolling cart and a visually grouped 280–320px settlement rail.
- Compact modal uses block flow and dialog scrolling at 960px and below.
- Cart lines clearly present product, SKU, list/minimum facts, quantity, final price, subtotal, and removal.
- Total, cash, QR, discard, Back, and Confirm have the approved visual hierarchy without changing their behavior.
- Controls retain accessible names, 44px targets, visible focus, forced-colors support, and pending disablement.
- Generic dialogs and non-Sales screens preserve their existing structure and styles.
- Focused and full frontend tests, test typecheck, production build, and `git diff --check` pass.

## Progress
- User selected the bounded single-slice strategy after review workload was estimated at 320–430 changed lines.
- Read-only mapping confirmed that `SaleScreen` owns checkout content while generic dialog semantics should remain untouched.

## Verification evidence
- T1 RED: focused Sales/generic-dialog suite failed 1/38 because the Sales checkout marker and approved structure were absent.
- T2 GREEN: Sales-owned markup and scoped responsive styling passed the focused suite 38/38.
- Initial independent readback found compact nested scrolling: the cart retained its desktop 320px scroll container while the compact dialog also scrolled.
- T2 correction RED: focused suite failed 1/38 on the missing compact cart overflow reset. GREEN: compact cart now uses natural height and visible overflow while the dialog remains the sole scrolling container; focused suite passed 38/38.
- T3 independent verification passed: focused 38/38, full frontend 239/239, test typecheck, production build, and `git diff --check`.
- Final candidate: 3 tracked source/test files, 143 additions and 65 deletions (208 changed lines), below the 400-line budget.
- Structural readback confirmed Sales-scoped CSS, unchanged action/confirmation wiring, no staged changes, and no edits to generic dialogs, `sale-flow.ts`, commands, IPC, Rust, SQLite, persistence, or dependencies.
- Existing non-blocking Vite warning: mixed static/dynamic Tauri API imports.
- Work-unit commit: `e288e75` (`feat(ui): redesign Sales checkout modal`).
- Native Windows desktop review found one blocking geometry defect: the final-price value and line subtotal overlapped in every cart row with realistic product names.
- Overlap correction RED: focused suite failed 1/38 on the missing collision-safe grid contract. GREEN: quantity, final price, and subtotal now use independent tracks with explicit gap/minimum width; focused suite passed 38/38.
- Overlap correction independent verification passed: focused 38/38, full frontend 239/239, test typecheck, production build, and `git diff --check`. Compact stacking and dialog-only scrolling remain intact.
- Native Windows screenshot confirmation of the corrected geometry remains pending.
