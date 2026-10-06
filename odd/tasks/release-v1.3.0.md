# Release v1.3.0

## Goal
Merge the action-notification preview and Catalog table scroll correction, prepare version 1.3.0, and publish a Windows NSIS release after verified update acceptance.

## Authorization and constraints
User explicitly requested merge and publication, selected version 1.3.0 and an unsigned Authenticode installer like the previous release. Version preparation is authorized on the existing feature branch. Prior explicit feature-branch commit authority applies; confirm external transfers and tag/publication scope before executing them. Never force/move/reuse tags, merge unrelated files, uninstall the existing app, or mutate business data during checks. Preserve six unrelated untracked ODD documents. Identifier stays com.repuestosautos.app. No dependency, Rust behavior, IPC, schema or root npm metadata changes.

## Preflight
Current feature branch feat/action-notifications-preview at eef639d; live origin/master e34f4351ccd1f134e648494bedeca2fdb5909eb7 is its merge base, ahead10/behind0. Local master e3ba3cf is older and is not the current release base. Latest stable v1.2.3 installer is Repuestos.Autos_1.2.3_x64-setup.exe (4,372,287 bytes); no v1.3.0 remote tag found. Three app versions are currently1.2.3: tauri.conf.json, Cargo.toml and root Cargo.lock. Root npm metadata has no app version. No repository Windows release workflow exists; use manual Windows build/transfer.

Previous candidate checks: 476/476 tests (zero failed/skipped/cancelled), test typecheck/build/diff PASS; production typecheck FAIL with93 unchanged baseline diagnostics. Windows user confirms table wheel scrolling fixed after556faee and notifications appeared to work. Contrast/modal/screen-reader and full installer update acceptance remain unverified. Native RDD observedOFF.

## Tasks
- [x] R1 (verified; committed 5f046abecf9ed32e14bc7fff7d233a2cabbf9943): Exactly three root app version entries changed to1.3.0 (+1/-1 each file). Writer and independent JSON/TOML parsing and whole-file single-replacement comparisons PASS; identifier/dependencies/signing unchanged. Parent diff/whitespace spot-check PASS. No meaningful behavior RED for metadata. ASSESS unassessable from unrelated untracked files required fulfilled independent verification; no fresh functional suite was claimed.
- [ ] R2 (in_progress; awaiting explicit master push authority): Independent traditional protection query404 (resource/access ambiguity) and repository rulesets successful withzero rulesets. Recheck remote master, preserve tracking/untracked files, integrate verified release candidate into master by fast-forward under ordinary repository policy and transfer exact build candidate to Windows only after explicit master push authority. Do not publish a tag yet.
- [ ] R3 (pending): Build unsigned Windows NSIS using npm run tauri -- build --features desktop --bundles nsis; record artifact SHA-256/size. Require verified backup on fixed NTFS and in-place update over previous installed version without uninstalling. Verify launch/restart, data, license, device-local Catalog access, Sales, Inventory, Reports, notifications and table/gallery/edit scrolling. Record failures honestly; no customer delivery while checks fail.
- [ ] R4 (pending): After accepted installer smoke and explicit tag/push/publication authority, create immutable annotated v1.3.0 tag for exact installer source, publish GitHub Release and attach that verified installer. Verify remote tag, release state and asset identity; state unsigned status and remaining production type debt.

## Verification policy
Version metadata has no meaningful behavior RED; use manifest/lock readback and proportionate runnable structural checks, preserving dependency graph. Applicable functional checks remain mandatory for source changes; installer smoke cannot be replaced by development preview or jsdom declaration tests. Linux cannot establish Windows update success.

## Evidence and next step
Version/signing selected by human; R1 committed5f046ab (chore(release): prepare application version1.3.0) after independent structural verification. No merge, tag, release or new remote transfer yet. Parent owns this document/mirror. Confirm explicit master push authority before fast-forward integration and Windows build transfer. No fresh tests/build/native/database checks were needed for the exact metadata-only change; prior476 tests/testtypes/build evidence and93baseline production diagnostics remain historical. Windows build, verified backup and all installer smoke checks are pending. Feature evidence: odd/tasks/action-notifications.md; deployment contract: docs/DEPLOYMENT_AND_LICENSING.md.
