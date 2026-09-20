# Change Proposal: Detect Sale Idempotency Conflicts

## Intent

Make sale idempotency safe to verify rather than treating every reuse of a request ID as a valid replay. A retry may return the original persisted sale only when the submitted sale intent is exactly the same; reuse of the request ID for a different intent must fail closed with a stable public conflict outcome.

## Problem and Outcome

The current sale idempotency record proves request-ID reuse but does not prove that the retried payload represents the same sale intent. A caller can therefore accidentally reuse a request ID with changed lines, captured prices, acknowledgements, or payment inputs and receive behavior intended only for an exact retry.

The outcome is bounded conflict detection at the existing sale persistence boundary. Exact replays return the original persisted summary without new writes or stock effects. Mismatched reuse returns typed `request_conflict` without changing sales, lines, payments, stock, or inventory movements. The compatibility policy preserves canonical identity versions v1 and v2, does not backfill historical rows, treats an all-null legacy identity as `request_conflict`, treats partial or malformed persisted identity as `persistence_failure`, and never reinterprets a stored v1 identity as v2.

## Agreed Scope

- Extend the existing `sales` table; do not create a separate idempotency table.
- Persist operation kind, payload version, canonical payload, and a SHA-256 digest for each newly created sale identity.
- Preserve both canonical sale identity versions. Define v1 as operation `confirm_sale`, payload version `1`, and a deterministic UTF-8 length-delimited canonical payload. Define v2 as operation `confirm_sale`, payload version `2`, using the same canonical fields plus the explicit nullable negotiated final unit price for each line.
- Encode each digest as lowercase hexadecimal SHA-256.
- Include in the canonical identity:
  - the submitted ordered lines, each containing product ID, quantity, captured unit price, captured revision, nullable acknowledged price, and nullable acknowledged revision;
  - payment fields for amount tendered and QR applied, each represented with an explicit null/value distinction.
- Make line order significant. Do not sort, deduplicate, or otherwise normalize submitted lines in a way that changes their order before identity comparison.
- Resolve an existing request ID before any new sale calculation or write. An exact request-ID and identity match returns the original persisted summary with no new writes or stock effects.
- Return typed public `request_conflict` for the same request ID with a different identity, with no writes or stock effects.
- Treat a legacy row whose four identity columns are all null as fail-closed: reusing its request ID returns `request_conflict`.
- Treat a partial or malformed persisted identity tuple—including an unsupported version, invalid canonical payload, or digest mismatch—as `persistence_failure`; never backfill it or reinterpret a persisted v1 identity as v2.
- Keep transaction ownership in `ConfirmSaleUseCase`; repositories remain mechanical and atomic, without owning business decisions or committing independently.
- Map the stable conflict at the command boundary, decode it in TypeScript, and show a specific neutral message in the UI.

## Explicit Decisions

| Topic | Decision |
| --- | --- |
| Storage location | Add identity columns to `sales`; no new table or parallel identity store. |
| Identity metadata | New sale identities use operation `confirm_sale` and payload version `1` or `2`; v1 preserves the established identity fields, while v2 additionally records the nullable negotiated final unit price. |
| Canonical form | Use a fixed field order and deterministic UTF-8 length-delimited fields. Nullable values carry explicit null/value tags; canonical numeric and identifier representations use the existing domain units and forms. JSON serialization, map iteration order, and presentation formatting are not identity mechanisms. |
| Digest | SHA-256 over the canonical payload, rendered as lowercase hexadecimal. |
| Line semantics | Submitted line order is significant and is preserved for identity comparison. |
| Excluded inputs | Request ID, catalog names/SKU, values resolved later from SQLite, derived total, timestamps, stock, and persisted outcome are excluded. |
| Replay | Same request ID plus the same operation/version/canonical identity returns the original persisted summary and performs no writes or stock effects. |
| Conflict | Same request ID plus a different identity returns typed public `request_conflict` and performs no writes or stock effects. |
| Legacy rows | Rows with all four identity columns null cannot be assumed equivalent to a retry and return `request_conflict`; partial or malformed identity data returns `persistence_failure`. No historical identity is backfilled. |
| Authority | `ConfirmSaleUseCase` owns lookup, comparison, and transaction orchestration; repositories only execute mechanical atomic persistence operations. |
| Version compatibility | Stored v1 and v2 identities are validated and compared only as their persisted versions; a stored v1 identity is never reinterpreted as v2. |
| Client behavior | The Tauri command exposes a stable error mapping, TypeScript decodes it, and React presents a neutral request-conflict message without deciding identity equivalence. |

