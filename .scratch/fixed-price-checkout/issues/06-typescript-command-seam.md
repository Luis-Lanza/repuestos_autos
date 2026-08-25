# 06 — TypeScript catalog and confirmation command seam

Status: approved

## Dependencies

05 Rust command contract.

## Scope

Send only product identity, positive quantity, nullable tendered cash, and nullable QR applied. Validate JavaScript safe-integer input before invoke. Consume the persisted authoritative response and remove client-authoritative unit price, applied cash, and change fields.

## Expected path groups

- `src/commands/catalog.ts`
- `src/commands/confirm-sale.ts`
- `src/catalog.test.js`
- `src/commands/confirm-sale.test.ts`

## Verification evidence

Run focused command tests, `npm test`, and `npm run build`. Evidence must show the exact invoke payload, rejection of unsafe/non-integer/negative money and invalid IDs/quantities, catalog terminology, persisted cash/QR/mixed responses, and backend error passthrough.

## Rollback

Revert both command modules and their tests together. The reduced TypeScript contract must not be deployed against the legacy Rust IPC contract.

## Cumulative-history boundary warning

This seam slice excludes React reducer and presentation work. Compare against its direct predecessor, exclude cumulative unrelated changes, and replan before 400 authored changed lines.

## Key Learnings

Client validation protects JSON representation; backend validation remains authoritative for business rules.
