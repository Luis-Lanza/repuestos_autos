# Design: Sellable Product Onboarding

## Technical Approach

Add category setup beside a deep `CreateProduct` interface spanning React → Tauri → Rust → SQLite. Rust validates values and owns one transaction writing product, typed values, search document, balance, and immutable opening movement. Existing search and checkout consume the commit. The dirty implementation is comparison evidence only; SDD did not guide it.

## Architecture Decisions

| Option | Tradeoff | Decision |
|---|---|---|
| Separate category setup and product creation vs one wizard command | A category may remain unused, but the product aggregate stays small | Separate commands; product creation is the atomic seam |
| SQL in application vs adapter receiving `Transaction` | Structure keeps SQLite out of orchestration | `CreateProductUseCase` begins/commits; `SqliteCatalogRepository` executes against its transaction |
| Current-only movement types vs v1 vocabulary | Broader checks avoid another table rebuild | Reserve `opening_stock`, `stock_entry`, `sale`, `return`, `adjustment`, `cancellation`; rows are immutable, non-zero, timestamped, with nullable reason/operator/source metadata |
| Leading-wildcard `LIKE` vs indexed search document | FTS requires query normalization and backfill but supports the 20,000-product target | Add FTS5 search documents, backfill legacy/common/typed values, sanitize terms, prefix-match, limit results, and update inside product transaction |
| Router vs local screen state | Router adds no value for two desktop screens | Keep an `App` screen enum with Sales ↔ Onboarding navigation |

## Data Flow

```text
OnboardingScreen → onboarding IPC → CreateProductUseCase
                                      ├─ domain validation
                                      └─ SQLite transaction
                                         ├─ product + typed values
                                         ├─ search document
                                         └─ balance + opening movement
                                                   ↓ commit
Sales search → indexed catalog query → fixed-price checkout
```

## File Changes

| File | Action | Description |
|---|---|---|
| `src-tauri/src/domain/catalog.rs` | Modify | Value types and validation invariants |
| `src-tauri/src/application/catalog/{mod.rs,repository.rs}` | Modify/Create | Use cases, repository interface, stable errors, transaction ownership |
| `src-tauri/src/infrastructure/sqlite/{mod.rs,catalog_repository.rs}` | Modify/Create | SQLite adapter, search, migrations |
| `src-tauri/src/infrastructure/sqlite/migrations/0005_catalog_onboarding_hardening.sql` | Create | Forward-only movement generalization, FTS/backfill, integrity checks |
| `src-tauri/src/commands/onboarding.rs`, `src-tauri/src/lib.rs` | Modify | Strict inputs, stable responses, command registration |
| `src/commands/onboarding.ts`, `src/ui/{app.ts,onboarding/*}` | Modify | Typed client, dynamic form, navigation |
| `src-tauri/tests/{product_onboarding.rs,sqlite_migrations.rs}`, `src/ui/onboarding/*.test.ts` | Modify | Vertical, migration, rollback, performance, and rendered-journey evidence |

## Interfaces / Contracts

`CreateProductInput` carries SKU, name, category ID, positive integer centavos, positive whole opening quantity, and definition/value pairs. Success returns identity, category, price, quantity, and active status. Expected failures return `{ kind: "error", code, message }`; stable codes (`duplicate_sku`, `missing_category`, validation codes, `persistence_failure`) never expose SQL. Reject unknown fields. Search returns committed active products and backend prices.

## Testing Strategy

| Layer | What to Test | Approach |
|---|---|---|
| Unit | Required/optional, number/option, duplicates, movement signs, error mapping | Rust/TypeScript focused tests |
| Integration | Atomic rollback at each write, immediate search and checkout, immutable movements | In-memory SQLite plus injected abort triggers |
| Migration | v0–v4 upgrade, future-version refusal, legacy fact/ID preservation, failed preflight | File-backed fixtures, `foreign_key_check`, reopen idempotency |
| Performance/UI | 20,000-product search under 1 second; category→product→sales journey | Release benchmark on target-class hardware; rendered interaction test |

## Threat Matrix

Local screen navigation triggers review, but introduces no executable/process boundary.

| Boundary | Minimum adversarial cases | Applicability | Design response | Planned RED tests |
|---|---|---|---|---|
| Documentation-like paths | `requirements.txt`, `CMakeLists.txt`, executable MDX, `README.sh` | N/A: no path execution | None | None |
| Git repository selection | `git -C`, relative/absolute paths | N/A: no VCS | None | None |
| Commit state | staged, `commit -a`, empty index | N/A: no VCS | None | None |
| Push state | tracking, first push, refspec | N/A: no VCS | None | None |
| PR commands | `--head`, environment prefix, composition | N/A: no PR automation | None | None |

## Migration / Rollout

Never rewrite applied v4. Transactional v5 preflights columns/foreign keys, preserves movement IDs, timestamps, and sale links, backfills FTS, validates counts and `foreign_key_check`, then advances `user_version`. Back up first. Roll back the application and restore its compatible snapshot; never delete confirmed sales.

## Current Implementation Gaps and Risks

- Catalog application code issues SQL, weakening the intended application/infrastructure seam.
- v4 restricts movements to `opening_stock|sale`, blocking future v1 types, without a v3→v4 corruption preflight.
- Leading-wildcard `LIKE`, duplicate joins, and no limit leave the one-second target unproven.
- Outer IPC failures and unstructured list errors do not share the stable onboarding envelope.
- Tests prove Rust integration and payload shaping, not rendered navigation or representative incompatible-database rollback.
- Persistent `minimum_unit_price` conflicts with catalog-price terminology; renaming needs a compatibility decision.
- Existing dirty changes exceed the 400-line review budget; `ask-on-risk` requires explicit acceptance or retrospective review slices before apply.

## Open Questions

None blocking; naming and review slicing remain follow-ups.
