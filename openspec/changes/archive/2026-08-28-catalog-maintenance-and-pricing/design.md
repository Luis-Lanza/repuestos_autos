# Design: Catalog Maintenance and Pricing

## Technical Approach

Add a deep Rust catalog-maintenance module behind an intent-based interface. The application use case owns an immediate SQLite transaction; domain transitions produce a persistence plan; the SQLite adapter guards revisions, replaces typed values, refreshes FTS, and appends audit atomically. Tauri exposes stable DTOs; TypeScript adapters and pure reducers isolate UI state. No delta spec exists yet.

## Architecture Decisions

| Decision | Alternatives / tradeoff | Choice and rationale |
|---|---|---|
| Maintenance seam | CRUD methods are shallow and scatter invariants. | `MaintainCatalogUseCase::execute(intent)` loads one aggregate and applies one domain transition. A small intent/result interface concentrates lifecycle, revision, audit, and validation rules. |
| Concurrency | Last-write-wins is simpler; locks held across UI edits are unsafe. | Use `revision` plus guarded `UPDATE ... WHERE id=? AND revision=?` inside `BEGIN IMMEDIATE`; zero rows maps to opaque `stale_catalog_record`. This survives future multi-connection access despite today’s process mutex. |
| Audit and FTS | Triggers reduce call-site work but hide intent and cannot express useful before/after evidence cleanly. | The adapter explicitly writes immutable before/after JSON audit rows and deletes/reinserts affected FTS rows in the same transaction. Any failure rolls back all three. |
| Cart repricing | Silently use the latest price surprises staff; freezing draft prices undercharges. | Draft lines capture price and revision. Confirmation remains authoritative and returns current line facts when stale; only an acknowledgement tied to that exact revision permits retry. |
| Lifecycle | Cascading category state is convenient but destroys independent product intent. | Category archive is blocked by active products; category changes never mutate product state. Product reactivation requires active category and valid typed values. Selling, alerts, and inventory require both active. |

## Data Flow

```text
UI reducer -> TS command adapter -> Tauri DTO -> MaintainCatalogUseCase
                                              -> domain transition
                                              -> SQLite transaction
                                                 row + values + FTS + audit

draft snapshot -> ConfirmSaleUseCase -> compare authoritative price/revision
                  | stale facts -> reducer acknowledgement -> same request_id retry
                  ` confirmed -> immutable sale-line snapshots
```

## File Changes

| File | Action | Description |
|---|---|---|
| `src-tauri/src/infrastructure/sqlite/migrations/0007_catalog_maintenance.sql` | Create | Add category activity/revisions, product revisions, immutable `catalog_audit`, constraints, and indexes; backfill without changing IDs or operational facts. |
| `src-tauri/src/domain/catalog.rs` | Modify | Add lifecycle states, maintenance intents, snapshots, transition plans, and validation. |
| `src-tauri/src/application/catalog/{mod.rs,repository.rs}` | Modify | Add use case and narrow load/apply repository contracts. |
| `src-tauri/src/infrastructure/sqlite/{mod.rs,catalog_repository.rs,sale_repository.rs,inventory_repository.rs}` | Modify | Run v7; implement guarded writes, FTS/audit transaction, and dual lifecycle filtering. |
| `src-tauri/src/{commands/catalog.rs,commands/confirm_sale.rs,lib.rs}` | Modify | Add registered stable maintenance DTOs and stale-price outcome mapping. |
| `src/commands/{catalog.ts,confirm-sale.ts}` | Modify | Validate and adapt allowlisted IPC payloads/results. |
| `src/ui/catalog/{catalog-maintenance-flow.ts,catalog-maintenance-screen.ts}` | Create | Pure reducer and rendering boundary for list/detail/edit/lifecycle flows. |
| `src/ui/{app.ts,sales/sale-flow.ts,sales/sale-screen.ts}` | Modify | Navigation plus captured-price stale/acknowledgement state. |
| `src-tauri/tests/{catalog_maintenance_domain.rs,catalog_maintenance_application.rs,catalog_maintenance_sqlite.rs,sqlite_migrations.rs,confirm_sale_application.rs}` | Create/Modify | RED seams and integration coverage. |
| `src/{commands/catalog.test.ts,commands/confirm-sale.test.ts,ui/catalog/catalog-maintenance-flow.test.ts,ui/sales/sale-flow.test.ts}` | Create/Modify | Adapter and reducer contracts. |

## Interfaces / Contracts

```rust
trait CatalogMaintenanceRepository {
    fn load(&self, tx: &Transaction<'_>, target: CatalogTarget) -> Result<CatalogSnapshot, MaintenanceError>;
    fn apply(&self, tx: &Transaction<'_>, plan: TransitionPlan) -> Result<CatalogSnapshot, MaintenanceError>;
}
```

IPC results remain tagged and opaque: `success`, `validation_error`, `lifecycle_blocked`, `stale_catalog_record`, or `persistence_failure`. Stale sale details include product ID, current price, and current revision; SQL details never cross IPC.

## Testing Strategy

| Layer | What to test | Approach |
|---|---|---|
| Domain/application | Every intent, independent lifecycle, stale revisions/prices, rollback ordering | Pure transition tests and repository doubles before implementation. |
| SQLite | v6→v7 preservation/reopen, guarded races, audit immutability, injected FTS/audit failure, 20,000-product rename, index plans | File/in-memory databases, two connections, failure triggers, `EXPLAIN QUERY PLAN`. |
| IPC/UI | DTO allowlists, stable errors, stale acknowledgement and second-change rejection | Tauri command seam plus TypeScript adapter/reducer tests. |

## Threat Matrix

Tauri IPC changes, so applicability was reviewed; it introduces no executable, shell, VCS, or PR boundary.

| Boundary | Applicability | Design response / RED tests |
|---|---|---|
| Documentation-like paths | N/A: no path classification or execution | None |
| Git repository selection | N/A: no Git invocation | None |
| Commit state | N/A: no commit automation | None |
| Push state | N/A: no push automation | None |
| PR commands | N/A: no PR automation | None |

## Migration / Rollout

Preflight v6 shape and foreign keys, back up, apply v7 transactionally, set `user_version=7`, reopen, then verify constraints, FTS parity, audit triggers, and preserved sale/stock/movement snapshots. Startup remains unavailable on failure. Rollback restores the backup with the prior app; never down-migrate or delete audit. Split delivery into autonomous schema/domain, SQLite/application, IPC/frontend, and sales-integration slices, each under 400 changed lines; ask before stacked-to-main if forecast exceeds budget.

## Open Questions

None.
