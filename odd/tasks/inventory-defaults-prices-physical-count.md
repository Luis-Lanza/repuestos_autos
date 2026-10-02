# Inventory defaults, optional prices, and physical-count authorization

## Objective
Keep Inventory on All Stock regardless of alert entry, preserve current product prices when Stock Entry prices are omitted, and require the shared Catalog password before a physical count mutates stock.

## Decisions
- Inventory always opens with All Stock; alerts remain informational and never select the Alerts filter automatically.
- Purchase, sale, and minimum-sale prices are optional in Stock Entry; each omitted value keeps the current persisted product value.
- Physical count uses the existing device-local Catalog password verifier. It creates no Inventory credential and immediately follows future Catalog password changes.
- Password verification is enforced in Rust before the physical-count application operation; the modal is not the security boundary.
- Omitted Stock Entry prices are a distinct caller intent from explicitly supplied prices, even when the effective persisted price is the same; new canonical idempotency encoding must preserve that distinction while decoding legacy identities.
- Physical-count authorization is evaluated against the current shared Catalog password immediately before mutation; it is a pure verification and does not unlock Catalog. A later concurrent password change does not revoke an already authorized count.

## Tasks
- [x] I1 Keep the Inventory entry state on All Stock and cover alert navigation. — Implemented in `de8f80b` (`fix(inventory): keep alert navigation on all stock`); focused mounted test and independent verification passed. Native review preflight stopped because RDD is disabled.
- [x] I2 Make all Stock Entry price inputs optional while preserving omitted persisted values. — Implemented in `9712a0a` (`feat(inventory): preserve omitted stock entry prices`); lock metadata synchronized separately in `24ba59a` (`chore(tauri): sync package lock version`). Frontend 35/35 and Rust 21/21 focused tests passed; native review preflight stopped because RDD is disabled.
- [x] I3 Require shared Catalog-password verification for physical-count confirmation end to end. — Implemented in `de61e77` (`feat(inventory): authorize physical counts with catalog password`); frontend 32/32 and Rust command 4/4 passed. Desktop IPC test remains blocked by missing GTK/GIO development libraries; native review preflight stopped because RDD is disabled.
- [~] I4 Run focused Rust/frontend checks and record Windows smoke cases. — Automated checks are green; Windows smoke remains pending. The pullable branch state, including the aligned App Shell regression test, is committed in `9e3f95b` (`test(inventory): align shell alert navigation`).

## Verification blockers
- [~] B1 Establish the W9 audit baseline and ownership before treating its failures as feature regressions.
- I4 is blocked: after fixing the stale App Shell expectation to All Stock, `npm test` passes 438/442. The four remaining failures are all in `src/ui/w9-evidence-audit.test.ts` (backup evidence hash, two historical candidate-path checks, and protected-path drift). B1 confirmed `src/ui/w9-evidence-audit.test.ts` itself is unchanged from `master`; the new `odd/tasks/inventory-defaults-prices-physical-count.md` is absent from `master` and explains the protected-path drift. The other three failures are stale/unproven-baseline W9 evidence and belong to separate W9 policy triage, not this Inventory feature.
- Windows smoke remains required for native desktop IPC because Linux lacks GTK/GIO development libraries. Exercise All Stock alert navigation, omitted/supplied price behavior, invalid/correct/new Catalog password physical counts, no mutation on rejection, and the fact that Catalog remains locked.

## Acceptance criteria
- Inventory opens on All Stock even when stock alerts exist or prompted navigation originated from an alert.
- Any subset of Stock Entry prices can be omitted; omitted values remain unchanged and supplied values retain validation.
- Physical count cannot mutate stock without the current Catalog password; wrong/missing credentials are rejected without mutation.
- All user-visible password states are accessible and the shared password change takes effect without duplication.
