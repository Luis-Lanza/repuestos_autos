# Confirm Sale Design

## Context

The repository currently contains product and architecture documentation but no application source, package manifests, migrations, or executable test runners. This change therefore establishes the smallest complete offline sales slice and its supporting scaffold without introducing product management, licensing, networking, or later inventory workflows.

The dependency direction remains:

```text
React UI
  -> typed Tauri command adapters
  -> Rust application use cases
  -> Rust domain rules
  -> SQLite infrastructure adapters
  -> SQLite
```

React owns draft and presentation state only. Rust owns authoritative validation and orchestration. SQLite provides transactional persistence and defense-in-depth constraints.

## Goals and Non-Goals

### Goals

- Search seeded active products and build a whole-unit draft cart.
- Confirm cash, QR, or mixed-payment sales through one typed command seam.
- Keep money as integer centavos and quantities as whole integers end to end.
- Make `ConfirmSaleUseCase` the owner of the complete confirmation transaction.
- Guarantee one persisted effect per UUID `request_id` and reconstruct every successful response from persisted records.
- Couple every stock decrement with one immutable negative inventory movement in the same transaction.
- Provide a phased implementation and verification plan despite no runners currently existing.

### Non-Goals

Product/category management, stock entry and adjustment, returns, cancellation, reports, backup/restore, licensing, identity and roles, customers, invoicing, payment gateways, barcode support, networking, synchronization, multi-store behavior, and fractional or measured quantities remain outside this design.

## Architectural Decisions

### 1. Use a deep confirm-sale module at the typed Tauri command seam

The external interface is one typed `confirm_sale` command accepting a complete confirmation request and returning a persisted sale summary or a stable typed error. The command adapter performs serialization, UUID/integer shape checks, and error mapping, then delegates once to `ConfirmSaleUseCase`. It does not query SQLite or duplicate sales rules.

This seam is also the primary slice-level test surface: tests submit the same typed request used by React and assert the returned summary and SQLite effects. UI tests can replace only the TypeScript command adapter; Rust tests can invoke the command handler with application state backed by a disposable SQLite database. This keeps the interface small while exercising substantial domain, transaction, and persistence behavior behind it.

A separate typed `search_products` command supports catalog discovery. It is read-only and does not participate in sale confirmation.

### 2. Keep domain values explicit

Rust domain types prevent freely interchangeable primitive values:

- `MoneyCentavos(i64)` accepts non-negative integer centavos where required and uses checked multiplication/addition for totals.
- `Quantity(i64)` accepts positive whole units only.
- `RequestId(Uuid)` represents the sale intent identifier.
- `Payment` is a tagged value with `Cash { amount_applied, amount_tendered, change_given }` and `Qr { amount_applied }` variants.

TypeScript command contracts use integer `number` fields suffixed with `_centavos` and whole-integer quantity fields. The adapter rejects non-safe integers before invocation. Formatting into Bs occurs only in React and never feeds authoritative calculations.

### 3. Make the application use case own transaction decisions

`ConfirmSaleUseCase` explicitly begins, commits, or rolls back one transaction through an injected SQLite transaction manager. Repository methods receive the active transaction context and cannot begin or commit independently.

The use case orchestrates request-id reservation, catalog reads, domain validation, inserts, conditional stock updates, movement appends, and summary reconstruction. Domain functions remain independent of Tauri and SQLite. Infrastructure adapters implement transaction-scoped reads and writes without making business decisions.

### 4. Use database uniqueness as the idempotency arbiter

`sales.request_id` has a `UNIQUE NOT NULL` constraint. Confirmation first attempts to insert a pending sale header with the supplied request ID inside the transaction, using conflict-aware insertion.

- If reservation succeeds, the transaction owns the new confirmation and continues.
- If reservation conflicts, the use case loads the complete existing sale aggregate by request ID, reconstructs the persisted summary, and returns it without validating or applying the retried payload.
- If the existing aggregate cannot be reconstructed completely, the command returns a persistence-integrity error rather than synthesizing a partial result.

