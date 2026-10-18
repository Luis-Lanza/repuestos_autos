# Release v1.4.2 — complete Sales table thumbnails

## Objective
Deliver an unsigned Windows x64 NSIS patch displaying complete Sales table thumbnails like Catalog, without changing persisted data, backup/restore or licensing.

## Authorization and constraints
User explicitly requested merge and release and confirmed v1.4.2 unsigned. After Windows acceptance and exact transferred-artifact verification, user explicitly authorized evidence commit/push, a new annotated v1.4.2 tag at tested source 6100c9ce6ade3210c53a11d06193cb9a066206e4, tag push and GitHub Release publication with the exact tested unsigned EXE. Preserve com.repuestosautos.app and existing tags. No private-key material, customer data, restore operations or installer execution on customer profiles.

## Tasks
- [x] R1: Fast-forward the verified thumbnail work into master and push; record the existing reviewed work-unit commit and verify remote identity.
- [x] R2: Prepare release/v1.4.2 with synchronized app versions, scoped release evidence and proportionate checks; commit/push build source for Windows.
- [ ] R3 (in progress; waiting for Windows): Build the unsigned NSIS installer on Windows, verify a fixed-NTFS backup and in-place 1.4.1 update, check thumbnail parity and data/license/Catalog/Sales/Inventory/Reports, verify backup/restore using synthetic data, then obtain final tag/publication authority.

## Acceptance and checks
- Source behavior is existing reviewed commit 5cb0b1b950cd67e9bb20e239ea051985b766ff32 plus version metadata only.
- No schema, Rust behavior, IPC, stored-image, backup or restore modifications.
- Version 1.4.2 in all required app manifests/lock entry; identifier unchanged.
- Build command on Windows: npm run tauri -- build --features desktop --bundles nsis.
- Record exact installer source commit, byte size and SHA-256 before publication; never substitute a rebuild for the tested bytes.
- No existing tag moved; GitHub Release publication remains blocked until acceptance.

## Evidence
- Preflight: master/origin/master a6bb43b; fix branch/origin bfd5348; master is an ancestor; no existing v1.4.2 tag or remote release branch observed.
- Thumbnail code checks: 23 ProductBrowser + 41 Sales mounted tests; independent 24 Chromium geometry checks; native review review-1fb06bf32ebe609e approved and acknowledged.
- User reports Windows visual fix works; this is not acceptance of a new 1.4.2 installer or update/restore rehearsal.
- Seven unrelated untracked task documents preserved.
- R2 verification: exactly three app-version substitutions, with dependencies and identifier unchanged; cargo metadata --offline --locked reports 1.4.2; focused ProductBrowser/Sales tests 64/64 pass, independently repeated. git diff --check passed.
- Native metadata review review-1c71033fc7ca19f4 approved and acknowledged. ASSESS unavailable due to untracked declaration; independent verification completed as conservative fallback.
- R2 build-source commit: 6100c9ce6ade3210c53a11d06193cb9a066206e4, pushed to origin/release/v1.4.2 and exact identity verified. Pin this source for Windows build; later tracker-only commits do not change executable source.
- User-provided Windows log reports successful optimized build (1m 39s) and one x64 NSIS bundle: C:\Users\windows-tauri\proyectos\repuestos_autos-search\src-tauri\target\release\bundle\nsis\Repuestos Autos_1.4.2_x64-setup.exe. Two nonfatal dead_code warnings name backup_response_diagnostic and reconcile_cleanup_evidence; they do not establish backup/restore runtime behavior. Subsequent user-provided PowerShell evidence confirms exact Windows HEAD 6100c9ce6ade3210c53a11d06193cb9a066206e4 and empty git status --short. Installer: 4,392,247 bytes; SHA-256 DC9A866C7B773197D58173E3B4D940CBD4B3DFB6A3E343A61D0DC8D7F32F3173; Authenticode NotSigned, matching approved profile. Independent Linux read-only inspection subsequently confirmed /tmp/Repuestos Autos_1.4.2_x64-setup.exe is 4,392,247 bytes and SHA-256 dc9a866c7b773197d58173e3b4d940cbd4b3dfb6a3e343a61d0dc8d7f32f3173, exactly matching the Windows-tested installer. file identifies a Nullsoft Installer self-extracting PE archive (installer launcher architecture is not the payload architecture). Windows update/restore acceptance is recorded below.
- R1 complete: master fast-forwarded to bfd53489b40e633ea6437e61a0720bab3f852aa8 and pushed; exact origin/master identity verified. Existing reviewed work-unit commit: 5cb0b1b950cd67e9bb20e239ea051985b766ff32. Created release/v1.4.2 from that integrated head.

- User confirmed installed 1.4.1 in the synthetic-data test profile with a backup, then reported "funciono 10 de 10" after the update/functional/restore checklist. User then explicitly answered "chi" to the focused question confirming restoration of the 1.4.1 backup and correct data after closing/reopening. The old-backup restore/restart gate is accepted as human-reported evidence.

- Final user authorization: "autoriz", directly answering evidence commit/push, new annotated v1.4.2 tag at 6100c9c, tag push and exact unsigned EXE publication. Fresh preflight confirmed unchanged source artifact hash/size, remote release head b4c0f31 and no existing v1.4.2 tag/release.

## Next step
Commit/push acceptance evidence, create/push the new annotated tag, upload exact tested installer to a draft, independently download/compare it, then publish only after successful verification. No master integration of release metadata or customer installation is implied.
