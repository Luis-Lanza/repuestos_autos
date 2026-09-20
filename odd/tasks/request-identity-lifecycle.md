# Request identity lifecycle

## Objective
Ensure each sale or inventory mutation request ID represents exactly one user intent, while an exact retry preserves the original ID.

## Problem
A request ID can survive edits to mutation-defining fields and be reused for a different payload, undermining later idempotency conflict protection.

## Scope
- Sale draft mutation fields: lines, quantities, payment inputs, and acknowledgement payload.
- Inventory mutation fields: product, operation, quantity/count, note, and reason.
- Focused flow and interaction tests.

## Constraints
- Ticket 06 only: no Rust, Tauri IPC, persistence, identifier format, or payload-conflict detection changes.
- Preserve exact-retry IDs after transport uncertainty.
- User has authorized implementation but has not authorized commits, push, or PR creation.
- TDD mode: not configured; use focused post-change checks.

## Delivery strategy
- ask-on-risk; forecast: approximately 120 authored changed lines.
- No commit boundary is authorized yet.

## Tasks
- [x] T1 Map current sale and inventory identity lifecycles; define exact invalidation transitions and focused test surfaces. Route: delegated exploration (4+ files). Evidence: sale and inventory reducers already reset IDs on listed mutations; mounted tests lack command-envelope retry/change coverage.
- [x] T2 Implement and test request-ID invalidation for sale and inventory payload edits. Route: delegated writer (multi-file write). Evidence: sale/inventory focused flow and mounted tests pass 50/50; inventory same-value actions are now no-ops.
- [x] T3 Independently verify focused sale and inventory identity behavior. Route: delegated verification. Independent verifier passed the focused suite (50/50); no ticket-scope findings.

## Acceptance criteria
- Editing any mutation-defining field invalidates the current request ID.
- Exact retries preserve their ID.
- New sale or adjustment starts with a fresh ID.
- Focused tests distinguish retry from changed intent in both flows.

## Progress
- 2026-09-20: Created from approved ticket 06.
- 2026-09-20: T1 mapped. The implementation is frontend-only; required work is mounted command-envelope coverage and confirmation of final-price transition behavior. Same-value actions should remain no-ops because they do not change mutation intent.
- 2026-09-20: T2 completed. Added flow/mounted regression coverage for exact retry, changed intent, and new intent; inventory same-value actions now preserve request identity. Focused command passed: 50 tests, 0 failures.
- 2026-09-20: Native risk assessment returned unavailable/empty output; follow the fail-closed plan with independent verification.
- 2026-09-20: T3 independently passed: 50 tests, 0 failures or skips. Native review inspection confirmed RDD is disabled, so no native review started. Ticket 06 and its board entry are now marked done.
- 2026-09-20: Committed the completed work unit as `fix(requests): reset identity on intent changes`; no push requested.

## Verification evidence
- Writer check: `npx tsx --test --import ./test/react-dom.ts src/ui/sales/sale-flow.test.ts src/ui/sales/sale-screen.mounted.test.ts src/ui/inventory/inventory-flow.test.ts src/ui/inventory/inventory-screen.mounted.test.ts` — 50 passed, 0 failed.
- Native assessment: unavailable (empty native output); independent verifier completed as fail-closed fallback.
- Independent check: `npx tsx --test --import ./test/react-dom.ts src/ui/sales/sale-flow.test.ts src/ui/sales/sale-screen.mounted.test.ts src/ui/inventory/inventory-flow.test.ts src/ui/inventory/inventory-screen.mounted.test.ts` — 50 passed, 0 failed, 0 skipped.
- Native review inspection: RDD disabled; no native review applicable.

## Next step
Ticket 06 is committed locally. The unrelated untracked `odd/tasks/developer-toolchain-rc-upgrade.md` remains outside this ticket; await explicit push authorization.
