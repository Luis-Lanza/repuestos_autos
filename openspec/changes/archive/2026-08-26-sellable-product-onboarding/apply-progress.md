# Apply Progress: Sellable Product Onboarding — Slices 1–3

**Mode**: Standard (`strict_tdd: false`)
**Delivery**: chained PR slice, `stacked-to-main`; no commit, push, or PR was created.
**Scope**: Slice 1 covered domain/application transaction ownership, SQLite persistence, and forward migration. Slice 2 covered the Tauri command contract, minimal onboarding UI, and indexed sales search. Slice 3 adds the release benchmark and closes bounded scope/regression verification.

## Completed Tasks

- [x] 1.1 Retrospectively audited the direct-route dirty implementation.
- [x] 1.2 Added migration RED/green evidence for the v5 forward upgrade.
- [x] 1.3 Added onboarding persistence/search-document and immutable-movement evidence.
- [x] 2.1 Added `CreateProductUseCase` and its transaction-scoped repository interface.
- [x] 2.2 Added `SqliteCatalogRepository` and immutable forward migration `0005`.
- [x] 3.1 Reconciled strict onboarding/search payloads, registered commands, and stable envelopes.
- [x] 3.2 Proved minimal Sales ↔ Onboarding navigation and typed onboarding feedback.
- [x] 3.3 Replaced legacy search joins with canonical FTS prefix retrieval and backend-price sale coverage.
- [x] 4.1 Added and ran a release-profile benchmark of global FTS prefix search with 20,000 products.
- [x] 4.2 Closed exclusion/regression evidence for checkout/history compatibility, static checks, full suites, and a bounded desktop launch.

## Audit and Remediation

The pre-existing direct-route code already passed its older focused tests, but it issued product persistence SQL from the application module and altered movement semantics in applied migration v4. This slice keeps v4 untouched, moves the create-product write path behind a SQLite repository seam, and adds transactional v5 preflight, FTS5 document backfill, movement-vocabulary expansion, and immutable-row reconstruction. Category, command, UI, checkout, and sales-search wiring remain outside this work unit.

## Evidence

| Stage | Command / result |
|---|---|
| Baseline audit | `cargo test --manifest-path src-tauri/Cargo.toml --test product_onboarding --test sqlite_migrations` — PASS, 10 tests before new assertions. |
| RED | Same command after the FTS-document assertion — FAIL: `no such table: catalog_product_search`. |
| GREEN | `cargo fmt --manifest-path src-tauri/Cargo.toml --check && cargo test --manifest-path src-tauri/Cargo.toml --test product_onboarding --test sqlite_migrations` — PASS, 12 tests (5 onboarding, 7 migration). |
| Static | `cargo clippy --manifest-path src-tauri/Cargo.toml --tests -- -D warnings` — PASS. |

### Work Unit Evidence

| Evidence | Result |
|---|---|
| Focused test command and exact result | `cargo test --manifest-path src-tauri/Cargo.toml --test product_onboarding --test sqlite_migrations` — exit 0; 12 passed, 0 failed. |
| Runtime harness command/scenario and exact result | N/A — this slice has no Tauri/UI process boundary; in-memory SQLite integration tests exercise the product transaction, migration/reopen, rollback, FTS document, foreign-key, and immutability paths. |
| Rollback boundary | Revert `application/catalog/{mod.rs,repository.rs}`, `infrastructure/sqlite/{mod.rs,catalog_repository.rs,migrations/0005_catalog_onboarding_hardening.sql}`, and the two focused Rust test files together; no command/UI or checkout behavior is removed. |

**Evidence revision**: `sha256:ac4c4c42f2f33d233ea304abc8ba38d55fb1ccca1b4c864c2bccaaf7948af79d`

## Outstanding Risks

- FTS5 documents are now persisted and backfilled, but query normalization, prefix/limit retrieval, the 20,000-product benchmark, and sales/UI wiring remain task 3.3/4.1 work.
- The migration tests cover version-zero/one flow and a representative version-four forward migration; production rollout still requires the backup procedure described in the proposal.

## Attempt Evidence

