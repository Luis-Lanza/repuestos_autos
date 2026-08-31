# Implementation Tasks: Post-sale Lifecycle

## Review Workload Forecast

| Field | Value |
| ------- | ------- |
| Estimated changed lines | 2,400–3,400 authored additions + deletions across ~26 files |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 schema → PR 2 domain/application contracts → PR 3 return transaction → PR 4 cancellation transaction → PR 5 command adapters → PR 6 history read model → PR 7 operator UI → PR 8 backup/integrity hardening |
| Delivery strategy | ask-on-risk |
| Chain strategy | feature-branch-chain |

Decision needed before apply: No — `feature-branch-chain` selected; PR 1 only is authorized.
Chained PRs recommended: Yes
Chain strategy: feature-branch-chain
400-line budget risk: High

The full change is well above the 400-line review budget. Before apply, obtain the delivery decision and select `stacked-to-main` or `feature-branch-chain`; apply only one bounded PR slice at a time. If any slice approaches 400 authored changed lines, split it at the stated verification/rollback boundary rather than combining unrelated layers.

## Execution Rules

- Follow each work unit in RED → GREEN → TRIANGULATE → REFACTOR order; keep its tests and documentation with the behavior.
- Use fixed SQL and `rusqlite::params!` for every value. Read `.claude/skills/sqlite-database-expert/references/advanced-patterns.md` and `security-examples.md` before migration/query implementation.
- Record the exact focused command/result, authored changed-line count, runtime scenario/result (or explicit `N/A`), and rollback boundary for every work unit.
- Preserve the existing React → Tauri → application/domain → SQLite dependency direction. Do not add refund, settlement, report, catalog, payment mutation, or Tauri capability behavior.

## Dependency-Ordered Work Units

### 0. Resolve delivery boundary before apply

- [x] **0.1 Delivery gate** — Record the selected chain strategy for this change and authorize only PR 1. Start: forecast is High and strategy is `ask-on-risk`. Finish: `stacked-to-main` or `feature-branch-chain` is explicitly recorded and the first PR boundary is approved. Verification: confirm each planned PR has an independent test and rollback boundary. Rollback: no source changes; leave apply blocked if no decision exists.

### 1. PR 1 — Additive schema-v10 foundation

- [x] **1.1 RED — Migration contract tests** — Extend `src-tauri/tests/sqlite_migrations.rs` with failing evidence for fresh v10 creation, v9→v10 preservation, idempotent reopen, invalid-v9 preflight, required tables/indexes/triggers, immutable fact tables, foreign-key links, movement/quantity constraints, and allowed legacy unlinked v9 correction movements.
- [x] **1.2 GREEN — Schema migration** — Add `src-tauri/src/infrastructure/sqlite/migrations/0010_post_sale_lifecycle.sql` and update `src-tauri/src/infrastructure/sqlite/mod.rs` to register 9→10, set `CURRENT_SCHEMA_VERSION = 10`, and validate the new structure. Create the five additive tables, specified indexes, update/delete rejection triggers, and defensive insert-validation triggers; do not rebuild or modify existing sale facts.
- [x] **1.3 TRIANGULATE — Constraint cases** — Add malformed return/cancellation fixtures proving zero cancellation lines have no movement, positive lines have exactly one matching movement, cumulative restoration cannot exceed sold quantity, and hostile text remains data. Run `cargo test --manifest-path src-tauri/Cargo.toml --test sqlite_migrations`.
- [x] **1.4 REFACTOR — Migration locality** — Remove duplicated v10 validation setup without widening the migration interface. Finish: schema-v9 rows compare logically equal before/after and malformed correction facts fail atomically. Runtime: `N/A` (migration integration tests exercise the SQLite boundary). Rollback: before accepted v10 facts, remove migration registration/file together; after accepted facts, retain schema and disable later feature surfaces.

### 2. PR 2 — Pure plans and application contracts

