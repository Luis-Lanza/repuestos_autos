# Design: Fixed-price checkout

## Context

The current checkout crosses React, a TypeScript Tauri command seam, a Rust command adapter, the sales application/domain, and a SQLite adapter. Today the client submits `negotiated_unit_price_centavos` and complete payment rows, including cash applied and change. Rust validates those values but does not own their derivation.

The existing SQLite schema already stores the facts required by the new behavior:

- `products.minimum_unit_price_centavos` stores the current product monetary value.
- `sale_lines.negotiated_unit_price_centavos` stores the actual unit price used by a historical sale.
- `sale_lines.minimum_unit_price_snapshot_centavos` stores the then-current minimum.
- `sale_payments` stores applied, tendered, and change facts.
- `sales.request_id` is unique.
- Inventory movements reference sale lines and are protected by immutable-update/delete triggers.

The design changes authority and public terminology without rewriting historical facts.

## Decisions

### 1. Use the existing monetary columns as backward-compatible physical storage

Migration version 2 is a semantic migration, not a destructive schema migration.

| Physical SQLite column | Fixed-price interpretation | Legacy interpretation |
| --- | --- | --- |
| `products.minimum_unit_price_centavos` | current catalog unit price | current minimum negotiable price |
| `sale_lines.negotiated_unit_price_centavos` | authoritative sale-time unit-price snapshot | negotiated sale-time unit price |
| `sale_lines.minimum_unit_price_snapshot_centavos` | compatibility snapshot; equal to the resolved catalog price for new fixed-price sales | historical minimum snapshot |
| `sale_payments.amount_applied_centavos` | backend-derived applied amount | historical applied amount |
| `sale_payments.amount_tendered_centavos` | submitted cash tender | historical cash tender |
| `sale_payments.change_given_centavos` | backend-derived change | historical change |

The Rust and TypeScript interfaces rename catalog and persisted-summary concepts to `catalog_unit_price` and `unit_price`, but SQLite column names remain unchanged. This avoids a table rebuild, preserves every old value exactly, and allows the previous binary to continue operating after rollback.

For every newly confirmed fixed-price sale, Rust resolves `products.minimum_unit_price_centavos` once inside the confirmation transaction and writes that same value to both sale-line price columns. The duplicated write is intentional compatibility data: the previous application sees a valid negotiated price equal to its minimum, while the new application exposes only one authoritative historical `unit_price_centavos`.

Legacy rows are never repriced or backfilled. The new reader maps the stored `negotiated_unit_price_centavos` to `unit_price_centavos`, because it is the actual price historically applied even when that row predates fixed-price checkout. The legacy minimum snapshot remains stored but is not exposed in the new cashier-facing summary contract.

### 2. Apply migration version 2 without changing table shapes

`migrate_if_needed` becomes ordered version handling:

1. Version `0`: apply `0001_confirm_sale.sql`, set `user_version = 1`, then continue.
2. Version `1`: in one SQLite transaction, run the version-2 compatibility preflight and set `user_version = 2`.
3. Version `2`: open normally.
4. Version greater than `2`: fail opening with an unsupported-schema-version error rather than writing an unknown schema.

The version-2 preflight verifies that the required tables and physical columns exist and runs `PRAGMA foreign_key_check`; it does not update sale, payment, stock, movement, request-ID, or product rows. Missing required columns or foreign-key violations abort the migration and leave `user_version` unchanged. The migration SQL file may contain only the documented compatibility assertions supported by the migration runner; the runner owns the version change transaction.

No new defaults, nullability, indexes, or constraints are introduced:

- sale-line price and snapshot columns remain `NOT NULL` and non-negative;
- cash tender/change remain nullable only for QR rows;
- QR tender/change remain `NULL`;
- request-ID uniqueness, stock checks, foreign keys, and movement immutability remain unchanged.

Rollback is forward-compatible: do not decrement `user_version` and do not drop or rewrite anything. The previous application currently initializes only version `0` and otherwise uses the existing tables, so it can operate against version `2`. It reads old and new fixed-price rows through the unchanged physical columns. If the previous application creates a negotiated sale after rollback, a later fixed-price application still reads its stored negotiated value as that sale's historical unit price; it does not claim that the legacy sale used fixed-price behavior.

### 3. Keep confirmation as one application-owned transaction and check idempotency first

`ConfirmSaleUseCase` remains responsible for the complete transaction. Its order is:

