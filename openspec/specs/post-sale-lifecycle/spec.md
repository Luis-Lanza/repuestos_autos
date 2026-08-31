# Post-sale Lifecycle Specification

## Purpose

Define the observable lifecycle for correcting a confirmed sale through item returns and whole-sale cancellation while preserving the sale's original historical facts and restoring inventory through additive correction history. This slice records inventory corrections only; it does not perform monetary settlement.

## Requirements

### Requirement: Multi-line Returns

The system MUST allow an operator to create one return from a Sales History sale detail containing one or more original sale lines. Each requested quantity MUST be a positive whole number and MUST be no greater than that line's remaining returnable quantity. Return eligibility MUST be determined by the original sale-line identity, not by product identity alone.

#### Scenario: Accept partial quantities from multiple original lines

- GIVEN a confirmed sale has two original lines, including any repeated product identity as distinct sale-line records
- WHEN the operator submits positive whole-unit quantities for both lines within each line's remaining returnable quantity
- THEN the system accepts both quantities in one return and reports the accepted quantity associated with each exact original sale-line identity

#### Scenario: Reject invalid or unavailable quantities without partial effects

- GIVEN a confirmed sale has returnable original lines
- WHEN the operator submits a zero, negative, fractional, unknown-line, or over-remaining quantity in a multi-line return
- THEN the system rejects the entire return
- AND no return record, stock increase, or correction movement is observable for any line in that request

#### Scenario: Return cannot exceed the sold quantity

- GIVEN an original sale line has prior accepted returns
- WHEN a new return requests more than the line's remaining returnable quantity
- THEN the system rejects the request and leaves all persisted facts and stock unchanged

### Requirement: Atomic Additive Stock Restoration

For every accepted positive return quantity, the system MUST increase the corresponding stock balance by exactly that quantity and MUST record exactly one immutable positive inventory movement linked to the original sale and original sale line. The return records, stock increases, and movements MUST become visible together or none of them may remain visible.

#### Scenario: Commit return records and stock restoration together

- GIVEN a valid multi-line return for two original sale lines
- WHEN the return is accepted
- THEN each accepted line has its return detail
- AND each accepted quantity has the exact matching stock increase
- AND each accepted quantity has one immutable positive movement linked to its original line

#### Scenario: Roll back all return effects on persistence failure

- GIVEN a valid return would affect multiple original sale lines
- WHEN persistence fails before the operation completes
- THEN no part of the return is persisted
- AND no stock balance or inventory movement from that return is observable

### Requirement: Whole-sale Cancellation and Residual Restoration

The system MUST allow a confirmed sale to be cancelled once with a non-blank cancellation reason, including after accepted returns. Cancellation MUST restore only the residual quantity not already returned for each original sale line. A fully returned sale MUST be cancellable as a status and audit event with zero stock restoration.

#### Scenario: Cancel before any returns

- GIVEN a confirmed sale has not received any accepted returns
- WHEN the operator submits a non-blank cancellation reason
- THEN the sale becomes cancelled
- AND each original line's sold quantity is restored exactly once
- AND the cancellation detail identifies the restored quantities

#### Scenario: Cancel after partial returns

- GIVEN a confirmed sale has accepted returns for some original line quantities
- WHEN the operator submits a non-blank cancellation reason
- THEN the sale becomes cancelled
- AND each line's restored quantity equals its sold quantity less its accepted returned quantity
- AND no already-returned unit is restored again

#### Scenario: Cancel a fully returned sale

- GIVEN every original sale line has no remaining returnable quantity
- WHEN the operator submits a non-blank cancellation reason
- THEN the sale becomes cancelled and the cancellation audit event is retained
- AND stock does not increase because the residual quantity is zero

#### Scenario: Reject invalid cancellation attempts without mutation

- GIVEN a sale is confirmed
- WHEN the operator submits a blank or whitespace-only reason
- THEN cancellation is rejected with no status, audit, stock, or movement change
- GIVEN a sale is already cancelled
- WHEN the operator submits another cancellation
- THEN the system rejects it without additional restoration or cancellation records

### Requirement: Idempotent Replay and Request Conflicts

Each return or cancellation intent MUST require a stable request identity and a canonical payload identity. Reusing the same request identity with the same canonical payload MUST replay the originally persisted outcome without creating duplicate records, movements, or stock changes. Reusing that identity with a different payload MUST return a stable conflict and MUST write nothing.

#### Scenario: Replay a committed return

- GIVEN a return request has committed with a request identity and canonical payload
- WHEN the same request identity and canonical payload are submitted again
- THEN the system returns the same persisted outcome
- AND return records, movements, and stock balances are unchanged by the replay

#### Scenario: Replay a committed cancellation

- GIVEN a cancellation request has committed with a request identity and canonical payload
- WHEN the same request identity and canonical payload are submitted again
- THEN the system returns the same persisted cancellation outcome
- AND no additional cancellation, restoration, or movement is created

#### Scenario: Reject conflicting request reuse

- GIVEN a request identity is already associated with a persisted operation
- WHEN that identity is submitted with a different canonical payload
- THEN the system returns a stable request-conflict result
- AND no record, status, movement, or stock balance is changed

### Requirement: Concurrent Eligibility and No Double Restoration

The system MUST evaluate sale eligibility, prior accepted returns, cancellation state, and residual quantities against a consistent persisted state so overlapping correction requests cannot cause cumulative restored quantities to exceed the original sold quantity.

#### Scenario: Overlapping returns do not over-return