- [x] **2.1 RED — Domain behavior tests** — Add focused tests beside `src-tauri/src/domain/sales/post_sale.rs` for multi-line/repeated-product returns, empty/duplicate/unknown/non-positive/over-remaining rejection, cancellation before/after returns, fully returned zero residuals, cancelled/non-confirmed rejection, trimmed non-blank reason, and checked `i64` arithmetic.
- [x] **2.2 GREEN — Domain plans** — Add `src-tauri/src/domain/sales/post_sale.rs` and export it from `src-tauri/src/domain/sales/mod.rs`; implement `RequestedReturnLine`, `OriginalSaleLine`, return/cancellation plans, and typed domain errors without persistence, hashing, IDs, or UI calculations.
- [x] **2.3 RED/GREEN — Application interface and identity** — Add failing tests then implement `src-tauri/src/application/sales/post_sale.rs` and exports in `src-tauri/src/application/sales/mod.rs`: request/result DTOs, `PostSaleError`, `PersistedRequest`, the narrow `PostSaleRepository` seam, canonical return ordering, cancellation reason length-prefixing, normalized `RequestId`, SHA-256 bytes/hash comparison, and the two-method `PostSaleLifecycleUseCase` interface.
- [x] **2.4 TRIANGULATE/REFACTOR — Identity conflicts** — Prove reordered return lines canonicalize identically while changed quantity, sale, reason, or operation conflicts; simplify only behind the same interface. Run the relevant library tests with `cargo test --manifest-path src-tauri/Cargo.toml post_sale`. Runtime: `N/A` (pure domain/application contracts). Rollback: remove only the unreferenced post-sale modules and exports.

### 3. PR 3 — Atomic multi-line return vertical backend

- [x] **3.1 RED — Return transaction tests** — Create return-focused cases in `src-tauri/tests/post_sale_lifecycle.rs` for exact original-line identity, multi-line commit, exact balance increments, one `return` movement per positive line, hostile bound values, rollback after an injected first-line failure, immutable original facts, replay, conflict, and two-connection over-return serialization.
- [x] **3.2 GREEN — SQLite return adapter/use case** — Add `src-tauri/src/infrastructure/sqlite/post_sale_repository.rs` and register it in `src-tauri/src/infrastructure/sqlite/mod.rs`. Implement the return path under one caller-owned `TransactionBehavior::Immediate`: request lookup, exact replay/conflict comparison, in-transaction fact loading, domain plan, request/header/line writes, checked stock update, movement insert, persisted-result reload, single commit, and explicit rollback mapping.
- [x] **3.3 TRIANGULATE — Failure and concurrency** — Exercise same-ID replay, reordered-line replay, mismatched payload conflict, unknown/wrong-sale lines, missing stock row, injected persistence failure, busy/storage mapping, and competing writers; assert rejected attempts leave no partial facts.
- [x] **3.4 REFACTOR — Keep eligibility in domain** — Remove adapter-side business decisions while retaining row-count and malformed-fact checks. Run `cargo test --manifest-path src-tauri/Cargo.toml --test post_sale_lifecycle return`. Runtime harness: focused Rust integration scenario against temporary SQLite. Rollback: remove the return adapter registration/path while retaining schema-v10 tables.

### 4. PR 4 — Whole-sale cancellation vertical backend

- [x] **4.1 RED — Cancellation transaction tests** — Extend `src-tauri/tests/post_sale_lifecycle.rs` for cancellation before returns, after partial returns, fully returned zero-stock cancellation, blank reason, cancel-once, return-versus-cancellation overlap, exact replay/conflict, immutable originals/payments, and normalized reason persisted only on cancellation movements.
- [x] **4.2 GREEN — Cancellation orchestration** — Complete the cancellation path in `src-tauri/src/application/sales/post_sale.rs` and `src-tauri/src/infrastructure/sqlite/post_sale_repository.rs`: derive residuals from persisted line facts inside `IMMEDIATE`, insert one audit line per original line, create movements only for positive residuals, update stock exactly once, derive lifecycle status from the cancellation header, and reload the persisted result before commit.
- [x] **4.3 TRIANGULATE — Serialization outcomes** — Prove overlapping return/cancellation produces one valid serialization, losing requests receive stable errors, fully returned cancellation creates no movement, and same request ID cannot cross operation kinds.
- [x] **4.4 REFACTOR — Shared transaction mechanics only** — Deduplicate replay/rollback plumbing without merging return and cancellation business plans. Run `cargo test --manifest-path src-tauri/Cargo.toml --test post_sale_lifecycle cancellation`. Runtime harness: focused Rust integration scenario against temporary SQLite. Rollback: disable only cancellation entry points; retain committed additive facts and return behavior.

