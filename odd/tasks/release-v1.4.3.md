# Release v1.4.3 — exact SKU preference

## Objective
Prepare an unsigned Windows x64 NSIS patch so exact product references do not return similar SKUs, without changing existing customer data or backup behavior.

## Authorization and constraints
User reported search works on Windows and requested release. User selected prepare_143_unsigned_commit_push: explicitly approved v1.4.3 unsigned preparation and versioned source commit/push for Windows build. Final authority subsequently granted via authorize_v143_final_publication after manual acceptance and exact transferred-artifact verification: evidence commit/push, new annotatedv1.4.3tag atf2a887150ff0217b068d7160ce4d0d360479c658,tagpush and exactunsignedEXEpublication after independent uploaded-byte verification. No master merge authorized or needed for this preparation; branch from verified feature HEAD0192c4c. Preserve com.repuestosautos.app, existing tags, seven unrelated untracked docs and customer privacy. No schema/storage/backup/restore/license changes. Do not access customer databases or private keys. Do not install over customer data. No reusing v1.4.2 tag/installer.

## Branch and baseline
- release/v1.4.3 from0192c4cb6f075b718bb9199f30cee3a43582aec7.
- Reviewed behavior commit4769042120b7c85418909e084570121348ab7b95; native review review-77a6fa513cfd920e approved and acknowledged.
- Latest stable release v1.4.2; preflight no remote v1.4.3 branch/tag; manifests currently1.4.2; tracked clean.

## Tasks
- [x] R1 (verified; build-source commit f2a887150ff0217b068d7160ce4d0d360479c658 pushed and exact remote verified): Synchronize app version1.4.3 in Tauri manifest, Cargo manifest and app Cargo.lock entry only; verify metadata/source invariants and focused tests, native review if consented, commit/push Windows build source and verify remote identity.
- [x] R2 (human acceptance complete; evidence committed with this document): Build unsigned NSIS from pinned source, record clean source/EXE size/SHA256/signature; verify a backup on fixed NTFS, update existing1.4.2 test installation without uninstalling, check exact/partial search, data/license/Catalog/Sales/Inventory/Reports and old-backup restore/restart using synthetic data.
- [ ] R3 (in progress; final publication authorized, exact installer verified): Transfer exact tested installer, verify bytes independently, obtain final annotated tag/push/publication authority; create draft/upload, independently download/hash, publish and read back stable release.

## Allowed preparation surfaces
- src-tauri/tauri.conf.json
- src-tauri/Cargo.toml
- src-tauri/Cargo.lock (app package version only)
- odd/tasks/release-v1.4.3.md (parent-owned evidence)

## Verification and acceptance
- Passive version-only metadata change: no meaningful behavioral RED; use structural diff/locked cargo metadata and focused regression checks. Existing behavior already has observed RED/GREEN.
- Exactly three1.4.2->1.4.3 substitutions; dependencies/features/identifier/schema/storage/backup code unchanged.
- cargo metadata --offline --locked --no-deps --format-version1 --manifest-path src-tauri/Cargo.toml reports app1.4.3.
- cargo test --locked --manifest-path src-tauri/Cargo.toml --test catalog_search --test catalog_browse passes; git diff --check.
- Build Windows: npm run tauri -- build --features desktop --bundles nsis.
- Installer expected src-tauri/target/release/bundle/nsis/Repuestos Autos_1.4.3_x64-setup.exe; exact path from actual build log authoritative.
- Pin tested executable source commit, size/SHA256 and NotSigned before publication; never replace accepted bytes with a rebuild.
- No release publication or client delivery until manual installer/update/backup acceptance and final human authority.

