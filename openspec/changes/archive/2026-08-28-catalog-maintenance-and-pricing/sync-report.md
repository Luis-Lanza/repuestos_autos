# Sync Report: Catalog Maintenance and Pricing

## Status

**synced** — The explicitly authorized safe requirements synchronization is complete. The change remains active until archival.

## Sync scope

| Domain | Change delta | Canonical target | Result |
| --- | --- | --- | --- |
| `sales` | `openspec/changes/archive/2026-08-28-catalog-maintenance-and-pricing/specs/catalog-maintenance-and-pricing/spec.md` | `openspec/specs/sales/spec.md` | Updated |
| `operational-inventory-control` | `openspec/changes/archive/2026-08-28-catalog-maintenance-and-pricing/specs/catalog-maintenance-and-pricing/spec.md` | `openspec/specs/operational-inventory-control/spec.md` | Updated |

## Delta applied

| Domain | Operation | Requirement |
| --- | --- | --- |
| `sales` | MODIFIED | `Whole-Unit Quantities and Fixed Catalog Price` |
| `operational-inventory-control` | MODIFIED | `Active Product Selection` |
| both | ADDED | None |
| both | REMOVED | None |

The two canonical requirement blocks were reconstructed as complete blocks. Existing scenarios were preserved and the requested stale-price acknowledgement and archived-category rejection scenarios were added. No unrelated canonical requirement or section was changed.

## Destructive-sync approval

The parent instruction explicitly authorized the safe retry after the destructive-merge guard blocked the partial delta. Affected requirements were:

- `sales / Whole-Unit Quantities and Fixed Catalog Price`: approximately 24 old lines replaced by a complete 38-line requirement block.
- `operational-inventory-control / Active Product Selection`: approximately 15 old lines replaced by a complete 29-line requirement block.

No requirement was removed. The replacements preserve the canonical scenarios and add only the approved behavior.

## Checks performed

- Read proposal, delta spec, design, tasks, verify report, configuration, and both canonical specifications.
- Confirmed native SDD status: `nextRecommended: archive`, `blockedReasons: []`, and 25/25 tasks complete.
- Confirmed verification PASS with evidence revision `sha256:cd9a942a5b01cbd13c5638336f89a3add3eb958dd622f0b7377858a299ec754f`.
- Read back both replaced canonical requirement blocks structurally.
- Ran `git diff --check` for both canonical specifications successfully.
- No product source, tests, staging, commits, branches, PRs, or unrelated change artifacts were modified.

## Active same-domain changes

Warning: active change `local-backup-and-restore` exists, but its spec domain is `local-backup-and-restore`; it does not touch either synchronized domain. No same-domain collision was detected.
