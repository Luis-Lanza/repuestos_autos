# 07 — React checkout draft state and request continuity

Status: approved

## Dependencies

06 TypeScript command seam.

## Scope

Remove editable line-price, cash-applied, and change state/actions/controls. Keep read-only catalog guidance, quantity, tendered cash, QR input, and one generated request ID across failed confirmation retries. Discard or successful confirmation starts a new intent.

## Expected path groups

- `src/ui/sales/sale-flow.ts`
- `src/ui/sales/sale-screen.ts`
- `src/ui/sales/sale-flow.test.ts`
- narrowly required UI test helpers

## Verification evidence

Run focused sale-flow tests, `npm test`, and `npm run build`. Evidence must show price is guidance only, obsolete authority actions are absent, cash/QR/mixed draft inputs submit correctly, retries preserve the UUID, and discard clears draft payment input.

## Rollback

Revert sale-flow/screen changes and their tests as one unit. The TypeScript seam remains valid independently.

## Cumulative-history boundary warning

Final persisted-summary rendering is excluded from this state slice. Inspect only its direct predecessor diff, keep cumulative unrelated paths out, and replan before 400 authored changed lines.

## Key Learnings

A request ID represents one checkout intent and must survive a failed attempt without allowing stale response facts into a later request.
