---
name: rust-application-integrity
description: "Trigger: Rust domain rule, application seam, transaction orchestration, typed error, checked arithmetic, idempotency payload. Protect this project's domain and application integrity."
license: Apache-2.0
metadata:
  author: gentleman-programming
  version: "1.0"
---

## Activation Contract

Use this skill for Rust domain invariants, application use cases, repository seams, transaction orchestration, typed errors, arithmetic, and idempotency. Do not place UI, IPC, or SQLite-specific details in domain modules.

## Hard Rules

- Encode invariants in domain types and constructors; reject invalid states before persistence.
- Use checked integer arithmetic for money, quantity, totals, deltas, and revisions; never use floating point for centavos.
- Keep application seams small and persistence-neutral; add an adapter seam only when behavior actually varies.
- Orchestrate each multi-write use case in one transaction owned by the application/infrastructure seam.
- Return bounded typed errors; translate database and transport details outside the domain.
- Make idempotency compare request identity, operation kind, payload version, and canonical payload digest.
- Canonicalize payloads deterministically before hashing; reject reuse with a different payload.

## Decision Gates

| Situation | Action |
| --- | --- |
| Value has validity rules | Introduce or reuse a domain type |
| Use case performs related writes | Require one transaction and rollback test |
| Caller needs alternate storage | Define a repository or transaction seam |
| Retry may repeat a mutation | Persist and compare canonical request identity |
| Arithmetic may overflow | Use checked operations and a typed failure |
| Error crosses IPC | Map it through `tauri-ipc-contracts` |

## Execution Steps

1. Trace the command, application use case, domain types, repository seam, and SQLite adapter.
2. State invariants and failure variants before changing orchestration.
3. Keep policy in domain/application code and mechanics in adapters.
4. Test success, invariant rejection, rollback, overflow, exact replay, and mismatched replay where applicable.
5. Verify no partial facts survive a failed transaction.

## Output Contract

Return changed seams, protected invariants, transaction ownership, typed error mapping, idempotency evidence, and residual integrity risks.

## References

- `references/project-integrity-map.md` — local domain, application, transaction, and idempotency paths.