## Evidence
- Feature automated checks:474 full default Rust,36 focused search/browse,17 release search (20,000products),three synthetic backup preservation targets passed; query-only snapshots/total_changes unchanged.
- User reports Windows search functioning, not installer/update/restore acceptance.
- Writer mvhbvvv6-6-xsj3 and independent verifier mvhbxbjp-7-3d0i confirm exactly three version substitutions; identifier/dependencies/features unchanged; locked offline metadata reports app1.4.3;36focused tests pass; git diff --check passes. No meaningful RED for metadata-only change. Six unchanged dead-code warnings; no failures.
- Native metadata review review-9440b310109219be approved and exact acknowledgement succeeded (authority burned revision sha256:8718a6edfdc43f26a57b9957856dc05453cb7439801a1918fbe770cc218978f6). Reviewed tree2340139b95421b98064251f45c969d713c0e92f3 matches committed metadata. ASSESS unavailable due undeclared untracked paths; independent verification used conservative fallback.
- R1 build-source commit f2a887150ff0217b068d7160ce4d0d360479c658 pushed to origin/release/v1.4.3; exact remote identity verified. Pin this executable source for Windows build; later tracker-only commits do not change executable source. No tag or GitHub release created; installer gates pending.

- User-provided Windows log reports successful optimized v1.4.3 build in54.25s and one x64 NSIS bundle at C:\Users\LUIS\velay\repuestos-autos-tes2\repuestos_autos\src-tauri\target\release\bundle\nsis\Repuestos Autos_1.4.3_x64-setup.exe. Two nonfatal dead-code warnings: backup_response_diagnostic and reconcile_cleanup_evidence. This is build evidence only, not runtime backup/restore evidence. Subsequent user PowerShell evidence confirms Windows HEAD ef3e535f6202dee9a6bc296eb64348725cb9cd77 and empty git status --short. Installer size4,393,920bytes; SHA256332deb41eabc0aae5ef000ea8295207414786b13dc998247ad8c4d98d6772d7c; Authenticode NotSigned, matching approved profile. Exact source/artifact identity recorded as user-reported evidence; installation/update/restore acceptance remains pending.

- User explicitly answered 'si' to focused confirmation that restoring the1.4.2synthetic backup completed successfully and both products kept correct prices/quantities after closing/reopening. Baseline accepted as human-reported; exact backup location/fixed NTFS not explicitly supplied.1.4.3in-place update and restore in new version remain pending.

- After the1.4.3in-place update/search/data/license/screens/old-backup checklist, user reports 'listo! para quejoya anda ben' (aggregate positive acceptance). Record successful rehearsal as human-reported, but request focused confirmation of1.4.2backup restored in1.4.3with correctdata after restart; fixedNTFSbackup placement remains not explicitly confirmed. Do not infer final publication authorization.

- User answered 'si esta bien pa podemos generar la release' directly to focused1.4.2backup restoration in1.4.3/correctdata after restart and internalNTFS question, following positive aggregate update acceptance. R2 accepted as human-reported synthetic test evidence; no customer installation or exhaustive platform certification. User requests release creation. Exact tested installer bytes still must be received/verified before tag/upload/publication.

## Next step
SCP receipt verified on Linux: /tmp/repuestos-release-v1.4.3-HLcNyNVu/Repuestos Autos_1.4.3_x64-setup.exe is4,393,920bytes with SHA256332deb41eabc0aae5ef000ea8295207414786b13dc998247ad8c4d98d6772d7c, exactly matching Windows-tested bytes. Read-only file identifies NSIS self-extracting archive; launcher architecture is not payload architecture. Remote release branch remains ef3e535; no remotev1.4.3tag observed. User explicitly selected authorize_v143_final_publication: evidence commit/push,newannotatedtagatf2a8871,tagpush and exactunsignedEXEpublicationauthorized. Freshpreflight confirms unchangedhash/source,releasebranchremoteef3e535,noexistinglocal/remotetagorGitHubreleasev1.4.3. Recordacceptancecommit,createimmutabletaganddraft,upload,independentlydownload/hash/cmpthenpublish. Do not rebuild/substitute installer or request customer DB/backup/license files. Then finalize delivery authority and immutable tag at tested build-source f2a887150ff0217b068d7160ce4d0d360479c658; draft/upload/downloadverify before publication.
