# Release v1.2.3

## Objective
Prepare a newer Windows NSIS installer containing the accepted Sales original-image zoom fix, then validate an in-place update over v1.2.2 before any publication.

## Authorization
User explicitly approved merging PR228 and preparing v1.2.3 with scoped commits/pushes. Unsigned installer preference preserved. Tag, GitHub Release and customer installation remain unauthorized; require separate approval after successful smoke. Never move/reuse v1.2.2 or any existing tag.

## Constraints
Keep identifier `com.repuestosautos.app`, schema, dependencies, licensing and import limits unchanged. Preserve six unrelated untracked task documents. Align only root app version fields in Cargo.toml/Cargo.lock/tauri.conf.json. Keep original zoom behavior/tests unchanged from accepted PR228. User explicitly approved generated outputs under `src-tauri/target/` and `dist/` for offline Cargo check and Vite build, while retaining the three-file source-edit limit; no tracked generated output changes. Do not equate user manual smoke with complete database-record equality.

## Tasks
- [x] R1 MERGED: PR228 merged with pinned HEAD3171fc79a3ff5a728dc87f46228a346cc5210f48 as `e34f4351ccd1f134e648494bedeca2fdb5909eb7`; fetched origin/master matches. Created `chore/release-1.2.3` at that merge; no branch deletion.
- [x] R2 VERIFIED: Exact three root version replacements independently verified against e34f435; unchanged dependencies/identifier, Cargo check and buildPASS. Version-only work-unit commit `4fbc4b0f1f832b2096f86af70853dbd8f03cbbd6`; release evidence documentation committed alongside delivery, push authorization granted.
- [ ] R3 IN PROGRESS: Build exact Windows NSIS installer and verify source/size/SHA256; verify fixed-NTFS backup, then manual in-place update over1.2.2 including data/license, local Catalog access, Sales/browse/checkout zoom, Inventory, Reports and restart.
- [ ] R4 Obtain separate tag/publication approval after smoke, create immutable annotated v1.2.3 tag and stable GitHub Release attaching exact tested installer, then verify publication identity. No customer installation inferred.

## Evidence and limitations
PR228OPEN/MERGEABLE, pinned HEAD3171fc7, no reported required checks (empty rollup is not CI PASS); master unprotected. Remote v1.2.3 tag absent and latest releasev1.2.2 at preflight. Accepted fix91focusedUI/10native independently passed, writer419npm/441Rust and buildpassed. Production typecheck has94 exactpre-existing diagnostics, zero candidate additions. Desktop automated IPC unexecuted due unavailable GTK/WebKit prerequisites. W9 last24/30FAIL, not rerun or changed. No all-checks-green claim.

## R2 verification
Writer muw2183v-k-79k6 completed exactly three version replacements, +3/-3. Structural assertions verify root versions1.2.3, stableidentifier and identical dependencies. Offline Cargo checkPASS (six existing dead-code warnings), npm buildPASS (75modules, existing mixed-import warning), diffcheckPASS. Metadata-only change has no meaningful behavioral RED; ordinary structural checks used. No generated tracked outputs. Independent verifier muw23ddu-l-fhr5 PASS: exact byte comparisons before/after rerun Cargo check and build, normalized parsed lock graph identical, identifier stable, untracked-document hashes preserved. Native ASSESS unassessable due untracked scope, RDDoff, runtimewriterlarge: conservative high-risk plan requires independent verification before commit/push.

## Next step
Push version/evidence commits on `chore/release-1.2.3`, then build on a clean Windows checkout at the delivered HEAD with `npm run tauri -- build --features desktop --bundles nsis`. Record exact git HEAD, empty git status, installer byte size and SHA256. Before installation, create/verify a backup on fixed NTFS; update the installed1.2.2 in place (no uninstall), then check all R3 items. Tag/publication remains separately unauthorized.
