# Archive Report: Durable Restore Filesystem Transitions

## Result

**PASS — archived successfully.** The completed change passed the archive gates and was moved intact to the dated archive path.

## Structured status and action context

```yaml
schemaName: gentle-ai.sdd-status
changeName: durable-restore-filesystem-transitions
artifactStore: hybrid
planningHome:
  root: /home/luis/velay/repuestos_autos/openspec
  changesDir: openspec/changes
changeRoot: /home/luis/velay/repuestos_autos/openspec/changes/durable-restore-filesystem-transitions
artifacts:
  proposal: done
  specs: done
  design: done
  tasks: done
  applyProgress: done
  verifyReport: done
  syncReport: done
taskProgress:
  total: 31
  complete: 31
  remaining: 0
  unchecked: []
applyState: all_done
dependencies:
  apply: all_done
  verify: all_done
  sync: all_done
  archive: ready
actionContext:
  mode: repo-local
  workspaceRoot: /home/luis/velay/repuestos_autos
  allowedEditRoots:
    - /home/luis/velay/repuestos_autos/openspec/changes/durable-restore-filesystem-transitions/**
    - /home/luis/velay/repuestos_autos/openspec/changes/archive/2026-09-01-durable-restore-filesystem-transitions/**
  warnings: []
nextRecommended: complete
```

## Artifacts read and preserved

- `proposal.md`
- `explore.md`
- `specs/backup-restore/spec.md`
- `design.md`
- `tasks.md`
- `apply-progress.md`
- `verify-report.md`
- `sync-report.md`
- `archive-report.md` (prior blocked report replaced)
- `openspec/config.yaml`
- `openspec/specs/backup-restore/spec.md` (read-only)

The active inventory contained 9 files, including the domain spec and all phase reports. The complete directory was moved; no artifact was deleted.

## Verification and sync accounting

- Verification: `PASS_WITH_WARNINGS`; 10/10 requirements, 26/26 scenarios, 0 blockers, 0 critical findings.
- Evidence revision: `sha256:c8cfb644e1d7905f190583a1a53da9e91cb4a9456a971433a65204443fbe329b`.
- Canonical domain: `backup-restore`, present at 17 requirements / 42 scenarios.
- Sync status: `synced`; canonical content was already synchronized and was not edited during archive.
- Exact delta: 9 ADDED requirements, 1 MODIFIED requirement, 0 REMOVED requirements; 24 added scenarios and 2 modified scenarios.
- ADDED: `Durable Prepared Evidence Before Disruption`; `Marker Progress Never Leads Durable Transitions`; `Marker Removal Follows Validated Readiness`; `Retain Fallback Sources Through Recovery Copy`; `Fail Closed on Required Durability Failure`; `Preserve Exact Restore Marker Compatibility`; `Bound the Durability Guarantee to Windows Local NTFS`; `Recycle Completed Restore Marker Sidecars`; `Require Real Windows NTFS Runtime Evidence`.
- MODIFIED: `Preserve Existing Restore and Ticket 02 Boundaries`.
- REMOVED: none.

The active delta requirement blocks matched the canonical specification exactly. No active same-domain collision was found; the other matching domain artifact is already archived.

## Validation and move

- Persisted tasks were re-read immediately before report replacement and showed no `- [ ]` implementation task markers.
- `git diff --check`: passed.
- Canonical requirement/scenario counts and exact delta-to-canonical block matching: passed.
- Active path before move: present.
- Archive target before move: absent.
- Archived path: `openspec/changes/archive/2026-09-01-durable-restore-filesystem-transitions/`.
- Active path after move: absent.
- Archive completeness after move: passed; all 9 files, including `sync-report.md` and final `archive-report.md`, are present.
- `/tmp` evidence was not edited or removed.
- No product or canonical specification was edited by archive; no commit or push was performed.

## Warnings

1. Verification accepted four unrelated pre-existing Windows Tauri mock IPC test failures after successful all-target compilation.
2. Historical cumulative W2 review footprint exceeded the 400-line budget without a recorded `size:exception`.
3. The canonical specification retains its pre-existing working-tree modification from the prior sync; archive did not change it.

## Destructive merge and exceptions

No archive-time sync or destructive canonical merge was performed. The sync report records parent authorization for persistence of the already-synced 9-added/1-modified baseline. No stale-checkbox reconciliation or partial-archive exception was used.

## Memory traceability

Hybrid archive report persisted under `sdd/durable-restore-filesystem-transitions/archive-report` as Engram observation **137**.
