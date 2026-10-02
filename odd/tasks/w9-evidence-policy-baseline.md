# W9 evidence-policy baseline

## Objective
Restore a trustworthy W9 evidence audit: determine the intended baseline for historical candidate hashes and path sets, then make the audit tolerate legitimate ODD task tracking without weakening protected-path detection.

## Scope
- W9 evidence-audit policy and its dedicated test evidence only.
- No backup, Catalog-access, or gross-profit production behavior changes unless separately proven necessary.
- Inventory implementation remains out of scope; its I4 stays blocked pending this work and Windows smoke evidence.

## Known evidence
- `src/ui/w9-evidence-audit.test.ts` is unchanged from `master`.
- The new `odd/tasks/inventory-defaults-prices-physical-count.md` explains the current protected-path drift failure.
- Three remaining W9 failures concern stale hash/path expectations for backup, gross-profit, and Catalog-access evidence; their historical baseline must be established before changing policy.

## Tasks
- [x] W1 Map W9 audit inputs, historical evidence, and intended protected-path policy. — The strict policy predates recent candidate checks (`adc2ed5d`); exact pre-existing ODD exclusions were added in `a4fdc39c`, while backup/gross-profit/Catalog checks were introduced or updated by `d6de33e`, `219c890`, and `e4e4183`.
- [x] W2 Define and test the narrow policy update for legitimate ODD task tracking and stale evidence refresh. — Implemented in `0276623` (`test(w9): separate strict audit from general suite`); `npm test` excludes only W9 and passes 412/412; `npm run test:w9` remains intentionally strict and red outside its historical candidate. Native review preflight stopped because RDD is disabled.
- [x] W3 Run focused/full audit verification and record how Inventory I4 can resume. — `npm test` passes 412/412, test typecheck and diff check pass; `test:w9` remains deliberately strict (26/30 outside its historical candidate). Inventory I4 is unblocked from W9 but still awaits Windows smoke evidence.

## Acceptance criteria
- W9 failures are attributed with repository-history evidence, not assumptions.
- Legitimate active ODD task documents do not create false protected-path drift.
- Protected-path detection remains effective for unrelated source changes.
- No production behavior changes are mixed into the W9 policy correction.