1. Begin SQLite transaction.
2. Reserve `request_id` using the existing unique `sales.request_id` insert.
3. If reservation conflicts, load and return the already-confirmed persisted summary immediately. Do not inspect request lines, resolve prices, derive payment values, write rows, or touch stock.
4. Resolve all requested product prices and construct authoritative sale lines.
5. Calculate the authoritative total using checked integer-centavo arithmetic.
6. Derive and validate the payment breakdown from the total and submitted tender values.
7. Persist sale lines and payment facts.
8. Conditionally deduct stock and append immutable inventory movements for each persisted line.
9. Mark the sale confirmed and load the persisted summary from SQLite.
10. Commit. Any error rolls back the reserved pending sale and every related change.

This order preserves retry behavior after catalog changes and keeps lines, payments, balances, movements, and summary state atomic.

## Deep modules and interfaces

### Catalog price resolution

Keep the seam at the application repository interface because the current SQLite adapter already owns product lookup and persistence access. Replace the shallow per-line negotiated-price method with one operation:

```rust
pub struct RequestedLine {
    pub product_id: i64,
    pub quantity: Quantity,
}

pub trait SaleRepository {
    fn reserve_or_load(
        &self,
        transaction: &Transaction<'_>,
        request_id: &RequestId,
    ) -> Result<Reservation, ConfirmSaleError>;

    fn resolve_lines(
        &self,
        transaction: &Transaction<'_>,
        requested: &[RequestedLine],
    ) -> Result<Vec<SaleLine>, ConfirmSaleError>;

    fn persist_confirmed(
        &self,
        transaction: &Transaction<'_>,
        request_id: &RequestId,
        sale: &Sale,
    ) -> Result<PersistedSaleSummary, ConfirmSaleError>;
}

pub enum Reservation {
    Reserved,
    ExistingConfirmed(PersistedSaleSummary),
}
```

`reserve_or_load` hides the unique-key insert and existing-summary readback so the use case has one idempotency operation and cannot accidentally reprice a conflict.

`resolve_lines` hides product lookup, active-state validation, money conversion, checked line-total calculation, and sale-time snapshot construction. It preserves request order and returns exactly one `SaleLine` per request line. The SQLite adapter parameterizes every product lookup. Duplicate product IDs are rejected by the use case before resolution so one product cannot create competing stock updates in the same sale.

`persist_confirmed` hides the physical compatibility column names and the coordinated line/payment/stock/movement writes, including persisted-summary readback. It remains inside the caller-owned transaction; it does not start or commit a nested transaction. This three-operation interface concentrates persistence invariants without adding another adapter or a hypothetical new seam.

The authoritative domain line becomes:

```rust
pub struct SaleLine {
    product_id: i64,
    quantity: Quantity,
    unit_price: MoneyCentavos,
    total: MoneyCentavos,
}

impl SaleLine {
    pub fn priced(
        product_id: i64,
        quantity: Quantity,
        unit_price: MoneyCentavos,
    ) -> Result<Self, SaleError>;
}
```

Callers cannot supply a minimum or negotiated price. The SQLite implementation writes `unit_price` into both legacy physical price columns.

### Derived payment computation

The payment calculation is a pure domain module with one interface:

```rust
#[derive(Clone, Copy)]
pub struct PaymentInput {
    pub amount_tendered: Option<MoneyCentavos>,
    pub qr_applied: Option<MoneyCentavos>,
}

pub struct PaymentBreakdown {
    payments: Vec<Payment>,
}

impl PaymentBreakdown {
    pub fn derive(
        total: MoneyCentavos,
        input: PaymentInput,
    ) -> Result<Self, PaymentError>;

    pub fn payments(&self) -> &[Payment];
}
```

For positive sale totals, derivation is exact and ordered:

1. Missing QR means zero QR. Explicit zero QR is treated as no QR payment row.
2. If QR exceeds total, return `PaymentError::QrExceedsTotal`.
3. `cash_applied = total - qr_applied`, using checked subtraction.
4. If `cash_applied > 0`, cash tender is required and must be at least `cash_applied`; otherwise return `CashTenderRequired` or `InsufficientCashTender`.
5. `change_given = amount_tendered - cash_applied`, using checked subtraction.
6. If `cash_applied == 0`, missing or zero cash tender produces no cash row; a positive tender is rejected as `UnexpectedCashTender` rather than silently ignored.
7. Emit QR first when positive, then cash when required. Applied amounts therefore sum to total by construction.

