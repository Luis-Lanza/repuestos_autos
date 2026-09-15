# Change Proposal: Detect Inventory Idempotency Conflicts

## Intent

Make inventory idempotency safe to verify rather than treating every reuse of a request ID as a valid replay. A retry may return the original persisted inventory result only when the submitted inventory intent is exactly the same; reuse of the request ID for a different intent must fail closed with a stable public conflict outcome.

## Problem and Outcome

The current inventory idempotency record proves request-ID reuse but does not prove that a retried inventory operation represents the same request. A caller can therefore reuse a request ID with changed quantity, count, or reason and receive behavior intended only for an exact retry.

The outcome is bounded conflict detection at the existing `inventory_movements` persistence boundary. Exact replays return the original persisted movement/result without a duplicate movement or stock effect. Mismatched reuse returns typed public `request_conflict` without changing stock or movement history.

## Agreed Scope

- Extend the existing `inventory_movements` table; do not create a separate idempotency table.
- Add nullable, additive identity fields for operation kind, payload version, canonical payload, and lowercase SHA-256 digest.
- Define version 1 as a deterministic Rust-owned UTF-8 length-delimited identity. Request ID is separate from the identity and is excluded from its payload.
- Include operation kind and product ID in the identity.
- For stock entry, include the requested positive quantity and exact optional note.
- For physical count, include the requested non-negative count and the existing domain-normalized nonblank reason.
- Exclude prior balance, resulting balance, calculated delta, timestamp, source reference, and other persisted effects from the identity.
- Resolve an existing request ID before any new inventory calculation or write. An exact request-ID and identity match returns the original persisted movement/result with no new movement or stock effect.
- Return typed public `request_conflict` for the same request ID with a different identity, with no movement or stock effect.
- Treat legacy rows with no verifiable identity as fail-closed: reuse of their request ID returns `request_conflict`.
- Treat malformed or partial identity data as a bounded `persistence_failure`; do not guess or reconstruct identity equivalence.
- Keep transaction ownership in the inventory repository. Do not broaden this change into ticket 17 seam refactoring.
- Map the stable conflict at the Rust command boundary, decode it in TypeScript, and show a specific neutral message in the Inventory UI without database or digest details.
- Ensure the unique-insert race fallback performs the same identity validation as the initial replay lookup.

## Explicit Decisions

| Topic | Decision |
| --- | --- |
| Storage location | Add nullable identity columns to `inventory_movements`; no separate idempotency table. |
| Identity metadata | New inventory identities use the operation kind and payload version `1`. |
| Canonical form | Rust owns a deterministic version-1 UTF-8 length-delimited encoding with fixed field order. Request ID is separate and excluded. |
| Operation identity | Identity includes operation kind and product ID. Stock entry includes requested positive quantity and exact optional note; physical count includes requested non-negative count and the existing domain-normalized nonblank reason. |
| Digest | SHA-256 over the canonical payload, rendered as lowercase hexadecimal. |
| Excluded inputs | Prior/resulting balance, calculated delta, timestamp, source reference, other persisted effects, and request ID are excluded. |
| Replay | Same request ID plus the same operation/version/canonical identity returns the original persisted movement/result and performs no writes or stock effects. |
| Conflict | Same request ID plus a different identity returns typed public `request_conflict` and performs no writes or stock effects. |
| Legacy rows | Rows lacking a verifiable identity cannot be assumed equivalent to a retry and fail closed with `request_conflict`. |
| Malformed data | Malformed or partial persisted identity returns bounded `persistence_failure`; equivalence is never inferred. |
| Transaction authority | The inventory repository retains transaction ownership. Ticket 17 seam refactoring is excluded. |
| Race fallback | A unique-insert race rechecks the existing request ID using the same identity validation as the initial lookup. |
| Client behavior | The Tauri command exposes a stable conflict mapping, TypeScript decodes it, and Inventory UI presents a neutral request-conflict message without persistence or digest details. |

## Invariants

1. Every newly persisted inventory movement has operation kind, payload version `1`, its canonical payload, and the matching lowercase SHA-256 digest.
2. The digest is calculated only from the Rust-owned canonical identity fields and cannot be affected by request ID, balances, calculated delta, timestamps, source references, or other persisted effects.
3. Two requests are exact replays only when their request ID and complete operation/version/canonical identity match, including the operation-specific requested values and optional-note state.
4. An exact replay returns the original persisted movement/result, creates no second movement, and applies no additional stock effect.
5. A conflicting reuse creates no movement or stock effect and returns only the stable typed conflict outcome.
6. A legacy row with no verifiable identity is never treated as an exact replay.
7. Malformed or partial identity data never produces a replay or conflict decision based on guessed equivalence; it returns bounded `persistence_failure`.
8. Inventory transaction ownership remains in the repository, and a failed validation or persistence path leaves no partial movement or stock effect.
9. The unique-insert race fallback applies the same identity validation rules as the initial request-ID lookup.
10. The UI cannot reinterpret a conflict as a successful inventory operation or expose database, canonical-payload, or digest details as operator-facing language.

## Affected Areas

