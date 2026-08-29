# Apply Progress: Sales History

## Status

- Change: `sales-history`
- Mode: Strict TDD
- Completed tasks: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 3.1, 3.2, 3.3
- Remaining tasks: None
- Delivery: stacked PR slice 3 targeting `main`; PR1 and PR2 are merged.

## TDD Cycle Evidence

| Task | Test File | Layer | Safety Net | RED | GREEN | TRIANGULATE | REFACTOR |
|---|---|---|---|---|---|---|---|
| 1.1 | `tests/sales_history.rs`, `tests/sqlite_migrations.rs` | SQLite integration | `cargo test --manifest-path src-tauri/Cargo.toml sales_history` passed before corrective changes. | Original index test failed with missing `sales_confirmed_history_idx`; corrective v8 fixture was added before production behavior changes. | Focused command passed 2/2; isolated v8→v9 migration passed. | New database chain, v8→v9 index/version, 102 matching rows, pending/out-of-range exclusions, and repeated reads. | Additive v9 migration retained; focused suite passed after formatting. |
| 1.2 | `tests/sales_history.rs` | SQLite integration | N/A (new reader/module in original cycle). | Query behavior tests preceded reader implementation in the original cycle. | Parameterized confirmed-only list/detail reader passed focused tests. | Tie ordering, fixed 101 over-fetch, payment method flags, and read immutability are exercised. | SQL and fetch constants remain private. |
| 1.3 | `tests/sales_history.rs` | Application + SQLite integration | N/A (new original test file). | Original range/snapshot tests were added before final reader behavior. | Invalid range, nullable snapshots, quantity, and monetary corruption map to typed errors. | Offset normalization, cash/QR ordering, zero/negative persisted facts, and unavailable snapshots use distinct cases. | Validation remains centralized in private conversion helpers. |
| 1.4 | `tests/sales_history.rs` | Application + SQLite integration | Focused test command passed before corrective encapsulation refactor. | Original model/interface tests preceded final page assembly behavior. | Page fields are private; only `from_overfetch` constructs pages and accessors expose read-only views. | Exactly 100 and 101 matching rows prove cap/`has_more`; no external limit exists. | Narrow interfaces and private construction preserved after `cargo fmt`. |
| 1.5 | `tests/sales_history.rs` | Approval/integration | Focused test command passed 2/2 before refactoring existing history files. | Approval test selected before page encapsulation refactor; it verifies page count, `has_more`, ordering, and repeated-read equality. | Focused command passed 2/2 after refactor. | Isolated v8 migration plus invalid quantity and negative money cases remain green. | `cargo fmt --manifest-path src-tauri/Cargo.toml` then focused and full Rust suites passed. |
| 2.1 | `tests/sales_history_commands.rs`, `src/commands/sales-history.test.ts`, `src-tauri/src/lib.rs` | Rust command + TypeScript adapter + Tauri mock IPC | Planned focused Rust command had 0 matching tests and frontend suite had 37 passing tests before RED. | Missing Rust command module and TypeScript adapter caused unresolved-import failures. | New command tests pass tagged list/detail success, `invalid_range`, `sale_not_found`, and opaque `persistence_failure`; TypeScript tests pass local range conversion and payload projection. | DST boundaries use independently constructed `-05:00` and `-04:00` midnights; mock IPC proves registration and no persistence mutation. | `cargo fmt`; response serialization is derived from existing read models to avoid duplicate centavo/snapshot mapping. |
| 2.2 | `tests/sales_history_commands.rs`, `src/commands/sales-history.test.ts` | Command/adapter integration | 2 Rust command tests and 40 frontend tests passed after RED corrections. | Tests referenced the absent command/adapter interfaces. | Added read-only commands, registered both Tauri handlers, and created typed TypeScript list/detail invokes. | Invalid range, unknown ID, storage failure, tagged success, nullable snapshots, cash payloads, and non-zero-offset DST conversion use distinct cases. | Shared serializable read models retain integer centavos, nullable snapshots, and tagged payments. |
| 2.3 | `src-tauri/src/lib.rs`, command/adapter tests | Read-only approval | Focused suites passed before final boundary inspection. | Registration test selected before final `with_read` review. | Tauri mock invoke test verifies both handlers are registered and leaves sales, payments, lines, stock, and movements unchanged. | Full Rust (124) and frontend (40) suites plus TypeScript check remain green. | Handlers only call `DatabaseState::with_read`; no UI/navigation path or write transaction was added. |
| 3.1 | `src/ui/sales/history-flow.test.ts` | Reducer + static React screen | `npm test -- --run 'src/ui/sales/*history*.test.ts'` passed 40 pre-existing frontend tests; the script expands the repository-wide glob. | Test first failed with `ERR_MODULE_NOT_FOUND` for the absent `history-flow.ts`. | The same command passed 44 tests after the flow/screen implementation. | Covers list and detail loading, selection/back, empty/error, 100-result narrowing notice, Bs centavos, unavailable snapshots, cash/QR payment facts, and app navigation. | Shared `formatBs` keeps all historical monetary display centavo-based; final focused and type checks passed. |
| 3.2 | `src/ui/sales/history-flow.ts`, `src/ui/sales/history-screen.ts`, `src/ui/app.ts` | React UI + Tauri adapter integration | 44 frontend tests passed before final verification. | The UI test referenced absent reducer and screen modules before implementation. | Reducer-driven screen calls only `salesHistoryCommands.list/detail`; list/detail failure remains visible and never inserts fallback records. | Tests use persisted nullable snapshots, cash and QR variants, bounded `has_more`, and return to the unchanged list state. | The history route is isolated from sales confirmation and uses the existing typed read-only adapter. |
| 3.3 | `src/ui/sales/history-flow.test.ts` | Refactor/runtime boundary | Focused and full frontend suites passed after final formatter/state review. | N/A — refactor task; behavior was already specified by the preceding RED tests. | `npm test` passed 44 tests; `npx tsc --noEmit` passed. | Reducer/static React tests are the strongest executable substitute for the unavailable desktop scenario. | `formatBs` centralizes Bs integer-centavo formatting; no historical/catalog fallback path was introduced. |
| 3.3 correction | `src/ui/sales/history-flow.test.ts`, `src/ui/sales/history-screen.ts` | Non-GUI production UI runtime substitute | 44 frontend tests and typecheck passed before the corrective RED test. | The correction test failed because `createSalesHistoryInteraction` was not exported. | The exported production interaction now drives the real typed `createSalesHistoryCommands` adapter through list/detail async calls; focused tests passed 46/46. | A second async error test proves list/detail failures clear data and never fabricate history. | Screen component now delegates its own async production interaction; focused, full, and type checks passed. |

