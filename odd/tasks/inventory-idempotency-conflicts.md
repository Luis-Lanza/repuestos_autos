# Inventory idempotency conflicts

## Objective
Close ticket 08 by specifying the already-implemented inventory identity contract without widening it to unrelated movement types.

## Approved policy
- Identity is required only for request-bearing `stock_entry` and `adjustment` operations.
- Preserve v1 length-delimited UTF-8 canonical payloads and lowercase SHA-256.
- Do not backfill legacy movements; reuse of all-null legacy identities returns `request_conflict`; partial/malformed identities return `persistence_failure`.
- Defer a concurrent unique-insert fallback test to follow-up work.

## Scope
Add the missing change-local OpenSpec specification; verify existing behavior and mark ticket 08 complete if it conforms. No production change is expected.

## Constraints
Do not change migrations, sales/returns/cancellations movement contracts, IPC, or UI behavior. No commit, push, or PR is authorized.

## Tasks
- [x] T1 Add the missing inventory OpenSpec delta for approved identity behavior. Route: delegated writer.
- [x] T2 Independently verify existing focused inventory behavior and specification. Route: delegated verification. Passed 9/9 filtered inventory tests.

## Acceptance criteria
- Only request-bearing inventory operations require identity metadata.
- Replay, conflict, legacy, malformed, and rollback behavior are specified.
- Existing focused tests pass.

## Progress
- 2026-09-20: User approved request-bearing scope and deferred concurrent-fallback testing.
- 2026-09-20: OpenSpec delta added and independently verified; ticket 08 and board entry marked done.

## Verification evidence
- `cargo test --manifest-path src-tauri/Cargo.toml inventory` — 9 passed, 0 failed. The concurrent unique-insert fallback remains explicitly deferred.

## Next step
Ticket 08 is complete locally; await explicit commit/push authorization.
