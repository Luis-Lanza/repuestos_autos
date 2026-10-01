# Backup and Restore Integrity Hardening

## Objective
Make backup and full-database restore safe enough to act as the customer's recovery mechanism: a restore replaces the complete SQLite database snapshot or fails closed while retaining recoverable evidence.

## Product Policy
- A backup is a consistent snapshot of the complete customer SQLite database at one point in time.
- A restore replaces the complete database; it never merges individual catalog, stock, or sale rows.
- The application preserves a protective current-database copy before replacement and recovers it after an interruption when possible.
- License files remain outside the SQLite backup/restore boundary.
- On uncertainty, startup must recover a validated retained database or fail closed; it must never silently create an empty database over recoverable evidence.

## Findings To Correct
- Missing current restore marker can strand valid rollback/protective files while startup creates a new empty canonical database.
- Recovery selection accepts only an old schema shape and can skip a valid fallback.
- Backup publication lacks complete directory durability/cleanup semantics and assumes hard-link support.
- Backup destination IPC can bypass the picker and follows symlinks/junctions.
- Invalid, expired, cancelled, and restarted restore stages can accumulate.
- Frontend maps `license_required` restore denial to a storage error.

## Recovery Defaults
- If canonical and fallback databases both validate after an interrupted restore, preserve canonical and retain fallback evidence; never overwrite a valid canonical automatically.
- Support the safe backup/restore path only on NTFS until removable FAT/exFAT durability is independently proven.

