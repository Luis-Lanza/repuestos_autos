# Apply Progress: Catalog Maintenance and Pricing

**Completed work units**: slice-1-catalog-foundation (stacked-to-main PR1); slice-2-catalog-sqlite-use-cases (stacked-to-main PR2); slice-3-catalog-lifecycle-ipc-ui (stacked-to-main PR3); slice-4-catalog-metadata-intents (stacked-to-main PR4); slice-5-catalog-sqlite-forward-migration (stacked-to-main PR5); slice-6-catalog-typed-ipc (stacked-to-main PR6)
**Mode**: Standard, behavior-first RED → GREEN → focused formatting refactor.

## Completed Tasks
- [x] 1.1 Migration RED coverage
- [x] 1.2 v7 migration and preflight
- [x] 1.3 Domain RED coverage
- [x] 1.4 Domain transition foundation
- [x] 2.1 Application RED coverage
- [x] 2.2 Immediate-transaction maintenance use case
- [x] 2.3 SQLite RED coverage
- [x] 2.4 Guarded SQLite lifecycle adapter
- [x] 3.1 Command/adapter RED coverage
- [x] 3.2 Registered allowlisted lifecycle maintenance IPC
- [x] 3.3 Catalog maintenance reducer RED coverage
- [x] 3.4 Lifecycle maintenance UI and navigation
- [x] 4.1 Metadata intent RED coverage
- [x] 4.2 Metadata intent/application seam
- [x] 5.1 SQLite/migration RED coverage
- [x] 5.2 Guarded SQLite metadata persistence and v8 migration
- [x] 6.1 Typed edit command-seam RED coverage
- [x] 6.2 Typed Rust/Tauri edit IPC and registration

## TDD Cycle Evidence
| Task | RED | GREEN | Refactor |
|---|---|---|---|
| 1.1 | `sqlite_migrations` failed: 3 failures | 12 passed | Existing migration tests updated for v7 |
| 1.2 | Same v6→v7 failures | 12 passed | `cargo fmt` |
| 1.3 | Missing public domain symbols | 4 passed | Typed-value fixture made explicit |
| 1.4 | Missing transition symbols | 4 passed | Named `TransitionPlan` |
| 2.1 | Missing maintenance use-case symbols | 1 passed | Used the production SQLite seam for stable outcomes |
| 2.2 | Same unresolved application imports | 1 passed | Immediate transaction owns load, transition, apply, and commit |
| 2.3 | Missing exported SQLite maintenance adapter | 3 passed | Added explicit search visibility coverage |
| 2.4 | Same adapter import failure | 3 passed | Kept lifecycle, FTS refresh, and audit in one adapter |

## Work Unit Evidence
| Evidence | Exact result |
|---|---|
| Focused tests | `cargo test --manifest-path src-tauri/Cargo.toml --test sqlite_migrations` — 12 passed; `--test catalog_maintenance_domain` — 4 passed |
| Runtime harness | N/A — this slice is an in-process SQLite migration/domain seam; IPC/UI runtime starts in PR3. |
| Rollback boundary | Revert `0007_catalog_maintenance.sql`, `sqlite/mod.rs`, `domain/catalog.rs`, and their two focused tests; prior v6 data is restored from backup, never down-migrated. |

## Work Unit Evidence: PR2
| Evidence | Exact result |
|---|---|
| Focused tests | `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_maintenance_application --test catalog_maintenance_sqlite --test catalog_search` — 9 passed |
| Runtime harness | N/A — this is an in-process repository/application and SQLite integration seam; IPC/UI runtime starts in PR3. |
| Rollback boundary | Revert the catalog application/repository adapter files and the two PR2 tests; no IPC/UI, sales, or inventory behavior is removed. |

**Checks**: `cargo fmt --check`, locked `cargo check`, and `clippy -D clippy::perf` passed; pre-existing non-perf Clippy warnings remain outside this slice.
**Native attempt**: complete; evidence `sha256:28a6ad7d045f118cab2eed9234660aa6389cf00ab7da8b79cde4203b30953f5f`; settlement `catalog-maintenance-slice1-settle-20260828-4`.
**PR2 checks**: `cargo fmt --check`, locked `cargo check`, `clippy -D clippy::perf`, and `--test sqlite_migrations` (12 passed) passed; pre-existing non-perf Clippy warnings remain outside this slice.
**PR2 authored source/test delta**: 399 additions + deletions, measured before settlement; within the 400-line budget.
**Native attempt**: complete; request `catalog-maintenance-slice2-20260828-1`; evidence `sha256:8caa84e0116f347b20929b9490cb46fc2f58a5b39b08fced050db90d705951ac`; settlement `catalog-maintenance-slice2-settle-20260828-1`.

