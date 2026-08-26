## Exploration: Operational inventory control

### Current State
The repository already has the persistence primitives for an Inventory vertical but no Inventory module, command, or screen. Schema version 5 stores a non-negative `stock_balances` read model and immutable `inventory_movements`. Product onboarding atomically creates a positive balance and one `opening_stock` movement; sale confirmation atomically performs guarded decrements and one negative `sale` movement per line. The production database is a local SQLite file reopened at startup, while Tauri serializes access through one mutex-protected connection.

The version-5 movement table reserves `stock_entry`, `return`, `adjustment`, and `cancellation`, but its only type-specific check covers `sale`. Consequently, the database currently permits invalid combinations such as negative stock entries, positive sales without proper links for non-sale types, adjustments without reasons, and opening stock with sale links. Immutability triggers prevent update and delete, but they do not make inserted facts semantically valid. Individual foreign keys also do not prove that a linked sale line belongs to the linked sale and product.

Catalog owns the canonical active-product search. `search_products_command` calls `catalog::search_active_products`, whose FTS query returns product identity, category, current balance, and price. Inventory UI can reuse the existing TypeScript `searchProducts` adapter without duplicating FTS or SQL. At confirmation, Inventory must independently resolve the active product and current balance inside its transaction rather than trust a stale search result.

Catalog layering is inconsistent. Product creation uses a repository trait, but that trait exposes `rusqlite::Transaction`, and `search_active_products`, `list_categories`, `load_category_fields`, and `create_category` issue SQL directly from the application module. These are reusable capabilities, but not yet clean seams. The Inventory slice should not broaden into a Catalog refactor: reuse the public search interface in the UI, and introduce an Inventory-owned application interface with a SQLite adapter for authoritative mutation and alert queries.

The canonical Sales specification requires catalog price on draft lines and a draft total, while `DraftLine` and the current sale screen omit both. This is a focused checkout defect, not a prerequisite for Inventory: Inventory selection only needs the existing search result and has no sale-cart pricing dependency. It must be explicitly out of scope rather than silently repaired here.

Current test capability contradicts the stale SDD init context and `openspec/config.yaml`: `package.json`, `Cargo.toml`, frontend tests, Rust integration tests, desktop command tests, and build scripts now exist. During exploration, `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`, and the desktop-feature Rust suite all passed. The init/config testing claims and 350-line budget are stale; this change uses the supplied 400-line review budget. PRD and architecture documents express useful broader intent but are currently untracked and therefore less durable than the tracked canonical OpenSpec specifications.

#### Inventory semantics and invariants

- A stock entry accepts a positive whole-unit quantity and increases the current balance by that quantity.
- An adjustment should accept an absolute observed physical count, including zero. Inside one transaction, the application reads the authoritative current balance, derives `quantity_delta = counted_quantity - current_quantity`, and records the counted result and required non-blank reason. A no-op count should be rejected because version 5 requires a non-zero movement and the slice promises one movement per confirmed operation.
- A forward-only version-6 migration should rebuild `inventory_movements` and preserve IDs, timestamps, and existing facts. Checks should require: `opening_stock` positive with no sale links; `stock_entry` positive with no sale links; `sale` negative with both links; `return` positive with both links; `adjustment` non-zero with no sale links and a trimmed non-empty reason; and `cancellation` positive with both links and a trimmed non-empty reason. Opening, entry, sale, and return reasons should remain optional under current product rules.
- Operational entry and adjustment movements should require a UUID request ID protected by a unique partial index. `source_reference` should remain available for external references rather than be overloaded as the retry key. Adjustment audit evidence should retain the absolute counted quantity or resulting balance in addition to the derived delta.
- A composite relationship should ensure sale-linked movements reference a sale line belonging to the same sale and product; individual foreign keys are insufficient. Migration preflight and `PRAGMA foreign_key_check` must fail without advancing `user_version` if legacy facts are incompatible.
- `stock_balances.quantity >= 0` remains the final database defense. Entry arithmetic must be checked for integer overflow. Adjustment to an absolute count is naturally non-negative; any future signed-delta path must use a guarded update. The application-owned transaction should use an immediate write transaction or equivalent serialization, then compute, update, append exactly one movement, read back the persisted result, and commit.
- SQLite should own `occurred_at` through its UTC `CURRENT_TIMESTAMP` default. Commands and UI must return the persisted timestamp, never a client clock value. An idempotent retry returns the original movement, timestamp, and resulting balance without recomputation.
- With the current single mutex-protected production connection, in-process writes are serialized. Database constraints, immediate transactions, guarded updates, and request-ID uniqueness are still required for restart retries, tests, and any future second connection.
- Low/out-of-stock status should be derived, not persisted: query active products joined to balances with `quantity <= 1`, classify `1` as low and `0` as out, and derive the counter from the same query semantics. Persisted alert state would duplicate the balance and create drift without adding value while the threshold is globally fixed.
- Restart proof should use a file-backed temporary production database: confirm an operation, close, reopen, retry the same request ID, and assert one movement, one balance effect, the original timestamp/result, and correctly derived alerts.