## Tasks
- [x] T1 Map recovery state machine, canonical/fallback source selection, destination filesystem assumptions, and exact test seams. Route: delegated exploration.
- [x] T2 Implement fail-closed startup recovery and version-aware fallback validation with crash-boundary tests. Route: delegated writer. Independent-review blocker resolved: recovery artifact inspection uses non-following metadata, ambiguous marker/temporary/sidecar evidence fails closed, and recovery opens existing validated databases without creation. Verified with backup/restore and all Rust package suites; Windows runtime behavior remains unverified.
- [x] T3 Harden backup publication and validate destination handles/paths. Route: delegated writer. Automated verification covers opaque picker tokens, path-free responses, NTFS/reparse checks, publication fault injection, and distinct durability/cleanup warnings; Windows compilation/runtime evidence remains pending.
- [x] T4 Correct frontend restore-denial decoding and add behavioral tests. Route: delegated writer. `license_required` now has a restore-specific bounded message.
- [ ] T5 Independently verify automated failures/recovery plus Windows fixed-NTFS and removable-media evidence. Route: delegated verifier.
- [x] T7 Implement deterministic restore-stage cleanup failure handling and injectable cleanup seams. Terminal stage deletion is now preceded by a durable bounded evidence record; cleanup failures propagate as `storage_unavailable`, and startup reconciliation/abandoned cleanup accept the same private adapter seam. Focused and full verification passed on this host; Windows behavior remains unverified.
- [x] T8 Complete deterministic cleanup-failure matrix coverage: parent-directory sync failures at evidence/artifact/evidence-removal boundaries; prepare invalid/unsupported and post-stage metadata/size/checksum failures; confirm expiry, unconfirmed, checksum-invalid, pre-marker and successful-install cleanup failures; expired/aged pruning; and startup reconciliation/abandoned-stage fail-closed behavior. Every injected failure asserts bounded storage failure plus retained evidence and appropriate artifact/marker ownership. Focused and full Rust suites passed; `git diff --check` passed after the task update.
- [x] T6 Resolve restore-stage audit blockers: inventory every named marker sidecar fail-closed, reclaim abandoned pre-rename stages only after fallback recovery completes, and persist bounded cleanup evidence when stage removal fails. Route: delegated writer. Follow-up audit correction routes every restore-stage terminal cleanup through one evidence-and-directory-sync path and preserves specific license denial feedback through restore prepare/confirm. Local verification passed: `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore`, `cargo test --manifest-path src-tauri/Cargo.toml`, `npx tsx --test --import ./test/react-dom.ts src/commands/backup.test.ts src/ui/backup/backup-flow.test.ts src/ui/backup/backup-screen.mounted.test.ts src/ui/app-shell.mounted.test.ts`, `npm run typecheck:tests`, and `git diff --check`. Windows fixed-NTFS/runtime evidence remains pending.
- [x] T9 Close Linux cleanup-accounting gaps: preserve a published backup as `Created` with `cleanup_warning` only when evidence-backed snapshot cleanup fails; propagate cleanup failures from post-stage validation, align recording evidence bytes with production newline semantics, and cover pruning from backup-creation and picker-selection callers. W9 now audits only this candidate's exact paths and frontend/backend evidence. Linux verification passed: backup/restore integration, full Rust package tests, focused backup UI tests, test typecheck, full npm tests, and `git diff --check`. Windows fixed-NTFS/runtime evidence remains pending.
- [x] T10 Resolve the final independent-audit blockers: stage supported fallback recovery through migration into the same-volume UUID stage and durably install that exact stage; restrict the durable recovery verifier to existing rollback/protective sources or UUID-named same-volume recovery stages; return a cleanup warning only when cleanup failure has durable evidence; bind every changed backup-integrity file to its exact unified-zero diff SHA-256 in W9 and cover tampering negatively. Linux verification passed: `cargo test --manifest-path src-tauri/Cargo.toml --test backup_restore`, `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, `npm run typecheck:tests`, `npm run build`, and `git diff --check`. Windows fixed-NTFS/runtime evidence remains pending.

## Acceptance Criteria
- A missing canonical database never results in a newly created empty database when validated recovery evidence exists.
- A current database with an invalid current schema falls back to a validated retained source when one exists.
- Every restore terminal path reclaims or durably accounts for its stage; startup bounds abandoned artifacts.
- Backup publication either returns a durable, validated backup or returns a safe failure without ambiguous success.
- Backup destinations cannot be redirected through unvalidated picker bypasses, symlinks, or junctions.
- Restore denial due to license state is accurately presented.
- Windows validation covers successful full restore, interrupted restore recovery, invalid backup, fixed NTFS, and expected behavior on removable FAT/exFAT media.

## T3 Review Corrections
- Backup and restore picker outputs are opaque, native-held tokens; token TTLs use monotonic deadlines. This correction does not add cleanup for abandoned restore stages or stage lifecycle retention.
- Backup creation responses expose only the generated file name and summary; final-directory durability and internal temporary cleanup warnings remain distinct. A final-directory sync failure is an explicit non-durable publication warning, never a durable-success guarantee.
- Destination validation checks NTFS and rejects currently observed reparse-point ancestors, then revalidates before publication. Publication still uses path-based Win32 operations, not a directory-handle-relative transaction. An adversary able to replace path components between validation and filesystem operations may still redirect access; this residual is not claimed to be race-proof.
- Windows desktop-gated compilation and fixed-NTFS runtime behavior require a Windows environment; neither is established by host-independent tests.

## Restore-stage lifecycle implementation
- Restore confirmation tokens remain in-memory, single-use capabilities with monotonic deadlines; expired/rejected/cancelled stages are reclaimed, and checksum or pre-marker installation failures cannot leave a replayable stage.
- Startup removes bounded abandoned stage/snapshot artifacts only when no marker or recovery evidence exists; ambiguous entries and cleanup failures leave storage unavailable. Active-marker stages remain owned by recovery.
- Stage/snapshot admission is bounded by a maximum of eight artifacts and 512 MiB per database; excess or ambiguous state returns a stable storage error. Windows durable installation and cleanup still require platform validation.

## Next Step
- Complete T5's automated verification and validate the full protocol on Windows fixed NTFS plus expected FAT/exFAT rejection.