- Parent-owned active token: `sha256:0ade3e3da8c775f847c71b21efa1fb998158c57ad5214470ba512023a75a06a7`.
- This executor did not acquire or settle the attempt.
- Diagnosis: existing direct-route persistence lacked the required adapter seam and forward-only v5 compatibility hardening; remediation is now covered by focused tests.
- Harness disposition: `reused`.
- Cleanup evidence: `git diff --check` passed; the pre-existing formatting-only diffs in `src/ui/sales/sale-flow.ts` and `src/ui/sales/sale-screen.ts` remain untouched.
- Process evidence: no commit, push, PR, subagent, attempt acquire, or attempt settle was performed.

## Slice 2: IPC, Onboarding UI, and Indexed Sales Search

**Mode**: Standard (`strict_tdd: false`)
**Delivery**: chained PR slice 2, `stacked-to-main`; no commit, push, or PR was created.

### Completed Tasks

- [x] 3.1 Reconciled strict onboarding/search payloads, registered Tauri commands, and list/create envelopes with SQL-free stable errors.
- [x] 3.2 Audited and proved local Sales ↔ Onboarding navigation, category setup, typed fields, validation feedback, product facts, opening stock, and persisted-result feedback.
- [x] 3.3 Replaced legacy global-search joins with canonical FTS5 retrieval, normalized prefix terms, a 20-result limit, and a command-seam proof that an onboarded product sells at its backend price.

### RED → GREEN Audit

| Stage | Command / result |
|---|---|
| RED: command/UI | `npx tsx --test src/commands/onboarding.test.ts src/ui/onboarding/onboarding-form.test.ts` — failed before `SCREEN`/`screenAfter` were exported. |
| RED: search | `cargo test --manifest-path src-tauri/Cargo.toml --test command_seam --test catalog_search` — failed before reconciliation: `no such table: product_searchable_values` after the canonical-document test removed the legacy table. |
| GREEN | `cargo test --manifest-path src-tauri/Cargo.toml --test command_seam --test catalog_search --test confirm_sale_application` — exit 0; 22 passed, 0 failed. |
| Frontend | `npx tsx --test src/commands/onboarding.test.ts src/ui/onboarding/onboarding-form.test.ts && npx tsc --noEmit && npm run build` — exit 0; 3 tests passed, typecheck passed, Vite built. |
| Static | `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo clippy --manifest-path src-tauri/Cargo.toml --tests -- -D warnings`, and `cargo check --manifest-path src-tauri/Cargo.toml --features desktop` — all exit 0. |

### Work Unit Evidence

| Evidence | Result |
|---|---|
| Focused test command and exact result | Rust command/search/sale command: exit 0, 22 passed. Frontend adapter/form: exit 0, 3 passed. |
| Runtime harness command/scenario and exact result | `timeout 30s npm run tauri:dev` started Vite (ready in 270 ms), compiled the desktop binary, and reached `Running target/debug/repuestos-autos`; the planned 30-second timeout then stopped the interactive process. Command-seam tests exercise valid/invalid onboarding and immediate sale in the same local database. |
| Rollback boundary | Revert only the slice-2 command, UI, canonical-search, and focused-test changes in `src-tauri/src/{application/catalog/mod.rs,commands/{catalog,onboarding}.rs,lib.rs}`, `src/{commands/onboarding.ts,ui/{app.ts,onboarding/{onboarding-screen.ts,onboarding-form.test.ts}}}`, `src/commands/onboarding.test.ts`, and `src-tauri/tests/{catalog_search.rs,command_seam.rs}`; do not modify sales formatting or phase-1 persistence/migration files. |

**Evidence revision**: `sha256:2c1067430c9b96176ff626cefba1475760720c3271d6ce81e484e2a9546c7645`

### Slice 2 Attempt Evidence