A repeated request ID always identifies the original persisted sale. The UI therefore creates one UUID when confirmation first begins, retains it through pending and error states, and reuses it for retries of that intent. It clears the ID only after moving to a new sale intent or explicitly discarding the draft. Disabling duplicate submit controls is useful feedback but is not the idempotency mechanism.

### 5. Reconstruct all successful summaries from persisted records

Neither a new confirmation nor a retry returns an in-memory draft as the authoritative result. After all writes for a new sale, the use case reloads the sale header, lines with product display data, and payments within the same transaction, verifies that the persisted aggregate is complete, commits, and returns the mapped summary. A duplicate request follows the same reconstruction path.

This ensures that an ambiguous commit followed by a retry produces the same sale identity, request ID, confirmed status, timestamp, lines, prices, quantities, payments, total, and confirmation outcome.

### 6. Protect stock with conditional writes and immutable evidence

For each confirmed line, the transaction executes a conditional decrement:

```sql
UPDATE stock_balances
SET quantity = quantity - :quantity
WHERE product_id = :product_id
  AND quantity >= :quantity;
```

An affected-row count other than one is an insufficient-stock or integrity failure and rolls back the complete transaction. After each successful decrement, the use case appends one negative `inventory_movements` row linked to the sale and line. Application repository interfaces expose append but no movement update/delete operation; SQLite triggers reject updates and deletes so immutability does not depend only on application discipline.

A standalone availability read may improve feedback, but it is never accepted as proof that stock can be sold.

## Command Contracts

### Product search

```ts
type SearchProductsRequest = { query: string };

type ProductSearchResult = {
  product_id: string;
  sku: string;
  name: string;
  category_name: string;
  searchable_fields: Record<string, string>;
  available_quantity: number;
  minimum_unit_price_centavos: number;
};
```

`search_products` returns only active seeded products matching normalized SKU, product name, category, or configured searchable category fields. The result is read-only; confirmation reloads current authoritative product, price, and stock data.

### Sale confirmation

```ts
type ConfirmSaleRequest = {
  request_id: string;
  lines: Array<{
    product_id: string;
    quantity: number;
    negotiated_unit_price_centavos: number;
  }>;
  payments: Array<
    | {
        method: "cash";
        amount_applied_centavos: number;
        amount_tendered_centavos: number;
        change_given_centavos: number;
      }
    | {
        method: "qr";
        amount_applied_centavos: number;
      }
  >;
};

type PersistedSaleSummary = {
  sale_id: string;
  request_id: string;
  status: "confirmed";
  confirmed_at: string;
  outcome: "confirmed";
  lines: Array<{
    product_id: string;
    sku: string;
    product_name: string;
    quantity: number;
    negotiated_unit_price_centavos: number;
    minimum_unit_price_snapshot_centavos: number;
    line_total_centavos: number;
  }>;
  payments: Array<
    | {
        method: "cash";
        amount_applied_centavos: number;
        amount_tendered_centavos: number;
        change_given_centavos: number;
      }
    | {
        method: "qr";
        amount_applied_centavos: number;
      }
  >;
  total_centavos: number;
};
```

The corresponding Rust DTOs use `i64` and UUID parsing, not floating-point values. Command failures map to a stable discriminated error contract with codes for invalid request shape, inactive or missing product, invalid quantity, price below current minimum, invalid payment, insufficient stock, and persistence integrity/failure. Messages are displayable but callers branch only on codes.

## Domain and Application Flow

