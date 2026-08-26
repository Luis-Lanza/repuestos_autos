```yaml
schema: gentle-ai.verify-result/v1
evidence_revision: sha256:b81aa20ecc0c7e1a47ec250775e5886f762e4df522b82f45c70e43406adeac8c
verdict: pass
blockers: 0
critical_findings: 0
requirements: 7/7
scenarios: 13/13
test_command: CARGO_BUILD_JOBS=1 cargo test --locked --manifest-path "src-tauri/Cargo.toml" --features desktop --lib command_surface_tests && cargo test --locked --manifest-path "src-tauri/Cargo.toml" && npm test && cargo test --locked --release --manifest-path "src-tauri/Cargo.toml" --test catalog_search -- --nocapture
test_exit_code: 0
test_output_hash: sha256:f97d135fddf77f200e7e01c28b92281cbacc7916f96924795ac5bcef819d02c4
build_command: cargo fmt --manifest-path "src-tauri/Cargo.toml" --check && CARGO_BUILD_JOBS=1 cargo clippy --locked --manifest-path "src-tauri/Cargo.toml" --all-targets --all-features -- -D warnings && CARGO_BUILD_JOBS=1 cargo check --locked --manifest-path "src-tauri/Cargo.toml" --features desktop && npx tsc --noEmit && npm run build -- --outDir "/tmp/opencode/sellable-product-onboarding-final-verify/vite-dist" --emptyOutDir
build_exit_code: 0
build_output_hash: sha256:6e9978ee128ae6a9281fea6fe20aad9f75818a5fe8698196ea28c7f8d4207c8c
```

## Verification Report

**Change**: sellable-product-onboarding
**Version**: N/A
**Mode**: Standard (`strict_tdd: false`)
**Artifact store**: Hybrid/both
**Status**: PASS WITH WARNINGS

### Executive Summary

All 10 tasks are complete and both artifact backends agree on proposal, 7 requirements, 13 scenarios, design, and tasks. Fresh runtime execution passed the two remediation tests, the complete 69-test Rust suite, 16 frontend tests, the release 20,000-product benchmark, formatting, Clippy, desktop Cargo check, TypeScript checking, Vite production build, and bounded desktop startup. The candidate implementation remained byte-identical throughout runtime checks.

### Completeness

| Metric | Value |
|---|---:|
| Requirements | 7 |
| Requirements compliant | 7 |
| Scenarios | 13 |
| Scenarios compliant | 13 |
| Tasks total | 10 |
| Tasks complete | 10 |
| Tasks incomplete | 0 |

### Requirement / Scenario Coverage Matrix

| Requirement | Scenario | Passing runtime evidence | Result |
|---|---|---|---|
| Fields | Valid values | `product_onboarding::creates_product_attributes_balance_and_opening_movement_atomically`; onboarding command seam | ✅ COMPLIANT |
| Fields | Invalid values | Catalog domain validation; `missing_required_field_persists_nothing`; invalid number/option cases | ✅ COMPLIANT |
| Constraints | Invalid product | Duplicate SKU, zero price, zero stock, unknown field/value integration cases | ✅ COMPLIANT |
| Opening | Valid persistence | Atomic product/value/balance/FTS/opening-movement test, including timestamp and immutability | ✅ COMPLIANT |
| Opening | Rollback | Injected opening-movement failure rolls back prior product writes | ✅ COMPLIANT |
| Result | Immediate sale | Onboard → FTS search by all configured fields → fixed-price checkout | ✅ COMPLIANT |
| Scope | Exclusions | `command_surface_tests::rejects_excluded_onboarding_operations_without_persistence_mutation` | ✅ COMPLIANT |
| Active Product Search and Cart | Search and add an active product | Seeded/operator-created FTS tests plus frontend cart reducer | ✅ COMPLIANT |
| Active Product Search and Cart | Archived or inactive products cannot be sold | Inactive search exclusion and confirmation no-effects tests | ✅ COMPLIANT |
| Active Product Search and Cart | Discarding or removing a draft cart line | Frontend draft remove/discard behavior plus `rejecting_draft_removal_and_discard_leaves_persistence_unchanged` snapshot | ✅ COMPLIANT |
| Active Product Search and Cart | Sell an onboarded product under unchanged rules | Command seam and product-onboarding checkout tests retain backend price/history snapshot | ✅ COMPLIANT |
| Confirm-Sale Scope Exclusions | Product-management workflows are separate | Sales ↔ Onboarding navigation plus rejected management commands | ✅ COMPLIANT |
| Confirm-Sale Scope Exclusions | External and future workflows do not affect confirmation | Local cash/QR/mixed confirmation suite executes without external services | ✅ COMPLIANT |

