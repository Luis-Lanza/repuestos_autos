# Delta for Sales

## ADDED Requirements

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

## MODIFIED Requirements

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
