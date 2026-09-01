---
name: sqlite-database-expert
description: "Trigger: SQLite, rusqlite, SQL query, schema migration, transaction, online backup, restore, crash recovery. Implement secure and recoverable persistence for this desktop application."
license: Apache-2.0
metadata:
  author: gentleman-programming
  version: "1.0"
---

## Activation Contract

Use this skill for SQLite queries, schema changes, transactions, integrity checks, online backup, restore, and startup recovery. Use `rust-application-integrity` for domain policy and application orchestration.

## Hard Rules

- Bind every data value with `rusqlite` parameters; interpolate only fixed, code-owned identifiers from a closed allowlist.
- Enable foreign keys on every connection and keep constraints as a second line of defense.
- Add forward-only, numbered migrations; never rewrite a migration that may have shipped.
- Apply each schema step and `PRAGMA user_version` update in the same transaction.
- Reject databases newer than the supported schema and validate required columns, indexes, triggers, and foreign keys.
- Wrap related writes in one transaction and propagate failure without partial commit.
- Use SQLite online backup APIs for live snapshots; never copy an open database file directly.
- Stage and validate restores before replacement; preserve a recovery source and durable restore marker across crash points.

## Decision Gates

| Situation | Action |
| --- | --- |
| User data enters SQL | Read `references/security-examples.md` |
| Schema or restore changes | Read `references/crash-safety.md` |
| Multiple facts must agree | Use one transaction |
| Dynamic identifier is requested | Map an enum or allowlisted token to fixed SQL |
| Live backup is needed | Use `rusqlite::backup::Backup` |
| Startup finds a restore marker | Validate canonical storage, then recover from a validated fallback |

## Execution Steps

1. Trace the application seam, SQLite adapter, migration version, and focused tests.
2. Add parameterized SQL and explicit row decoding.
3. Add the next migration plus compatibility and post-migration validation.
4. Exercise rollback, foreign-key failure, older/newer schema, malformed backup, and crash recovery as relevant.
5. Verify integrity before exposing a restored connection.

## Output Contract

Return changed queries or migrations, transaction scope, validation and recovery checks, focused commands, and residual data-loss risk.

## References

- `references/security-examples.md` — secure rusqlite query patterns.
- `references/crash-safety.md` — project migration, backup, restore, and recovery protocol.
