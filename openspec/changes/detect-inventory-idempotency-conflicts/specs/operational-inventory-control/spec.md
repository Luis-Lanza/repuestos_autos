# Delta for Operational Inventory Control

## ADDED Requirements

### Requirement: Request-Bearing Inventory Identity Scope

Only request-bearing `stock_entry` and `adjustment` operations MUST require a request identity and canonical payload identity. The adjustment operation is the physical-count operation whose persisted movement type is `adjustment`. Opening-stock, sale, return, and cancellation movements MUST retain their existing contracts and MUST NOT be required to provide this identity by this change.

#### Scenario: Require identity for a stock entry

- GIVEN a request-bearing `stock_entry` operation
- WHEN it is persisted
- THEN it has a request ID and a complete versioned canonical identity

#### Scenario: Require identity for an adjustment

- GIVEN a request-bearing `adjustment` operation
- WHEN it is persisted
- THEN it has a request ID and a complete versioned canonical identity

#### Scenario: Preserve unrelated movement contracts

- GIVEN an opening-stock, sale, return, or cancellation movement
- WHEN it is persisted or read
- THEN this change does not require inventory idempotency identity fields for that movement
- AND its existing movement contract remains unchanged

### Requirement: Version 1 Inventory Canonical Identity

Each newly persisted request-bearing inventory operation MUST persist operation kind, payload version `1`, the canonical payload, and a matching lowercase hexadecimal SHA-256 digest. Version 1 MUST be a Rust-owned deterministic UTF-8 length-delimited encoding with this fixed field order: `inventory/v1` marker, operation kind, product ID, requested quantity or count, and operation-specific data. A stock entry MUST encode the exact optional-note state (`null` or `value` followed by the note); an adjustment MUST encode its domain-normalized nonblank reason. The request ID, prior balance, resulting balance, calculated delta, timestamp, source reference, and other persisted effects MUST be excluded from the identity.

#### Scenario: Persist a stable stock-entry identity

- GIVEN a valid stock entry with a product, positive whole-unit quantity, and optional note
- WHEN its identity is built
- THEN the payload uses the version-1 fixed field order and length-delimited UTF-8 encoding
- AND the digest is SHA-256 of that payload rendered in lowercase hexadecimal
- AND changing only the request ID does not change the identity

#### Scenario: Persist a stable adjustment identity

- GIVEN a valid adjustment with a product, non-negative count, and normalized nonblank reason
- WHEN its identity is built
- THEN the payload includes the operation kind, product ID, requested count, and normalized reason
- AND it uses payload version `1` and the matching lowercase SHA-256 digest

#### Scenario: Exclude derived effects from identity

- GIVEN two requests with the same operation-specific input identity
- WHEN their prior balance, calculated delta, resulting balance, timestamp, or source reference differs
- THEN those differences do not make the canonical identities different

### Requirement: Exact Replay and Request Conflict

The inventory repository MUST resolve an existing request ID before performing a new inventory calculation or write. The original persisted result MUST be returned as an exact replay only when the request ID and the complete stored operation kind, payload version, canonical payload, and digest match the submitted identity. Reusing the request ID with a different complete identity MUST return the stable typed `request_conflict` outcome and MUST create no movement or stock effect.

#### Scenario: Replay an exact request

- GIVEN a request-bearing inventory operation has committed with a complete valid identity
- WHEN the same request ID and identical identity are submitted again, before or after restart
- THEN the original persisted movement and result are returned
- AND no new movement, balance update, calculation-dependent mutation, or other write occurs

#### Scenario: Reject a changed request identity

- GIVEN a request ID is already associated with a valid persisted inventory operation
- WHEN the same request ID is submitted with a changed product, requested value, optional-note state, normalized reason, operation kind, canonical payload, or digest
- THEN the operation returns `request_conflict`
- AND stock balance and movement history remain unchanged

### Requirement: Fail-Closed Legacy and Malformed Identities

The system MUST NOT backfill or reconstruct identity fields for historical movements. When all four identity fields are null—operation kind, payload version, canonical payload, and digest—a request-ID reuse MUST return `request_conflict`. Any partial, malformed, unsupported, non-canonical, or digest-inconsistent persisted identity MUST return bounded `persistence_failure`. Neither outcome may repair, reinterpret, or rewrite the persisted movement.

#### Scenario: Reject an all-null legacy identity

- GIVEN a request-bearing `stock_entry` or `adjustment` request ID belongs to a legacy movement whose four identity fields are all null
- WHEN the request ID is reused
- THEN the operation returns `request_conflict`
- AND the legacy movement, balance, and request facts remain unchanged

#### Scenario: Reject partial or malformed identity data

- GIVEN a persisted identity has missing fields, an unsupported version or operation kind, malformed canonical bytes, non-canonical fields, or a digest that does not match
- WHEN its request ID is reused
- THEN the operation returns `persistence_failure`
- AND no identity or inventory fact is repaired, backfilled, or rewritten

### Requirement: Atomic Rollback and No-Write Guarantees

Inventory transaction ownership MUST remain in the repository. Validation failures, identity conflicts, malformed persisted identities, persistence failures, and failed commits MUST leave no partial movement, balance change, or result. Conflict and failure paths MUST be settled before any new inventory effect is committed.

#### Scenario: Roll back a failed inventory operation

- GIVEN validation or persistence fails while confirming a request-bearing operation
- WHEN the operation ends
- THEN the transaction rolls back
- AND no movement, balance change, identity, or result remains from that attempt

#### Scenario: Keep a conflict side-effect free

- GIVEN an existing request ID produces `request_conflict` or `persistence_failure`
- WHEN the command returns
- THEN no new inventory calculation or write is committed
- AND the existing movement, stock balance, and identity remain unchanged

### Requirement: Stable Inventory Conflict Mapping

The Tauri inventory commands MUST expose `request_conflict` and `persistence_failure` as stable public error codes. The TypeScript inventory adapter MUST allowlist and preserve those codes while replacing native, database, canonical-payload, and digest details with bounded neutral messages. The Inventory UI MUST never present a conflict as success and MUST show a specific neutral retry instruction for an already-used request ID; persistence failures MUST use the generic inventory-operation failure message.

#### Scenario: Map a request conflict through IPC and UI

- GIVEN confirmation reuses a request ID with different inventory data
- WHEN the Rust command, TypeScript adapter, and Inventory UI handle the result
- THEN IPC exposes code `request_conflict`
- AND the adapter preserves that code without exposing backend details
- AND the UI tells the operator that the request ID was already used with different inventory data
- AND no success result or stock refresh is published

#### Scenario: Bound a persistence failure through IPC and UI

- GIVEN confirmation encounters malformed identity data or a persistence failure
- WHEN the result crosses IPC and reaches the Inventory UI
- THEN IPC exposes code `persistence_failure`
- AND the adapter and UI use a generic inventory-operation failure message
- AND database, payload, digest, and native error details are not exposed

### Requirement: Deferred Concurrent Insert-Race Verification

The unique-insert fallback for concurrent requests MUST remain outside this ticket's verification scope. A dedicated test that creates a concurrent unique-insert race and verifies the fallback identity decision is deferred to follow-up work; this deferral MUST NOT weaken the initial request-ID lookup, exact-replay, conflict, legacy, malformed-identity, or rollback guarantees defined here.

#### Scenario: Record the deferred race test

- GIVEN verification is performed for this change
- WHEN the inventory idempotency tests are selected
- THEN the concurrent unique-insert fallback test is recorded as deferred follow-up work
- AND the focused non-concurrent inventory verification remains required