### 5. PR 5 — Rust/Tauri and TypeScript command seams

- [x] **5.1 RED — Rust command contracts** — Add `src-tauri/tests/post_sale_commands.rs` and update inline tests in `src-tauri/src/lib.rs` for strict request parsing, tagged success/error responses, stable error codes, no SQL leakage, inventory-only wording, both command registrations, and removal of obsolete excluded-command assertions.
- [x] **5.2 GREEN — Rust commands** — Add `src-tauri/src/commands/post_sale.rs`; update `src-tauri/src/commands/mod.rs` and `src-tauri/src/lib.rs` with `create_sale_return_command` and `cancel_sale_command`, owned/`deny_unknown_fields` request types, managed database state, and stable error mapping. Do not change capabilities.
- [x] **5.3 RED/GREEN — TypeScript adapter** — Add failing `src/commands/post-sale.test.ts`, then implement `src/commands/post-sale.ts` with discriminated unions, runtime guards, snake_case payloads under `{ request }`, rejection/malformed-response mapping, and one UUID retained across retries. Keep all eligibility arithmetic out of TypeScript.
- [x] **5.4 TRIANGULATE/REFACTOR — Seam parity** — Verify Rust/TypeScript casing, result fields, malformed payloads, conflict codes, and forbidden monetary language; keep adapters thin. Run `cargo test --manifest-path src-tauri/Cargo.toml --test post_sale_commands` and `npm test -- src/commands/post-sale.test.ts`. Runtime harness: invoke both commands from a development build or record `N/A` if the native shell is unavailable. Rollback: unregister/hide commands and remove the TS adapter while retaining backend facts.

### 6. PR 6 — Sales History lifecycle read model

- [x] **6.1 RED — Repository/history tests** — Extend `src-tauri/tests/sales_history.rs` for cancelled-sale discoverability, derived status, `has_corrections`, exact line IDs and persisted quantities, ordered return detail, cancellation detail including zero lines, unchanged original snapshots/totals/payments, and fixed parameterized filters.
- [x] **6.2 GREEN — Rust read model** — Update `src-tauri/src/application/sales/history.rs` and `src-tauri/src/infrastructure/sqlite/sale_history_repository.rs` to project lifecycle/corrections from additive facts while retaining `s.status = 'confirmed'` as the base reader condition.
- [x] **6.3 RED/GREEN — Command and TS history contracts** — Extend `src-tauri/tests/sales_history_commands.rs` and `src/commands/sales-history.test.ts` first, then update `src-tauri/src/commands/sales_history.rs` and `src/commands/sales-history.ts` with guarded summary/detail correction shapes and unchanged payment/money fields.
- [x] **6.4 TRIANGULATE/REFACTOR — Historical immutability** — Compare originals before/after return+cancellation and test legacy confirmed sales with no corrections. Run `cargo test --manifest-path src-tauri/Cargo.toml --test sales_history --test sales_history_commands` and `npm test -- src/commands/sales-history.test.ts`. Runtime harness: load corrected and uncorrected sale details through the command seam. Rollback: revert projections to the prior read-only shape; do not delete correction facts.

### 7. PR 7 — Accessible correction workflows in Sales History

- [x] **7.1 RED — UI flow tests** — Extend `src/ui/sales/history-flow.test.ts` for intent UUID retention, exact-line selection, whole-number/local validation, pending disablement, successful reload, stale/conflict value retention, reload/retry, cancellation reason/confirmation, cancelled/fully-returned action rules, focus/alert/`aria-busy`, keyboard operation, 44px targets, and forbidden refund/payment-reversal language.
  - Cumulative PR7A–PR7D automated evidence completes this test-only contract; PR7E separately records the display-capable manual runtime completion for task 7.4.
