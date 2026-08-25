# 08 — Persisted checkout presentation

Status: approved

## Dependencies

07 React continuity.

## Scope

Render draft catalog values as read-only guidance and successful checkout as SQLite-persisted authoritative facts. Use `catalog_unit_price_centavos` for guidance and `unit_price_centavos` for history; render cash tender/applied/change and QR applied without allowing response values into a later request.

## Expected path groups

- `src/ui/sales/catalog-result.ts`
- `src/ui/sales/persisted-summary.ts`
- `src/ui/sales/catalog-result.test.ts`
- `src/ui/sales/persisted-summary.test.ts`

## Verification evidence

Run focused presentation tests, `npm test`, and `npm run build`. Evidence must cover stored price differing from guidance, cash change, QR-only absent cash fields, mixed payment, stable errors, and separation of persisted response from later draft submission.

## Rollback

Revert these presentation modules/tests. If the complete reduced checkout experience must be reverted, do so in reverse dependency order through the Rust contract; do not change schema history.

## Cumulative-history boundary warning

Keep this to presentation and focused tests. Audit against the immediate predecessor, do not inherit unrelated cumulative files, and replan before 400 authored changed lines.

## Key Learnings

The screen must make the authority boundary visible: draft prices can change, persisted sale facts cannot.