| Area | Proposed impact |
| --- | --- |
| SQLite schema and migration | Add four nullable identity columns to `inventory_movements` while preserving legacy rows, request-ID behavior, and existing movement facts. |
| Rust inventory domain/application | Define the version-1 canonical identity, calculate and compare the digest and identity fields, distinguish legacy/malformed records, and preserve atomic inventory behavior. |
| Inventory repository | Read and persist identity fields, retain transaction ownership, and validate the same identity during unique-insert race fallback. |
| Tauri command boundary | Expose a stable public mapping for `request_conflict` and the bounded persistence failure. |
| TypeScript adapter | Decode the typed conflict outcome without relying on message text or persistence details. |
| Inventory UI | Show a specific neutral request-conflict message without database or digest details. |
| Verification | Cover exact replay, changed identity, operation-specific fields, legacy and malformed identity records, race fallback, no-write guarantees, and rollback/atomicity. |

## Out of Scope

- A new idempotency or request-history table.
- Changing request-ID generation or UUID format.
- Changing inventory entry, physical-count, non-negative-stock, or no-op policy.
- Including derived or persisted effects in the identity.
- Sale idempotency conflict handling; sale behavior is owned by the sale conflict ticket.
- Touching legacy sale paths; ticket 16 owns their cleanup.
- Ticket 17 repository/application seam refactoring.
- Returns, cancellations, suppliers/costs, catalog editing, reports, backup/restore, roles, cloud, multi-store, or other workflows outside this inventory capability.
- Reconstructing canonical identities for legacy rows from incomplete historical data.
- Broad cryptography or identifier redesign beyond SHA-256 digesting of the version-1 identity.

## Migration and Backward Compatibility

- Apply an additive SQLite migration that extends `inventory_movements` with nullable identity columns so existing databases and historical movements remain readable.
- Do not backfill legacy identity values. Their original submitted payload is not verifiable from persisted movement facts, and reconstructed values must not be treated as authoritative.
- New inventory writes populate all identity fields using the version-1 Rust canonicalization rules. Application validation must require a complete identity for new rows while remaining compatible with nullable legacy values.
- Preserve existing request-ID uniqueness and movement history. A request ID belonging to a legacy row with no verifiable identity returns `request_conflict` rather than being accepted as a replay.
- A row with malformed or partial identity data returns bounded `persistence_failure`; it is not silently upgraded or interpreted as a conflict.
- Existing balances, movement IDs, timestamps, links, reasons, request IDs, and other historical facts remain unchanged.
- Pre-v12 binaries reject a v12 database through the existing forward-only schema-version guard; they cannot read v12 schemas.

## Risks and Mitigations

### Canonicalization drift

Different layers could encode the same inventory intent differently, producing false conflicts or false replays. Define one Rust-owned version-1 canonicalization routine with fixed field order, UTF-8 length-delimited fields, and explicit representation for optional notes. Cover operation-specific boundary values and normalized reasons in verification.

### Digest comparison is mistaken for authority

A digest alone is not a sufficient identity proof. Persist and validate operation kind, payload version, canonical payload, and digest together; treat malformed or incomplete records as bounded `persistence_failure` rather than guessing.

### Legacy compatibility surprises

Legacy rows cannot prove payload equivalence, so old retries may receive a conflict. Document the fail-closed behavior, preserve the original movement and stock facts, and keep any historical reconstruction outside this ticket.

### Partial effects on conflict

A conflict discovered after writes could leave duplicate or partial inventory facts. Resolve and compare the existing request ID before new calculation or writes, and retain the repository-owned atomic transaction as protection for the remaining paths.

### Unique-insert race divergence

Concurrent requests could use different validation behavior depending on whether the request-ID row was found initially or after a uniqueness failure. Route both paths through the same identity validation so the winner is replayed only for an exact identity and all mismatches remain conflict-free.

### Neutral operator messaging

A technical error could be presented as a successful retry or expose implementation details. Keep the command error stable, decode it explicitly in TypeScript, and show a neutral message that the request ID was already used for different inventory data.

## Rollback

- Before release, remove this OpenSpec change and revert the schema, inventory application/repository, command, TypeScript, UI, and verification changes together.
- After the migration has added identity columns or new inventory rows, prefer a forward-compatible rollback: disable the new conflict behavior and UI entry point without deleting identity data, movement history, or stock facts.
- Keep the additive columns nullable for supported v12 readers and historical rows; pre-v12 binaries still reject schema version 12 through the existing forward-only guard.
- Never repair rollback by deleting or rewriting inventory movements, identities, balances, or other immutable facts. Any later correction must use an explicit forward change.

## Success Criteria

- New inventory operations persist operation kind, payload version `1`, the deterministic canonical payload, and its lowercase SHA-256 digest in `inventory_movements`.
- Identity includes operation kind and product ID; stock entry includes requested positive quantity and exact optional note; physical count includes requested non-negative count and the existing domain-normalized nonblank reason.
- Identity excludes request ID, prior/resulting balance, calculated delta, timestamp, source reference, and other persisted effects.
- Exact same-request replays return the original persisted movement/result without a duplicate movement, recalculation-dependent mutation, or stock change.
- Same request ID with any changed identity field returns typed public `request_conflict` and leaves movement history and stock unchanged.
- Legacy request IDs with no verifiable identity fail closed with `request_conflict`; malformed or partial identity records return bounded `persistence_failure`.
- The unique-insert race fallback performs the same identity validation as the initial replay lookup.
- Transaction verification demonstrates no partial facts for exact replay, conflict, malformed/partial identity, and persistence-failure paths.
- The command boundary and TypeScript adapter preserve the stable conflict type, and the Inventory UI displays a specific neutral message without database or digest details.
- Sale paths remain untouched, and ticket 17 seam refactoring remains out of scope.
