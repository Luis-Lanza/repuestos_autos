# Tasks: Fixed-price checkout

## Review Workload Forecast

| Field | Value |
| ------- | ------- |
| Estimated changed lines | Approximately 2,400–3,100 authored additions + deletions for the complete change; each remaining slice is forecast at 100–380 lines and MUST remain at or below 400 |
| 400-line budget risk | High for the full change; Low–Medium per remaining slice |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 migration (complete) → PR 2 domain rules (complete) → PR 3 application transaction core (complete) → PR 4 application failure hardening → PR 5 SQLite reservation and price resolution → PR 6 SQLite atomic persistence/readback → PR 7 Rust command contract → PR 8 TypeScript command seam → PR 9 React checkout state → PR 10 persisted presentation → PR 11 final acceptance evidence |
| Delivery strategy | ask-on-risk; user selected splitting for the detected risk |
| Chain strategy | stacked-to-main |

Decision needed before apply: No
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

The former combined application/transaction/idempotency block was forecast at approximately 1,125 changed lines and is replaced by PRs 3–6. PRs 1–3 are completed work and MUST NOT be reopened without concrete failing evidence. Apply exactly one remaining PR slice at a time. Before implementation and again before review, count authored additions plus deletions with `git diff --stat`; stop and replan before crossing 400 changed lines. Tests and OpenSpec evidence changed for a slice count toward its review budget.

### Stacked-to-main dependency chain

```text
PR 1 (complete) → PR 2 (complete) → PR 3 (complete)
  → PR 4 → PR 5 → PR 6 → PR 7 → PR 8 → PR 9 → PR 10 → PR 11
```

Each PR targets `main` only after its predecessor merges. Its diff MUST contain only its named work unit. Each PR must record start state, finish state, dependency, focused verification result, runtime harness result or explicit `N/A`, actual changed-line count, and rollback boundary.

## 1. PR 1 — Non-destructive migration and legacy compatibility (complete)

- [x] 1.1 **RED — Add the version-1 legacy database fixture and migration assertions.** Added `src-tauri/tests/fixtures/version1_fixed_price_legacy.sql` and migration coverage in `src-tauri/tests/sqlite_migrations.rs` for preserved historical rows, idempotent reopen, failed preflight, future-version rejection, and legacy query/write compatibility.
- [x] 1.2 **GREEN — Implement ordered migration handling.** Added `src-tauri/src/infrastructure/sqlite/migrations/0002_fixed_price_checkout.sql` and updated `src-tauri/src/infrastructure/sqlite/mod.rs` for versions 0–2 without table rebuilds, backfills, or schema downgrade.
- [x] 1.3 **TRIANGULATE — Cover compatibility failures.** Proved missing-column and foreign-key failures leave version/data unchanged and successful migration preserves row counts, keys, request IDs, stock, movements, and monetary columns.
- [x] 1.4 **REFACTOR — Consolidate migration helpers and verify.** `cargo test --manifest-path src-tauri/Cargo.toml --test sqlite_migrations` passed 5 tests; rustfmt and diff checks passed.

**Finish:** schema versions 0–2 open according to policy and legacy facts remain intact. **Rollback:** revert the migration runner/tests and remove migration 0002; never downgrade or rewrite a database already at version 2.

## 2. PR 2 — Authoritative sale-line and payment domain rules (complete)

- [x] 2.1 **RED — Specify authoritative pricing and payment derivation.** Updated `src-tauri/tests/sale_domain.rs` for priced lines, overflow, cash-only, QR-only, mixed, invalid tender, ordering, and aggregate payment invariants.
- [x] 2.2 **GREEN — Implement the domain contract.** Updated `src-tauri/src/domain/sales/mod.rs` with authoritative `unit_price`, `PaymentInput`, and `PaymentBreakdown::derive` while retaining persisted payment facts and typed errors.
- [x] 2.3 **TRIANGULATE — Cover boundary values and ordering.** Proved zero/maximum boundaries, QR-before-cash ordering, omission of zero-valued rows, and exact applied totals.
- [x] 2.4 **REFACTOR — Tighten invariant ownership and verify.** `cargo test --manifest-path src-tauri/Cargo.toml --test sale_domain` passed 13 tests; clippy, rustfmt, and diff checks passed.

**Finish:** the domain owns authoritative prices and derived payment facts. **Rollback:** revert only `src-tauri/src/domain/sales/mod.rs` and `src-tauri/tests/sale_domain.rs`; no persisted data changes.

## 3. PR 3 — Application transaction and idempotency core (complete)

