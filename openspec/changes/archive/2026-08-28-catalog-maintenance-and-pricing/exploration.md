## Exploration: Catalog Maintenance and Pricing

### Current State
The application can create categories and onboard active products, but it cannot correct, archive, reactivate, or reprice them. This turns onboarding mistakes and ordinary price changes into an operational dead end: staff would need direct SQLite edits or duplicate records, both of which threaten search consistency and historical interpretation. The real product problem is therefore not generic CRUD; it is safe maintenance of the current sellable catalog while preserving confirmed sales and immutable inventory facts.

Canonical vocabulary should distinguish:

- **Catalog Product** — the mutable current description of an item, including SKU, name, category-specific values, lifecycle state, and **Current Catalog Price**.
- **Current Catalog Price** — the positive integer-centavo price used only when a new sale is authoritatively confirmed.
- **Historical Sale-Line Unit Price Snapshot** — the immutable unit price stored on a confirmed sale line. It is a sale fact and MUST NOT be recomputed from the Catalog Product.
- **Archived Catalog Product/Category** — retained catalog data unavailable to new operational work; archival is not deletion.

The database already supports part of that model. `products.active` filters global FTS search, Sales rejects inactive products during confirmation, and Inventory rejects inactive products during stock operations. `sale_lines` stores SKU/name snapshots plus the resolved unit price in legacy compatibility columns (`negotiated_unit_price_centavos` and `minimum_unit_price_snapshot_centavos`). Current code writes the same authoritative Catalog Price to both columns, and idempotent sale retries load the original stored facts before repricing. Product, sale-line, movement, and balance foreign keys mean products must remain physically present.

Categories have no lifecycle state. Product/category rows have no revision, updated timestamp, or catalog audit history. Category and product update commands do not exist. Category listing currently includes every category, while global search filters only `products.active`; it has no category-state predicate. The FTS table is manually written during product onboarding and backfilled by migration, with no triggers, so editing SKU, name, category name, or attribute values without rebuilding affected search documents would leave search stale. Price and stock are read from base tables at query time and are not copied into the FTS document.

The Sales draft holds product identity, SKU, name, and quantity, but not the displayed Catalog Price despite the canonical Sales specification requiring it. Search shows price guidance; confirmation independently resolves product activity and Current Catalog Price inside its transaction. Therefore a price change after search applies to an unconfirmed sale, an archived product fails confirmation, and a confirmed/idempotent sale never changes. There is no persisted cart, cross-screen cart synchronization, or concurrent process in the intended one-computer deployment; however, sequential stale forms remain possible, and `DatabaseState` serializes commands but does not prevent a later stale edit from overwriting an earlier committed edit.

Inventory remains related but separate. Archiving a product must retain its balance and immutable movements; current alert and operation queries already exclude inactive products. Reactivation would expose the preserved balance again. Catalog maintenance must never create, rewrite, or delete inventory movements, and product metadata edits must not alter stock.

#### Product decisions and evidence-based defaults

| Question | Evidence-based recommendation | Status |
|---|---|---|
| What does “edit category” include first? | Rename only. Defer adding/removing/retyping fields and changing option sets because existing product values need explicit compatibility/backfill semantics. | Recommended default; product decision remains open. |
| May a product change category? | No in the first slice. Category reassignment changes the attribute schema and search meaning and needs a dedicated migration workflow. | Recommended default. |
| How is a category archived? | Reject while it has active products. Require explicit product archival first; category reactivation MUST NOT reactivate products implicitly. | Recommended default; bulk cascade behavior remains open. |
| Can a product with stock be archived? | Yes, with explicit confirmation showing current stock. Preserve balance/history; exclude it from Search, Sales confirmation, Inventory operations, and alerts. | Recommended default. |
| What blocks product reactivation? | Its category must be active and all stored attribute values must still validate. Preserved stock becomes operationally visible again. | Recommended default. |
| What happens to an active cart after repricing? | The unconfirmed sale uses the new Current Catalog Price at confirmation. The draft price is advisory; the UI should detect and clearly surface a changed price before payment is accepted rather than silently treating the old display as authoritative. | Domain rule is settled; acknowledgement UX remains open. |
| What happens to an active cart after archival? | Confirmation rejects it as `inactive_product`; the draft stays editable so the operator can remove or replace the line. | Existing behavior; formalize it. |
| Are SKU and category names reusable after archival? | No. Keep identities unique across active and archived records to avoid ambiguous history, search, and reactivation. | Recommended default. |
| Is catalog-change attribution required? | Record timestamped changes without an operator identity because v1 has no accounts or roles. | Consistent with PRD. |