**Compliance summary**: 13/13 scenarios and 7/7 requirements compliant.

### Mandatory Remediation Proof

The production `command_builder` and Tauri `MockRuntime` were inspected in `src-tauri/src/lib.rs` and executed fresh. The first test dispatched `update_product_command`, `archive_product_command`, and `import_products_command`; every command was rejected. The second dispatched `remove_draft_cart_line_command` and `discard_draft_cart_command`; both were rejected. Before/after SQLite snapshots were equal across `stock_balances`, `sales`, `sale_payments`, `sale_lines`, and `inventory_movements`.

```text
CARGO_BUILD_JOBS=1 cargo test --locked --manifest-path "src-tauri/Cargo.toml" --features desktop --lib command_surface_tests
exit 0
2 passed; 0 failed; 0 ignored.
Focused output SHA-256: fc725f19536f97d4eb5cb88b3a2996cdbfbe906d27760dfdf4ea2d2b4b2d0b98
```

### Build and Test Execution

#### Complete tests and release benchmark

```text
CARGO_BUILD_JOBS=1 cargo test --locked --manifest-path "src-tauri/Cargo.toml" --features desktop --lib command_surface_tests && cargo test --locked --manifest-path "src-tauri/Cargo.toml" && npm test && cargo test --locked --release --manifest-path "src-tauri/Cargo.toml" --test catalog_search -- --nocapture
exit 0
2 focused remediation tests, 69 default Rust tests, and 16 frontend tests passed; 0 failed/ignored/skipped.
Release benchmark: 20,000 products; 20 bounded prefix results; 8.606828 ms <= 1 second; benchmark binary 5/5 passed.
Output SHA-256: f97d135fddf77f200e7e01c28b92281cbacc7916f96924795ac5bcef819d02c4
```

#### Formatting, linting, checks, and frontend build

```text
cargo fmt --manifest-path "src-tauri/Cargo.toml" --check && CARGO_BUILD_JOBS=1 cargo clippy --locked --manifest-path "src-tauri/Cargo.toml" --all-targets --all-features -- -D warnings && CARGO_BUILD_JOBS=1 cargo check --locked --manifest-path "src-tauri/Cargo.toml" --features desktop && npx tsc --noEmit && npm run build -- --outDir "/tmp/opencode/sellable-product-onboarding-final-verify/vite-dist" --emptyOutDir
exit 0
rustfmt, Clippy with warnings denied, desktop Cargo check, TypeScript no-emit, and Vite production build passed.
Output SHA-256: 6e9978ee128ae6a9281fea6fe20aad9f75818a5fe8698196ea28c7f8d4207c8c
```

#### Bounded desktop startup

```text
XDG_DATA_HOME="/tmp/opencode/sellable-product-onboarding-final-verify/appdata" timeout 30s npm run tauri:dev -- --config '{"build":{"devUrl":"http://localhost:41731","beforeDevCommand":"npm run dev -- --host 127.0.0.1 --port 41731 --strictPort"}}'
exit 124 (expected timeout bound)
Vite ready in 266 ms on isolated port 41731; Cargo finished; `target/debug/repuestos-autos` reached Running before timeout.
Output SHA-256: 4524c42814949e4efabedebe803424bbed90a8b72ee2d53d309ba3ce38cba69d
```

No repository process remained on port 41731 or at the desktop binary path after timeout. The unrelated pre-existing `/workspace/frontend` Vite process on port 5173 was not touched.

