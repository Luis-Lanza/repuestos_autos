# Sales Specification

## Purpose

Provide the first offline point-of-sale flow for the store operator: discover active seeded products, build a valid cart with catalog-derived prices, accept cash and/or QR payment, and confirm a sale with consistent persisted sales and stock records.

## Requirements

### Requirement: Active Product Search and Cart

The system MUST provide one global search entry point over the seeded active catalog. Search results MUST expose enough information to select a product and assess its availability, including name, SKU, category, available whole-unit stock, and current catalog price. The operator MUST be able to add an available product to a draft cart and remove it without changing persisted stock.

#### Scenario: Search and add an active product

- GIVEN the seeded catalog contains an active product
- WHEN the operator searches using a matching product name, SKU, category, or searchable category field
- THEN the system shows the matching active product with its current catalog price
- AND the operator can add that product to the draft cart

#### Scenario: Archived or inactive products cannot be sold

- GIVEN a product is not active
- WHEN the operator searches or attempts to confirm a cart containing that product
- THEN the product is not available for sale
- AND no sale or stock effect is persisted

#### Scenario: Discarding or removing a draft cart line

- GIVEN a product is in the draft cart
- WHEN the operator removes the line or discards the draft sale
- THEN the draft no longer contains that line
- AND persisted stock, sales, payments, and inventory movements remain unchanged

### Requirement: Whole-Unit Quantities and Fixed Catalog Price

The system MUST accept only positive whole-number quantities for sale lines. A newly added line MUST display the product's current catalog price, but the operator MUST NOT be able to edit that price. At confirmation, the backend MUST resolve the authoritative current catalog price and persist it as the sale line's historical price snapshot. Catalog management may update a product's price only for future sales; it MUST NOT alter confirmed sale lines.

#### Scenario: Resolve and persist the catalog price at confirmation

- GIVEN an active product has a current catalog price
- WHEN the operator adds it to the cart
- THEN the line quantity is a positive whole unit
- AND the line displays the current catalog price without allowing an operator price edit
- WHEN the operator confirms a valid sale
- THEN the backend resolves the authoritative current catalog price
- AND the persisted sale line contains that resolved price as its historical snapshot

#### Scenario: Catalog price updates affect only future sales

- GIVEN a confirmed sale line has a persisted catalog-price snapshot
- WHEN catalog management changes the product's catalog price
- THEN the confirmed sale line retains its original snapshot
- AND a later confirmed sale uses the new catalog price

#### Scenario: Reject fractional, zero, or negative quantity

- GIVEN a product is in the draft cart
- WHEN the operator enters a fractional, zero, or negative quantity
- THEN the system rejects the quantity
- AND confirmation cannot persist a sale or stock effect

### Requirement: Payment Integrity

The system MUST support cash-only, QR-only, and mixed cash-and-QR payments. The confirmation request MUST contain only tendered cash and an optional QR-applied amount; the backend MUST derive cash applied and change from the authoritative sale total after applying QR. Every payment line MUST persist its applied amount, and cash MUST additionally persist tendered amount and derived change. All monetary values MUST be evaluated as integer centavos of Bs. QR applied plus derived cash applied MUST equal the sale total exactly; QR applied MUST be non-negative and MUST NOT exceed the sale total; tendered cash MUST cover the remaining amount; and `amount_tendered - amount_applied = change_given`.

#### Scenario: Confirm a cash-only sale with derived change

- GIVEN the cart total is a positive amount
- WHEN the operator enters tendered cash greater than the cart total and no QR amount
- THEN the backend derives cash applied equal to the cart total
- AND derives change equal to tendered cash minus the cart total
- AND persists the cash payment with applied, tendered, and change amounts in integer centavos

#### Scenario: Confirm a QR-only sale

- GIVEN the cart total is a positive amount
- WHEN the operator submits a QR amount equal to the total and no cash payment
- THEN the system confirms the sale
- AND persists the QR applied amount equal to the total
- AND persists no cash tender or change

#### Scenario: Confirm a mixed sale

- GIVEN the cart total is a positive amount
- WHEN the operator provides a QR amount less than the total and tendered cash covering the remainder
- THEN the backend derives cash applied as `total - QR applied`
- AND derives change as `tendered cash - cash applied`
- AND persists both payment lines with their authoritative details

#### Scenario: Reject QR overpayment

- GIVEN the cart total is a positive amount
- WHEN the submitted QR amount exceeds the total
- THEN confirmation is rejected
- AND no sale, payment, stock deduction, or inventory movement is persisted

#### Scenario: Reject insufficient tender for the remaining amount

- GIVEN the submitted QR amount is no greater than the total
- WHEN tendered cash is less than `total - QR applied`
- THEN confirmation is rejected
- AND no sale, payment, stock deduction, or inventory movement is persisted

### Requirement: Atomic Sale Confirmation and Stock Integrity

The system MUST confirm a sale, its lines, its payment lines, stock balance decrements, and one immutable negative inventory movement for each confirmed line as one atomic operation. Stock MUST never become negative. If any validation or stock requirement fails, the complete confirmation MUST be rolled back.

#### Scenario: Persist a successful multi-line confirmation atomically

