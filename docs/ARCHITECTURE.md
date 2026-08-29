# Version-1 Architecture

## Decision

Use a local **modular monolith**: one Windows desktop process, one SQLite database, and no network dependency for business operations.

## Implementation status

This document describes the target v1 architecture. Catalog onboarding and maintenance, fixed-price sale confirmation, operational inventory, and read-only sales history are implemented. Backup and restore are implemented with Fedora evidence; Windows task 4.1 evidence remains deferred. Returns, cancellation, and reporting remain planned: no current Rust application/command or UI implementation exists for them.

```text
React UI
   │ user intent (e.g. confirm sale)
   ▼
Tauri command boundary
   ▼
Rust application use cases
   ▼
Domain rules ────────────────┐
   ▼                          │
SQLite repositories           │
   ▼                          │
SQLite database ◀─────────────┘
```

## Responsibilities

| Layer | Owns | Must not own |
| --- | --- | --- |
| React + TypeScript | Screens, form state, local display validation, search presentation | Stock, payment, or catalog-price authority |
| Tauri command boundary | Typed request/response contract between UI and Rust | Business-rule duplication |
| Rust application | Use cases, transaction boundaries, orchestration, authoritative catalog-price resolution, error mapping | UI state or repository-owned transaction boundaries |
| Rust domain | Fixed sale price, derived cash payment values, stock availability, state transitions | SQLite or filesystem APIs |
| Infrastructure | SQLite repositories, migrations, backup/restore filesystem access, database constraints | Business decisions |

## Modules

```text
src/
  ui/                       React screens and components
src-tauri/src/
  commands/                 Tauri-facing input/output adapters
  application/
    catalog/                Categories, fields, products
    inventory/              Stock entry and adjustments
    sales/                  Implemented: sale confirmation, payments, and read-only history; planned: cancellation and return
    reporting/              Planned sales queries and aggregation
    backup/                 Implemented export and validated restore
    settings/               Local application settings
  domain/                   Business rules and domain types
  infrastructure/
    sqlite/                 Repositories and migrations
    filesystem/             Backup file operations
```

## SQLite data model

| Area | Principal tables | Key design choice |
| --- | --- | --- |
| Catalog | `categories`, `attribute_definitions`, `products`, `product_attribute_values` | Category fields are defined once and values are typed/searchable per product. |
| Stock | `inventory_movements`, `stock_balances` | Movements are immutable audit evidence; balances are a transactionally updated read model for fast availability checks. |
| Sales | `sales`, `sale_lines`, `sale_payments` | `sales.request_id` is unique for idempotency. At confirmation, a line stores the backend-resolved catalog price as its historical sale-time snapshot. Payments store applied amount; cash also stores operator-entered tendered amount and system-derived change. Read-only history queries return bounded calendar-filtered summaries and load persisted detail on demand. |
| Corrections (planned) | `sale_cancellations`, `returns`, `return_lines` | Reversals preserve the original sale and create compensating stock movements. |
| Audit metadata | Relevant operational records | `created_at`, `updated_at` where applicable, required `reason` for adjustments/cancellations, and nullable `operator_id` reserved for future attribution. |

Store all money as integer centavos of Bs, never floating-point values. Model money as a domain type, not a freely interchangeable integer.

All inventory quantities use positive integers because every product is sold by whole units. Fractional quantities and measured-goods support are out of scope.

## Critical transaction: confirm sale

`ConfirmSaleUseCase` owns the entire transaction. Repositories receive the transaction context; they never independently begin or commit transactions.

1. When confirmation starts, React generates and retains a UUID `request_id` for that sale intent. Repeated clicks and retries reuse it rather than generating another UUID.
2. Begin one SQLite transaction. Attempt to reserve the request ID by inserting the sale header with `request_id` protected by `UNIQUE` (for example, `INSERT ... ON CONFLICT DO NOTHING RETURNING id`).
   - If the insert succeeds, this transaction owns the new sale and continues.
   - If it conflicts, read and return the existing sale for that request ID without creating lines, payments, or stock movements.
3. Validate the products are active and quantities are valid. Resolve each product's authoritative current catalog price in the backend, calculate the sale total from those prices, and store each resolved price as the sale-line snapshot. Do not accept an operator-supplied line price.
4. For cash, accept tendered amount and derive the cash amount applied and change from the total after QR amounts. Validate non-negative values, sufficient tendered cash, and that derived cash applied plus QR amounts applied equals the sale total.
5. Insert the sale lines and payments for the newly reserved sale.
6. For each line, atomically decrement its balance inside the same transaction:

   ```sql
   UPDATE stock_balances
   SET quantity = quantity - :quantity
   WHERE product_id = :product_id
     AND quantity >= :quantity;
   ```

   If any update affects zero rows, report insufficient stock and roll back the complete transaction. Do not rely on a prior standalone `SELECT` availability check.
7. Append one negative immutable inventory movement per line.
8. Commit everything or roll back everything.

### Planned critical transaction: return or cancellation

`ReturnSaleUseCase` and `CancelSaleUseCase` are planned use cases. They would own their complete transactions and use the original sale lines as the source of truth for eligible quantities.

- A return is allowed only for a confirmed, non-cancelled sale. For each return line, the use case calculates `remaining_returnable = sold_quantity - sum(previously_returned_quantity)` and rejects any quantity above that amount before appending its positive stock movement.
- If a sale with prior returns is cancelled, cancellation restores only `sold_quantity - sum(previously_returned_quantity)` for each sale line. Therefore, a return and cancellation cannot restore the same sold unit twice.
- Returns, cancellations, and manual adjustments use the same application-owned transaction pattern: validate, create an immutable event, append movements, update balances, commit.

### Database invariants

SQLite is a second line of defense, not merely storage. Enable `PRAGMA foreign_keys = ON` for every connection and enforce foreign keys, uniqueness, and relevant checks such as positive quantities where required and non-negative monetary values. Application/domain validation remains authoritative for cross-record business rules.

## Backup boundary

The implemented backup service exports a database-safe snapshot selected by the operator to external storage. Restore validates the selected backup, requires confirmation, and replaces the local database only through a controlled shutdown/reopen flow. Fedora evidence exists; Windows task 4.1 evidence remains deferred.

## Explicit v1 boundaries

- No HTTP API, cloud, synchronization, server, or authentication service.
- No SQL issued from React.
- No inventory mutation without an immutable movement.
- The UI may retry a confirmation only with the same `request_id`; a new sale attempt requires a new UUID.
- If multi-computer or multi-store operation becomes necessary, introduce an API and central database as a separate architecture evolution rather than stretching the local database model.
