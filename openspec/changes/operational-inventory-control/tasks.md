# Tasks: Operational Inventory Control
## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 1,804–1,954; PR 0 = 404; slices 240–360 |
| Review budget lines | 400 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Proposed slices | PR 0 planning → PR 1 migration → PR 2 domain → PR 3 SQLite → PR 4 IPC → PR 5 UI/verification |
| Delivery strategy | ask-on-risk |
| Chain strategy | stacked-to-main |
| Size exception | PR 0 only: 404 planning; none implementation |
| Implementation boundary | One slice/apply run |

Decision needed before apply: No
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High
### Suggested Work Units

| Unit | Start → finish | PR / estimate | Test | Runtime | Rollback |
|------|----------------|---------------|------|---------|----------|
| 0 | five OpenSpec files → planning PR | PR 0 (~404; approved) | artifact review | N/A—planning | planning artifacts |
| 1 | v5 DB → v6/preflight | PR 1 (~330) | `cargo test --manifest-path src-tauri/Cargo.toml --test sqlite_migrations` | N/A—fixture | migration/migrator/tests/config |
| 2 | none → domain/use cases | PR 2 (~270) | `cargo test --manifest-path src-tauri/Cargo.toml --test inventory_domain --test inventory_application` | N/A—library | domain/application/tests |
| 3 | repository seam → SQLite authority | PR 3 (~360) | `cargo test --manifest-path src-tauri/Cargo.toml --test inventory_sqlite` | N/A—SQLite | adapter/index/tests |
| 4 | unregistered IPC → contracts/reducer | PR 4 (~240) | `cargo test --manifest-path src-tauri/Cargo.toml --test inventory_commands && npx tsx --test src/commands/inventory.test.ts src/ui/inventory/inventory-flow.test.ts` | N/A—seam | commands/contracts/flow/tests |
| 5 | wired workflow → UI/evidence | PR 5 (~300) | `npm test && npm run build && cargo test --manifest-path src-tauri/Cargo.toml` | `npm run tauri:dev`: entry/adjust/retry/restart/alerts | UI/app wiring/evidence/docs |
PR 0 is planning, not `sdd-apply`.

## Phase 1: Capability and Migration Foundation

- [x] 1.1 (~40) Refresh runners, budget, init metadata in `openspec/config.yaml`; preserve config.
- [x] 1.2 (~120) RED tests in `src-tauri/tests/sqlite_migrations.rs`: v5 reopen, invalid signs/links/reasons, composite links, rollback/version.
- [x] 1.3 (~170) Create migration `0006_operational_inventory_control.sql`; update `sqlite/mod.rs` preflight/copy/v6/triggers.

## Phase 2: Inventory Domain and Application

- [ ] 2.1 (~110) Tests: quantities, reasons, classification, overflow, no-op, stale projections, results.
- [ ] 2.2 (~160) Create `domain/inventory.rs` and `application/inventory/{mod.rs,repository.rs}` confirmation/alert use cases.

## Phase 3: SQLite Authority

- [ ] 3.1 (~150) Adapter tests: atomicity, retries, restart, intervening writes, alerts, constraints, `EXPLAIN`, <100 ms reads.
- [ ] 3.2 (~210) Create `infrastructure/sqlite/inventory_repository.rs`; update indexes/tests for `BEGIN IMMEDIATE`, checked arithmetic, guarded updates, alerts.

## Phase 4: IPC and Frontend Workflow

- [ ] 4.1 (~120) Command-seam tests; create `commands/inventory.rs`, register commands in `src-tauri/src/lib.rs`, keep errors opaque.
- [ ] 4.2 (~120) TypeScript contract/reducer tests; create `src/commands/inventory.ts` and `src/ui/inventory/inventory-flow.ts` with allowlisting/UUID retention.
- [ ] 4.3 (~180) Create `src/ui/inventory/inventory-screen.ts`; update `src/ui/app.ts` for Catalog search, forms, stale notice, alerts/navigation.

## Phase 5: Verification and Closure

- [ ] 5.1 (~80) Evidence for restart, concurrency, migration rejection/preservation, sales alerts, exclusions; threat matrix N/A.
- [ ] 5.2 (~40) Run focused suites, `npm test`, `npm run build`, Rust suites; update checklist/notes; preserve Sales/cart pricing.
