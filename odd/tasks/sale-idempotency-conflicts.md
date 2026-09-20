# Sale idempotency conflicts

## Objective
Close roadmap ticket 07 by formalizing the existing sale request-identity contract and proving migration compatibility for legacy sale rows.

## Problem
The active Tauri path already compares canonical sale identity and returns `request_conflict` for a mismatched retry, but the OpenSpec change does not describe negotiated-price v2 identities and migration tests do not exercise legacy-row replay through the production path.

## Approved policy
- Preserve canonical payload versions v1 and v2.
- Do not backfill identities for historical sale rows.
- Treat an all-null legacy identity as `request_conflict`; treat a partial/malformed persisted identity as `persistence_failure`.
- Never reinterpret a persisted v1 identity as v2.

## Scope
- Amend the existing `detect-sale-idempotency-conflicts` OpenSpec proposal/spec.
- Add focused migration/production-path evidence for legacy identity behavior.
- Mark ticket 07 complete only after observed checks.

## Constraints
- Do not redesign request identifiers, command registration, capabilities, or inventory idempotency.
- Do not alter the legacy internal confirmation path; it belongs to ticket 16.
- Preserve the public `request_conflict` contract.
- User has authorized this policy and implementation, but has not authorized commits, pushes, or PR creation.
- TDD mode: not configured; use focused post-change checks.

## Delivery strategy
- ask-on-risk; forecast: approximately 180 authored changed lines.
- No commit boundary is authorized yet.

## Tasks
- [x] T1 Reconcile approved policy with the existing OpenSpec proposal/spec and ticket tracker. Route: delegated writer. Evidence: existing OpenSpec change now specifies v1/v2 preservation and no-backfill compatibility.
- [x] T2 Correct and test persisted payload-version/canonical-marker consistency. Route: delegated writer. Persisted version now must match canonical marker; invalid tuples return `persistence_failure` with rollback preserved.
- [x] T3 Independently verify OpenSpec alignment and focused Rust migration/application behavior. Route: delegated verification. Passed 61/61 with no findings.

## Acceptance criteria
- Canonical v1/v2 compatibility and legacy behavior are specified.
- Exact replay returns the original sale; mismatched replay returns `request_conflict` without partial facts.
- Legacy migrated rows fail closed through the production path.

## Progress
- 2026-09-20: User approved v1+v2 without backfill after read-only mapping found the active Tauri path already implements conflict detection.
- 2026-09-20: T1/T2 completed. OpenSpec now matches the versioned policy; a production-path pre-v11 migration regression was added. Writer command passed 60 tests and diff check.
- 2026-09-20: Native risk assessment returned unavailable/empty output; independent verification is required.
- 2026-09-20: Independent verification found a deterministic contract gap: persisted `payload_version` is not cross-validated against the canonical marker, so an inconsistent persisted tuple can return `request_conflict` instead of `persistence_failure`. Reopened T2 for a bounded correction.
- 2026-09-20: T2 correction completed: version-marker consistency is validated before replay/conflict; focused Rust tests passed 61/61.
- 2026-09-20: T3 independently passed 61/61 with no findings. Native review inspection confirmed RDD is disabled; no native review started.

## Verification evidence
- Writer check: `cargo test --manifest-path src-tauri/Cargo.toml --test confirm_sale_application --test confirm_sale_use_case --test sqlite_migrations` — 60 passed.
- Writer check: `git diff --check` — passed.
- Native assessment: unavailable (empty native output).
- Independent command passed 60/60, but verification failed on the persisted payload-version/canonical-marker consistency gap; correction was applied.
- Correction and independent check: `cargo test --manifest-path src-tauri/Cargo.toml --test confirm_sale_application --test confirm_sale_use_case --test sqlite_migrations` — 61 passed, 0 failed.

## Next step
Ticket 07 is complete locally; await explicit commit/push authorization.
