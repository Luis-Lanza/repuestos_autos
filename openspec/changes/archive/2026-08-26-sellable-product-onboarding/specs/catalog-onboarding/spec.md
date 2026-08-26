# Catalog Onboarding Specification

## Purpose

Retrospective; SDD did not guide implementation.

## Requirements

### Requirement: Fields

Staff MUST create/list categories. Fields MUST specify name, type (text/number/option), and required/optional status. Values MUST match; required supplied, optional MAY be absent, options defined.

#### Scenario: Valid values

- GIVEN typed fields
- WHEN staff submits matching values
- THEN values accepted

#### Scenario: Invalid values

- GIVEN a value is missing, wrongly typed, or an unknown option
- WHEN onboarding is requested
- THEN rejected; no result persists

### Requirement: Constraints

Onboarding MUST create active one-category product with name, unique SKU, positive integer-centavo price, and positive whole-unit stock.

#### Scenario: Invalid product

- GIVEN SKU exists, price is non-positive, or stock is not positive whole units
- WHEN onboarding is requested
- THEN rejected; catalog and inventory are unchanged

### Requirement: Opening

Onboarding MUST atomically persist product, values, balance, and exactly one immutable, timestamped positive `opening_stock` movement. Failure MUST leave them unchanged.

#### Scenario: Valid persistence

- GIVEN onboarding rules pass
- WHEN onboarding succeeds
- THEN all persist with one positive timestamped movement

#### Scenario: Rollback

- GIVEN persistence fails during onboarding
- WHEN the operation ends
- THEN none remains

### Requirement: Result

On success, the system MUST return persisted result and make product immediately searchable globally by name, SKU, category, or configured value. It MUST remain compatible with fixed-price checkout/history snapshots.

#### Scenario: Immediate sale

- GIVEN onboarding returned successfully
- WHEN staff searches and confirms a sale
- THEN found, sold at catalog price, and price retained

### Requirement: Scope

It MUST NOT add editing/archiving, Excel import, images/barcodes, suppliers/purchase costs, fractional inventory, accounts/roles, cloud sync, or multi-store operation. It MUST NOT change checkout pricing/payment, idempotency, or history.

#### Scenario: Exclusions

- GIVEN an excluded workflow is requested
- WHEN the onboarding capability is used
- THEN it is unavailable and checkout/history remain unchanged
