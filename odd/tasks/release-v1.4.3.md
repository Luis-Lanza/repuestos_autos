# Release v1.4.3 — exact SKU preference

## Outcome
Published stable release: https://github.com/Luis-Lanza/repuestos_autos/releases/tag/v1.4.3
Published at 2026-10-21T00:05:59Z. GitHub readback confirms draft=false, prerelease=false and latest=v1.4.3, with exactly one installer asset.

## Objective and authorization
Deliver an unsigned Windows x64 NSIS patch giving eligible exact product codes precedence over broader search, without changing existing customer data or backup behavior.
User approved prepare_143_unsigned_commit_push, then explicitly selected authorize_v143_final_publication after Windows acceptance and exact transferred-artifact verification: evidence commit/push, new annotated v1.4.3 tag at tested source f2a887150ff0217b068d7160ce4d0d360479c658, tag push and exact unsigned installer publication after independent uploaded-byte verification.
No master merge or customer installation was authorized/performed. Preserve com.repuestosautos.app, existing tags, seven unrelated untracked documents, customer data and private keys.

## Tasks
- [x] R1: Synchronize the three app versions, verify source invariants and focused tests, complete native metadata review, commit/push Windows build source and verify remote identity. Work-unit commit: f2a887150ff0217b068d7160ce4d0d360479c658.
- [x] R2: Build unsigned Windows NSIS, pin source/artifact identity, verify an internal fixed-NTFS backup and manually accept the 1.4.2-to-1.4.3 in-place update plus old-backup restoration/restart with synthetic records. Acceptance evidence committed/pushed as e515bbb8787e23cc831b13b7d12a89e51446ce5c.
- [x] R3: Receive the exact tested installer, verify bytes, obtain explicit final authority, create/push annotated tag, upload to draft, independently download/compare, publish and read back stable release. Publication closure is recorded by the commit containing this document.

## Source and scope
- Branch release/v1.4.3 from 0192c4cb6f075b718bb9199f30cee3a43582aec7.
- Reviewed behavior commit 4769042120b7c85418909e084570121348ab7b95; native review review-77a6fa513cfd920e approved and acknowledged.
- Release preparation changed exactly one 1.4.2-to-1.4.3 app-version entry in each of src-tauri/tauri.conf.json, src-tauri/Cargo.toml and src-tauri/Cargo.lock. Identifier, dependencies and features unchanged.
- Build-source commit f2a887150ff0217b068d7160ce4d0d360479c658. Windows build HEAD ef3e535f6202dee9a6bc296eb64348725cb9cd77 differs only by tracker evidence; Windows git status --short was empty.
- No schema, persisted-data, backup/restore, stored-image or licensing implementation changes. No customer database or private-key material accessed.

## Automated verification
- Feature verification: 474 default Rust tests, 36 focused search/browse tests, 17 release search tests including two 20,000-product checks, and three targeted synthetic backup-source preservation checks passed. Query-only execution, table/FTS/schema snapshots and total_changes() confirm no search writes.
- Version writer mvhbvvv6-6-xsj3 and independent verifier mvhbxbjp-7-3d0i confirmed exactly three substitutions, locked offline cargo metadata reporting app 1.4.3, 36 focused tests passing and clean git diff --check.
- No meaningful RED for passive version-only metadata; prior behavioral change has observed RED/GREEN.
- Native metadata review review-9440b310109219be approved and exact acknowledgement burned authority at revision sha256:8718a6edfdc43f26a57b9957856dc05453cb7439801a1918fbe770cc218978f6. Reviewed tree 2340139b95421b98064251f45c969d713c0e92f3 matches metadata source.
- ASSESS initially unavailable due undeclared untracked paths; conservative independent verification completed. Whole-file formatting differences predate the feature; introduced regions were normalized without unrelated reformatting.
- Known unused helper warnings remain; no failing checks. Full clean-install/offline/crash/accessibility coverage was not performed.

## Windows manual evidence
- User initially had no installed 1.4.2. Installed the published previous release and created own synthetic BKR6E-11/BKR6EIX-11 records; no customer data required.
- User confirmed 1.4.2 backup restoration with correct products/prices/quantities after close/reopen.
- User reported successful 1.4.3 update/search/data/license/screen rehearsal, then explicitly confirmed restoration of the 1.4.2 backup in 1.4.3 with correct data after restart and internal NTFS backup placement.
- These checks are human-reported acceptance, not independent exhaustive Windows certification.
- Windows build command: npm run tauri -- build --features desktop --bundles nsis. User log reports optimized build in 54.25 seconds and one x64 NSIS bundle. Nonfatal warnings: backup_response_diagnostic and reconcile_cleanup_evidence.

## Installer identity
- Exact accepted local file: /tmp/repuestos-release-v1.4.3-HLcNyNVu/Repuestos Autos_1.4.3_x64-setup.exe
- Size: 4,393,920 bytes.
- SHA-256: 332deb41eabc0aae5ef000ea8295207414786b13dc998247ad8c4d98d6772d7c
- Authenticode: NotSigned, matching explicit approval. Same tested bytes published; no rebuild/substitution.
- SCP receipt independently matched Windows size/hash on Linux. The NSIS PE32 launcher identification is not the payload architecture.

## Publication evidence
- Annotated tag v1.4.3 object 09550de4895da16481e5118308310d2f5d9bea48 peels to f2a887150ff0217b068d7160ce4d0d360479c658; remote verified. No existing tag moved.
- GitHub release 418999701, node RE_kwDOUAEpZM4Y-W2V, target f2a887150ff0217b068d7160ce4d0d360479c658.
- Exactly one asset: 660846457, Repuestos.Autos_1.4.3_x64-setup.exe, expected size/digest.
- Independent verifier mvhcp7hg-8-84o1 downloaded the actual uploaded asset to /tmp/repuestos-v1.4.3-independent-3TBdGf3u/Repuestos.Autos_1.4.3_x64-setup.exe. Size and SHA-256 matched; byte-for-byte cmp exit 0.
- Fresh prepublication checks reconfirmed draft identity, target, one asset, local/download hashes and unchanged remote tag. One PATCH draft=false published the authorized release; numeric release and latest readbacks confirmed success.
- GitHub reported immutable=false: tag preservation follows the release procedure; platform-enforced immutability was not established.
- Download: https://github.com/Luis-Lanza/repuestos_autos/releases/download/v1.4.3/Repuestos.Autos_1.4.3_x64-setup.exe

## Next step
Release is published. Customer installation remains a separate operation: create/verify a backup first, close the app and update in place without uninstalling or deleting data. Optional source/metadata integration into master requires a separate decision; master was not modified by this release.
