# Archive Report: Catalog Maintenance and Pricing

## Status

**PASS WITH WARNINGS** — Requirements were safely synchronized, verification passed, and the completed change is ready for archival.

## Final readiness

- Native SDD status: `nextRecommended: archive`; `blockedReasons: []`.
- Tasks: 25/25 complete, 0 pending.
- Verification: PASS; evidence revision `sha256:cd9a942a5b01cbd13c5638336f89a3add3eb958dd622f0b7377858a299ec754f`.
- Approved issue: #101.
- PR8 delta: 395 raw lines / 386 behavior lines; within the 400-line cap.

## Artifacts read

- `proposal.md`
- `specs/catalog-maintenance-and-pricing/spec.md`
- `design.md`
- `tasks.md`
- `apply-progress.md`
- `verify-report.md`
- `sync-report.md`
- `openspec/config.yaml`
- Canonical `openspec/specs/sales/spec.md`
- Canonical `openspec/specs/operational-inventory-control/spec.md`

## Canonical specifications updated

| Domain | Operation | Requirement |
| --- | --- | --- |
| `sales` | MODIFIED | `Whole-Unit Quantities and Fixed Catalog Price` |
| `operational-inventory-control` | MODIFIED | `Active Product Selection` |

ADDED requirements: none. REMOVED requirements: none. Only the two explicitly authorized complete requirement-block replacements were applied. Existing scenarios were preserved; stale-price acknowledgement and archived/inactive category selection rejection were added.

## Destructive merge

The parent instruction explicitly approved the safe retry. The replaced blocks were approximately 24 and 15 prior lines respectively. No canonical requirement was deleted, and no partial delta was accepted.

## Warnings

- Active `local-backup-and-restore` is unrelated to the synchronized domains; no same-domain collision was found.
- The verification report records a non-blocking base-only `backup_restore` fixture mismatch (schema 6 vs 8), tracked by approved issue #101.
- `tauri:dev` was N/A because protected pre-existing `dist/` could not be safely mutated.

## Preservation

No product source/tests, Git staging, commits, branches, PRs, or unrelated `local-backup-and-restore` artifacts were modified.

## Archive destination

`openspec/changes/archive/2026-08-28-catalog-maintenance-and-pricing/`