- Parent-owned active token: `sha256:0886b693e783714dc4bd6d499163fbfab65e1a365abc70232451948e565fcbe0`.
- This executor did not acquire or settle the attempt.
- Diagnosis: direct-route onboarding lacked strict request deserialization and a stable category-list envelope; search still depended on unbounded leading-wildcard joins instead of the persisted FTS5 document.
- Harness disposition: `reused`.
- Cleanup evidence: `git diff --check` passed; the pre-existing formatting-only diffs in `src/ui/sales/sale-flow.ts` and `src/ui/sales/sale-screen.ts` remain untouched.
- Process evidence: no commit, push, PR, subagent, attempt acquire, or attempt settle was performed.

## Slice 3: Benchmark and Scope / Regression Closure

### Completed Tasks

- [x] 4.1 Extended `src-tauri/tests/catalog_search.rs` with a deterministic release-profile benchmark: one transaction inserts 20,000 active products, balances, and canonical FTS documents before timing only the `bench` prefix query.
- [x] 4.2 Ran the complete Rust and frontend suites, formatting, Clippy, desktop compile, typecheck, production frontend build, bounded desktop launch, and a command/UI scope scan. Existing fixed-price price-resolution, payment, idempotency, and persisted-history tests remained green.

### Benchmark Methodology and Result

The test preloads exactly 20,000 active catalog products in one SQLite transaction; each product has a balance and a canonical FTS document. It then times only `search_active_products(&connection, "bench")`, which performs normalized FTS prefix retrieval and asserts the 20-result limit plus matching result names. The release command ran on `Linux 7.1.5-101.fc43.x86_64 x86_64` with 16 logical CPUs.

