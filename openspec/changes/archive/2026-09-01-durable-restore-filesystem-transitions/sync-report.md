# Sync Report: Durable Restore Filesystem Transitions

```yaml
schema: gentle-ai.sync-report/v1
change: durable-restore-filesystem-transitions
status: synced
artifact_store: hybrid
sync_execution: validation-only-report-persistence
canonical_content_changed_this_run: false
domains_synced:
  - backup-restore
canonical_files_updated_by_prior_sync:
  - openspec/specs/backup-restore/spec.md
canonical_counts:
  requirements: 17
  scenarios: 42
delta_counts:
  added_requirements: 9
  added_scenarios: 24
  modified_requirements: 1
  modified_scenarios: 2
  removed_requirements: 0
  renamed_requirements: 0
preserved_unrelated_counts:
  requirements: 7
  scenarios: 16
verification:
  verdict: PASS_WITH_WARNINGS
  blockers: 0
  critical_findings: 0
  requirements: 10/10
  scenarios: 26/26
  evidence_revision: sha256:c8cfb644e1d7905f190583a1a53da9e91cb4a9456a971433a65204443fbe329b
canonical_sha256: sha256:46425a3a50285f342324506bf43d997781af7a3995db7e34898f9e8dc62b37f1
delta_sha256: sha256:96745909f178340a49a2609824942cae3014543e6e6eee0895b52f1739822621
active_same_domain_collisions: []
destructive_sync:
  removed_requirements: []
  large_modified_blocks:
    - Preserve Existing Restore and Ticket 02 Boundaries
  approval: parent explicitly authorized persistence of the already-synced result and supplied the accepted 9-added/1-modified/0-removed baseline; no destructive action was performed in this run
next_recommended: sdd-archive
```

## Sync Result

The `backup-restore` delta is already fully represented in `openspec/specs/backup-restore/spec.md`. Validation found no drift, so this run persisted only the missing sync report and did not alter canonical content.

## Requirement Changes

### ADDED

1. `Durable Prepared Evidence Before Disruption`
2. `Marker Progress Never Leads Durable Transitions`
3. `Marker Removal Follows Validated Readiness`
4. `Retain Fallback Sources Through Recovery Copy`
5. `Fail Closed on Required Durability Failure`
6. `Preserve Exact Restore Marker Compatibility`
7. `Bound the Durability Guarantee to Windows Local NTFS`
8. `Recycle Completed Restore Marker Sidecars`
9. `Require Real Windows NTFS Runtime Evidence`

These nine added requirements contribute 24 scenarios, and every full requirement block exactly matches the canonical specification.

### MODIFIED

- `Preserve Existing Restore and Ticket 02 Boundaries` — full requirement block and both scenarios exactly match the canonical specification.

### REMOVED

None.

### RENAMED

None.

## Preserved Canonical Content

Seven unrelated canonical requirement blocks and their 16 scenarios remain present:

1. `Recover After a Post-Disruption Restore Failure`
2. `Validated Readiness`
3. `Explicit Unavailable Terminal State`
4. `No Stale Connection Use During Recovery`
5. `Compatibility with Existing Recovery-Source Ordering`
6. `Bounded and Truthful Restore Results`
7. `Safe Error Disclosure`

## Validation Evidence

- Parsed delta sections independently and found 9 ADDED, 1 MODIFIED, 0 REMOVED, and 0 RENAMED requirements.
- Compared every complete ADDED and MODIFIED requirement block against the canonical file by exact requirement name and content; all 10 matched.
- Counted the canonical specification at 17 requirements and 42 scenarios.
- Confirmed the delta contains 10 requirements and 26 scenarios: 24 scenarios in ADDED requirements and 2 scenarios in the MODIFIED requirement.
- Confirmed 7 unrelated canonical requirements and 16 unrelated scenarios remain preserved.
- Confirmed no other active change contains `specs/backup-restore/spec.md`; the only other same-domain delta is already archived at `openspec/changes/archive/2026-08-31-recover-after-live-restore-failure/specs/backup-restore/spec.md`.
- Confirmed the sync report was missing before this run.
- Observed `openspec/specs/backup-restore/spec.md` as modified in the working tree, consistent with the prior sync; this run did not edit it.
- Reviewed `verify-report.md`: `PASS_WITH_WARNINGS`, 0 blockers, 0 critical findings, 10/10 requirements, and 26/26 scenarios.
- Verification evidence revision: `sha256:c8cfb644e1d7905f190583a1a53da9e91cb4a9456a971433a65204443fbe329b`.
- No product tests were rerun because the delegated task required structural sync validation only and prohibited product edits.

## Structured Status and Action Context

- Active change selection: explicit and unambiguous — `durable-restore-filesystem-transitions`.
- Status contract resolution: project override was absent; the global installed contract at `/home/luis/.pi/agent/gentle-ai/support/sdd-status-contract.md` was used.
- Artifact store: hybrid with authoritative file-backed OpenSpec content.
- Proposal, domain spec, design, tasks, and verify report are present; all 31 implementation tasks are complete.
- Sync report state before this run: missing. Sync report state after this run: present.
- Sync dependency: ready and satisfied because verification is passing with no blocker or critical finding.
- Action context: repo-local workspace `/home/luis/velay/repuestos_autos`.
- Allowed edit surface: only `openspec/changes/durable-restore-filesystem-transitions/sync-report.md`; this report is within that surface.
- Canonical and delta specification paths were treated as read-only.
- `rules.sync`: no sync-specific rules are configured in `openspec/config.yaml`.

## Warnings

1. The accepted Windows all-target/all-feature run compiled successfully but four unrelated pre-existing Tauri mock IPC tests failed; this is not represented as a passing runner.
2. The historical cumulative W2 review footprint exceeded 400 changed lines without a recorded `size:exception`.
3. The canonical specification has an existing working-tree modification from the prior sync; no canonical file was changed during this report-only run.

## Next Recommendation

The file-backed sync is validated and documented with no unresolved collision or destructive-sync blocker. The next recommended phase is `sdd-archive`; this run did not archive, commit, or push anything.
