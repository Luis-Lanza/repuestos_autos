# 01 — Non-destructive migration and legacy compatibility

Status: approved

## Dependencies

None. This is the foundation for all following fixed-price checkout work.

## Scope

Add ordered SQLite schema-version handling through version 2. Version 2 is a semantic compatibility preflight: preserve tables, historical rows, physical price/payment columns, request IDs, stock, and movements without backfill or table rebuild. Reject unknown future versions without mutation.

## Expected path groups

- `src-tauri/src/infrastructure/sqlite/mod.rs`
- `src-tauri/src/infrastructure/sqlite/migrations/0002_fixed_price_checkout.sql`
- `src-tauri/tests/sqlite_migrations.rs`
- `src-tauri/tests/fixtures/`

## Verification evidence

Run `cargo test --manifest-path src-tauri/Cargo.toml --test sqlite_migrations`. Evidence must show version-1 facts are unchanged, reopen is idempotent, preflight failure preserves version/data, future versions fail without writes, and legacy query/write shapes remain compatible.

## Rollback

Revert migration-runner and test changes only. Never downgrade a database already at version 2, drop columns, or rewrite historical rows; the prior binary must continue using the unchanged physical schema.

## Cumulative-history boundary warning

Review only this migration/fixture/test boundary against its predecessor. Do not absorb domain, command, UI, or pre-existing worktree changes; replan before exceeding 400 authored changed lines.

## Key Learnings

The migration changes interpretation and version handling, not persisted historical facts.