| Command | Exact result |
|---|---|
| `cargo test --release --manifest-path src-tauri/Cargo.toml --test catalog_search -- --nocapture` | Exit 0; 5 passed, 0 failed; the 20,000-product prefix query took `10.099391ms`, below the `<= 1s` target. |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Exit 0; 69 Rust tests passed, 0 failed. |
| `npm test` | Exit 0; 16 frontend tests passed, 0 failed. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings && cargo check --manifest-path src-tauri/Cargo.toml --features desktop && npx tsc --noEmit && npm run build` | Exit 0; formatting, Clippy, desktop check, TypeScript check, and Vite production build passed. |
| `timeout 30s npm run tauri:dev` | Expected timeout exit 124 after Vite was ready in 293ms and `target/debug/repuestos-autos` started; this is a bounded launch smoke, not an automated desktop interaction. |

### Scope and Regression Closure

- The command/UI source scan found no edit, archive, replenishment, adjustment, cancellation, report, backup, licensing, barcode, fractional-stock, Excel-import, or return-processing workflow. `0005_catalog_onboarding_hardening.sql` contains only the design-approved inert movement-vocabulary reservation (`stock_entry`, `return`, `adjustment`, `cancellation`); it exposes no command, UI, or mutation path.
- Rust command-seam and confirm-sale suites proved backend-price resolution, cash/QR/mixed payments, request-ID idempotency, immutable stock movements, and persisted sale-line/history snapshots. Frontend tests proved reduced confirmation payloads and persisted-summary presentation.
- The pre-existing formatting-only changes in `src/ui/sales/sale-flow.ts` and `src/ui/sales/sale-screen.ts` were not edited. `git diff --check` passed.

### Work Unit Evidence

| Evidence | Result |
|---|---|
| Focused test command and exact result | `cargo test --release --manifest-path src-tauri/Cargo.toml --test catalog_search -- --nocapture` — exit 0; 5 passed, 0 failed; benchmark query `10.099391ms`; 20 returned prefix matches from 20,000 products. |
| Runtime harness command/scenario and exact result | `timeout 30s npm run tauri:dev` — Vite ready in 293ms and the desktop binary started; expected timeout ended the interactive process. Command-seam tests provide the automated onboarding-to-search-to-sale scenario. |
| Rollback boundary | Revert only the benchmark test in `src-tauri/tests/catalog_search.rs` and this slice's task/apply evidence; no production onboarding, checkout, history, UI, or migration behavior was changed. |

### Remaining Evidence Limits

- The benchmark proves the release binary's SQLite search path on this Linux 16-logical-CPU runner; it does not establish a hardware-equivalence claim for an unspecified Windows deployment machine.
- Migration coverage uses disposable SQLite databases and does not replace a real-store backup/restore rehearsal.
- The bounded desktop smoke proves startup only; rendered click-through was not automated because no browser/desktop interaction runner is configured.

**Slice 3 evidence revision**: `sha256:b55b04ea2859795523f0d143f8cc62e5c2f5785434b9504c6da52b072aff1f6f`

### Slice 3 Attempt Evidence

- Parent-owned active token: `sha256:0f59b43a7b4aeaed86458d9624023ce3e740086e746dedada27cffd02c9a3f99` for `apply-onboarding-slice-3-20260826`; this executor did not acquire or settle it.
- Diagnosis: the remaining risk was unmeasured performance and unclosed regression/scope evidence, not a production defect. The release benchmark passed and all requested suites/checks are green.
- Harness disposition: reused; no new runtime harness was created.
- Cleanup evidence: `git diff --check` passed; user formatting and unrelated untracked/generated paths remain preserved.
- Process evidence: no commit, push, PR, subagent, attempt acquire, or attempt settle was performed.

## Bounded Remediation: Close Two Negative Evidence Gaps

**Work unit**: `close-two-negative-evidence-gaps`
**Mode**: Standard; stacked-to-main remediation slice
**Changed lines**: 122 authored additions/deletions, below the 400-line limit
**Remediates**: `sha256:ee1d5057148b6d022c312346e78fa665de646358530d87eeeffac6a85dc570c9`

`command_builder` now owns the production `generate_handler!` registration and is reused by a Tauri `MockRuntime` test app. The two behavior-first tests dispatch unavailable product-management and draft-cart operation names through that registration seam, then compare complete SQLite persistence snapshots. Each snapshot includes stock balances and the sales, payment, sale-line, and inventory-movement facts, proving rejected operations cannot mutate persistence.

### RED → GREEN Evidence

| Stage | Command / exact result |
|---|---|
| RED | `CARGO_TARGET_DIR="/tmp/opencode/sellable-product-onboarding-remediation-target" cargo test --locked --manifest-path src-tauri/Cargo.toml --features desktop --lib command_surface_tests` — failed before the test harness existed: `tauri::test` was not enabled and `command_builder` was unresolved. |
| GREEN | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR="/tmp/opencode/sellable-product-onboarding-remediation-target" cargo test --locked --manifest-path src-tauri/Cargo.toml --features desktop --lib command_surface_tests` — exit 0; 2 passed, 0 failed. |
| Regression | `CARGO_TARGET_DIR="/tmp/opencode/sellable-product-onboarding-remediation-regression-target" cargo test --locked --manifest-path src-tauri/Cargo.toml` — exit 0; 69 Rust tests passed, 0 failed. `npm test` — exit 0; 16 frontend tests passed, 0 failed. |
| Static | `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `npx tsc --noEmit`, `npm run build -- --outDir "/tmp/opencode/sellable-product-onboarding-remediation-dist" --emptyOutDir`, and `git diff --check` — each exit 0. `cargo check --features desktop` and Clippy were attempted but could not finish after `/tmp/opencode` hit the runner's disk quota; their failures were environmental (`Disk quota exceeded` / linker bus error), not compiler or lint findings. |

### Work Unit Evidence

| Evidence | Command / boundary | Exact result |
|---|---|---|
| Focused GREEN | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR="/tmp/opencode/sellable-product-onboarding-remediation-target" cargo test --locked --manifest-path src-tauri/Cargo.toml --features desktop --lib command_surface_tests` | Exit 0; 2 passed, 0 failed. |
| Runtime harness | Tauri `MockRuntime` IPC dispatch through the same `command_builder` used by desktop `run()` | Excluded `update_product_command`, `archive_product_command`, `import_products_command`, `remove_draft_cart_line_command`, and `discard_draft_cart_command` were rejected; both before/after SQLite snapshots were equal. |
| Rollback boundary | `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, and this remediation evidence | Revert the `tauri` test feature, shared command builder, and its two command-surface tests without changing onboarding, checkout, inventory, or protected sales formatting. |

### Remediation Diagnosis and Process Evidence

- **Diagnosis**: The failed verification found missing executable negative proof, not a product defect. These two tests close the command-surface exclusion and draft removal/discard persistence-invariance scenarios without adding any workflow.
- **Harness disposition**: `reused` — Tauri's built-in mock runtime exercised real command registration and IPC dispatch without launching a native webview.
- **Cleanup evidence**: `git diff --check` passed. Protected `src/ui/sales/sale-flow.ts` and `src/ui/sales/sale-screen.ts` formatting-only changes and unrelated untracked/generated paths were not edited. Temporary Cargo targets were kept under `/tmp/opencode`; one oversized temporary target was removed after a disk-quota failure.
- **Process evidence**: No subagent, commit, push, PR, attempt acquire, or attempt settle occurred. Parent-owned token `sha256:47c2802d9289905ba312413e4d6eb610a583fa83d60f92807cdc1f6614ba0ac1` remains active and untouched.
- **Static-check limitation**: A fresh verifier should rerun desktop `cargo check` and Clippy with sufficient temporary disk space; the focused runtime proof, default Rust suite, frontend suite, formatter, TypeScript check, Vite build, and diff check passed.

```yaml
schema: gentle-ai.remediation-result/v1
lineage_id: sha256:da771aa3f730a8ffffe330600d9a53ab7582b3eeb284fa21be9d1adccd214c5b
generation: 6
fix_batch: 2
failed_evidence_revision: sha256:ee1d5057148b6d022c312346e78fa665de646358530d87eeeffac6a85dc570c9
evidence_revision: sha256:f14aee63b7b2977cf3e5de6a950775e556d2cfa720b435e92ee621fe460355bb
status: complete
mode: Standard
focused_tests: passed
runtime_harness: passed
rollback_boundary: recorded
next_recommended: sdd-verify
```
```json
{
  "schema": "gentle-ai.remediation-evidence/v1",
  "lineage_id": "sha256:da771aa3f730a8ffffe330600d9a53ab7582b3eeb284fa21be9d1adccd214c5b",
  "generation": 6,
  "fix_batch": 2,
  "failed_evidence_revision": "sha256:ee1d5057148b6d022c312346e78fa665de646358530d87eeeffac6a85dc570c9",
  "evidence_revision": "sha256:f14aee63b7b2977cf3e5de6a950775e556d2cfa720b435e92ee621fe460355bb",
  "commands": [
    {
      "command": "CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/opencode/sellable-product-onboarding-remediation-target cargo test --locked --manifest-path src-tauri/Cargo.toml --features desktop --lib command_surface_tests",
      "exit_code": 0,
      "result": "2 passed, 0 failed"
    },
    {
      "command": "CARGO_TARGET_DIR=/tmp/opencode/sellable-product-onboarding-remediation-regression-target cargo test --locked --manifest-path src-tauri/Cargo.toml",
      "exit_code": 0,
      "result": "69 Rust tests passed, 0 failed"
    },
    {
      "command": "npm test",
      "exit_code": 0,
      "result": "16 frontend tests passed, 0 failed"
    },
    {
      "command": "cargo fmt --manifest-path src-tauri/Cargo.toml --check; npx tsc --noEmit; npm run build -- --outDir /tmp/opencode/sellable-product-onboarding-remediation-dist --emptyOutDir; git diff --check",
      "exit_code": 0,
      "result": "formatter, TypeScript, Vite build, and diff check passed"
    }
  ],
  "runtime_harness": {
    "status": "passed",
    "command": "CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/opencode/sellable-product-onboarding-remediation-target cargo test --locked --manifest-path src-tauri/Cargo.toml --features desktop --lib command_surface_tests",
    "result": "Tauri MockRuntime dispatched unavailable onboarding and draft commands through command_builder; persistence snapshots were unchanged",
    "na_reason": ""
  },
  "rollback": {
    "boundary": "src-tauri/Cargo.toml, src-tauri/src/lib.rs, and bounded remediation evidence",
    "evidence": "Revert the shared command registration seam and its two negative runtime tests without touching product, checkout, inventory, or sales UI behavior."
  }
}
```
