# Tasks: Sales History

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 650–850 authored lines (Rust, TypeScript, migration, and tests) |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 Rust read slice → PR 2 command/adapters → PR 3 UI/navigation; each targets `main` and merges in order |
| Delivery strategy | ask-on-risk |
| Chain strategy | stacked-to-main |

Decision needed before apply: No
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|------|------|-----------|----------------------|-----------------|-------------------|
| 1 | Persisted read models, v9 index, bounded summary/detail reader | PR 1 | `cargo test --manifest-path src-tauri/Cargo.toml sales_history` | In-memory SQLite migration/read scenario; no desktop runtime needed | Revert history reader, migration, and Rust read tests only |
| 2 | Read-only Tauri commands plus date/IPC adapters | PR 2 | `cargo test --manifest-path src-tauri/Cargo.toml sales_history_commands`; `npm test -- --run src/commands/sales-history` | `npm run tauri:dev` invoke list/detail with a seeded database | Revert command registration and adapter files without touching UI |
| 3 | Sales history UI states, navigation, and end-to-end wiring | PR 3 | `npm test -- --run src/ui/sales/*history*.test.ts` | `npm run tauri:dev`: Sales → History → select → Back, including empty/error/loading | Revert history screen/flow/app wiring and UI tests |

## Phase 1: Read Foundation (PR 1)

- [x] 1.1 **RED:** Add in-memory SQLite tests for v8→v9 migration, partial chronological index, half-open boundaries, deterministic `(confirmed_at DESC, id DESC)` ordering, `LIMIT 101` cap/`has_more`, and repeated-read immutability.
- [x] 1.2 **GREEN:** Add `time`, migration `0009_sales_history_index.sql`, module registration, and parameterized `sale_history_repository.rs` summary/detail queries restricted to confirmed sales.
- [x] 1.3 **RED:** Add reader/application tests for invalid ranges, fixed page construction, persisted snapshot `None` handling, positive whole quantities/money validation, payment variants/order including cash tendered/change, and typed errors.
- [x] 1.4 **GREEN:** Create `application/sales/history.rs` models/interfaces; parse/normalize RFC3339 bounds, validate ranges, enforce private fixed overfetch assembly, counts/method flags, detail ordering, and `persisted_data_invalid` mapping without catalog joins.
- [x] 1.5 **REFACTOR:** Keep SQL/constants private, enforce parameter binding and read-only connection access, then rerun focused Rust tests.

## Phase 2: Command and Adapter Slice (PR 2)

- [x] 2.1 **RED:** Add command tests for `invalid_range`, `sale_not_found`, `persistence_failure`, tagged success/error payloads, registration, and `DatabaseState::with_read` usage; add TypeScript tests for independent local-midnight DST conversion and UTC normalization inputs.
- [x] 2.2 **GREEN:** Implement `src-tauri/src/commands/sales_history.rs`, exports/registration in `commands/mod.rs` and `lib.rs`, plus `src/commands/sales-history.ts` typed invokes and RFC3339 range conversion.
- [x] 2.3 **REFACTOR:** Verify no write transaction/command path is exposed and preserve integer-centavo/nullable snapshot payloads.

## Phase 3: UI Slice (PR 3)

- [x] 3.1 **RED:** Add reducer/screen tests for history navigation, selection/back, loading, empty, error, bounded-page “more” notice, Bs formatting, unavailable SKU/name, payment details, and no fabricated data.
- [x] 3.2 **GREEN:** Implement `history-flow.ts`, `history-screen.ts`, and `app.ts` wiring for list/detail retrieval and all states.
- [x] 3.3 **REFACTOR:** Run focused frontend tests and the Sales runtime scenario; confirm browsing/reloading never mutates persisted business records.
