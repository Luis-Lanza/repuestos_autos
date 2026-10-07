# Release v1.4.0

## Goal and authorization
Publish a Windows NSIS release containing repeatable product names, globally unique SKU validation, and the historical recovery-evidence compatibility correction. User explicitly requested a release and confirmed v1.4.0 with an unsigned Windows installer ("dale!"), a fresh exception to the deployment guide's signed-installer recommendation.

Preparation and version alignment are authorized. User explicitly approved verified metadata/evidence commits, release/v1.4.0 push, fast-forward integration into master and master push ("autorizo paaa, usa el skill"). Annotated tag creation/push, GitHub upload/publication and customer delivery still require explicitly scoped approval before execution. Prior release and feature delivery permissions are not inherited. No source behavior changes, dependency updates, real/customer database access or automatic native app launch are authorized.

## Source and preservation
- Release branch: release/v1.4.0, from verified feat/repeated-product-names source 9bcac8d7d525a0f3cb6f3336445dbc3968b93ba1.
- Preflight master local/remote: 4430d33b1c1f7df8784e03b14469cd324c4b99d7; fast-forward ancestry observed. Recheck before integration.
- Remote/local v1.4.0 tag and release branch absent at preflight. Recheck before creation; never force, move or reuse tags.
- Latest published stable: v1.3.1. Its installer, tag and tested source remain immutable. Later evidence commits are not installer build sources.
- Preserve com.repuestosautos.app and all six unrelated untracked ODD documents. Parent owns this document, full Engram mirror and visible todo projection; one bounded writer at a time.
- The feature has independently observed 457 default Rust tests and eight startup regressions passing. Earlier 479 frontend tests/build passed before the recovery-only Rust correction; zero frontend diff in that correction. Human confirmed source 9bcac8d and aggregate Windows functional acceptance ("funciona todo joyita"); individual-case proof and installer acceptance are not inferred.

## Tasks
- [ ] R1 (in_progress; independent verification PASS, commit authorized): Align only the application versions in src-tauri/tauri.conf.json, src-tauri/Cargo.toml and the repuestos-autos package entry in src-tauri/Cargo.lock to 1.4.0. Preserve the identifier, dependency graph and all unrelated bytes. Observe JSON/TOML parsing, exact baseline replacement comparison, lockfile package/dependency equality and diff checks. Version-only metadata has no meaningful behavioral RED; use structural verification, native assessment and the returned verifier plan. Close only with verified work-unit commit identity after explicit approval.
- [ ] R2 (pending): With explicit approval, commit verified metadata/docs, integrate by fast-forward into master, push the authorized refs and verify remote identities/current policy. Pin the exact committed clean Windows build source. Record integration evidence separately; no tag or publication yet.
- [ ] R3 (pending): On authorized Windows infrastructure, build the NSIS installer from the pinned source; record filename, size, SHA-256, embedded version and unsigned status. Verify backup on fixed NTFS and an in-place update over 1.3.1 without uninstalling. Obtain human acceptance of app launch, preserved data/backups/license/device-local Catalog access, Sales, Inventory, Reports and repeatable names/distinct SKU. Stop on failed checks; preserve recovery artifacts. Record untested clean/offline installation, license-negative, restore/crash or accessibility cases honestly, not as passed.
- [ ] R4 (pending): After exact installer acceptance and explicit publication/tag approvals, create a new immutable annotated v1.4.0 tag pointing to the tested build source, push the tag, attach verified installer bytes, independently download/compare size/hash/bytes, and publish the GitHub Release. Disclose unsigned status, schema24 forward migration and retained limitations. Record release URL, asset identity and final outcome; customer delivery is separately authorized.

## R1 allowed writer surfaces
- src-tauri/tauri.conf.json
- src-tauri/Cargo.toml
- src-tauri/Cargo.lock

## Verification and operational gates
- Metadata writer must not edit this document or unrelated feature trackers. No lockfile dependency regeneration, installations, version changes outside the three app declarations, or unrelated formatting.
- RDD last observed off. After writer completion call assessment and follow its exact plan; unassessable is treated as high risk and requires an independent verifier. Applicable functional checks remain separate from native review.
- Windows build command: npm run tauri -- build --features desktop --bundles nsis. Expected bundle directory: src-tauri\target\release\bundle\nsis\. Linux checks do not prove Windows build/NTFS durability/desktop IPC.
- Vite build is not TypeScript verification. Retained production typing debt is 92 diagnostics; the historical test-typecheck excludes TypeScript test roots. No new candidate failures are waived.
- Before installation, verify a backup on fixed NTFS. Same-PC update requires no new license; SQLite backup does not contain file-based license or device-local Catalog-access configuration. Never include license private keys or customer files in artifacts.
- Schema24 cannot be assumed downgrade-compatible with v1.3.1/schema23. Do not advise uninstalling, deleting retained recovery evidence, or reinstalling an older app as a rollback.
- Record aggregate human acceptance as aggregate; no invented case-level, crash, browser, assistive-technology or restore evidence. A newly built installer must be tested even though development source already passed smoke checks.

## Progress
Read-only explorer muxkalun-16-wija mapped the three version declarations, deployment guidance and prior release evidence. Parent independently checked GitHub stable v1.3.1 metadata and Git refs. Version/signing approved; release branch and this tracker created before any version writes. R1 writer muxkgabh-17-lilj completed exactly three version replacements (+3/-3). JSON/TOML parsing, full-byte baseline comparisons and unchanged dependency graph/identifier checks PASS; offline locked Cargo metadata reports root1.4.0; diff check PASS. Parent spot-check confirms only version changes. Native assessment is unassessable because unrelated untracked files require declarations; RDD off returns the high-risk independent-verifier plan. Independent verifier muxkikk6-18-oa0q PASS: all three full files equal baseline with one replacement, parsed semantics match after restoring old target version, complete516-package lock graph unchanged, identifier preserved; offline locked metadata and diff checks exit0. Git branch/HEAD, tracked/untracked hashes and scope remain unchanged. Current GitHub master protected=false and rulesets empty, with remote4430d33; release branch not yet remote. Verified metadata/docs commits, release-branch push, master fast-forward integration and master push are now explicitly authorized; execution is next. No meaningful behavioral RED applies to passive metadata; no Windows build or functional proof is inferred. No new commit, master integration, tag, installer build or publication yet.