- [x] **7.2 GREEN — State flow** — Update `src/ui/sales/history-flow.ts` with return/cancellation intent, validation, pending/error/success-reload actions, stable request identity across retries, and no optimistic quantity changes.
- [x] **7.3 GREEN — Presentation adapter** — Update `src/ui/sales/history-screen.ts` to render text lifecycle status, separate `Original sale items`, `Original payment facts`, and `Inventory correction history`; add labelled return and confirmed cancellation forms using persisted remaining/restored quantities only.
- [x] **7.4 TRIANGULATE/REFACTOR — Interaction/a11y states** — Cover repeated-product lines, fully returned confirmed sale, cancelled sale, command conflict, and malformed command result; extract presentation helpers only when they reduce interface complexity. Run `npm test -- src/ui/sales/history-flow.test.ts`. Runtime harness: `npm run tauri:dev`, manually record return, retry, cancel, keyboard/focus, and corrected-history scenarios. Rollback: hide correction actions/history sections while keeping commands and persisted facts intact.
  - Cumulative automated evidence covers the required state matrix and retained `npm test` result; display-capable human observation now records a return, invalid-value correction and successful resubmission in the same still-open form, cancellation, keyboard-only operation, first-invalid-field focus, and corrected-history facts. The prior Wayland Gdk Error 71 is retained as launcher history; no command is rerun for this evidence-only completion.

### 8. PR 8 — Backup/restore compatibility and final integrity hardening

- [x] **8.1 RED — Lifecycle backup fixture** — Extend `src-tauri/tests/backup_restore.rs` with a v10 database containing original sale facts, returns, cancellation (including zero residual lines), linked movements, and balances; assert staging versions 1–10 and restored fact equality/consistency.
- [x] **8.2 GREEN — v10 restore validation** — Update `src-tauri/src/infrastructure/sqlite/backup.rs` and shared validation in `src-tauri/src/infrastructure/sqlite/mod.rs` so `stage_and_validate` migrates staged v1–v9 copies to v10 and `validate_restored_database` checks v10 structure, foreign keys, request/header identity, movement links, cumulative restoration, and complete cancellation line coverage without rejecting legacy unlinked v9 movements.
- [x] **8.3 TRIANGULATE — Corruption matrix** — Add targeted failures for missing triggers/indexes, mismatched movement links, excess cumulative restoration, missing cancellation lines, and changed original facts; prove the selected backup source remains unchanged on failure.
- [x] **8.4 REFACTOR/REGRESSION — Full boundary verification** — Consolidate validation helpers without changing `OperationalFacts`; run `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore --test sqlite_migrations`, then full `cargo test --manifest-path src-tauri/Cargo.toml` and `npm test`. Runtime harness: restore a lifecycle-bearing backup and open its Sales History detail via `npm run tauri:dev`. Rollback: keep schema/data readable and disable feature surfaces; never down-migrate or delete accepted facts.
  - Manual smoke evidence confirms schema-v10 backup creation, a successful UI restore, restoration to the earlier expected state after creating a new sale, and visible restored return/correction history.

## Final Scope and Evidence Check

- [x] **9.1 Requirement traceability** — Confirm automated evidence covers multi-line returns, atomic stock restoration, residual cancellation, replay/conflict, concurrent no-double-restoration, immutable originals, cancelled history visibility, inventory-only language, v9 migration, v10 backup/restore, and every explicit exclusion.
- [x] **9.2 Diff-budget check** — Before each PR, record `git diff --stat` and authored additions + deletions. Split any slice that reaches the 400-line budget at its verification/rollback boundary; do not use generated artifacts to hide authored size.
- [x] **9.3 Final regression evidence** — Record exact results for `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and the Tauri runtime scenarios. Verify no unexpected `Cargo.toml`, capability, report, export, payment, catalog, or backup-UI changes exist.
  - Final evidence: Rust `cargo test --manifest-path src-tauri/Cargo.toml` passed 158 tests; frontend `npm test` passed 62 tests; `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` and `git diff --check` passed. Retained human smoke evidence covers real return/cancellation stock restoration, schema-v10 backup restore, return to the earlier state after a later sale, and visible restored correction history.
  - Final inventory contains only post-sale lifecycle artifact, SQLite backup-validation, and backup test changes. It contains no `Cargo.toml`, Tauri capability/configuration, report/export, payment, catalog, or backup-UI change.
