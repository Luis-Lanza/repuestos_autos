# Tasks: Sellable Product Onboarding

Existing direct-route code is retrospective evidence, not completed SDD work. Keep tasks unchecked until apply audits, proves, or remediates each requirement.

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | Runtime 700–1,000; SDD artifact-only ~100 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 → PR 2 → PR 3 |
| Delivery strategy | chained — user-resolved |
| Chain strategy | stacked-to-main |

Decision needed before apply: No — chained delivery resolved.
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|------|------|-----------|----------------------|-----------------|-------------------|
| 1 | Domain, repository seam, and forward migration | PR 1 | `cargo test --manifest-path src-tauri/Cargo.toml --test product_onboarding --test sqlite_migrations` | N/A — persistence boundary is proven by SQLite integration tests | Revert catalog Rust changes and migration 0005 only |
| 2 | IPC contract and onboarding UI | PR 2 | `npx tsx --test src/commands/onboarding.test.ts src/ui/onboarding/onboarding-form.test.ts` | `npm run tauri:dev`: create category, then valid/invalid product | Revert onboarding command/UI files and screen navigation |
| 3 | Indexed search and unchanged checkout integration | PR 3 | `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_search --test confirm_sale_application` | `npm run tauri:dev`: search onboarded SKU and complete cash sale | Revert search adapter and onboarding-to-sales wiring; retain PR 1–2 |

## Phase 1: Reconciliation and RED Tests

- [x] 1.1 Audit `src-tauri/src/{domain,application,infrastructure,commands}`, `src/commands`, and `src/ui` against design; record existing evidence and gaps for apply-progress without treating code presence as completion.
- [x] 1.2 RED: extend `src-tauri/tests/sqlite_migrations.rs` with v0–v4 upgrades, future-version refusal, legacy movement ID/timestamp/sale-link preservation, failed preflight, `foreign_key_check`, and reopen-idempotence cases.
- [x] 1.3 RED: extend `src-tauri/tests/product_onboarding.rs` for typed values, duplicate/missing inputs, non-positive price/stock, rollback at each write, exactly one immutable opening movement, and stable errors.

## Phase 2: Domain and Persistence Remediation

- [x] 2.1 Reconcile `src-tauri/src/domain/catalog.rs` and `src-tauri/src/application/catalog/{mod.rs,repository.rs}` so Rust owns validation, stable error mapping, and `CreateProductUseCase` transaction ownership; reject unknown fields.
- [x] 2.2 Reconcile `src-tauri/src/infrastructure/sqlite/{mod.rs,catalog_repository.rs}` and create migration `0005_catalog_onboarding_hardening.sql`; preserve applied v4, preflight legacy data, generalize immutable movements, backfill FTS5, and atomically persist product, values, search document, balance, and opening movement.

## Phase 3: IPC, UI, and Sales Wiring

- [x] 3.1 RED then reconcile `src-tauri/src/commands/onboarding.rs`, `src-tauri/src/lib.rs`, and `src/commands/onboarding.ts` for strict payloads, registered commands, list/create envelopes, and SQL-free stable errors.
- [x] 3.2 RED then reconcile `src/ui/app.ts` and `src/ui/onboarding/*` for category listing, dynamic text/number/option fields, validation feedback, and Sales ↔ Onboarding navigation.
- [x] 3.3 RED then reconcile search and sales seams in `src-tauri/src/application/catalog`, `src-tauri/src/infrastructure/sqlite`, and `src/ui/sales/*` so active products search globally, are limited/prefix-matched, and sell at backend price with unchanged history snapshots.

## Phase 4: Verification and Scope Closure

- [x] 4.1 Add a release-mode 20,000-product benchmark proving or explicitly reporting the one-second search target on target-class hardware.
- [x] 4.2 Verify exclusions and regression behavior: no editing/archiving/import/barcodes/fractional stock, and no checkout pricing, payment, idempotency, or history changes; document unproven migration/UI/performance claims.
