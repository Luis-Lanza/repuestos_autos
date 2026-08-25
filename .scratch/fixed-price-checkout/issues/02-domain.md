# 02 — Authoritative sale-line and payment domain rules

Status: approved

## Dependencies

01 Migration, so the domain contract is delivered against the supported persistence-version policy.

## Scope

Replace negotiated line construction with authoritative priced lines. Add pure payment derivation from total, optional QR applied, and optional cash tender; enforce exact applied totals, QR-first ordering, tender sufficiency, and checked integer-centavo arithmetic.

## Expected path groups

- `src-tauri/src/domain/sales/mod.rs`
- `src-tauri/tests/sale_domain.rs`

## Verification evidence

Run `cargo test --manifest-path src-tauri/Cargo.toml --test sale_domain`. Cover priced-line totals/overflow; cash-only, QR-only, mixed, QR-over-total, missing/insufficient/unexpected cash, zero-row omission, ordering, and aggregate payment invariants.

## Rollback

Revert only the sales domain module and its focused tests. No database migration or confirmed-sale history is part of this boundary.

## Cumulative-history boundary warning

Keep this to domain behavior and tests. Do not include application, SQLite adapter, command, or UI changes from cumulative worktree history; replan before 400 authored changed lines.

## Key Learnings

Money stays in integer centavos, and payment derivation belongs to the domain rather than the client.
