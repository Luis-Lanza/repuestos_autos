# Release v1.2.3

## Objective
Prepare a newer Windows NSIS installer containing the accepted Sales original-image zoom fix, then validate an in-place update over v1.2.2 before any publication.

## Authorization
User explicitly approved merging PR228 and preparing v1.2.3 with scoped commits/pushes. Unsigned installer preference preserved. After explicit verified-backup/full-smoke confirmation, user separately authorized immutable annotated v1.2.3 tag/push and GitHub Release attaching exact tested installer, conditional on independent Linux SHA256 verification first. Customer installation remains unauthorized. Never move/reuse v1.2.2 or any existing tag.

## Constraints
Keep identifier `com.repuestosautos.app`, schema, dependencies, licensing and import limits unchanged. Preserve six unrelated untracked task documents. Align only root app version fields in Cargo.toml/Cargo.lock/tauri.conf.json. Keep original zoom behavior/tests unchanged from accepted PR228. User explicitly approved generated outputs under `src-tauri/target/` and `dist/` for offline Cargo check and Vite build, while retaining the three-file source-edit limit; no tracked generated output changes. Do not equate user manual smoke with complete database-record equality.

## Tasks
- [x] R1 MERGED: PR228 merged with pinned HEAD3171fc79a3ff5a728dc87f46228a346cc5210f48 as `e34f4351ccd1f134e648494bedeca2fdb5909eb7`; fetched origin/master matches. Created `chore/release-1.2.3` at that merge; no branch deletion.
- [x] R2 VERIFIED: Exact three root version replacements independently verified against e34f435; unchanged dependencies/identifier, Cargo check and buildPASS. Version-only work-unit commit `4fbc4b0f1f832b2096f86af70853dbd8f03cbbd6`; release evidence documentation committed alongside delivery, push authorization granted.
- [x] R3 MANUAL PASS: User confirms verified fixed-NTFS backup and in-place1.2.2→1.2.3 data/license, Sales/checkout original zoom with Catalog locked, Inventory, Reports and restart. Windows source/size/SHA256 recorded below; exact transferred artifact independently verified before R4 publication. Acceptance evidence is documentation-only; no rebuild required.
- [x] R4 PUBLISHED: Stable latest release404262192 https://github.com/Luis-Lanza/repuestos_autos/releases/tag/v1.2.3, draftfalse/prereleasefalse. Annotatedtagbff7ebfc270ac890bd6c39d2e64bd985e09300cd peels to tested672d488334d574ad79162efdaccd78ec2e52e305, remoteverified. Asset614277341 `Repuestos.Autos_1.2.3_x64-setup.exe`,4372287bytes, serverdigestsha256:3c6705eca8d1dcb4a94e833a6e93198cdb695b850eab19827166dd8bd49d9958matches exacttestedinstaller; verified in draft before publish. No customer installation inferred.

## Evidence and limitations
PR228OPEN/MERGEABLE, pinned HEAD3171fc7, no reported required checks (empty rollup is not CI PASS); master unprotected. Remote v1.2.3 tag absent and latest releasev1.2.2 at preflight. Accepted fix91focusedUI/10native independently passed, writer419npm/441Rust and buildpassed. Production typecheck has94 exactpre-existing diagnostics, zero candidate additions. Desktop automated IPC unexecuted due unavailable GTK/WebKit prerequisites. W9 last24/30FAIL, not rerun or changed. No all-checks-green claim.

## R2 verification
Writer muw2183v-k-79k6 completed exactly three version replacements, +3/-3. Structural assertions verify root versions1.2.3, stableidentifier and identical dependencies. Offline Cargo checkPASS (six existing dead-code warnings), npm buildPASS (75modules, existing mixed-import warning), diffcheckPASS. Metadata-only change has no meaningful behavioral RED; ordinary structural checks used. No generated tracked outputs. Independent verifier muw23ddu-l-fhr5 PASS: exact byte comparisons before/after rerun Cargo check and build, normalized parsed lock graph identical, identifier stable, untracked-document hashes preserved. Native ASSESS unassessable due untracked scope, RDDoff, runtimewriterlarge: conservative high-risk plan requires independent verification before commit/push.

## R3 Windows artifact evidence
Pre-build user readback: release branch HEAD `672d488334d574ad79162efdaccd78ec2e52e305`, empty `git status --short`. Delivered version commit4fbc4b0 and evidence672d488 match remote. User reports NSIS installer `Repuestos Autos_1.2.3_x64-setup.exe`, size **4372287 bytes**, SHA256 **3C6705ECA8D1DCB4A94E833A6E93198CDB695B850EAB19827166DD8BD49D9958**. Build exit/output and post-build clean status not supplied. Transferred installer independently verified by muw2w5rv-m-sbkg: regular nonsymlink, exactsize/SHA256 and MZ/PEboundsPASS; bootstrapPE32 is not payload architecture or Authenticode proof. Compared694trackedfilesagainsttestedsource, all byte-identical except authorized release task documentation. Tag/release absence rechecked before creating either. User explicitly confirmed verified fixed-NTFS backup and entire installed-app checklist after reporting 'listo anda bien papu'. Manual smoke PASS is human-reported; not automated IPC proof or complete before/after database equality.

## Next step
Release complete; download attached exact tested installer. Do not replace/recompile the asset or move/reuse any existing tag. Customer-specific backup/update verification remains necessary. Master contains the zoomfix merge, while v1.2.3versionpreparation remains on released branch/tag; separate metadata PR/merge is not authorized. Final publication evidence is documentation-only on the release branch; no production change since tested672d488. No customer installation authority inferred.