### Task Completion — 10/10

| Task | Status | Independent evidence |
|---|---|---|
| 1.1 implementation audit | ✅ | Current source compared with proposal/spec/design |
| 1.2 migration evidence | ✅ | 7 tests: v0, v1, v4, future refusal, corruption preflights, FK, fact preservation, reopen |
| 1.3 onboarding evidence | ✅ | 5 integration tests plus 2 domain tests |
| 2.1 domain/use-case seam | ✅ | Rust validation and `CreateProductUseCase` transaction ownership |
| 2.2 SQLite adapter/migration | ✅ | Repository, v5 FTS backfill, legacy preservation, immutable movements |
| 3.1 IPC contracts | ✅ | Strict deserialization, registered commands, stable envelopes |
| 3.2 UI/navigation | ✅ | Typed form behavior and Sales ↔ Onboarding screen enum |
| 3.3 search/sales wiring | ✅ | FTS prefix/limit, inactive exclusion, backend-priced checkout |
| 4.1 20k benchmark | ✅ | Release query completed in 8.606828 ms |
| 4.2 exclusions/regressions | ✅ | Fresh negative command-surface snapshots and complete regressions |

### Correctness (Static Evidence)

| Requirement | Status | Notes |
|---|---|---|
| Fields | ✅ Implemented | Typed definitions/values; required/optional and option membership enforced |
| Constraints | ✅ Implemented | Active one-category products; unique SKU; positive centavos/whole stock |
| Opening | ✅ Implemented | One Rust-owned transaction writes product, values, balance, FTS, and movement |
| Result | ✅ Implemented | FTS includes name/SKU/category/configured values; checkout resolves backend price |
| Scope | ✅ Implemented | Only five approved desktop commands are registered; excluded workflows reject |
| Active Product Search and Cart | ✅ Implemented | Active FTS, draft-only state, no-effect rejection, fixed-price persistence |
| Confirm-Sale Scope Exclusions | ✅ Implemented | Existing local cash/QR/mixed flow and idempotent history remain intact |

### Coherence (Design)

| Decision | Followed? | Notes |
|---|---|---|
| Separate category setup and product creation | ✅ Yes | Separate registered commands |
| Use-case transaction with SQLite adapter | ✅ Yes | `CreateProductUseCase` commits; adapter performs transaction-scoped writes |
| General immutable movement vocabulary | ✅ Yes | v5 vocabulary plus update/delete abort triggers |
| Canonical FTS5 prefix search | ✅ Yes | Backfill, sanitized terms, active filter, 20-result limit |
| Local screen enum instead of router | ✅ Yes | `SCREEN`/`screenAfter` drive navigation |

### Migration, FTS, Atomicity, and Checkout Regression

- Migration compatibility passed from new v0, representative v1, and direct v4 stores through v5; future schemas and corrupt legacy/v4 stores were refused without mutation; movement identity, timestamps, sale links, foreign keys, and reopen idempotency were checked.
- FTS5 backfill and canonical-document search passed for name, SKU, category, configured values, prefix matching, inactive exclusion, and the 20-result cap.
- Onboarding atomicity passed for valid persistence and injected late-write rollback, including one immutable positive timestamped `opening_stock` movement.
- Fixed-price checkout/history passed for onboarded products, backend price resolution, cash/QR/mixed payments, idempotent retries, stock movements, and persisted sale-line snapshots.

### Candidate Unchanged Evidence

- Initial candidate hash: `sha256:3a90d7e758715c7669c958e706b52b9bf72cb704e5f63fbf52ff4b8ca4271d6e`.
- Final pre-report candidate hash: `sha256:3a90d7e758715c7669c958e706b52b9bf72cb704e5f63fbf52ff4b8ca4271d6e`.
- `git diff --check` exited 0 before and after runtime execution.
- Tracked diff remained 15 files, 867 insertions, and 30 deletions; untracked implementation and generated/tooling paths remained unchanged.
- Only the verification report is eligible for persistence after validator admission; source, tasks, and apply-progress were not modified.

### Findings