- [x] 3.1 **RED — Specify reservation-first orchestration.** Added `src-tauri/tests/confirm_sale_application.rs` coverage for duplicate rejection, reservation-first order, existing-confirmed short-circuit, payment handoff, and no calls after resolution failure.
- [x] 3.2 **GREEN — Add the deep repository interface.** Added `RequestedLine`, `Reservation`, `ConfirmSaleRepository::reserve_or_load`, ordered `resolve_lines`, and caller-owned `persist_confirmed` in `src-tauri/src/application/sales/repository.rs` and `src-tauri/src/application/sales/mod.rs`.
- [x] 3.3 **GREEN — Orchestrate the transaction.** Updated `src-tauri/src/application/sales/confirm_sale.rs` and `src-tauri/src/application/sales/application_contract.rs` for duplicate rejection, idempotency-first resolution, derived payment, persisted-summary handoff, commit-on-success, and rollback-on-error.

**Evidence:** focused application tests passed 4 tests; sale-domain tests passed 13; legacy `confirm_sale_use_case` tests passed 8; clippy, rustfmt, and diff checks passed. The source/test delta was 327 changed lines. **Finish:** the application core exists while compatibility adapters still compile. **Rollback:** revert the application contract/interface/use case and repository-double tests; PRs 1–2 remain valid.

## 4. PR 4 — Application transaction failure hardening

**Forecast:** 100–180 changed lines. **Depends on:** PR 3. **Out of scope:** SQLite implementation changes, command DTOs, and frontend files.

- [x] 4.1 **RED — Complete transaction-order failure coverage.** Extended `src-tauri/tests/confirm_sale_application.rs` with call-order and rollback assertions for invalid payment, missing/inactive product, corrupt or non-confirmed reservation, and `persist_confirmed` failure. Proved no later repository operation occurs after the first error and an existing confirmed reservation does not inspect lines or derive payments.
- [x] 4.2 **GREEN/TRIANGULATE — Complete typed application failures.** Adjusted only `src-tauri/src/application/sales/application_contract.rs` and `src-tauri/src/application/sales/repository.rs` where the failing corrupt-reservation case demonstrated a gap. No string-matched errors, nested transactions, production `unwrap`/`expect`, or duplicate interfaces were added.
- [x] 4.3 **REFACTOR — Verify the isolated application module.** Ran `cargo test --manifest-path src-tauri/Cargo.toml --test confirm_sale_application`, `cargo test --manifest-path src-tauri/Cargo.toml --test sale_domain`, clippy, rustfmt check, and diff check.

**Finish:** every application-level failure has explicit call-order and rollback evidence through the repository interface. **Rollback:** revert only PR 4 test additions and any directly proven application corrections; PR 3 remains compilable and usable. **Runtime:** `N/A`; the repository-double target is the focused application harness.

## 5. PR 5 — SQLite reservation, catalog-price resolution, and idempotent readback

**Forecast:** 260–380 changed lines. **Depends on:** PR 4. **Out of scope:** stock/movement failure matrix, public command DTO replacement, and frontend changes.

- [x] 5.1 **RED — Specify reservation and authoritative line resolution.** Refactor the smallest necessary fixture surface in `src-tauri/tests/confirm_sale_use_case.rs` to submit `RequestedLine` plus `PaymentInput`. Add focused cases proving reservation precedes product lookup, current catalog prices are resolved in request order, duplicate/missing/inactive products have no effects, and retry after repricing returns the original stored summary without another stock deduction.
- [x] 5.2 **GREEN — Implement the SQLite adapter for the application interface.** Update `src-tauri/src/infrastructure/sqlite/sale_repository.rs` to implement `ConfirmSaleRepository::reserve_or_load` and ordered, parameterized `resolve_lines`. Implement the minimum `persist_confirmed` path needed for a complete cash or mixed happy path by delegating to existing local persistence helpers where safe; keep transaction ownership with the application and write the resolved price to both compatibility columns.
- [x] 5.3 **TRIANGULATE — Prove historical and idempotent readback.** In `src-tauri/tests/confirm_sale_use_case.rs`, change the catalog price after confirmation and assert ordinary readback and same-request retry preserve the stored unit price/payment facts and exactly one sale/line/payment/movement set.
- [x] 5.4 **REFACTOR — Keep the adapter deep and verify.** Consolidate reservation/summary queries inside `sale_repository.rs`, keep SQL parameterized, and retain compatibility methods only when a concrete caller still needs them. Run the focused use-case test, clippy, and rustfmt check.

**Finish:** SQLite satisfies reservation-first idempotency and authoritative price resolution through the new repository interface, with one verified end-to-end persistence path. **Rollback:** revert `sale_repository.rs` and the PR 5 fixture/test changes together; PRs 1–4 and the legacy adapter path remain intact. **Runtime:** `cargo test --manifest-path src-tauri/Cargo.toml --test confirm_sale_use_case` is the SQLite runtime harness.