The existing `Payment` persisted shape remains:

```rust
pub enum Payment {
    Cash {
        amount_applied: MoneyCentavos,
        amount_tendered: MoneyCentavos,
        change_given: MoneyCentavos,
    },
    Qr {
        amount_applied: MoneyCentavos,
    },
}
```

`Sale::new(lines, payment_breakdown)` still independently enforces non-empty lines, checked total arithmetic, and `sum(amount_applied) == total`. This is defense in depth at the aggregate invariant owner, not duplicate client calculation.

## Cross-seam contracts

### React draft

Draft state contains catalog guidance but no price authority:

```ts
type DraftLine = {
  product_id: number;
  sku: string;
  product_name: string;
  quantity: number;
  catalog_unit_price_centavos: number;
};

type DraftPaymentInput = {
  amount_tendered_centavos: string;
  qr_applied_centavos: string;
};
```

The cart renders catalog price as read-only guidance and states that confirmation uses the current backend price. Remove `line_price_changed`, cash-applied, and cash-change actions and controls. The UI may show an estimated draft total, applied cash, or change, but those values are presentation-only and never enter the command payload. After success, replace the draft display with the persisted response so a price change between search and confirmation is visible.

Retain one generated UUID in `state.request_id` across failed retries. Only discard/new-sale creates a new intent.

### TypeScript command seam

Exact request shape:

```ts
type ConfirmSaleRequest = {
  request_id: string;
  lines: Array<{
    product_id: number;
    quantity: number;
  }>;
  payment: {
    amount_tendered_centavos: number | null;
    qr_applied_centavos: number | null;
  };
};
```

`null` means the payment method was not supplied. TypeScript validates safe integers before invoke: product IDs and quantities must be positive; monetary inputs, when non-null, must be non-negative safe integers. Business validation against the authoritative total remains in Rust.

Exact success shape:

```ts
type PersistedSaleSummary = {
  sale_id: number;
  request_id: string;
  status: "confirmed";
  confirmed_at: string;
  outcome: "confirmed";
  lines: Array<{
    product_id: number;
    sku: string;
    product_name: string;
    quantity: number;
    unit_price_centavos: number;
    line_total_centavos: number;
  }>;
  payments: Array<
    | {
        method: "cash";
        amount_applied_centavos: number;
        amount_tendered_centavos: number;
        change_given_centavos: number;
      }
    | { method: "qr"; amount_applied_centavos: number }
  >;
  total_centavos: number;
};

type ConfirmSaleResponse =
  | ({ kind: "success" } & PersistedSaleSummary)
  | { kind: "error"; code: string; message: string };
```

Persisted output reports facts loaded from SQLite, not an in-memory projection of the request.

### Rust Tauri command adapter

The deserialization structs mirror the TypeScript request and use `#[serde(deny_unknown_fields)]` on the top-level request, each line, and the payment input. Therefore legacy fields such as `negotiated_unit_price_centavos`, `payments`, `amount_applied_centavos`, or `change_given_centavos` cause `invalid_request` before the use case starts; they are never silently authoritative and no persistence occurs.

The adapter owns shape conversion only:

- parse UUID to `RequestId`;
- convert quantity to `Quantity`;
- convert nullable centavos to `Option<MoneyCentavos>`;
- map typed application/domain errors to stable command codes;
- map persisted domain names to the exact response shape.

It does not resolve price, total the sale, or calculate payment values.

### Rust application and domain

Application request:

```rust
pub struct ConfirmSaleRequest {
    pub request_id: RequestId,
    pub lines: Vec<RequestedLine>,
    pub payment: PaymentInput,
}
```

Use typed errors instead of matching database/domain strings:

```rust
pub enum ConfirmSaleError {
    InvalidRequest,
    DuplicateProduct,
    ProductMissing,
    ProductInactive,
    InvalidQuantity,
    MoneyOverflow,
    QrExceedsTotal,
    CashTenderRequired,
    InsufficientCashTender,
    UnexpectedCashTender,
    InsufficientStock,
    PersistedDataInvalid,
    Persistence,
}
```

The command adapter maps QR/tender errors to `invalid_payment`; missing/inactive product and insufficient stock retain their existing codes. Persistence details are logged internally if logging is introduced, but the command response remains `persistence_failure` without SQL text.

## Invariant ownership