**CRITICAL**: None.

**WARNING**

1. The bounded desktop harness proves startup, not a rendered category → product → sales click-through; no desktop interaction runner is configured.
2. Task 1.2 says v0–v4 upgrades, while direct starting-version fixtures are v0, v1, and v4; task 1.3 says rollback at each write, while the integration suite injects one late-write failure. Transactional behavior passes, but the task wording is broader than direct fixture breadth.
3. The implementation remains a broad dirty worktree above the 400-line review budget. Stacked-to-main delivery must preserve the planned slices and exclude unrelated generated/tooling trees.

**SUGGESTION**

1. Add automated desktop interaction coverage when a stable WebDriver/WebKit harness is available.
2. Rehearse backup/restore rollback against a representative production database before deployment.

### Diagnosis

The prior failure was an evidence gap, not an observed production defect. The remediation tests now provide executable proof for excluded command rejection and draft removal/discard persistence invariance, closing the prior `11/13` result to `13/13`.

### Harness Disposition

`reused` — existing Cargo, Node, Tauri `MockRuntime`, SQLite integration, release benchmark, and bounded desktop harnesses were reused. The desktop launch used isolated port 41731 to avoid the unrelated process on port 5173.

### Cleanup Evidence

Runtime/build outputs are isolated under `/tmp/opencode/sellable-product-onboarding-final-verify`; `git diff --check` passes; candidate hashes match; and no launched desktop or port-41731 process remains. No unrelated data was deleted.

### Process Evidence

No subagent, commit, push, PR, source edit, task update, apply-progress update, attempt acquire, or attempt settle occurred. Parent token `sha256:66dad98ee80ed5c126a828a59558b3a22626eb10a3516e5ad89b0eff062db595` was neither acquired nor settled. Prior failed evidence `sha256:ee1d5057148b6d022c312346e78fa665de646358530d87eeeffac6a85dc570c9` and remediation settlement evidence `sha256:7bf2a607fc71ea7ff028bdf7cdd3f8c4107d8c75ce175d58e8af2de5335edc29` were bound into this fresh evidence.

### Canonical Verification Evidence

The exact preimage below hashes to `sha256:b81aa20ecc0c7e1a47ec250775e5886f762e4df522b82f45c70e43406adeac8c`:

```text
change=sellable-product-onboarding
candidate_state=unchanged
candidate_state_hash=sha256:3a90d7e758715c7669c958e706b52b9bf72cb704e5f63fbf52ff4b8ca4271d6e
prior_failed_evidence_revision=sha256:ee1d5057148b6d022c312346e78fa665de646358530d87eeeffac6a85dc570c9
remediation_settlement_evidence=sha256:7bf2a607fc71ea7ff028bdf7cdd3f8c4107d8c75ce175d58e8af2de5335edc29
test_output_hash=sha256:f97d135fddf77f200e7e01c28b92281cbacc7916f96924795ac5bcef819d02c4
build_output_hash=sha256:6e9978ee128ae6a9281fea6fe20aad9f75818a5fe8698196ea28c7f8d4207c8c
startup_output_hash=sha256:4524c42814949e4efabedebe803424bbed90a8b72ee2d53d309ba3ce38cba69d
focused_remediation_test_output_hash=sha256:fc725f19536f97d4eb5cb88b3a2996cdbfbe906d27760dfdf4ea2d2b4b2d0b98
git_diff_check_exit=0
requirements_complete=7/7
scenarios_complete=13/13
critical_findings=0
protected_sales_ui=formatting-only
artifact_writes=verify-report-only
attempt_token_action=neither-acquired-nor-settled
parent_token=sha256:66dad98ee80ed5c126a828a59558b3a22626eb10a3516e5ad89b0eff062db595
```

### Verdict

**PASS WITH WARNINGS**

All 13 scenarios have passing runtime coverage and all build/static gates pass. Remaining warnings concern harness breadth, task wording, rollout rehearsal, and review slicing rather than requirement noncompliance.

### Next Recommended

Archive after the parent records this final evidence against the active attempt and completes the normal archive gate.