## Work Unit Evidence: PR3 Lifecycle IPC and UI
| Evidence | Exact result |
|---|---|
| Focused tests | `npx tsx --test src/commands/catalog.test.ts src/ui/catalog/catalog-maintenance-flow.test.ts` — 4 passed after blocker remediation; `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_maintenance_commands` — 2 passed; prior desktop command-surface evidence remains 3 passed. |
| Checks | `npx tsc --noEmit`, `npx vite build` with an isolated temporary output directory and redirected output, prior `cargo fmt --check`, and prior `cargo check --locked` passed. |
| Runtime harness | N/A — `npm run tauri:dev` can write the protected, pre-existing untracked `dist/` directory; it cannot be cleaned safely without risking unrelated local artifacts. |
| Rollback boundary | Revert the lifecycle catalog command, UI catalog directory, app navigation, and catalog command/reducer tests; no metadata-editing, sales, or inventory behavior is removed. |

**PR3 authored source/test delta**: 369 additions + deletions, excluding OpenSpec evidence; within the 400-line cap.
**PR3 authorized boundary**: This completed PR3 covers lifecycle IPC/UI only. Metadata editing (SKU/name/typed values/price) is an explicit unchecked PR4 vertical slice; stale-cart and inventory behavior is now PR5. No sales or inventory work was included.
**PR3 blocker remediation**: Successful catalog records are explicitly projected so backend-only fields do not cross the TypeScript adapter; stale conflicts now render a keyboard-reachable reload control. No metadata task was completed.
**PR3 independent verification**: Passed with evidence `sha256:e55a75b50caf17b0984082208fd10d91f2110b7895a36541af310a2ac23218e0`; TypeScript 4/4, Rust IPC 2/2, desktop command surface 1/1, typecheck, redirected Vite build, rustfmt, locked cargo check, and Clippy performance checks passed.
**Native attempt**: Finished as failed because its immutable objective still included metadata and price editing. The failed outcome records that superseded scope mismatch, not a candidate defect; the maintainer explicitly authorized the five-PR boundary before final verification.

## Work Unit Evidence: PR4 Metadata Intents and Validation
| Evidence | Exact result |
|---|---|
| Focused tests | `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_maintenance_domain --test catalog_maintenance_application --test catalog_maintenance_sqlite --test catalog_maintenance_commands --test confirm_sale_use_case` — 31 passed. |
| Checks | `cargo fmt --check`, locked `cargo check`, and `cargo clippy --all-targets --locked -- -D clippy::perf` passed; existing non-perf warnings remain. |
| Runtime harness | N/A — PR4 is an in-process domain/application seam. `tauri:dev` may write protected `dist/`, and typed IPC begins in PR5. |
| Rollback boundary | Revert metadata intent/domain validation, metadata repository seam, and the two focused domain/application tests; no SQLite metadata writes, typed IPC, frontend, sales, or inventory behavior is removed. |

**PR4 authored source/test delta**: 399 additions + deletions, excluding OpenSpec evidence and protected pre-existing Sales diffs; within the 400-line cap.
**Historical seven-PR boundary**: superseded by maintainer-authorized eight-PR chain after PR5 verification separated the former SQLite/IPC scope.
**Eight-PR boundary**: PR5 is SQLite guarded persistence and v7→v8 forward migration; PR6 is typed Rust/Tauri edit IPC; PR7 is TypeScript/accessibility edit UI; PR8 is sales, stale-cart, and inventory behavior.
**PR4 boundary verification**: SQLite repository, Tauri command, and all catalog frontend/adapter files matched `be0f1e2`; protected Sales files matched `90c499108e47bfec8d64c6f681e136c14d34b248` at PR4 completion.

## Work Unit Evidence: PR5 SQLite Persistence and Forward Migration
| Evidence | Exact result |
|---|---|
| Focused/regression tests | Independent verification: 47/47 passed across migration/SQLite, domain/application, search, and historical-price regressions. |
| Checks | `cargo fmt --check`, locked `cargo check`, and `cargo clippy --all-targets --locked -- -D clippy::perf` passed. |
| Runtime harness | N/A — PR5 is an in-process migration/repository slice; typed Tauri IPC is PR6. |
| Migration compatibility | `0007_catalog_maintenance.sql` remains unchanged. Forward v8 migration preflights normalized product-name duplicates; valid v7 databases preserve facts/attributes, enforce uniqueness, and reopen idempotently; duplicate v7 data fails before schema/version advancement. |
| Rollback boundary | Revert PR5 SQLite repository, registry, v8 migration, and the two focused Rust test files; no IPC, UI, sales, or inventory behavior is removed. |