### Affected Areas
- `src-tauri/src/infrastructure/sqlite/migrations/0005_catalog_onboarding_hardening.sql` — current weak movement checks establish the compatibility baseline; it must remain unchanged while a new forward-only migration strengthens invariants.
- `src-tauri/src/infrastructure/sqlite/mod.rs` — migration ceiling, version-6 execution, schema preflight, and restart behavior.
- `src-tauri/src/application/catalog/mod.rs` — canonical search capability to reuse; also contains direct SQL that should not be copied into Inventory.
- `src-tauri/src/application/catalog/repository.rs` — existing repository seam leaks SQLite transaction types and should not be imitated as the Inventory interface.
- `src-tauri/src/application/inventory/` — proposed deep module for previewing and confirming stock entry/physical-count adjustment plus querying alerts.
- `src-tauri/src/domain/inventory.rs` — proposed whole-unit operation validation and movement semantics without SQLite knowledge.
- `src-tauri/src/infrastructure/sqlite/inventory_repository.rs` — proposed adapter for authoritative product/balance reads, atomic writes, idempotent readback, and derived alert projections.
- `src-tauri/src/commands/` and `src-tauri/src/lib.rs` — typed Tauri request/response adapters and command registration.
- `src/commands/catalog.ts` — existing global search adapter reused unchanged by Inventory UI.
- `src/commands/inventory.ts` — proposed narrow IPC adapter with retained UUID request IDs and safe-integer validation.
- `src/ui/app.ts` and proposed `src/ui/inventory/` — navigation, search-driven operation form, projected balance, confirmation result, and dedicated alert list/counter.
- `src-tauri/tests/sqlite_migrations.rs` — version-5-to-6 preservation, invalid legacy preflight, link/sign/reason checks, and migration reopen coverage.
- Proposed Rust Inventory integration/command tests and TypeScript adapter/state tests — atomicity, retry, concurrency defense, status thresholds, and restart persistence.
- `openspec/config.yaml` — stale project/testing context discovered during exploration; updating it is outside this phase's allowed artifact scope.
- `src/ui/sales/sale-flow.ts` and `src/ui/sales/sale-screen.ts` — unrelated formatting-only local diffs that must remain untouched; the checkout display defect is also excluded.

### Approaches
1. **Signed-delta adjustment** — the operator enters a positive or negative quantity change, which is persisted directly as the movement delta.
   - Pros: Small command shape; directly matches movement storage; convenient when the operator already knows “add/remove N.”
   - Cons: Easy to reverse the sign at the counter; weak evidence of what was physically counted; requires the operator to calculate the discrepancy; makes later audits unable to distinguish an input mistake from the observed stock.
   - Effort: Low

2. **Absolute physical-count adjustment** — the operator enters the observed count; the application derives the signed delta from the authoritative balance in the transaction and persists both the movement delta and counted/resulting quantity.
   - Pros: Matches stocktake workflow; captures stronger audit evidence; removes mental subtraction and sign errors; projected balance is the entered count; supports zero stock naturally.
   - Cons: Requires an authoritative transactional read and an additional audit field; concurrent changes must be serialized before deriving the delta; a no-op count needs explicit behavior.
   - Effort: Medium

