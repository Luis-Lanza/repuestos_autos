# Release v1.4.3 — exact SKU preference

## Objective
Prepare an unsigned Windows x64 NSIS patch so exact product references do not return similar SKUs, without changing existing customer data or backup behavior.

## Authorization and constraints
User reported search works on Windows and requested release. User selected prepare_143_unsigned_commit_push: explicitly approved v1.4.3 unsigned preparation and versioned source commit/push for Windows build. Final tag/publication remains pending installer acceptance and separate authority. No master merge authorized or needed for this preparation; branch from verified feature HEAD0192c4c. Preserve com.repuestosautos.app, existing tags, seven unrelated untracked docs and customer privacy. No schema/storage/backup/restore/license changes. Do not access customer databases or private keys. Do not install over customer data. No reusing v1.4.2 tag/installer.

## Branch and baseline
- release/v1.4.3 from0192c4cb6f075b718bb9199f30cee3a43582aec7.
- Reviewed behavior commit4769042120b7c85418909e084570121348ab7b95; native review review-77a6fa513cfd920e approved and acknowledged.
- Latest stable release v1.4.2; preflight no remote v1.4.3 branch/tag; manifests currently1.4.2; tracked clean.

## Tasks
- [x] R1 (verified; build-source commit f2a887150ff0217b068d7160ce4d0d360479c658 pushed and exact remote verified): Synchronize app version1.4.3 in Tauri manifest, Cargo manifest and app Cargo.lock entry only; verify metadata/source invariants and focused tests, native review if consented, commit/push Windows build source and verify remote identity.
- [ ] R2 (in progress; awaiting human Windows build/acceptance): Build unsigned NSIS from pinned source, record clean source/EXE size/SHA256/signature; verify a backup on fixed NTFS, update existing1.4.2 test installation without uninstalling, check exact/partial search, data/license/Catalog/Sales/Inventory/Reports and old-backup restore/restart using synthetic data.
- [ ] R3 (pending; separate authorization required): Transfer exact tested installer, verify bytes independently, obtain final annotated tag/push/publication authority; create draft/upload, independently download/hash, publish and read back stable release.

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

## Next step
On Windows fetch/switch/pull release/v1.4.3, verify clean source, build NSIS1.4.3 and report output plus EXE size/SHA256/signature. Do not install on customer PC; manual test-profile update/backup/restore gates follow artifact identification. Final tag/publication authority pending.
