# 09 — Canonical fixed-price checkout specification

Status: approved

## Dependencies

08 Persisted presentation.

## Scope

Update the canonical sales specification with fixed-price authority, sale-time historical prices, backend-derived payment behavior, migration compatibility, strict cross-seam request/response rules, idempotent retry semantics, atomic failure behavior, and explicit out-of-scope boundaries.

## Expected path groups

- `openspec/specs/sales/spec.md`
- focused specification evidence only; no source-code expansion

## Verification evidence

Read back the canonical spec and trace each requirement to the completed migration, domain, application, SQLite, Rust command, TypeScript seam, and React presentation evidence. Confirm it names cash-only, QR-only, mixed, invalid QR/tender, repricing, retry, atomicity, and rollback compatibility.

## Rollback

Revert only the canonical-spec update. No runtime behavior, database data, or delivery evidence is changed by this documentation work unit.

## Cumulative-history boundary warning

Do not use a documentation slice to carry source changes or unrelated historical diffs. Compare the spec-only boundary against its predecessor and replan before 400 authored changed lines.

## Key Learnings

The canonical spec preserves the contract that every interface and test slice implements.