## Invariants

1. Every newly persisted confirm-sale row has operation kind `confirm_sale`, payload version `1` or `2`, its versioned canonical payload, and the matching lowercase SHA-256 digest.
2. The digest is calculated only from the versioned canonical identity fields and cannot be affected by request ID, catalog display data, later SQLite lookups, derived totals, timestamps, stock, or persisted outcome.
3. Two requests are exact replays only when their request ID and complete operation/version/canonical identity match, including ordered lines and explicit null/value states.
4. An exact replay creates no sale, line, payment, movement, or stock effect and does not rewrite the original persisted facts.
5. A conflicting reuse creates no sale, line, payment, movement, or stock effect and returns only the stable typed conflict outcome.
6. An all-null legacy identity is never treated as an exact replay and returns `request_conflict`; partial or malformed persisted identity returns `persistence_failure`.
7. A persisted v1 identity is never reinterpreted as v2, and historical rows are never backfilled.
8. The application transaction remains atomic and is owned by `ConfirmSaleUseCase`; repository methods do not commit independently.
9. The UI cannot reinterpret a conflict as a successful sale or expose cryptographic details as operator-facing business language.

## Affected Areas

| Area | Proposed impact |
| --- | --- |
| SQLite schema and migration | Add the four sale-identity columns while preserving existing request-ID uniqueness and readability of legacy rows. |
| Rust sales domain/application | Preserve versioned v1/v2 canonical identities, calculate/compare the digest, fail closed for legacy rows, and keep the comparison in `ConfirmSaleUseCase`. |
| SQLite repository | Read and persist identity fields as mechanical operations within the application-owned transaction. |
| Tauri command boundary | Expose a stable public mapping for `request_conflict`. |
| TypeScript and React | Decode the typed conflict and show a specific neutral retry/request-conflict message. |
| Verification | Cover exact replay, changed identity, legacy request-ID reuse, no-write guarantees, and atomic transaction behavior. |

## Out of Scope

- A new idempotency or request-history table.
- Changing the request-ID generation or UUID format.
- Broad cryptography or identifier redesign beyond SHA-256 digesting of these versioned identities.
- Idempotency conflict handling for inventory adjustments or other operations.
- Changes to sale pricing, payment derivation, stock rules, persisted summaries, or sale lifecycle behavior except where needed to preserve replay/conflict semantics.
- The legacy duplicate confirm-sale API. It remains explicitly excluded and is reserved for future ticket 16 cleanup.
- Returns, cancellations, refunds, payment reversals, synchronization, or other workflows outside sale confirmation.
- Reconstructing a canonical identity for legacy rows from incomplete historical data.

## Migration and Backward Compatibility

- Apply an additive SQLite migration that extends `sales` with nullable identity columns so existing databases and historical rows remain readable.
- Do not backfill legacy identity values: their original submitted payload is not verifiable from persisted sale facts, and derived or reconstructed values must not be presented as authoritative.
- New confirm-sale writes populate all identity columns and use the applicable v1 or v2 canonicalization rules. Constraints or application validation must require these fields for new rows while remaining compatible with null legacy values.
- Preserve the existing unique request-ID behavior. A request ID already owned by a row whose identity columns are all null returns `request_conflict`, rather than being accepted as a replay.
- A request ID owned by a row with a partial, malformed, unsupported, or version-inconsistent identity returns `persistence_failure`; the application must not repair or reinterpret that row.
- Existing sale lines, payments, stock balances, inventory movements, timestamps, outcomes, and historical summaries remain unchanged.
- A prior application that ignores additive columns may continue reading existing sales; rollback must not delete newly stored identity data or alter historical sale facts. Reintroducing the new application later will continue to fail closed for rows created without a verifiable identity.

