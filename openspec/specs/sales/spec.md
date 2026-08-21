# Sales Specification

## Purpose

Provide the first offline point-of-sale flow for the store operator: discover active seeded products, build a valid cart, accept cash and/or QR payment, and confirm a sale with consistent persisted sales and stock records.

## Requirements

### Requirement: Active Product Search and Cart

The system MUST provide one global search entry point over the seeded active catalog. Search results MUST expose enough information to select a product and assess its availability, including name, SKU, category, available whole-unit stock, and minimum sale price. The operator MUST be able to add an available product to a draft cart and remove it without changing persisted stock.

#### Scenario: Search and add an active product

- GIVEN the seeded catalog contains an active product
- WHEN the operator searches using a matching product name, SKU, category, or searchable category field
- THEN the system shows the matching active product with its available stock and minimum sale price
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

### Requirement: Whole-Unit Quantities and Price Floor

The system MUST accept only positive whole-number quantities for sale lines. Each newly added line MUST be prefilled with the product's current configured minimum sale price. The operator MAY enter a higher negotiated unit price, but the system MUST reject confirmation when any negotiated unit price is below the product's current minimum. A confirmed line MUST retain the negotiated unit price and the applicable minimum-price snapshot.

#### Scenario: Prefill and accept a valid negotiated price

- GIVEN an active product has a configured minimum sale price
- WHEN the operator adds it to the cart
- THEN the line quantity is a positive whole unit
- AND the line unit price is prefilled with the configured minimum
- WHEN the operator changes the unit price to a higher value and confirms a valid sale
- THEN the persisted sale line contains the higher negotiated price and the minimum-price snapshot

#### Scenario: Reject fractional, zero, or negative quantity

- GIVEN a product is in the draft cart
- WHEN the operator enters a fractional, zero, or negative quantity
- THEN the system rejects the quantity
- AND confirmation cannot persist a sale or stock effect

#### Scenario: Reject a price below the current minimum

- GIVEN a cart line's negotiated unit price is below the product's current configured minimum
- WHEN the operator attempts to confirm the sale
- THEN confirmation is rejected
- AND no sale, payment, inventory movement, or stock deduction is persisted

### Requirement: Payment Integrity

The system MUST support cash-only, QR-only, and mixed cash-and-QR payments. Every payment line MUST record the amount applied to the sale. Cash MUST additionally record amount tendered and change given. All monetary values MUST be evaluated as integer centavos of Bs. The sum of applied payment amounts MUST equal the sale total exactly, and cash values MUST be non-negative and satisfy `amount_tendered - amount_applied = change_given`.

#### Scenario: Confirm a cash-only sale with change

- GIVEN the cart total is a positive amount
- WHEN the operator submits one cash payment whose applied amount equals the total and whose tendered amount exceeds the applied amount by the recorded change
- THEN the sale is confirmed
- AND the persisted payment records the applied amount, tendered amount, and change in integer centavos

#### Scenario: Confirm a QR-only sale

- GIVEN the cart total is a positive amount
- WHEN the operator submits a QR payment whose applied amount equals the total
- THEN the sale is confirmed
- AND the persisted payment records the QR applied amount
- AND no cash tender or change is required

#### Scenario: Confirm a mixed cash-and-QR sale

- GIVEN the cart total is a positive amount
- WHEN the operator submits cash and QR payments whose applied amounts together equal the total
- AND the cash tendered amount, applied amount, and change are consistent
- THEN the sale is confirmed
- AND both payment lines are persisted with their respective details

#### Scenario: Reject unequal applied payments

- GIVEN the sum of applied payment amounts is less than or greater than the sale total
- WHEN the operator attempts to confirm
- THEN confirmation is rejected
- AND no sale, payment, inventory movement, or stock deduction is persisted

#### Scenario: Reject inconsistent cash values

- GIVEN a cash payment has a negative value or its tendered amount minus applied amount does not equal its change
- WHEN the operator attempts to confirm
- THEN confirmation is rejected
- AND no sale or stock effect is persisted

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

The system MUST generate and retain one UUID `request_id` for each sale intent when confirmation begins. Repeated clicks and retries for that intent MUST reuse the same request ID. The system MUST return the original persisted sale for a repeated request ID and MUST NOT duplicate any sale, line, payment, inventory movement, or stock deduction.

#### Scenario: Retry a successful confirmation

- GIVEN a sale was confirmed with a retained UUID request ID
- WHEN the operator retries confirmation using that same request ID
- THEN the system returns the original persisted sale
- AND the returned sale identity and request ID match the first confirmation
- AND sale, line, payment, movement, and stock counts remain unchanged after the retry

#### Scenario: Retain the request ID after an ambiguous or failed UI attempt

- GIVEN confirmation has begun for a sale intent
- WHEN the UI enters a pending or error state and the operator retries that same intent
- THEN the UI uses the retained request ID rather than generating a new one
- AND a successful retry cannot create a duplicate sale for that intent

### Requirement: Persisted Sale Summary

The system MUST return a sale summary reconstructed from persisted records after a successful confirmation or idempotent retry. The summary MUST include the sale identity, UUID request ID, status, timestamp, line items, product information, positive whole-unit quantities, negotiated unit prices, payment breakdown, total in Bs, and confirmation outcome.

#### Scenario: Display the persisted summary after confirmation

- GIVEN a sale confirmation succeeds
- WHEN the confirmation response is received
- THEN the operator can view the persisted sale summary
- AND the summary contains the sale identity, request ID, status, timestamp, products, quantities, negotiated prices, payment breakdown, and total formatted in Bs

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
