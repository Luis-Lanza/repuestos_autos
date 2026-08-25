# 04 — SQLite authoritative persistence and stored-fact readback

Status: approved

## Dependencies

03 Application core.

## Scope

Implement the application repository with parameterized catalog-price resolution, reservation/load behavior, and caller-transaction-owned persistence. Write resolved prices to both compatibility columns; atomically persist lines, derived payments, stock deductions, immutable movements, confirmation state, and SQLite-loaded summaries.

## Expected path groups

- `src-tauri/src/infrastructure/sqlite/sale_repository.rs`
- `src-tauri/tests/confirm_sale_use_case.rs`
- narrowly required SQLite test fixtures/helpers

## Verification evidence

Run `cargo test --manifest-path src-tauri/Cargo.toml --test confirm_sale_use_case` and `cargo test --manifest-path src-tauri/Cargo.toml --test sqlite_migrations`. Prove repricing affects only new sales, retry returns stored facts once, all payment modes persist exact facts, and invalid payment/stock/readback failures leave counts, stock, and movements unchanged.

## Rollback

Revert this adapter/test slice while retaining the application interface. Never alter migration version 2 or rewrite confirmed history.

## Delivery-size exception

For local-commit preparation only, an explicit size exception is recorded for this estimated cumulative group of ~542 authored changed lines. Authoritative transaction persistence, stored-fact readback, and SQLite-focused tests remain together because they are one rollback-safe persistence unit. This record does not imply approval, readiness, staging, a commit, a branch, or a PR.

## Cumulative-history boundary warning

This slice excludes public command DTOs and frontend files. Inspect the real predecessor diff and exclude cumulative unrelated paths. Cumulative uncommitted history cannot prove historical work-unit boundaries or reconstructed line counts; replan if unrelated paths appear. The tracker-wide <=400 authored-additions-plus-deletions policy remains in effect for every other work unit.

## Key Learnings

The SQLite adapter resolves catalog prices inside the same transaction that protects payments, stock, movements, and persisted summary readback.
