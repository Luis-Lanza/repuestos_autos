# 03 — Application transaction and idempotency core

Status: approved

## Dependencies

01 Migration and 02 Domain.

## Scope

Make `ConfirmSaleUseCase` own one transaction with reservation-first idempotency. Define repository operations for reserve-or-load, ordered line resolution, and caller-owned confirmed persistence. Existing confirmed requests return stored facts before repricing, recalculation, or stock mutation.

## Expected path groups

- `src-tauri/src/application/sales/confirm_sale.rs`
- `src-tauri/src/application/sales/application_contract.rs`
- `src-tauri/src/application/sales/repository.rs`
- `src-tauri/src/application/sales/mod.rs`
- `src-tauri/tests/confirm_sale_application.rs`

## Verification evidence

Run `cargo test --manifest-path src-tauri/Cargo.toml --test confirm_sale_application` and `cargo test --manifest-path src-tauri/Cargo.toml --test sale_domain`. Evidence must prove call order, existing-confirmed short-circuit, rollback on every first error, duplicate-product rejection, and no nested transaction ownership.

## Rollback

Revert the application contract, repository interface, use case, and repository-double tests as one unit. Keep migration and domain work intact.

## Delivery-size exception

For local-commit preparation only, an explicit size exception is recorded for this estimated cumulative group of ~437 authored changed lines. The transaction contract, idempotency behavior, repository interface, and focused tests are retained together because they form one rollback-safe behavioral unit. This record does not imply approval, readiness, staging, a commit, a branch, or a PR.

## Cumulative-history boundary warning

Do not combine real SQLite persistence, IPC, or frontend work with this application-interface slice. Compare only the intended paths to its actual predecessor. Cumulative uncommitted history cannot prove historical work-unit boundaries or reconstructed line counts; replan if unrelated paths appear. The tracker-wide <=400 authored-additions-plus-deletions policy remains in effect for every other work unit.

## Key Learnings

Idempotency is a transaction-order guarantee: load an existing confirmation before resolving mutable catalog prices.
