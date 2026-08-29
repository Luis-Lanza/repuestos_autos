# Delta for Catalog Maintenance and Pricing

## ADDED Requirements

### Requirement: Metadata Edits and Validation

Staff MUST edit category names and product SKU, name, and typed values. Names/SKU MUST be non-blank and unique; typed values MUST match definitions. Reassignment, identity reuse, and hard deletion remain unavailable. Success returns revision.

#### Scenario: Reject invalid metadata
- GIVEN a value is blank, mistyped, or a name/SKU is duplicated after normalization
- WHEN staff submits the edit
- THEN a stable outcome returns and no catalog fact changes

### Requirement: Independent Lifecycle

Category and product state MUST be independent. Category archival MUST fail with active products and never cascade. Product archival MUST preserve stock, movements, and sales. Product reactivation requires active category and valid values; category reactivation never reactivates products.

#### Scenario: Enforce lifecycle boundaries
- GIVEN an active product blocks category archival or an archived category blocks product reactivation
- WHEN staff requests it
- THEN a stable lifecycle outcome returns with no state/inventory mutation

### Requirement: Pricing, Visibility, and Search Consistency

Prices MUST be integer centavos and affect later sales only. Sale search and inventory selection MUST include active products; maintenance views MUST expose archived records. Changes MUST atomically align fact, FTS/search, and audit.

#### Scenario: Reprice and refresh
- GIVEN a confirmed line has P and its product is searchable
- WHEN staff changes the catalog price to Q
- THEN the next search shows Q, later sales use Q, and the line remains P

#### Scenario: Reject partial refresh
- GIVEN a catalog update or FTS refresh fails
- WHEN the command ends
- THEN no update, partial search, or audit is observable

### Requirement: Revisions, Audit, and Stable Outcomes

Every mutation MUST require its expected revision. A stale revision MUST return opaque, stable `stale_catalog_record`; other failures MUST return stable outcomes. Each success MUST append exactly one immutable audit record with entity, operation, before/after, revision, and timestamp; records MUST NOT change or be deleted.

#### Scenario: Resolve concurrent edits
- GIVEN two commands read revision R
- WHEN both submit changes and one commits first
- THEN one succeeds, the other returns `stale_catalog_record`, and the loser changes neither fact nor audit

### Requirement: Migration, Restart, and Accessible States

Migration MUST be additive and transactional, preserve IDs, attributes, stock, movements, and confirmed sale facts, and fail before advancement for incompatible data. Restart MUST retain catalog, search, revision, and audit. UI MUST expose text/programmatic loading, unavailable, validation, conflict, stale-price, success, and failure states, with keyboard-reachable recovery.

#### Scenario: Migrate and reopen
- GIVEN valid legacy data and a completed migration
- WHEN the application restarts
- THEN preserved facts/state remain usable

#### Scenario: Surface recoverable conflict
- GIVEN a stale revision or cart price is detected
- WHEN the response renders
- THEN the UI announces the reason without color alone, blocks unsafe confirmation, and offers retry/reconfirm

## MODIFIED Requirements

### Requirement: Whole-Unit Quantities and Fixed Catalog Price

Sales MUST retain each draft price, compare it with the authoritative price at confirmation, and require acknowledgement plus reconfirmation when prices differ. Confirmed line prices MUST be immutable; repricing MUST never rewrite them.
(Previously: Sales used the catalog price without stale-cart acknowledgement.)

#### Scenario: Acknowledge stale price
- GIVEN a draft captured P and the authoritative price is Q
- WHEN confirmation occurs without acknowledgement
- THEN `stale_catalog_record` blocks the sale; acknowledgement and refreshed review permit reconfirmation at Q

### Requirement: Active Product Selection

Inventory MUST select active products only and preserve balances and immutable movements through lifecycle changes.
(Previously: Inventory selected active products without catalog lifecycle visibility.)

#### Scenario: Reject archived selection
- GIVEN the selected product is archived
- WHEN an inventory operation is submitted
- THEN a stable unavailable outcome returns and balance/history remain unchanged

## Non-Goals

Returns, reports, imports, promotions, costs, deletion, reassignment, scheduling, persisted carts, accounts, sync, and schema evolution are excluded.