| Invariant | Owner |
| --- | --- |
| Safe integer JSON/JS representation | TypeScript command seam; repeated by Rust deserialization/domain conversion |
| No negotiated price or derived payment input | TypeScript type plus Rust `deny_unknown_fields` command adapter |
| UUID request ID | `RequestId` domain type |
| Same request ID returns original persisted facts before repricing | `ConfirmSaleUseCase` transaction order plus unique `sales.request_id` |
| Positive whole quantity | `Quantity` domain type and SQLite `CHECK` |
| One requested line per product | `ConfirmSaleUseCase` |
| Product exists and is active | SQLite sale repository during `resolve_lines` |
| Catalog price resolved at confirmation | SQLite sale repository inside the confirmation transaction |
| Checked line and sale totals in integer centavos | `SaleLine` and `Sale` domain types |
| QR limit, cash applied, tender sufficiency, and change | `PaymentBreakdown::derive` |
| Applied payment sum equals authoritative total | `Sale` aggregate; SQLite persists the already-valid breakdown |
| Sale lines, payments, stock, movements, and confirmation are atomic | `ConfirmSaleUseCase` transaction |
| Stock cannot become negative | Conditional SQLite stock update plus `CHECK (quantity >= 0)` |
| Inventory movement is append-only | SQLite foreign keys and immutable triggers |
| Historical line/payment readback uses stored facts | `SqliteSaleRepository::load_confirmed` |
| Legacy rows are not repriced | Version-2 migration policy and summary reader |

## Data flow

```text
Catalog search
  SQLite physical minimum price
    -> Rust catalog result as catalog_unit_price_centavos
    -> TypeScript/React read-only draft guidance

Confirmation
  React: request_id + product/quantity + nullable cash tender/QR
    -> TypeScript safe-integer validation
    -> Tauri command strict deserialization
    -> ConfirmSaleUseCase transaction
       -> request-ID reservation / existing summary short-circuit
       -> SQLite product lookup and catalog price resolution
       -> SaleLine checked totals
       -> PaymentBreakdown derived values
       -> Sale aggregate validation
       -> SQLite line/payment writes
       -> conditional stock deductions + immutable movements
       -> confirmed sale update
       -> persisted summary readback
    -> authoritative response replaces draft
```

## File change plan

### React and TypeScript

- `src/commands/catalog.ts`: expose `catalog_unit_price_centavos`; adapt the existing backend catalog field in the same vertical change.
- `src/commands/confirm-sale.ts`: replace negotiated/payment-row input with the exact reduced request, safe-integer validation, and renamed persisted line output.
- `src/ui/sales/sale-flow.ts`: remove editable price and derived-payment state/actions; retain catalog guidance, tender inputs, and request-ID retry continuity.
- `src/ui/sales/sale-screen.ts`: render read-only catalog price; submit only product/quantity and tender inputs; remove cash-applied/change controls.
- `src/ui/sales/catalog-result.ts` and `src/ui/sales/persisted-summary.ts`: use catalog/final unit-price terminology and render authoritative payment facts.
- Associated `.test.ts` files: update contracts, reducer behavior, display, rejected legacy fields, and retry request-ID expectations.

### Rust

- `src-tauri/src/commands/catalog.rs` and `src-tauri/src/application/catalog/mod.rs`: rename the outward catalog field while reading the unchanged physical product column.
- `src-tauri/src/commands/confirm_sale.rs`: strict reduced request, shape conversion, typed error mapping, and renamed persisted output.
- `src-tauri/src/application/sales/confirm_sale.rs`: orchestrate idempotency-first resolution, payment derivation, persistence, and stock mutation in one transaction.
- `src-tauri/src/application/sales/repository.rs`: deepen the repository interface around reservation, line resolution, confirmed persistence, and readback.
- `src-tauri/src/domain/sales/mod.rs`: replace negotiated line construction with authoritative priced lines and add pure `PaymentInput`/`PaymentBreakdown` derivation.
- `src-tauri/src/infrastructure/sqlite/sale_repository.rs`: resolve catalog prices, write both compatibility price columns, persist derived payments, preserve stock/movement behavior, and map legacy rows to the renamed summary.
- `src-tauri/src/infrastructure/sqlite/mod.rs`: ordered schema-version handling, version-2 preflight, and unknown-future-version failure.
- `src-tauri/src/infrastructure/sqlite/migrations/0002_fixed_price_checkout.sql`: document/apply the non-destructive compatibility migration; no historical-row updates.
- Rust integration/domain tests: replace negotiated-input fixtures and add migration/readback cases.

