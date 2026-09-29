# Checkout Stock Snapshot and Product Details

## Objective
Make each product line in Sales **Revisar y cobrar** show its captured stock availability and offer **Ver detalles**, opening the same complete Sales product-detail experience without compromising checkout modal behavior.

## Approved Decisions
- Show the label `Stock al agregar: N` beside the product identity and before `Quitar`.
- This is a browse/add snapshot, not a reservation or a live availability guarantee; sale confirmation remains authoritative for stock validation.
- `Ver detalles` must present the same Sales content: image, SKU, category, primary location, stock, prices, and attributes.
- The detail overlay must be explicitly handed off from checkout so focus, Escape, backdrop, close action, and invoker restoration remain correct with nested dialogs.

## Constraints
- Preserve the existing Sales browse snapshot; do not fetch a fresh Catalog detail or add IPC.
- Keep availability in the existing product-information column; do not add an action column.
- Keep `Quitar` and confirmation behavior unchanged.
- Do not imply stock is reserved or bypass confirmation-time insufficient-stock validation.
- No commit, push, or pull request is authorized.

## TDD and Checks
- Mode: standard; strict TDD disabled by `openspec/config.yaml`.
- Determine focused frontend test command before implementation and record evidence below.

## Tasks
- [x] T1 Retain the required product browse snapshot on draft lines and expose `Stock al agregar` in checkout. Route: delegated writer. Evidence: focused flow and mounted tests passed.
- [x] T2 Reuse the complete Sales detail experience safely from the checkout dialog, including nested focus/overlay behavior. Route: delegated writer. Evidence: focused product-browser and Sales mounted tests passed.
- [ ] T3 Independently verify snapshot truthfulness, checkout behavior, accessibility, and regression scope. Route: delegated verifier. Evidence: automated verification passed; Windows smoke remains pending.
- [x] T4 Preserve checkout detail thumbnails across browse-page changes. Route: delegated correction. Scope: keep a SaleScreen-lifetime product/revision thumbnail cache, retain no stale or mismatched revision, add no checkout-detail fetch or confirmation payload field, and extend only the exact W9 screen-path policy. Evidence: focused checkout regression, revision-mismatch coverage, and full frontend audit passed.

## Acceptance Criteria
- Every checkout line displays `Stock al agregar: N` using the captured browse snapshot.
- The availability label cannot be interpreted as a reservation or confirmation-time guarantee.
- A checkout line exposes `Ver detalles` before `Quitar`.
- The checkout detail experience shows the same product facts and image behavior as Sales browse details.
- Closing the detail experience restores focus to its checkout trigger and leaves checkout open; Escape/backdrop behavior remains correct.
- Existing price editing, remove, payment, discard, return-to-cart, and confirmation flows remain verified.

## Progress
- 2026-09-29: User requested captured product availability and the same complete Sales detail view inside checkout.
- 2026-09-29: User chose truthful copy `Stock al agregar: N` because cart stock is a browse/add snapshot and confirmation revalidates authoritative stock.
- 2026-09-29: Verification found checkout details could lose the thumbnail after browsing away. User chose a product/revision SaleScreen-lifetime cache and authorized exact W9 admission for `src/ui/sales/sale-screen.ts`.
- 2026-09-29: User authorized exact W9 admission for this task record and the already changed `src/ui/sales/sale-flow.ts` and `sale-flow.test.ts` paths.

## Evidence
- Automated: focused Sales flow/screen/ProductBrowser suite passed (71 tests); thumbnail screen suite passed (35 tests); `npm test` passed (367 tests); `npm run typecheck:tests` and `git diff --check` passed. Independent verification confirmed snapshot truthfulness, unchanged confirmation payload/no checkout-detail fetch, full shared detail content, nested focus/dismissal behavior, bounded exact W9 paths, cross-page thumbnail retention, and same-product wrong-revision rejection.

## Next Step
- Push a smoke-validation checkpoint if the user authorizes it, then run the Windows checkout detail smoke sequence.