3. **Persisted alert status** — update an alert/status column or table after every inventory event.
   - Pros: Very cheap alert reads; can retain status-transition history if that later becomes a requirement.
   - Cons: Duplicates balance-derived truth; every mutation path must maintain it; drift and migration repair become possible; no benefit at the current fixed threshold.
   - Effort: Medium

4. **Derived alert query** — classify active products from current balances at read time and derive list and counter from one projection.
   - Pros: Single source of truth; immediately correct after any committed event and after restart; no write-path coupling; simple fixed-threshold semantics.
   - Cons: Executes a query on each refresh; needs a targeted index or query-plan check if catalog size grows substantially.
   - Effort: Low

### Recommendation
Build a bounded Inventory vertical around a small application interface: preview an operation from the latest read model, confirm it authoritatively in one SQLite transaction, and list derived alerts. Use positive quantity for `stock_entry` and absolute physical count for `adjustment`; require a reason only for adjustment. The preview is advisory, while confirmation recalculates from the current balance and returns the persisted old balance, delta, resulting balance, movement ID, and database timestamp.

Use a retained UUID request ID for each operation intent. On confirmation, begin an immediate transaction, return an existing movement for that request ID if present, revalidate that the product is active, derive and validate the result, update the balance, insert exactly one immutable movement, read back the result, and commit. A failed operation rolls back both balance and movement; the same request ID can be retried. Keep alerts as derived active-product queries.

The bounded first slice includes: selecting one active product through existing global search; one-product stock entry; one-product physical-count adjustment with required reason; projected balance; atomic/idempotent confirmation; exactly one immutable movement; derived low/out alert counter and list; and file-backed restart proof. Explicit non-goals are Catalog search/SQL changes, product editing or archiving, the checkout price/total defect, returns, cancellations, suppliers, purchase cost, batches, barcode flows, per-product thresholds, remote notifications, roles/operator identity, reports, backup/restore, cloud synchronization, and multi-store concurrency.

Product decisions still open are whether stock entries may carry an optional note, the user-facing wording/order of alert rows, and whether a no-op physical count should be rejected or acknowledged without a movement. Recommended defaults are optional entry note, out-of-stock before low-stock then product name, and rejection of no-op confirmation. None blocks proposal if those defaults are made explicit; changing no-op behavior to persist an audit event would require relaxing the non-zero movement invariant and the “exactly one movement” semantics.

### Risks
- A table-rebuild migration can lose or reinterpret historical movement facts unless it copies IDs/timestamps exactly and preflights legacy compatibility.
- Existing version-5 constraints permit invalid reserved movement combinations, so migration must detect incompatible rows rather than silently normalize them.
- Absolute-count adjustments can overwrite an intervening sale if confirmation derives from a stale preview; authoritative derivation must occur under the write transaction and the UI must show the recalculated result.
- Request-ID uniqueness alone is insufficient if balance mutation occurs before retry detection or outside the same transaction.
- The current Catalog application layer contains SQL and a SQLite-leaking repository trait; copying that pattern would couple Inventory business logic to persistence.
- The likely vertical exceeds the 400-line review budget across migration, Rust, IPC, UI, and tests. With `ask-on-risk`, task planning should recommend reviewable chained slices before apply.
- Untracked PRD/architecture files may not be available to other contributors; proposal/spec must carry all accepted behavior into tracked OpenSpec artifacts.
- `openspec/config.yaml` and SDD init memory materially understate current test capability and use an obsolete review budget.

### Ready for Proposal
Yes. The proposal should adopt physical-count adjustments, derived alerts, database-owned timestamps, UUID idempotency, a version-6 forward-only invariant migration, and the bounded first slice above. It should explicitly state that the checkout draft-price/total defect is neither a prerequisite nor included, and should surface the no-op adjustment default for confirmation without blocking planning.