## Test Summary

- **Total sales-history command tests**: 2 Rust command integration tests, 1 Tauri mock invoke seam test, and 3 TypeScript adapter tests.
- **Total tests passing**: 124 broader Rust tests (prior slice); 46 frontend tests; 0 doc tests.
- **Layers used**: Rust SQLite command integration, Tauri mock IPC registration/read-only seam, TypeScript adapter/date conversion, and reducer/static React screen integration plus an injected production command-adapter interaction; desktop GUI could not connect to this environment’s Wayland display.
- **Approval tests (refactoring)**: Tauri mock invokes both handlers then compares persistence snapshots.
- **Pure functions created**: `localDateRangeToUtc` converts separate local browser midnights to RFC3339 UTC bounds.

## Work Unit Evidence

| Evidence | Result |
|---|---|
| Focused Rust test command | `cargo test --manifest-path src-tauri/Cargo.toml sales_history_commands` — passed, 2 tests. |
| Focused frontend test command | `npm test -- --run src/commands/sales-history` — passed, 40 tests; this repository script expands its complete test glob, so `--run` does not narrow it. |
| Runtime harness | `cargo test --manifest-path src-tauri/Cargo.toml --features desktop registers_read_only_sales_history_commands_at_the_tauri_command_seam` — passed, 1 Tauri mock IPC test proving registered invokes and unchanged persistence. No GUI dev server was started. |
| Type check | `npx tsc --noEmit` — passed. |
| Broader commands | `cargo test --manifest-path src-tauri/Cargo.toml` — passed, 124 tests and 0 doc tests; `npm test` — passed, 40 tests. |
| Source diff budget | 383/400 authored source lines (code and tests, excluding SDD artifact bookkeeping). |
| Rollback boundary | Revert the history command module/registration, serializable command-boundary derives, adapter, and command tests; no UI, migration, or persisted business data is changed. |
| Focused PR3 frontend test command | `npm test -- --run 'src/ui/sales/*history*.test.ts'` — passed, 44 tests. The package script expands all `src/**/*.test.*` files, so `--run` does not narrow execution. |
| PR3 runtime harness | `timeout 30s npm run tauri:dev` compiled and launched the desktop binary, but GTK failed before manual interaction with `Gdk-Message: Error 71 (Protocol error) dispatching to Wayland display`. No `tauri`, `vite`, or app process remained afterward. |
| PR3 strongest runtime substitute | Reducer/static React screen tests exercised Sales → History state, list/detail retrieval states, selection/back, persisted nullable snapshot rendering, and read-only adapter usage; passed as part of 44 frontend tests. |
| PR3 source diff budget | 192/400 authored code-and-test additions (186 new UI/test lines plus 6 app additions; excludes SDD bookkeeping). |
| PR3 rollback boundary | Revert `history-flow.ts`, `history-screen.ts`, `history-flow.test.ts`, and the app history route/button. This removes only read-only history UI and leaves sales, catalog, payments, stock, and persisted records untouched. |
| PR3 evidence revision | `sha256:e2a6045ce4fc8cd3c7653e1c3e03b4615168282d4f958283d8981f57e222c07d` over `src/ui/app.ts`, `history-flow.ts`, `history-screen.ts`, and `history-flow.test.ts`. |
| Corrective non-GUI UI runtime harness | `npm test -- --run 'src/ui/sales/*history*.test.ts'` — passed, 46 tests. It runs `createSalesHistoryCommands` → `createSalesHistoryInteraction` → reducer → `HistoryScreen`, proving async list/detail/back, command names/payload identity, loading, persisted `Unavailable` snapshots, and no fallback data. |
| Corrective source diff budget | 256/400 authored code-and-test additions (250 new UI/test lines plus 6 app additions; excludes SDD bookkeeping). |
| Corrective evidence revision | `sha256:df5be0003a215c35bf61450bc79d8ab4e9310df4705d3fcd55016812139081e7` over `src/ui/app.ts`, `history-flow.ts`, `history-screen.ts`, and `history-flow.test.ts`; distinct from failed `sha256:e2a6045ce4fc8cd3c7653e1c3e03b4615168282d4f958283d8981f57e222c07d`. |
| Corrective cleanup | `ps` found no `repuestos-autos` or `vite` process after tests; GTK was not re-run. |