### Affected Areas
- `docs/PRD.md` — Requires category editing/archival, product active status, future-only Catalog Price changes, and preserved sale history.
- `docs/ARCHITECTURE.md` — Assigns authoritative Catalog Price resolution to Rust/SQLite and treats confirmed sale facts and inventory movements as historical evidence.
- `openspec/specs/catalog-onboarding/spec.md` — Onboarding currently excludes editing and archival; maintenance should extend Catalog without weakening atomic onboarding.
- `openspec/specs/sales/spec.md` — Already defines future-only repricing, backend confirmation-time resolution, immutable historical snapshots, inactive-product rejection, and idempotent retries.
- `openspec/specs/operational-inventory-control/spec.md` — Inventory selection, operations, and alerts are active-product-only and must remain consistent after archival/reactivation.
- `src-tauri/src/domain/catalog.rs` — Existing Catalog validation can be extended for update intents and current-value validation.
- `src-tauri/src/application/catalog/` — Currently mixes direct SQL, onboarding orchestration, and a SQLite-leaking creation trait; this is the natural seam for a deeper Catalog Maintenance module.
- `src-tauri/src/infrastructure/sqlite/catalog_repository.rs` — Product persistence and FTS document construction live here; maintenance needs transactional compare-and-swap, audit writes, and canonical search-document refresh.
- `src-tauri/src/infrastructure/sqlite/sale_repository.rs` — Must continue resolving the current product price and activity at confirmation while reading historical line snapshots from `sale_lines`.
- `src-tauri/src/infrastructure/sqlite/inventory_repository.rs` — Already rejects inactive products and filters inactive alerts; category lifecycle introduces an effective-availability rule that must be shared rather than duplicated.
- `src-tauri/src/infrastructure/sqlite/migrations/` and `sqlite/mod.rs` — A forward-only additive migration is needed for category activity, revisions, timestamps/audit history, indexes, and migration preflight.
- `src-tauri/src/commands/{catalog.rs,onboarding.rs}` and `src-tauri/src/lib.rs` — Need narrow maintenance commands, stable errors, strict payloads, and registration.
- `src/commands/` and `src/ui/onboarding/` — The current creation-only screen should become a Catalog workspace with listing, edit, lifecycle, conflict, and confirmation states.
- `src/ui/sales/` — Must preserve backend authority and make stale draft price/activity outcomes understandable; current draft-price omission is adjacent existing debt.
- `src/ui/inventory/` — Must refresh selection/alerts after lifecycle changes without owning Catalog rules.
- `src-tauri/tests/sqlite_migrations.rs`, Catalog/Sales/Inventory integration tests, and TypeScript flow tests — Need preservation, stale-edit, search-refresh, lifecycle, repricing, and history evidence in later phases.

### Approaches
1. **Table-oriented CRUD commands** — Add separate update/archive/reactivate commands that directly mutate category and product rows, with each caller handling validation, FTS refresh, and conflicts.
   - Pros: Smallest initial code change; closely matches the current direct-SQL Catalog functions.
   - Cons: Shallow interface; duplicates lifecycle and search rules; makes partial FTS/audit updates likely; stale writes remain last-write-wins; spreads effective-availability semantics across Sales and Inventory.
   - Effort: Medium

2. **Deep Catalog Maintenance module with optimistic revisioning** — Expose a small intent-based interface for loading an editable Catalog record, applying validated metadata/price changes, and changing lifecycle state. A SQLite adapter owns compare-and-swap, search-document refresh, audit append, and transactionality.
   - Pros: Concentrates validation, category/product lifecycle, revision conflicts, FTS consistency, and auditability behind one interface; preserves existing Sales and Inventory seams; supports safe future expansion.
   - Cons: Requires an additive migration and some Catalog refactoring; category-field evolution still needs a later dedicated design.
   - Effort: Medium/High

3. **Fully temporal catalog versions** — Create immutable product/category versions and make every sale, search result, and inventory operation reference an effective version.
   - Pros: Complete catalog history and precise reconstruction of every past description.
   - Cons: Much larger model and migration; duplicates history already captured where Sales needs it; complicates search, foreign keys, Inventory, and reactivation without a stated product need.
   - Effort: High

### Recommendation
Choose the deep **Catalog Maintenance** module with optimistic revisioning, but keep the first slice deliberately narrow.