1. React searches through the typed search adapter and adds a product to local draft state with quantity `1` and the returned minimum price as the initial negotiated price.
2. React permits local add/remove/edit behavior and immediate integer/payment feedback without mutating persisted stock.
3. On the first confirm attempt for the draft intent, React generates and stores a UUID and builds a `ConfirmSaleRequest` using centavo and whole-unit fields.
4. The Tauri command adapter parses the DTO and invokes `ConfirmSaleUseCase` with injected dependencies.
5. The use case begins one SQLite transaction and attempts request-id reservation.
6. On a request-id conflict, it reconstructs and returns the existing persisted summary with no new effects.
7. For a new reservation, it loads current product records, active state, minimum prices, and stock references in the transaction.
8. Domain construction validates non-empty valid lines, positive whole quantities, non-negative prices, checked line totals, negotiated prices at or above current minima, valid payment variants, exact applied-payment equality, and cash tender/change consistency.
9. The use case persists sale lines with both negotiated and minimum-price snapshot centavos, then persists payment lines.
10. It conditionally decrements every stock balance and appends one linked negative immutable movement per sale line. Any failure returns an error and rolls back all preceding effects.
11. It marks the reserved sale confirmed, reconstructs the complete summary from persisted rows, commits, and returns that summary.
12. React replaces the confirmation form with the persisted summary and formats centavos as Bs for display.

## SQLite Design

Versioned migrations create the minimum tables needed by this slice:

- `categories` and searchable category field/value tables needed by global search;
- `products` with SKU, name, active state, category reference, and minimum price centavos;
- `stock_balances` with one row per product and `quantity >= 0`;
- `sales` with unique UUID text `request_id`, status, total centavos, and timestamps;
- `sale_lines` with product reference, positive quantity, negotiated price, minimum-price snapshot, and line total centavos;
- `sale_payments` with a constrained method discriminator, non-negative applied amount, and nullable cash-only tender/change columns guarded by method-specific checks;
- `inventory_movements` with product, sale, sale-line references, a negative whole-unit delta, and creation timestamp.

Foreign keys are enabled on every connection. Checks enforce representable row-level invariants such as non-negative centavos, positive sale quantities, non-negative balances, valid payment column shapes, and negative sale movement quantities. Cross-row price, total, and payment equality remain authoritative domain/application validations.

Seed data is inserted by migration through the same catalog and stock tables used by future adapters. Seeds include active categories, searchable fields, active products with minimum prices, and stock balances; at least one inactive product supports exclusion verification. No seed is hard-coded in React or Rust control flow.

Indexes cover `sales.request_id`, active product search fields, category/searchable values, sale-line/payment foreign keys, and inventory movement references.

## React State Design

A sale screen reducer owns:

- search query and results;
- draft lines and editable whole quantities/centavo price input state;
- cash and QR payment input state;
- confirmation state (`idle`, `pending`, `error`, `confirmed`);
- the retained request ID; and
- the returned persisted summary.

UI validation prevents obvious malformed submission and gives keyboard-friendly feedback, but the command can still reject stale price, inactive product, payment, or stock conditions. An error preserves the draft and request ID. A confirmed result is rendered only from the returned persisted summary.

## Planned File Changes

The implementation phase is expected to introduce the following structure; exact scaffold filenames may follow the generated Tauri 2 conventions while preserving these module responsibilities:

```text
package.json                         React/TypeScript scripts and dependencies
src/
  ui/sales/                          search, cart, payment, and summary modules
  commands/catalog.ts               typed search adapter
  commands/confirm-sale.ts          typed confirm-sale adapter and shared DTOs
src-tauri/Cargo.toml                 Rust/Tauri/SQLite dependencies and test setup
src-tauri/src/
  commands/catalog.rs               Tauri search adapter
  commands/confirm_sale.rs          Tauri confirmation adapter and error mapping
  application/catalog/              search orchestration
  application/sales/confirm_sale.rs transaction-owning use case
  domain/                            money, quantity, sale, and payment rules
  infrastructure/sqlite/            connection, transaction, repositories
  infrastructure/sqlite/migrations/ schema and seeds
```

No product-management, licensing, network, return, cancellation, reporting, backup, or synchronization modules are introduced by this change.

## Error and Transaction Semantics

- Domain or application rejection before commit rolls back the complete transaction.
- Conditional stock failure rolls back sale headers, lines, payments, prior line decrements, and movements.
- SQLite uniqueness resolves concurrent/repeated request IDs; application-only pre-checks are insufficient.
- A persistence error during reconstruction prevents a success response and triggers rollback for a newly owned transaction.
- A commit whose client-visible result is ambiguous is resolved by retrying the same request ID and reconstructing the persisted aggregate.
- Database/internal details are logged locally where configured but are not exposed as unstable UI branching strings.

