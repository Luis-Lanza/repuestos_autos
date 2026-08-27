# Operational Inventory Control Specification

## Purpose

Provide offline stock entry, count reconciliation, immutable history, and active-product alerts.

## Requirements

### Requirement: Active Product Selection

The system MUST reuse Catalog global search for active products and MUST NOT duplicate semantics.

#### Scenario: Select an active product

- GIVEN an active product matches global search
- WHEN the operator selects it
- THEN inventory entry and adjustment are available

#### Scenario: Reject an inactive product

- GIVEN the selected product is inactive
- WHEN an operation is submitted
- THEN a stable error returns; balance/history are unchanged

### Requirement: Positive Stock Entry

An entry MUST accept positive whole-unit quantity and optional note. Confirmation MUST resolve authoritative balance in its transaction.

#### Scenario: Confirm an entry

- GIVEN an active product and valid quantity
- WHEN confirmed with an optional note
- THEN balance increases and the result includes the note

#### Scenario: Reject invalid entry

- GIVEN quantity is zero, negative, fractional, or overflows
- WHEN confirmed
- THEN a stable error returns and inventory is unchanged

### Requirement: Absolute Physical-Count Adjustment

An adjustment MUST accept absolute whole-unit target `>= 0` and non-blank reason. Its transaction MUST derive signed delta from current balance, reject no-op with a stable error, and preserve non-negative stock.

#### Scenario: Reconcile a count

- GIVEN the target differs from current balance
- WHEN confirmed with a valid reason
- THEN the signed delta applies and target balance persists

#### Scenario: Reject invalid adjustment

- GIVEN target is negative, fractional, equals current, or reason is blank
- WHEN confirmed
- THEN a stable error returns and balance/history are unchanged

### Requirement: Atomic Idempotent Inventory Operations

Both MUST use one retained UUID request ID. Success MUST atomically persist exactly one immutable movement, resulting balance, database-owned timestamp, and result. Retry MUST return it without mutation, including after restart.

#### Scenario: Retry a committed operation

- GIVEN a request ID has committed
- WHEN resubmitted before or after restart
- THEN the original result returns without a second movement or balance change

#### Scenario: Roll back failure

- GIVEN validation or persistence fails
- WHEN the operation ends
- THEN no movement, balance change, or result remains

### Requirement: Authoritative Concurrent Confirmation

Confirmation MUST recalculate adjustment delta from current balance, not preview; intervening operations MUST NOT be overwritten.

#### Scenario: Confirm after an intervening change

- GIVEN a preview used an earlier balance
- WHEN another operation commits first
- THEN confirmation uses the newer balance and preserves both operations

### Requirement: Derived Inventory Alerts

Alerts MUST be read-derived, never persisted, and include active products only: `0` is **Out of stock**, `1` is **Low stock**, and `>1` is absent. They MUST order out-of-stock, low-stock, then deterministic product order, refreshing after inventory operations and sales.

#### Scenario: Refresh and order alerts

- GIVEN an operation or sale changes balances
- WHEN alerts are read
- THEN labels, counts, filtering, and order are returned

### Requirement: Forward-Only Movement Compatibility

A forward-only migration MUST preserve valid legacy opening/sale movement IDs, timestamps, and facts. It MUST preflight invalid rows and composite sale/product links, fail before schema advancement, and enforce per-type signs, links, reasons, request IDs, and immutability.

#### Scenario: Reopen valid legacy history

- GIVEN legacy opening and sale movements are valid
- WHEN migration and restart occur
- THEN facts remain readable and correctly linked

#### Scenario: Reject incompatible history

- GIVEN a sign, link, reason, or composite relationship is invalid
- WHEN migration is attempted
- THEN the database is preserved and schema does not advance

### Requirement: Explicit Scope Exclusions

This capability MUST NOT change checkout cart-price defect, returns, cancellations, suppliers/costs, catalog editing, reports, backup/restore, roles, cloud, or multi-store.

#### Scenario: Keep excluded workflows separate

- GIVEN the operator uses inventory control
- WHEN an excluded workflow is requested
- THEN it is unavailable and existing capabilities remain unchanged