## Implementation Notes

- The isolated v8 fixture applies migrations 0001–0008, sets `user_version = 8`, then proves the production migration creates the v9 index without rewriting business facts.
- Summaries query only confirmed sales with parameter-bound half-open SQLite timestamp bounds, deterministic `confirmed_at DESC, id DESC` ordering, and fixed 101-row over-fetch.
- `SaleHistoryPage` has private fields and a crate-private enforced constructor; adapters receive read access only through accessors.
- Detail reads persisted line snapshots directly; legacy `NULL` SKU/name stays unavailable and never falls back to the catalog.
- Corrupt persisted quantities or monetary values map to `HistoryError::PersistedDataInvalid`.
- Command handlers map `InvalidRange` to `invalid_range`, missing detail to `sale_not_found`, and persistence or invalid persisted data to opaque `persistence_failure`.
- Date conversion uses `new Date(year, month, day)` for each local midnight independently; bare `YYYY-MM-DD` is never parsed as UTC.

## PR3 UI Notes

- The corrective non-GUI harness is a real async path through the production TypeScript command adapter and the screen-owned interaction, not a reducer-only or static-only assertion.
- UI state intentionally renders legacy `null` SKU/product name as `Unavailable`; it never consults catalog data or fabricates a historical line.
- The screen uses the existing typed `salesHistoryCommands` adapter only, so UI browsing cannot reach a write command.
- The desktop runtime compiled but the host Wayland connection failed before manual navigation; the spawned dev/Vite/app processes were confirmed absent after exit.

## Final Verification Remediation

- Failed evidence revision remediated: `sha256:426ffddc9a09b3eb7196e13de11ea29b1cb35dac8d65755eaa6412450f762e7b`.
- RED preserved from the supplied verified failure: `cargo fmt --manifest-path src-tauri/Cargo.toml --check` exited 1 because rustfmt required the `matches!` assertion in `src-tauri/tests/sales_history_commands.rs:43-46` to span multiple lines. It was not rerun before normalization.
- GREEN: `cargo fmt --manifest-path src-tauri/Cargo.toml` exited 0 and changed only that assertion’s layout (4 additions, 1 deletion); no behavior changed.
- Focused check: `cargo fmt --manifest-path src-tauri/Cargo.toml --check` exited 0.
- Focused test: `cargo test --manifest-path src-tauri/Cargo.toml sales_history_commands` exited 0; 2 passed, 0 failed.
- Process cleanup: no `cargo` or `rustfmt` process remained after the commands.
- Rollback boundary: revert only the layout-only assertion change in `src-tauri/tests/sales_history_commands.rs`; no production behavior or unrelated artifact is affected.
- Remediation evidence revision: `sha256:3f3c69f62cae59fe3db2010ce17a6f5aebe24d338056a769d747b6fc7617da1a`, distinct from the failed verification revision. Fresh independent verification remains required.
