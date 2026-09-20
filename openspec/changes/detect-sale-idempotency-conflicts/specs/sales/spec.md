# Delta for Sales

## ADDED Requirements

### Requirement: Versioned Sale Identity Compatibility

The system MUST preserve canonical sale identity versions v1 and v2. Both versions MUST persist operation kind `confirm_sale`, their own payload version, a deterministic canonical payload, and its matching lowercase SHA-256 digest. Version v1 MUST retain the established line, acknowledgement, and payment identity fields. Version v2 MUST additionally encode the explicit nullable negotiated final unit price for each line. A persisted identity MUST be validated against its stored version before comparison; a persisted v1 identity MUST NOT be reinterpreted as v2.

#### Scenario: Preserve an exact v1 replay

- GIVEN a confirmed sale stores a valid `confirm_sale` v1 identity
- WHEN the same request ID and the same v1 identity are submitted again
- THEN the system returns the original persisted sale
- AND it performs no sale, payment, stock, or inventory write

#### Scenario: Preserve an exact v2 replay

- GIVEN a confirmed sale stores a valid `confirm_sale` v2 identity including negotiated final-price nullability
- WHEN the same request ID and the same v2 identity are submitted again
- THEN the system returns the original persisted sale
- AND it performs no sale, payment, stock, or inventory write

#### Scenario: Do not reinterpret a stored v1 identity as v2

- GIVEN a request ID belongs to a confirmed sale with a valid persisted v1 identity
- WHEN the retry requires the v2 identity contract
- THEN the system does not reinterpret or upgrade the stored v1 payload
- AND it returns the bounded `persistence_failure` outcome without changing persisted facts

### Requirement: Legacy Sale Identity Fail-Closed Policy

The system MUST NOT backfill or reconstruct canonical identities for historical sale rows. A persisted sale whose operation kind, payload version, canonical payload, and digest are all null MUST be treated as an unverifiable legacy identity and reused request IDs MUST return `request_conflict`. A persisted sale with any partial, malformed, unsupported, or digest-inconsistent identity MUST return `persistence_failure`.

#### Scenario: Reject an all-null legacy identity as a conflict

- GIVEN a confirmed sale created before sale identity migration has all four identity columns null
- WHEN its request ID is submitted through sale confirmation
- THEN the system returns `request_conflict`
- AND the historical sale, lines, payments, stock, and inventory movements remain unchanged

#### Scenario: Fail closed for partial or malformed persisted identity

- GIVEN a persisted sale has only some identity columns, an unsupported version, an invalid canonical payload, or a digest that does not match the payload
- WHEN its request ID is submitted through sale confirmation
- THEN the system returns the bounded `persistence_failure` outcome
- AND it does not repair, backfill, reinterpret, or rewrite the persisted identity or sale facts

### Requirement: Pre-v11 Migration Replay Regression

The production database migration path MUST preserve a confirmed sale created before migration 0011 without fabricating identity values. After migration to the current schema, reusing that sale's request ID MUST fail closed with `request_conflict` through the production confirmation path.

#### Scenario: Migrate a pre-v11 confirmed sale without enabling an unverifiable replay

- GIVEN a pre-v11 database contains a confirmed sale and its request ID
- WHEN the production database is opened and migrated to the current schema
- THEN the sale remains readable and its four identity columns remain null
- WHEN confirmation reuses that request ID
- THEN the production path returns `request_conflict`
- AND no new sale, line, payment, stock, or inventory fact is persisted