- GIVEN a cart contains multiple active products with valid positive whole-unit quantities and sufficient stock
- AND the payment amounts exactly cover the sale total
- WHEN the operator confirms the sale
- THEN one confirmed sale and all of its lines and payments are persisted
- AND each line's stock balance is decremented by its quantity
- AND one immutable negative inventory movement is persisted for each line
- AND all persisted effects belong to the same successful confirmation

#### Scenario: Roll back when one line lacks sufficient stock

- GIVEN a multi-line cart has sufficient stock for some lines but insufficient stock for another line
- WHEN the operator confirms the sale
- THEN confirmation is rejected for insufficient stock
- AND no sale header, sale line, payment line, inventory movement, or partial stock decrement remains persisted
- AND no stock balance becomes negative

### Requirement: Idempotent Confirmation

The system MUST generate and retain one UUID `request_id` for each sale intent. Repeated confirmation attempts for that intent MUST reuse the same request ID. Once a request ID has a persisted sale, a retry MUST return that persisted sale and MUST NOT resolve a new catalog price, recalculate payment values, or duplicate any sale, line, payment, inventory movement, or stock deduction.

#### Scenario: Retry after a catalog price changes

- GIVEN a sale was confirmed with a retained request ID and its line price was persisted
- WHEN the catalog price changes and the operator retries with the same request ID
- THEN the system returns the original persisted sale and payment breakdown
- AND the returned historical price and payment values are unchanged
- AND no duplicate sale data or additional stock deduction is created

#### Scenario: Retry an unsuccessful attempt

- GIVEN an attempt with a request ID failed before persistence
- WHEN the operator retries the same intent with that request ID and valid inputs
- THEN the system may confirm one sale using the price resolved for the successful attempt
- AND it creates no duplicate records for that request ID

### Requirement: Persisted Sale Summary

The system MUST return a sale summary reconstructed from persisted records after a successful confirmation or idempotent retry. The summary MUST include the sale identity, UUID request ID, status, timestamp, line items, product information, positive whole-unit quantities, sale-time catalog-price snapshots, payment breakdown, total in Bs, and confirmation outcome.

#### Scenario: Display the persisted summary after confirmation

- GIVEN a sale confirmation succeeds
- WHEN the confirmation response is received
- THEN the operator can view the persisted sale summary
- AND the summary contains the sale identity, request ID, status, timestamp, products, quantities, sale-time catalog prices, payment breakdown, and total formatted in Bs

#### Scenario: Reconstruct the same summary on retry

- GIVEN a sale already exists for the submitted request ID
- WHEN the operator retries confirmation
- THEN the system returns a summary reconstructed from the persisted sale records
- AND it represents the same sale details and confirmation outcome as the original response

### Requirement: Confirm-Sale Scope Exclusions

The confirm-sale slice MUST remain limited to seeded catalog discovery, draft cart creation, sale confirmation, cash/QR payment recording, and atomic sales-and-stock persistence. It MUST NOT require product-management workflows, stock entry or manual adjustment, returns, cancellation, reports, backup or restore, licensing enforcement, user accounts or roles, customer accounts, invoicing, payment gateways, barcode hardware, cloud services, synchronization, multi-store support, or fractional/measured quantities.

#### Scenario: Product-management workflows are unavailable in this slice

- GIVEN the operator is using the confirm-sale flow
- WHEN the operator needs to create, edit, archive, or import a product or category
- THEN the confirm-sale slice provides no such operation
- AND seeded data remains the only catalog source for this slice

#### Scenario: External and future workflows do not affect confirmation

- GIVEN the application is operating offline
- WHEN the operator confirms an in-scope cash, QR, or mixed-payment sale
- THEN confirmation does not require a network service, licensing check, payment gateway, account, invoice, barcode device, or multi-store synchronization
- AND returns, cancellations, stock adjustments, reports, and backup/restore are not performed as part of confirmation

### Requirement: Confirmation Inputs Exclude Negotiated Values

The confirmation request MUST accept product identity and positive whole-unit quantity for each line, plus tendered cash and an optional QR-applied amount. It MUST NOT accept operator-supplied line prices, cash-applied amounts, or change values as authoritative inputs. The backend MUST resolve prices and derive payment values.

#### Scenario: Reject negotiated price and derived-payment authority

- GIVEN a checkout request contains an operator-supplied line price, cash-applied amount, or change value
- WHEN the request is submitted for confirmation
- THEN the system rejects the request or ignores those fields without using them as authority
- AND no sale, payment, stock deduction, or inventory movement is persisted

### Requirement: Migration and Legacy Compatibility

The system MUST define and apply an explicit SQLite migration and backward-compatibility policy before rollout. Existing confirmed sales and payment records MUST remain readable without repricing, deletion, or loss of request-ID, stock, movement, line, or payment history. The policy MUST define legacy defaults or nullability, constraint rollout, and rollback behavior.

#### Scenario: Open an existing database after migration

- GIVEN a database contains confirmed sales created under the previous schema
- WHEN the migrated application reads those sales
- THEN it returns a compatible persisted summary without changing their historical prices or payment facts
- AND existing stock balances, inventory movements, and request-ID uniqueness remain intact

#### Scenario: Roll back application behavior after migration

- GIVEN the migration has completed and the previous application contract must be restored
- WHEN rollback is performed according to the documented compatibility policy
- THEN confirmed-sale history and stock integrity are preserved
- AND the previous application can operate without interpreting newly introduced data as a different historical sale
