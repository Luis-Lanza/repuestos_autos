# Release v1.2.2

## Objective
Publish a new Windows installer restoring assigned locations in Sales and checkout details without reusing v1.2.1.

## Authority and constraints
- User explicitly approved v1.2.2, merge PR227, version alignment, scoped commits/pushes, and immutable annotated tag/GitHub Release only after installer update acceptance over1.2.1.
- Preserve unsigned installer preference and identifier com.repuestosautos.app.
- No dependency, schema, licensing, Catalog-access, data or unrelated behavior changes.
- Preserve six unrelated untracked task documents; no client installation or destructive Git operations.
- Remote master currently de45dbd, unprotected; no rulesets or check rollup returned. Empty checks are not CI PASS. PR227 current head40623f4 and CLEAN.
- W9 last24/30FAIL remains a disclosed policy/evidence limitation; no audit changes. Automated desktopIPC tests unexecuted; developer Windows smoke manually accepted, not installer acceptance.
- Delivery strategy ask-on-risk; forecast <100 authored metadata/evidence lines, one release preparation slice.

## Tasks
- [x] R1 Integrate PR227 and align application root versions to1.2.2 on chore/release-1.2.2. Writer/independent verifierPASS; authorized work-unit commit a0d5124fc0e86f6363cb94aa318db65e648c2fb5 observed and pushed. Only3version fields plus this task document committed. Parent handles pinned authorized Git delivery; delegated worker changes Cargo.toml,Cargo.lock,tauri.conf.json. Use parsed structural verification, not RED/GREEN: passive version metadata has no meaningful failing behavior test. Commit versions/tests evidence as one work unit; dependencies/identifier unchanged.
- [ ] R2 Build Windows NSIS and record installer/update manual acceptance. IN PROGRESS: awaiting user-assisted Windows build from a0d5124 or its documentation-only successor. User runs npm run tauri -- build --features desktop --bundles nsis. Obtain artifact size/SHA256; verify backup on fixed NTFS before in-place test over1.2.1 (no uninstall), data/license/device-local Catalog access/Sales/Inventory/Reports/restart. Clarify disposable developer fixtures vs customer data. No artifact or acceptance inferred from dev smoke.
- [ ] R3 Publish authorized immutable annotated v1.2.2 tag and GitHub Release with exact tested installer attached. Verify released tree equals tested build, digest identity and public release state. No tag reuse/movement; do not publish before R2PASS. PR/merge of separate release metadata requires explicit authorization if repository policy chooses that route.

## Checks and evidence
Previous bugfix: frontend415/415, focused87/87, typecheck/buildPASS; writer full defaultRust439/439 and independent focused36/36PASS. Windows developer smoke reportedPASS for assigned/unassigned/browse/checkout/lockedCatalog. No new release checks yet.

## Progress
Release skill and deployment guide read. Authorization confirmed; v1.2.2 remote tag absent at preflight. Pinned PR227 merge observed as0b9b78521ce62838a700bbc50022e495e6a82c2a; chore/release-1.2.2 starts at this fetched origin/master. Version-only writer completed exactly3one-line hunks1.2.1->1.2.2; parsed structural comparison and offlineCargo metadataPASS, dependencies/identifier unchanged. Parent hunk readback/diff-checkPASS. NativeASSESSunassessable due untracked scope,RDDoff; Independent structural verifierPASS: parsed/textual HEAD comparison proves exactly3version replacements, offlineCargo metadata1.2.2PASS, diffcheckPASS and7untracked document hashes unchanged during verification. No Windows evidence inferred. This Linux environment cannot build/test Windows installer; user-assisted Windows steps required.

## Next step
User fetches/switches/pulls chore/release-1.2.2 on Windows, reports clean tracked checkout and git rev-parse HEAD, builds NSIS, supplies log/size/SHA256. Then confirm fixed-NTFS verified backup and in-place1.2.1->1.2.2 smoke on test PC. Continue to tag/publication only after exact installer acceptance. Metadata PR/merge route not yet authorized.