Its external interface should express operator intent rather than tables: load one editable Category or Catalog Product, update allowed metadata using `expected_revision`, change Current Catalog Price, archive, and reactivate. The SQLite adapter should execute each intent in one transaction: reload current state, reject a mismatched revision with `stale_catalog_record`, validate cross-record rules, update the base row, refresh all affected FTS documents, append one timestamped audit record, increment revision, read back the persisted result, and commit. Stable failures should distinguish validation, duplicate identity, inactive/missing category, lifecycle conflict, stale edit, and opaque persistence failure.

Effective operational availability should be one canonical Catalog rule: a product is available only when both product and category are active. Search, Sales confirmation, Inventory confirmation, and alerts should consume that rule through a small projection/query interface rather than copy predicates. Category archival should be blocked while active products remain; category reactivation should restore only the category, never product states. Product archival may preserve non-zero stock after explicit UI confirmation because no stock fact changes.

Use an additive forward-only schema migration (expected next version: v7): add category activity plus integer revisions and database-owned update timestamps to categories/products; add an append-only catalog-change audit table; add supporting indexes; preserve every existing ID, active product, price, attribute value, balance, movement, sale line, and timestamp. Do not rename the legacy physical price columns in this slice. Treat `products.minimum_unit_price_centavos` as compatibility storage for Current Catalog Price and both sale-line price columns as immutable compatibility storage for the Historical Sale-Line Unit Price Snapshot. A physical rename/rebuild adds risk without product value.

The first slice should include:

- Category list/detail, rename, archive, and reactivate with revision checks.
- Catalog Product list/detail, edits to SKU, name, existing category-specific values, and Current Catalog Price; category reassignment excluded.
- Product archive/reactivate with effective-availability validation and stock-preservation confirmation.
- Atomic FTS refresh for product edits and all products affected by a category rename.
- Future-sale repricing with proof that confirmed/idempotent sale facts never change.
- Stable stale-edit handling, timestamped catalog audit records, and persisted readback.
- Search, Sales, Inventory, and alert consistency after lifecycle changes.
- Transactional migration preflight, foreign-key validation, reopen proof, and backup-before-rollout/restore-for-rollback guidance.

Explicit non-goals should be category field-definition evolution, option deletion/remapping, product category reassignment, bulk edit/import, hard delete, SKU reuse, price scheduling/history analytics, discounts/negotiation, purchase cost/suppliers, stock mutation, returns/cancellation, accounts/operator attribution, persisted carts, multi-device concurrency, and live cross-screen synchronization. The existing Sales draft-price display defect should be addressed only as much as required to present current guidance and stale-confirmation outcomes; it must not become a checkout redesign.

### Risks
- Category-field edits can invalidate existing product values; including them without explicit backfill and compatibility rules would corrupt the Catalog model.
- Manual FTS maintenance creates stale-search risk unless every metadata transaction refreshes the canonical document before commit.
- A category rename can touch many FTS rows and should be bounded, transactional, and performance-tested at 20,000 products.
- Last-write-wins updates would silently lose sequential edits even with one mutex-protected connection; expected revisions are still required.
- Silent repricing of a displayed draft can surprise the operator or invalidate payment inputs; proposal/spec must define the acknowledgement/error flow.
- Category archival cascades can accidentally reactivate products or hide stocked products; blocking active children is safer than implicit bulk mutation.
- Audit records can drift from base rows if they are not written in the same transaction and protected from update/delete.
- Rebuilding or renaming historical price storage could reinterpret confirmed sales; the first slice should avoid it and verify byte-for-byte sale fact preservation.
- The complete vertical will likely exceed the 400-line review budget. Under `ask-on-risk`, task planning should recommend stacked-to-main work units before apply.
- `openspec/config.yaml` still contains stale change-specific wording and a 350-line sentence, but exploration must not modify unrelated configuration; the supplied 400-line budget governs this change.

### Ready for Proposal
Yes. Automatic proposal generation can adopt the deep Catalog Maintenance module, optimistic revisioning, additive v7 migration, explicit Current Catalog Price versus Historical Sale-Line Unit Price Snapshot vocabulary, blocked category archival with active children, future-only repricing, and the bounded first slice above. The proposal should preserve the unresolved product choices as explicit decisions: whether category field-definition evolution belongs in a later change, whether price changes require an explicit active-cart acknowledgement, and whether the business ever wants bulk category archival. None blocks a narrow first proposal if the recommended defaults are accepted provisionally.