## Risks and Mitigations

### Canonicalization drift

Different layers could encode the same intent differently, producing false conflicts or false replays. Keep version-aware Rust-owned canonicalization routines, use fixed field order and explicit null/value tags, and test representative boundary values and reordered lines.

### Digest comparison is mistaken for authority

A digest alone is not a substitute for the canonical identity. Persist and compare the operation kind, payload version, canonical payload, and digest together; treat malformed or incomplete new identity data as non-replayable rather than guessing.

### Legacy compatibility surprises

Legacy rows cannot prove payload equivalence, so old retries may receive a conflict. Document this fail-closed behavior clearly, preserve the original persisted sale for history, and keep any recovery or migration of missing identity outside this ticket.

### Version compatibility

Keep v1 and v2 as distinct canonical contracts. Validate the stored payload against its stored version before comparing it with the incoming request. A stored v1 payload remains v1 even when the incoming request uses v2; it is not upgraded, decoded, or compared as v2.

### Partial effects on conflict

A conflict discovered after writes could leave duplicate or partial facts. Resolve and compare the existing request ID before any new sale effects, and retain the application-owned atomic transaction as the final protection.

### Duplicate confirm-sale API divergence

The legacy duplicate confirm-sale API may not expose or enforce the new conflict contract. Keep it out of this change as agreed, document the compatibility risk, and reserve its removal or cleanup for ticket 16 rather than widening this scope.

### Neutral operator messaging

A technical error could be presented as a successful retry or expose implementation details. Keep the command error stable, decode it explicitly in TypeScript, and show a neutral message instructing the operator that the request ID was already used for different sale data.

## Rollback

- Before release, remove this OpenSpec change and revert the schema, application, command, TypeScript, and UI changes together.
- After the migration has added identity columns or new sale rows, prefer a forward-compatible rollback: disable the new conflict behavior and UI entry point without deleting identity data, sale history, payments, movements, or stock facts.
- Keep the additive columns nullable so a prior application can continue reading the database without interpreting the new fields as changed historical sale data.
- Never repair rollback by deleting or rewriting a sale, its identity, lines, payments, stock, or inventory movements. Any later correction must use an explicit forward change.

## Success Criteria

- New confirmed sales persist `confirm_sale`, payload version `1` or `2`, the corresponding deterministic canonical payload, and its lowercase SHA-256 digest in `sales`.
- Identity includes every submitted ordered line and the specified payment fields, including explicit null/value distinctions; v2 additionally includes the nullable negotiated final unit price, and both versions exclude all listed non-identity data.
- Exact same-request replays return the original persisted summary with no new writes, recalculation-dependent effects, duplicate records, or stock changes.
- Same request ID with any changed identity field, changed null/value state, or changed line order returns typed public `request_conflict` and leaves all sale and stock facts unchanged.
- Reuse of a request ID belonging to an all-null legacy row fails closed with the same bounded conflict while keeping the legacy sale readable and unchanged; a pre-v11 confirmed sale migrated to the current schema demonstrates this through the production confirmation path.
- Partial or malformed persisted identity fails closed with the bounded `persistence_failure` outcome, and a persisted v1 identity is never reinterpreted as v2.
- Transaction tests demonstrate that exact replay and conflict paths are no-write paths and that normal confirmation remains atomic.
- The command boundary and TypeScript adapter preserve the stable conflict type, and the UI displays a specific neutral message.
- The legacy duplicate confirm-sale API remains out of scope and is documented as a ticket-16 cleanup risk.