**PR5 authored source/test delta**: five paths, 381 additions + 19 deletions = 400; OpenSpec evidence excluded from native accounting.
**PR5 functional coverage**: guarded expected revisions; atomic names/SKU/price/typed-value replacement; audit and FTS refresh including category rename; historical sale prices unchanged; Sales files preserved at `90c499108e47bfec8d64c6f681e136c14d34b248`.
**PR5 independent PASS evidence**: `sha256:6a0682008fbb561b0193347533ad54ef9dff22cf0c39a6ec3c2b5edc95c4ec0f`.
**Native attempt outcome**: failed only because its immutable objective still included typed IPC, now assigned to PR6; the PR5 candidate itself passed independent verification.

## Work Unit Evidence: PR6 Typed Metadata Read/Edit IPC
| Evidence | Exact result |
|---|---|
| Focused tests | Catalog command tests 3/3; application/SQLite/migration regressions 25/25. |
| Command harness | Desktop MockRuntime command surface 4/4, including registered detail-read and edit paths. |
| Checks | `cargo fmt --check`, locked desktop `cargo check`, and `cargo clippy --features desktop --all-targets --locked -- -D clippy::perf` passed. |
| Runtime harness | N/A — no desktop process launched; safe MockRuntime exercised both command paths. |
| Rollback boundary | Revert the four PR6 application/command/registration/test paths; PR5 persistence and all frontend, sales, and inventory behavior remain intact. |

**PR6 authored source/test delta**: four paths, 376 additions + 3 deletions = 379; OpenSpec evidence excluded from native accounting.
**PR6 behavior**: typed metadata detail-read and edit DTOs, nested unknown-field denial, explicit response projection, stable opaque outcomes, category-specific definitions/current values, and production/test registration.
**PR6 independent PASS evidence**: `sha256:194c7ec59da4f56c489d2994d18d6e5843269f46cb432914b4c192966e77f1f2`.
**Native attempt**: passed and remediated PR5 evidence `sha256:6a0682008fbb561b0193347533ad54ef9dff22cf0c39a6ec3c2b5edc95c4ec0f`.

## Work Unit Evidence: PR7 Accessible Metadata Editing
| Evidence | Exact result |
|---|---|
| Focused tests | Catalog adapter/flow behavior 9/9, including detail retry identity, validation, pending controls, success refresh, and conflict reload. |
| Checks | `npx tsc --noEmit` and redirected Vite production build passed; whitespace validation passed. |
| Accessibility | Native labeled controls, field-specific validation associations, form-level alerts, `aria-busy`, keyboard-reachable retry/reload, archived-state text, and centavos guidance verified. |
| Runtime harness | N/A — `tauri:dev` was not run because protected pre-existing `dist/` must remain untouched; reducer, callback, SSR, typecheck, and production-build seams were exercised. |
| Rollback boundary | Revert the five PR7 TypeScript adapter/flow/screen/test paths; Rust IPC, persistence, sales, and inventory remain intact. |

**PR7 authored source/test delta**: five paths, 159 additions + 15 deletions = 174; OpenSpec evidence excluded from native accounting.
**PR7 independent PASS evidence**: `sha256:b419573d83a1688def0244fe2ec48b57931bee3a904c7cb366b3dc5f182ed90e`.
**Native attempt**: passed after remediating detail retry identity, success-announcement lifetime, and field-scoped validation semantics.

## Work Unit Evidence: PR8 Sales and Inventory Integration
| Evidence | Exact result |
|---|---|
| Focused tests | Rust 43/43; focused frontend 11/11; full frontend 37/37; MockRuntime 4/4. |
| Broader regressions | Rust suite excluding the proven base-only backup fixture failure passed 119/119. |
| Checks | Rustfmt, locked desktop cargo check, Clippy performance gate, TypeScript typecheck, redirected Vite build, and whitespace validation passed. |
| Runtime harness | N/A — `tauri:dev` was not run because protected pre-existing `dist/` cannot be safely mutated; in-process SQLite/application, MockRuntime, reducer/callback, and production-build seams were exercised. |
| Rollback boundary | Revert the 20 PR8 source/test behavior paths while preserving the two pre-existing Sales formatting hunks and all catalog PR1–PR7 work. |

**PR8 authored behavior delta**: 346 additions + 40 deletions = 386 after excluding the protected 7-addition/2-deletion Sales formatting baseline.
**PR8 behavior**: exact stale-price/revision acknowledgements, second-change rejection, immutable confirmed line prices, typed IPC/frontend outcomes, and active category/product gates for sales, inventory operations, search, and alerts.
**PR8 independent PASS evidence**: `sha256:9b229e98c48db78d7d81d7af9cea135136582c586409395a650916f1f31c9405`.
**Known baseline debt**: `backup_restore::creates_a_consistent_snapshot_before_releasing_the_live_database_mutex` expects schema 8 from a fixture that migrates only through v6; the identical 6-versus-8 failure is reproduced on base `585617b`.
**Native attempt**: passed at 386 behavior lines; protected Sales formatting remained outside the candidate authority.