- GIVEN two return requests concurrently target the remaining quantity of the same original sale line
- WHEN both requests complete
- THEN the sum of accepted return quantities is no greater than the original sold quantity
- AND any rejected request has no partial effects

#### Scenario: Overlapping cancellation and return do not double-restore

- GIVEN a confirmed sale has residual returnable quantity
- WHEN a cancellation and a return overlap for that sale
- THEN the committed correction history is internally consistent
- AND cumulative returned quantity plus cancellation-restored quantity for every original line is no greater than its sold quantity
- AND stock increases exactly match the committed correction quantities

### Requirement: Immutable Original Sale Facts

The system MUST preserve the original sale's line quantities, product snapshots, sale-time prices, totals, and payment records as immutable historical facts. Returns and cancellation MUST add correction history without rewriting or deleting those original facts.

#### Scenario: Original facts remain unchanged after corrections

- GIVEN a confirmed sale has original line snapshots, quantities, totals, and payment records
- WHEN one or more returns and a cancellation are accepted
- THEN the original facts remain logically identical to their pre-correction values
- AND the correction detail is available separately from those facts

#### Scenario: Correction references the original line

- GIVEN a sale contains multiple lines for the same product
- WHEN a return or cancellation restoration is accepted
- THEN each correction quantity can be traced to the exact original sale-line record
- AND no correction is attributed solely by product identity

### Requirement: Sales History Correction Visibility

Cancelled sales MUST remain visible in Sales History list and detail views. Sales History detail MUST show the lifecycle status, accepted return detail, cancellation-restored quantities, original line snapshots, and original payment facts. Historical access MUST remain read-only with respect to correction facts.

#### Scenario: List and detail retain a cancelled sale

- GIVEN a confirmed sale has been cancelled, with or without prior returns
- WHEN an operator opens Sales History
- THEN the sale remains discoverable in the list
- AND its detail identifies the cancelled status and all accepted correction quantities

#### Scenario: Detail distinguishes original and correction facts

- GIVEN a cancelled sale has original payments and at least one inventory correction
- WHEN the operator views its detail
- THEN original sale-time lines, snapshots, totals, and payments remain identifiable as original facts
- AND returned and cancellation-restored quantities are shown as correction detail

### Requirement: Inventory-correction Language and No Refund Semantics

The system MUST describe returns and cancellation as inventory corrections only. User-facing commands, responses, persisted correction detail, and Sales History presentation MUST NOT claim or imply a refund, reimbursement, payment reversal, credit, settlement, refund amount, or cash-drawer adjustment. Original payment facts MUST remain informational and unchanged.

#### Scenario: Correction copy does not imply reimbursement

- GIVEN an operator starts, accepts, retries, or views a return or cancellation
- WHEN the system presents the operation or its result
- THEN the language identifies inventory quantities restored or corrected
- AND it does not present a refund, reimbursement, reversal, credit, settlement, or amount returned to the customer

#### Scenario: Payments remain informational

- GIVEN a sale has one or more persisted payment records
- WHEN a return or cancellation is accepted
- THEN the payment records remain unchanged and visible as original payment facts
- AND no monetary correction record is created

### Requirement: Migration and Backup Compatibility

The change MUST preserve existing confirmed-sale data when an existing schema-v9 database is upgraded. The upgraded schema MUST remain readable by the application's existing confirmation and Sales History behavior, and backup/restore validation MUST accept databases containing the additive return and cancellation history without data loss or corruption. No new backup behavior is required beyond this compatibility.

#### Scenario: Upgrade an existing confirmed-sale database

- GIVEN a valid pre-change schema-v9 database containing confirmed sales, original lines, inventory balances, movements, and payments
- WHEN the database is opened after the forward migration
- THEN the existing records remain readable and unchanged
- AND existing sale confirmation and Sales History access continue to work
- AND the database can accept new lifecycle correction records

#### Scenario: Restore a database containing lifecycle history

- GIVEN an upgraded database contains returns, cancellation detail, and additive inventory movements
- WHEN it is backed up and restored using the existing backup/restore validation path
- THEN the restored database preserves the original sale facts and all correction history
- AND stock balances and movement history remain internally consistent
- AND the restored database remains readable by the application

#### Scenario: Feature rollback does not erase accepted facts

- GIVEN a migrated database has accepted correction history
- WHEN the lifecycle UI or commands are temporarily disabled
- THEN the additive schema and accepted correction records remain readable
- AND no original facts or immutable movements are deleted or rewritten

### Requirement: Explicit Scope Exclusions

This lifecycle MUST NOT introduce refunds, payment reversals, payment gateway operations, cash-drawer settlement, refund allocation, reports, analytics, exports, exchanges, replacement sales, credits, customer accounts, restocking fees, damaged-item disposition, actor or role attribution, return reasons, partial cancellation as a separate operation, original-sale editing or deletion, multi-store or cloud synchronization, fractional quantities, catalog changes, or new backup behavior beyond schema compatibility.

#### Scenario: Excluded workflows remain unavailable

- GIVEN an operator uses the post-sale lifecycle
- WHEN the operator completes a return or whole-sale cancellation
- THEN no refund, payment reversal, exchange, credit, customer-account, report, export, or settlement workflow is created or implied
- AND partial correction is represented only through item returns, not a separate partial-cancellation operation

#### Scenario: Original sale and catalog remain out of scope

- GIVEN a sale has accepted post-sale corrections
- WHEN the operator views or manages the sale
- THEN original sale lines, snapshots, totals, and payments cannot be edited or deleted by this lifecycle
- AND no catalog change, fractional quantity, actor attribution, or multi-store synchronization behavior is introduced