## 6. PR 6 — SQLite atomic persistence, stock integrity, and stored-fact summaries

**Forecast:** 280–390 changed lines. **Depends on:** PR 5. **Out of scope:** catalog/Tauri public renames and frontend files.

- [x] 6.1 **RED — Specify atomic persistence failures.** Added reusable count/stock snapshots in `src-tauri/tests/confirm_sale_use_case.rs` for QR overpayment, insufficient tender, later-line stock failure followed by same-request retry, and corrupt confirmed-summary readback. Each failure leaves the snapshot unchanged.
- [x] 6.2 **GREEN — Complete caller-transaction-owned persistence.** Verified `persist_confirmed` writes lines, derived payments, conditional stock deductions, immutable movements, confirmation status, and SQLite summary readback in the caller-owned transaction. Corrected corrupt confirmed-summary readback to return `ConfirmSaleError::Persistence`, so the adapter preserves the documented persistence-failure boundary without a nested transaction.
- [x] 6.3 **TRIANGULATE — Prove payment modes and persisted-source output.** Added focused cash-only, QR-only, and mixed assertions for exact stored applied/tendered/change columns. Existing focused cases continue to prove trigger protection, request-ID uniqueness, repriced idempotent readback, and failed-attempt retry.
- [x] 6.4 **REFACTOR — Remove superseded SQLite compatibility paths.** Retained the legacy helper after concrete caller discovery found `confirm_sale.rs` still uses `SaleRepository::load_summary`; no unused compatibility path was removed. Ran the focused use-case and migration tests, clippy, rustfmt check, and diff check.

**Finish:** one SQLite transaction owns lines, payments, stock, movements, status, and stored-summary readback for every payment mode and failure path. **Rollback:** revert PR 6 persistence/test changes while retaining PR 5 reservation and resolution; never rewrite migration version 2 or confirmed history. **Runtime:** the focused SQLite integration targets are the runtime harness.

## 7. PR 7 — Rust catalog and strict Tauri command contract

**Forecast:** 240–350 changed lines. **Depends on:** PR 6. **Out of scope:** TypeScript and React changes.

- [x] 7.1 **RED/GREEN — Rename the outward catalog contract.** Update `src-tauri/tests/catalog_search.rs`, catalog cases in `src-tauri/tests/command_seam.rs`, `src-tauri/src/application/catalog/mod.rs`, and `src-tauri/src/commands/catalog.rs` to expose `catalog_unit_price_centavos` while retaining the physical SQLite column.
- [x] 7.2 **RED — Reject legacy confirmation authority.** Add table-driven JSON cases in `src-tauri/tests/command_seam.rs` for forbidden negotiated price, payment rows, applied cash, and change fields, plus nullable tender/QR and malformed UUID/quantity/money shapes.
- [x] 7.3 **GREEN — Implement the exact persisted command interface.** Update `src-tauri/src/commands/confirm_sale.rs` with `#[serde(deny_unknown_fields)]` at every request level, shape-only conversion, typed error mapping, and the exact persisted response. Change `src-tauri/src/lib.rs` only if command registration requires it.
- [x] 7.4 **TRIANGULATE/REFACTOR — Verify the Rust seam.** Cover cash, QR, mixed, repriced summary, and idempotent response without duplicating lower-layer matrices. Run command-seam and catalog tests, the complete Rust suite, clippy, and rustfmt check.

**Finish:** Rust accepts only the reduced request and returns persisted authoritative facts. **Rollback:** revert command/catalog DTOs and seam tests together; PR 6 remains internally valid, but a mismatched IPC pair must not be deployed. **Runtime:** command-seam integration tests exercise serialization and dispatch.

## 8. PR 8 — TypeScript catalog and confirmation command seam

**Forecast:** 220–320 changed lines. **Depends on:** PR 7. **Out of scope:** React reducer and presentation changes.

- [x] 8.1 **RED — Lock the exact IPC payload.** Updated `src/commands/confirm-sale.test.ts` and `src/catalog.test.js` to require only request ID, product/quantity, nullable tender/QR, and `catalog_unit_price_centavos`; unsafe, non-integer, negative money and non-positive identity/quantity reject before invoke.
- [x] 8.2 **GREEN — Implement reduced TypeScript interfaces.** Updated `src/commands/confirm-sale.ts` and `src/commands/catalog.ts` with the design request/success unions, safe-integer validation, and no client-authoritative price, applied-cash, or change fields.
- [x] 8.3 **TRIANGULATE/REFACTOR — Verify response and error stability.** Covered cash, QR, mixed, persisted-price differences, and backend error passthrough; removed obsolete command types. Focused tests, `npm test`, and `npm run build` passed.