## Verification Strategy

No frontend, Rust, or end-to-end runner is currently configured, so this design does not claim executable tests. Implementation must first add the minimal manifests and test configuration, then run only commands actually introduced and present in the repository.

The typed confirm-sale command seam should cover the primary behavior matrix against a disposable SQLite database:

- cash-only, QR-only, mixed-payment, and multi-line success;
- fractional, zero, and negative quantity rejection at the command shape/domain edges;
- inactive product and below-current-minimum rejection;
- unequal payment total and inconsistent/negative cash rejection;
- one-line insufficient stock and partial-progress multi-line rollback;
- exactly one negative movement per persisted line;
- repeated request ID returning the same persisted summary with unchanged row counts and stock;
- summary reconstruction from database records rather than request values;
- foreign-key enablement, request-id uniqueness, row checks, and movement update/delete rejection;
- active global search over seeded name, SKU, category, and searchable fields;
- UI request-ID retention across pending/error retry and replacement only for a new intent.

Domain tests should exercise checked money arithmetic, quantity construction, price-floor snapshots, sale totals, and payment invariants without SQLite. Focused React tests should exercise reducer state and the typed command adapter replacement rather than duplicating Rust rules. A packaged Tauri smoke path can be added after the lower-level runners exist; until then, verification reports unavailable runtime checks explicitly.

## Phased Implementation Approach

1. **Minimal scaffold and database bootstrap:** add Tauri 2, React/TypeScript, Rust manifests, SQLite connection setup, foreign-key enablement, migrations, seeds, and runner configuration. Verify schema and seed readback once the newly added runners exist.
2. **Rust domain and application core:** implement value types, payment and sale invariants, transaction-scoped repository interfaces, `ConfirmSaleUseCase`, conditional stock mutation, immutable movements, idempotency, and persisted summary reconstruction. Add domain and disposable-database tests beside this behavior.
3. **Typed command seam:** implement search and confirm-sale DTOs, Tauri adapters, stable error mapping, and command-seam integration tests. This phase proves the full Rust-to-SQLite behavior before UI orchestration.
4. **React sales flow:** implement typed adapters, search, cart, payment entry, retained request-ID state, confirmation feedback, and persisted summary display with focused UI tests.
5. **Integrated verification and packaging check:** run the configured frontend and Rust suites, exercise the local React-to-Tauri flow, and record any unavailable Windows packaging evidence honestly. Do not add out-of-scope workflows to satisfy integration setup.

The tasks phase must split these phases into reviewable vertical work units, forecast changed lines against the configured 350-line review budget, and trigger the `ask-on-risk` decision before apply if the forecast exceeds the budget or recommends chained delivery.

## Rollout and Recovery

This is an unreleased first slice, so rollout begins with disposable local development databases created from versioned migrations and seeds. During development, schema corrections use forward migration edits appropriate to the unreleased disposable database and recreate it when necessary.

After real sale data exists, rollback must not delete or rewrite sales or movements. Disable the confirmation entry point while preserving the database, then ship a forward corrective migration or release. Any failed confirmation remains self-recovering through transaction rollback; ambiguous success is recovered by retrying the retained request ID.

## Risks and Mitigations

- **First-slice scaffold exceeds review capacity:** phase work vertically, forecast against 350 changed lines, and stop for the configured delivery decision.
- **Rule duplication across UI and persistence:** keep the typed command seam thin and all authoritative cross-record rules in domain/application code.
- **Duplicate effects after retries:** retain the UI request ID, enforce SQLite uniqueness, and reconstruct the original aggregate on conflict.
- **Overselling after a stale availability read:** rely on conditional decrement within the owned transaction.
- **Incomplete retry response:** persist every summary source atomically and reject incomplete reconstruction.
- **Money precision loss:** accept only safe integer centavos in TypeScript and use checked `i64` domain arithmetic in Rust.
- **False verification confidence:** distinguish planned evidence from runnable evidence until manifests and runners are actually added.
