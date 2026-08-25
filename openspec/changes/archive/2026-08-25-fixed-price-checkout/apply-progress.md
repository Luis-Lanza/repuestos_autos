# Apply Progress: Fixed-price checkout

## Cumulative completed work before PR 9

- PR 1: completed migration and legacy compatibility evidence.
- PR 2: completed authoritative sale-line and derived-payment domain rules.
- PR 3: completed application transaction and idempotency contract.
- PR 4: completed transaction-order failure hardening.
- PR 5: completed SQLite reservation, catalog-price resolution, and idempotent readback.
- PR 6: completed SQLite atomic persistence, stock integrity, and stored-fact summaries.
- PR 7: completed Rust catalog and strict Tauri command contract.
- PR 8: completed TypeScript catalog and confirmation command seam.

Detailed PR 1–8 evidence remains recorded in the change task history and prior apply executor output; this file continues from that completed state.

## PR 9 — React checkout draft state and request continuity

Completed tasks: 9.1, 9.2, 9.3.

### Completed work

- Replaced negotiated/minimum draft price fields with read-only `catalog_unit_price_centavos` guidance.
- Replaced payment-row draft state with string-backed tendered-cash and QR inputs.
- Removed editable unit-price, cash-applied, and cash-change actions, controls, and request construction.
- Reduced checkout submission to request ID, product/quantity lines, and nullable tender inputs.
- Preserved a request ID through failed confirmation retries; discard after success or cancellation clears draft inputs for the next intent.

### Files changed

- `src/ui/sales/sale-flow.test.ts`
- `src/ui/sales/sale-flow.ts`
- `src/ui/sales/sale-screen.ts`
- `openspec/changes/fixed-price-checkout/tasks.md`
- `openspec/changes/fixed-price-checkout/apply-progress.md`

### TDD Cycle Evidence

| Cycle | Evidence |
| --- | --- |
| RED | Updated the sale-flow test for catalog guidance and string payment drafts. `npx tsx --test src/ui/sales/sale-flow.test.ts` failed 3 assertions because legacy price/payment state remained. |
| GREEN | Replaced legacy sale-flow state/actions and reduced the screen request. The focused test target passed 4 tests. |
| TRIANGULATE | Added cash-only, QR-only, mixed-input, failed-retry continuity, and post-success discard assertions. |
| REFACTOR | Consolidated payment draft input into one flat object and removed unused client-side centavo parsing branches. |

### Verification

- `npx tsx --test src/ui/sales/sale-flow.test.ts` — 4 passed.
- `npm test` — 11 passed.
- `npm run build` — passed.
- `git diff --check` — passed.
- TypeScript diagnostics scoped to the PR 9 files: no errors in `sale-flow.ts`, `sale-flow.test.ts`, or `sale-screen.ts`. The full direct `tsc` invocation now reports only two intentionally deferred PR 10 presentation errors in `catalog-result.ts` and `persisted-summary.ts`.

### Scope and delivery

- Deviation from design: none.
- Remaining tasks: PRs 10–11.
- Workload/PR boundary: stacked-to-main PR 9, `react-checkout-draft-state-and-request-continuity`; source/test delta is 318 additions + deletions before OpenSpec evidence, below the 400-line cap. No commit created.
- Rollback boundary: revert the three PR 9 React files and their task/progress evidence together; PR 8 remains the reduced command interface.

## PR 10 — Persisted checkout presentation

Completed tasks: 10.1, 10.2, 10.3.

### Completed work

- Renamed catalog-result presentation from obsolete minimum-price terminology to read-only `catalog_unit_price_centavos` guidance.
- Renamed persisted line presentation from obsolete negotiated-price terminology to the stored `unit_price_centavos` fact.
- Added presentation coverage for cash change, QR-only payment without cash fields, and mixed QR/cash payment facts.
- Preserved the prior reduced submission boundary: persisted-summary values remain display-only, while existing sale-flow coverage confirms only draft product, quantity, and tender values enter later requests.

### Files changed

- `src/ui/sales/catalog-result.ts`
- `src/ui/sales/catalog-result.test.ts`
- `src/ui/sales/persisted-summary.ts`
- `src/ui/sales/persisted-summary.test.ts`
- `openspec/changes/fixed-price-checkout/tasks.md`
- `openspec/changes/fixed-price-checkout/apply-progress.md`

### TDD Cycle Evidence

| Cycle | Evidence |
| --- | --- |
| RED | Updated presentation fixtures to the renamed catalog and persisted interfaces. `npx tsx --test src/ui/sales/catalog-result.test.ts src/ui/sales/persisted-summary.test.ts` failed: both obsolete source fields produced `Bs NaN`. |
| GREEN | Replaced the two obsolete field reads with `catalog_unit_price_centavos` and `unit_price_centavos`; focused presentation tests passed 3 tests and `npx tsc --noEmit --pretty false` passed. |
| TRIANGULATE | Added QR-only and mixed-payment formatting assertions; cash-change formatting remains covered by the persisted cash fixture. The full suite verified stable backend errors and cash/QR/mixed payload behavior. |
| REFACTOR | Retained the existing flat presentation helpers, removed the obsolete field accesses, and confirmed no TypeScript diagnostics remain. |