**Finish:** TypeScript sends the strict reduced payload and consumes persisted authoritative responses. **Rollback:** revert both command modules and their tests; do not deploy against the legacy Rust IPC contract. **Runtime:** record any existing invoke harness, otherwise `N/A` with reason.

## 9. PR 9 — React checkout draft state and request continuity

**Forecast:** 250–360 changed lines. **Depends on:** PR 8. **Out of scope:** final persisted-summary presentation and full cross-stack acceptance.

- [x] 9.1 **RED — Specify draft state and request-ID continuity.** Updated `src/ui/sales/sale-flow.test.ts` to prove catalog price is guidance, obsolete editable-price/derived-cash actions are absent, tender strings remain inputs, failed retries retain one UUID, and discard/success starts the next intent.
- [x] 9.2 **GREEN — Simplify checkout state and submission.** Updated `src/ui/sales/sale-flow.ts` and `src/ui/sales/sale-screen.ts` to remove editable price, cash-applied, and change state/actions/controls and submit only product/quantity plus tender inputs.
- [x] 9.3 **TRIANGULATE/REFACTOR — Prove intent boundaries.** Covered cash, QR, mixed drafts, failed retry continuity, and discard clearing payment draft values so persisted response facts cannot seed a later request. Ran focused sale-flow tests, `npm test`, and `npm run build`.

**Finish:** draft state contains guidance and operator inputs only, and retries preserve one request ID. **Rollback:** revert sale flow/screen and their tests together; PR 8 remains the command interface. **Runtime:** focused UI tests; desktop interaction is deferred to PR 11.

## 10. PR 10 — Persisted checkout presentation

**Forecast:** 220–350 changed lines. **Depends on:** PR 9. **Out of scope:** licensing, product management, discounts, refunds, gateways, and schema work.

- [x] 10.1 **RED/GREEN — Distinguish guidance from persisted facts.** Update `src/ui/sales/catalog-result.test.ts`, `src/ui/sales/persisted-summary.test.ts`, `src/ui/sales/catalog-result.ts`, and `src/ui/sales/persisted-summary.ts` so draft guidance uses `catalog_unit_price_centavos` and success shows stored `unit_price_centavos` with authoritative cash/QR facts.
- [x] 10.2 **TRIANGULATE — Cover authoritative presentation.** Add cases for a stored price differing from guidance, cash change, QR without cash fields, mixed payment, stable errors, and no response-derived values entering a later submission.
- [x] 10.3 **REFACTOR — Remove obsolete presentation types.** Resolve TypeScript exhaustiveness errors, then run focused presentation tests, `npm test`, and `npm run build`.

**Finish:** checkout clearly distinguishes draft guidance from persisted authoritative results. **Rollback:** revert the four presentation files/tests; if the reduced draft becomes unusable, revert PRs 10 → 7 in reverse order.

## 11. PR 11 — Cross-stack acceptance and delivery evidence

**Forecast:** 80–200 changed lines, limited to test gaps and OpenSpec evidence; any implementation defect discovered here requires a separately replanned slice rather than expanding this PR above 400 lines. **Depends on:** PR 10.

- [x] 11.1 Ran and recorded the full frontend and Rust suites, production build, clippy, rustfmt check, and diff check in `apply-progress.md`. The current aggregate worktree stat is 2,101 additions + 640 deletions across tracked files; PR 11 itself changes OpenSpec evidence only and remains below 400 lines.
- [x] 11.2 Attempted `npm run tauri:dev`; Vite started and the Tauri desktop binary launched. Manual interaction is `N/A`: this executor has no interactive desktop input/inspection channel or UI automation to operate the launched window or query its live SQLite state. The exact cash, QR, mixed, rejection, repricing, retry, and persistence facts are covered by deterministic integration evidence recorded in `apply-progress.md`.
- [x] 11.3 Mapped every sales-spec scenario to named test evidence, including migration/rollback compatibility, command rejection before persistence, and no-effects failure cases. Scope audit found no licensing, product-management, discount, refund, or gateway implementation files.
- [x] 11.4 Recorded the dependency, focused verification, runtime boundary, rollback boundary, known actual count, and conventional-message plan for PRs 1–11 in `apply-progress.md`. Counts not captured by prior uncommitted slices are explicitly identified rather than reconstructed or fabricated.

**Finish:** every requirement has traceable evidence and every slice respects the 400-line review budget. **Rollback:** documentation/evidence can be reverted independently; any code correction must use its own bounded, dependency-aware slice.
