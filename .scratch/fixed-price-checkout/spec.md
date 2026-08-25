# Fixed-price checkout local delivery tracker

## Purpose

Track delivery slices for fixed-price checkout: confirmation resolves catalog prices in Rust, persists sale-time snapshots, and derives payment facts from tendered cash and optional QR value.

> **Triage metadata exception:** `Status:` is intentionally omitted from this tracker and every local issue under explicit user authorization because `docs/agents/triage-labels.md` is missing. No issue state is implied.

## Delivery model

- Local Markdown tracker only; this is not a GitHub issue set and contains no GitHub issue numbers.
- Planned work units follow a stacked-to-main dependency order, but this tracker does not claim branch, PR, approval, or merge readiness.
- Each implementation slice must keep tests and evidence with its behavior, record its actual changed-line count, and stay at or below 400 authored additions plus deletions unless a maintainer explicitly accepts an exception.

## Authorized delivery-size exceptions

These are local-commit preparation records only. They do not create a branch or PR, nor do they imply approval, readiness, staging, or a completed commit.

- 03 Application core: explicit size exception accepted for an estimated cumulative group of ~437 authored changed lines because the transaction contract, idempotency behavior, repository interface, and their focused tests form one rollback-safe behavioral unit.
- 04 SQLite persistence: explicit size exception accepted for an estimated cumulative group of ~542 authored changed lines because authoritative transaction persistence, stored-fact readback, and SQLite-focused tests must remain together.
- 10 Archive evidence: explicit size exception accepted for an estimated cumulative group of ~1,153 authored changed lines because cross-stack delivery evidence, verification records, and archive traceability are one evidence unit.
- All other work units retain the <=400 authored-additions-plus-deletions policy.

## Cumulative-history boundary warning

The repository worktree may contain cumulative uncommitted history from prior slices. A local issue's expected path groups define its intended boundary; they do **not** authorize carrying unrelated earlier changes into that work unit. Cumulative uncommitted history cannot prove historical work-unit boundaries or reconstructed line counts. Before delivery, inspect the slice diff against its actual predecessor and replan if unrelated paths or more than 400 authored changed lines appear, except for the explicitly recorded size exceptions above.

## Dependency chain

`01 Migration → 02 Domain → 03 Application core → 04 SQLite persistence → 05 Rust command contract → 06 TypeScript command seam → 07 React continuity → 08 Persisted presentation → 09 Canonical spec → 10 Archive evidence`

## Local issues

1. [01 Migration](issues/01-migration.md)
2. [02 Domain](issues/02-domain.md)
3. [03 Application core](issues/03-application-core.md)
4. [04 SQLite persistence](issues/04-sqlite-persistence.md)
5. [05 Rust command contract](issues/05-rust-command-contract.md)
6. [06 TypeScript command seam](issues/06-typescript-command-seam.md)
7. [07 React continuity](issues/07-react-continuity.md)
8. [08 Persisted presentation](issues/08-persisted-presentation.md)
9. [09 Canonical spec](issues/09-canonical-spec.md)
10. [10 Archive evidence](issues/10-archive-evidence.md)

## Key Learnings

- Catalog price and payment calculations are backend authority; UI prices are guidance only.
- Existing SQLite physical columns support a non-destructive semantic migration and forward-compatible rollback.
- Request-ID idempotency must short-circuit before price resolution or stock mutation.
