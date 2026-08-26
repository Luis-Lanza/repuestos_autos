## Exploration: Sellable Product Onboarding

### Current State
Canonical product documentation requires manual category and product creation, category-defined typed fields, integer-centavo catalog prices, positive whole-unit inventory, immutable timestamped movements, global active-product search, and fixed-price checkout. The architecture assigns UI state to React, typed IPC adaptation to Tauri commands, business validation and transaction ownership to Rust application/domain modules, and constraints/persistence to SQLite.

The dirty worktree already contains a directly implemented onboarding vertical; this exploration assesses it as evidence and does not claim SDD guided it. The implementation exposes separate list/create-category and create-product commands. Product creation validates the selected category and typed values, then persists the active product, attribute values, stock balance, and positive `opening_stock` movement in one SQLite transaction. Existing global search and confirm-sale paths consume the created product, and integration tests exercise rollback, search, and fixed-price checkout. OpenSpec configuration is stale: it still says application manifests, source, and executable tests do not exist.

### Affected Areas
- `docs/PRD.md` — Canonical behavior for categories, searchable configured fields, active products, whole-unit stock, audit movements, and fixed-price sales.
- `docs/ARCHITECTURE.md` — Defines layer ownership, transaction authority, catalog terminology, and SQLite invariants.
- `src/ui/onboarding/` — Minimal category/product forms and dynamic field input rendering.
- `src/commands/onboarding.ts` — Narrow TypeScript interface to onboarding commands.
- `src-tauri/src/commands/onboarding.rs` — Tauri-facing response and stable error adapter.
- `src-tauri/src/application/catalog/mod.rs` — Catalog onboarding orchestration, validation calls, transaction ownership, persistence, and search integration.
- `src-tauri/src/domain/catalog.rs` — Category-field and product-attribute validation rules.
- `src-tauri/src/infrastructure/sqlite/migrations/0004_product_onboarding.sql` — Typed attribute schema and opening-movement persistence constraints.
- `src-tauri/tests/product_onboarding.rs` — Atomicity, validation, searchability, and checkout evidence.

### Approaches
1. **Separate category setup from an atomic product aggregate command** — Keep category creation/listing as catalog setup, then treat product, attribute values, opening balance, and opening movement as one application-owned transaction.
   - Pros: Small operator workflow; supports selecting existing categories; preserves one deep product-creation interface; keeps all-or-nothing inventory integrity; reuses current search and checkout seams.
   - Cons: A newly created category can remain unused if product creation fails; category and product onboarding are not one transaction.
   - Effort: Medium

2. **Single wizard command for optional category plus product creation** — Submit a complete category-or-selection and product payload to one transaction.
   - Pros: Entire first-time setup succeeds or fails together; no unused category after a failed product creation.
   - Cons: Larger conditional interface; duplicates category selection/creation concerns; makes existing-category onboarding harder to reason about; reduces reuse and module depth.
   - Effort: High

### Recommendation
Use the separate category setup plus atomic product aggregate approach. The product goal permits creating or selecting a category, while the critical invariant is that product, typed values, balance, and positive opening movement persist together. Keep Rust authoritative, expose only typed intent over Tauri, and preserve the existing global-search and fixed-price-checkout interfaces. The proposal should define behavior from the canonical docs, while treating current code only as implementation evidence requiring later verification.

### Risks
- Global search uses leading-wildcard `LIKE`; the searchable-value index cannot prove the PRD target of initial results within one second at 20,000 products. Benchmarking or an FTS/index strategy is needed before claiming the performance requirement.
- SQLite enforces typed value shape but does not itself guarantee that an attribute definition belongs to the product's category or that an option value belongs to its definition; current Rust validation is the authoritative defense.
- Persistent columns still use legacy `minimum_unit_price` terminology while public contracts use catalog price, creating semantic debt across catalog and checkout modules.
- Current frontend tests cover IPC payloads and attribute-value shaping, but not the rendered operator journey or immediate post-create discovery through the UI.
- The migration rebuilds `inventory_movements`; legacy preservation is tested, but rollback and failure behavior for partially incompatible real databases require explicit proposal/design treatment.
- The already-implemented dirty vertical exceeds the 400 changed-line review budget even before all untracked files are counted. Under `ask-on-risk`, retrospective review slicing or explicit size-risk acceptance is required; SDD must not be represented as having guided that implementation.

### Ready for Proposal
Yes. The proposal should formalize the canonical product intent and non-goals, explicitly label the implementation as pre-existing evidence, preserve the atomic product aggregate seam, and carry forward search-performance, schema-integrity, migration, UI-journey, and review-size risks.