## Failure conditions and transaction result

| Condition | Error code | Persistence result |
| --- | --- | --- |
| Unknown/legacy authority field in JSON | `invalid_request` | command rejected before transaction |
| Invalid UUID, empty lines, duplicate product, unsafe/invalid shape | `invalid_request` (quantity retains `invalid_quantity`) | no transaction effects |
| Product missing or inactive | `missing_product` / `inactive_product` | full rollback |
| Arithmetic overflow or persisted money outside domain range | `persistence_failure` for corrupted persistence; `invalid_request` only for malformed input | no partial effects |
| QR greater than authoritative total | `invalid_payment` | full rollback |
| Required cash tender missing or insufficient | `invalid_payment` | full rollback |
| Positive cash tender when QR covers the total | `invalid_payment` | full rollback |
| Insufficient stock | `insufficient_stock` | lines, payments, balances, and movements roll back |
| Existing confirmed request ID | success with original summary | no new writes or stock change |
| Existing non-confirmed/corrupt reservation | `persistence_failure` | no new writes |
| SQLite write/readback/commit failure | `persistence_failure` | transaction rollback where SQLite permits; no success returned |
| Version-2 preflight failure or future schema version | database open failure | database content and version unchanged |

## Focused test strategy

### Domain tests

- `SaleLine::priced` calculates checked totals and rejects overflow.
- `PaymentBreakdown::derive` covers cash-only exact/with change, QR-only, mixed exact/with cash change, QR over total, missing cash, insufficient cash, and unexpected cash after full QR.
- `Sale` independently rejects an applied sum that differs from total.

### Rust application/SQLite integration tests

Use in-memory SQLite with foreign keys enabled and real migrations:

- confirmation ignores draft display price by changing the product catalog price before confirmation and asserting the newly resolved persisted price;
- changing catalog price after confirmation does not change summary readback;
- retry with the same request ID after a catalog change returns byte-equivalent business facts, keeps one sale/line/payment/movement set, and deducts stock once;
- a failed attempt can retry with the same request ID because the pending reservation rolled back;
- cash-only, QR-only, and mixed rows contain exact applied/tendered/change facts;
- QR overpayment and insufficient tender leave counts and stock balances unchanged;
- a later-line stock failure rolls back earlier line/payment/movement writes;
- missing/inactive products and duplicate product lines have no effects;
- persisted summary is loaded from SQLite and exposes `unit_price_centavos`, not the current product price;
- movement update/delete triggers and request-ID uniqueness remain enforced.

### Migration tests

Create a version-1 database fixture containing:

- a confirmed legacy negotiated sale whose negotiated price differs from its minimum snapshot;
- cash, QR, and mixed historical payments;
- request ID, stock balance, and inventory movement rows.

Then open it through production migration and assert:

- `user_version == 2`;
- all row counts, primary/foreign keys, request ID, stock quantities, movement deltas, and raw monetary columns are unchanged;
- the new summary reports the legacy negotiated value as historical `unit_price_centavos` and preserves payment facts exactly;
- a new fixed-price sale writes equal values to both physical sale-line price columns;
- reopening is idempotent;
- preflight failure leaves version `1` and all rows unchanged;
- a version greater than `2` is rejected without mutation;
- the previous application query/write shapes still operate against the migrated schema.

### Command-seam and React tests

- Rust JSON deserialization rejects negotiated price, payment rows, cash applied, and change fields.
- TypeScript sends exactly request ID, product/quantity, and nullable tender fields.
- unsafe/non-integer values fail before invoke.
- reducer has no editable line-price or derived-cash action and retains request ID after a failed confirmation.
- draft catalog price is read-only guidance; persisted summary replaces it with the authoritative stored price and full payment breakdown.

## Rollout

1. Ship version-2 migration handling and the complete vertical contract change together; mixed old UI/new backend contracts are unsupported.
2. On application start, migrate before exposing checkout. If migration fails, block confirmation and surface a database-open failure; never fall back to client-authoritative values.
3. Do not rewrite or batch-update historical sale data during rollout.
4. Roll back by installing the previous application while leaving the version-2 database untouched. The physical schema remains version-1 compatible.
5. If a future change renames or constrains physical columns, it requires a separate table-rebuild migration and cannot reuse this rollback guarantee.