### Verification

- `npx tsx --test src/ui/sales/catalog-result.test.ts src/ui/sales/persisted-summary.test.ts` — 3 passed.
- `npm test` — 12 passed.
- `npm run build` — passed.
- `npx tsc --noEmit --pretty false` — passed; the two deferred PR 10 diagnostics are resolved.
- `git diff --check` — passed.

### Scope and delivery

- Deviation from design: none.
- Remaining tasks: PR 11.
- Workload/PR boundary: stacked-to-main PR 10, `persisted-checkout-presentation`; the scoped implementation/test delta was 40 additions + deletions before OpenSpec evidence, below the 400-line cap. No commit created.
- Rollback boundary: revert the four presentation files/tests and this PR 10 task/progress evidence together; PR 9 remains the draft-state dependency.

## PR 11 — Cross-stack acceptance and delivery evidence

Completed tasks: 11.1, 11.2, 11.3, 11.4.

### Verification

| Command | Exact result |
| --- | --- |
| `npx tsx --test src/commands/confirm-sale.test.ts src/catalog.test.js src/ui/sales/sale-flow.test.ts src/ui/sales/catalog-result.test.ts src/ui/sales/persisted-summary.test.ts` | Passed: 12 tests. |
| `npm test` | Passed: 12 tests, 0 failures. |
| `npm run build` | Passed: Vite transformed 32 modules and emitted `dist/index.html` plus the production JavaScript bundle. |
| `cargo test --manifest-path src-tauri/Cargo.toml --test sqlite_migrations --test sale_domain --test confirm_sale_application --test confirm_sale_use_case --test command_seam --test catalog_search` | Passed: 48 integration tests (5 migration, 13 domain, 9 application, 14 SQLite use-case, 4 command seam, 3 catalog). |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Passed: 48 integration tests, 0 unit tests, and 0 doc tests. |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings` | Passed with no warnings. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | Passed. |
| `git diff --check` | Passed. |

### Desktop runtime boundary

`npm run tauri:dev` was attempted with `DISPLAY=:0`, `WAYLAND_DISPLAY=wayland-0`, the Tauri CLI, and the desktop entrypoint available. Vite became ready, Cargo built the desktop feature, and `target/debug/repuestos-autos` launched. Manual checkout execution is `N/A`: this executor has no interactive desktop input/inspection channel or UI automation to operate that window, seed a live sale, or inspect its SQLite state. The process was stopped by the bounded 45-second harness timeout; no manual scenario result is claimed.

### Sales specification traceability

| Sales spec scenario | Concrete evidence |
| --- | --- |
| Reject negotiated price and derived-payment authority | `command_seam::rejects_legacy_authority_and_invalid_request_shapes_before_confirmation`; `confirm-sale.test.ts` sends only the reduced payload. |
| Open an existing database after migration | `sqlite_migrations::migrates_version_one_without_rewriting_legacy_facts_and_reopens_idempotently`; verifies unchanged legacy row facts and idempotent reopen. |
| Roll back application behavior after migration | `sqlite_migrations::migrates_version_one_without_rewriting_legacy_facts_and_reopens_idempotently` verifies physical-column compatibility; migration design documents forward-compatible rollback without downgrade or rewrites. |
| Confirm cash-only sale with derived change | `sale_domain::cash_only_derives_exact_applied_amount_and_change`; `confirm_sale_use_case::confirms_a_multi_line_cash_sale_with_persisted_stock_movements_and_summary`. |
| Confirm QR-only sale | `sale_domain::qr_only_derives_one_qr_payment`; `confirm_sale_use_case::confirms_qr_only_and_mixed_payment_sales`. |
| Confirm mixed sale | `sale_domain::mixed_payment_emits_qr_before_cash_and_derives_exact_or_change`; `confirm_sale_use_case::confirms_qr_only_and_mixed_payment_sales`. |
| Reject QR overpayment | `sale_domain::qr_above_total_is_rejected`; `confirm_sale_use_case::rejects_invalid_authoritative_payments_without_persisting_any_effects`. |
| Reject insufficient tender | `sale_domain::missing_or_insufficient_cash_for_remaining_total_is_rejected`; `confirm_sale_use_case::rejects_invalid_authoritative_payments_without_persisting_any_effects`. |
| Retry after a catalog price changes | `confirm_sale_use_case::reservation_short_circuits_repriced_or_missing_retries_to_stored_facts`; `command_seam::returns_repriced_persisted_summary_for_idempotent_retries`. |
| Retry an unsuccessful attempt | `confirm_sale_use_case::rolls_back_later_stock_failure_and_allows_the_same_request_to_retry`; `confirm_sale_application::application_failures_stop_in_order_and_roll_back_the_reservation`. |

Additional no-effects coverage: missing/inactive/duplicate requests are covered by `confirm_sale_use_case::rejects_inactive_missing_stale_and_unequal_payment_requests_without_effects`; later stock failure by `rolls_back_every_effect_when_a_later_line_has_insufficient_stock`; command shape rejection occurs before confirmation in the command-seam test. Migration preflight and future-version failures preserve rows/version in the remaining `sqlite_migrations` tests.

### Scope and delivery evidence

The tracked aggregate worktree stat before PR 11 evidence was 2,101 additions + 640 deletions across 30 files. A scope audit of modified and untracked change files found only checkout, catalog, Tauri/Rust, SQLite migration/test, OpenSpec, and project-support assets; no licensing, product-management, discount, refund, or payment-gateway implementation entered the change.

| PR | Dependency | Focused verification / runtime | Actual source/test delta | Rollback boundary | Conventional message plan |
| --- | --- | --- | --- | --- | --- |
| 1 | Baseline | `sqlite_migrations` (5 passed); runtime: migration harness | Not preserved in the cumulative uncommitted worktree | Migration runner/tests plus migration 0002; never downgrade version 2 | `feat(sales): preserve fixed-price migration compatibility` |
| 2 | PR 1 | `sale_domain` (13 passed); runtime: domain harness | Not preserved in the cumulative uncommitted worktree | Domain module and domain tests only | `feat(sales): derive authoritative payment facts` |
| 3 | PR 2 | Application (4 at completion), domain (13), legacy use-case (8); runtime: repository-double harness | 327 | Application contract/interface/use case and repository-double tests | `feat(sales): add idempotent confirmation transaction core` |
| 4 | PR 3 | `confirm_sale_application`, `sale_domain`; runtime: repository-double harness | Not preserved in the cumulative uncommitted worktree | PR 4 failure tests and directly proven application corrections | `test(sales): harden confirmation transaction failures` |
| 5 | PR 4 | `confirm_sale_use_case`; runtime: SQLite integration harness | Not preserved in the cumulative uncommitted worktree | `sale_repository.rs` and PR 5 fixture/tests together | `feat(sales): resolve fixed prices through SQLite reservations` |
| 6 | PR 5 | `confirm_sale_use_case`, `sqlite_migrations`; runtime: SQLite integration harness | Not preserved in the cumulative uncommitted worktree | PR 6 persistence/tests while retaining PR 5 resolution | `feat(sales): persist atomic fixed-price sale facts` |
| 7 | PR 6 | `command_seam`, `catalog_search`, full Rust suite; runtime: command serialization/dispatch | Not preserved in the cumulative uncommitted worktree | Rust command/catalog DTOs and seam tests together | `feat(commands): expose strict fixed-price checkout contract` |
| 8 | PR 7 | command-focused TypeScript tests, `npm test`, `npm run build`; runtime: invoke harness N/A | Not preserved in the cumulative uncommitted worktree | TypeScript command modules and their tests | `feat(commands): send reduced checkout payload` |
| 9 | PR 8 | `sale-flow.test.ts` (4 passed), `npm test`, `npm run build`; runtime: focused UI tests | 318 before OpenSpec evidence | Sale flow/screen and their evidence | `feat(ui): retain checkout intent across retries` |
| 10 | PR 9 | presentation tests (3 passed), `npm test`, `npm run build`; runtime: focused UI tests | 40 before OpenSpec evidence | Presentation files/tests and their evidence | `feat(ui): present persisted checkout facts` |
| 11 | PR 10 | Full frontend/Rust suites, build, clippy, rustfmt, diff check; manual desktop N/A as bounded above | 74 additions + 4 deletions = 78 (OpenSpec evidence only) | PR 11 evidence only; code defects require a new slice | `docs(sdd): record fixed-price checkout acceptance evidence` |

Historical PR 1–2 and 4–8 counts were not captured in the available file artifacts, and their changes now share files in one cumulative uncommitted worktree. They are therefore explicitly not reconstructed from the aggregate diff. The reported PR 3, 9, and 10 counts are the completion-time values in the task/progress history; every PR's forecast was at or below 400 lines. No commit was created.

### Files changed

- `openspec/changes/fixed-price-checkout/tasks.md`
- `openspec/changes/fixed-price-checkout/apply-progress.md`

### Scope and delivery

- Deviation from design: none. No product behavior was added.
- Remaining tasks: none.
- Workload/PR boundary: stacked-to-main PR 11, `cross-stack-acceptance-and-delivery-evidence`; 74 additions + 4 deletions = 78, evidence-only and below the 400-line cap.
- Rollback boundary: revert only PR 11 task/progress evidence. Any implementation correction must be replanned as a new bounded, dependency-aware slice.
- Commit status: no commit created or requested.
