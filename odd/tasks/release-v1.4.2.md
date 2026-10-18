# Release v1.4.2 — complete Sales table thumbnails

## Objective
Deliver an unsigned Windows x64 NSIS patch displaying complete Sales table thumbnails like Catalog, without changing persisted data, backup/restore or licensing.

## Authorization and constraints
User explicitly requested merge and release and confirmed v1.4.2 unsigned. Merge/push and release preparation are authorized. Do not publish/tag until the exact versioned installer passes Windows acceptance and final delivery authorization. Preserve com.repuestosautos.app and existing tags. No private-key material, customer data, restore operations or installer execution on customer profiles.

## Tasks
- [x] R1: Fast-forward the verified thumbnail work into master and push; record the existing reviewed work-unit commit and verify remote identity.
- [ ] R2 (in progress): Prepare release/v1.4.2 with synchronized app versions, scoped release evidence and proportionate checks; commit/push build source for Windows.
- [ ] R3: Build the unsigned NSIS installer on Windows, verify a fixed-NTFS backup and in-place 1.4.1 update, check thumbnail parity and data/license/Catalog/Sales/Inventory/Reports, verify backup/restore using synthetic data, then obtain final tag/publication authority.

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
- R2 build-source commit and remote verification: being recorded; no Windows build or installer acceptance claimed.
- R1 complete: master fast-forwarded to bfd53489b40e633ea6437e61a0720bab3f852aa8 and pushed; exact origin/master identity verified. Existing reviewed work-unit commit: 5cb0b1b950cd67e9bb20e239ea051985b766ff32. Created release/v1.4.2 from that integrated head.

## Next step
Complete R2 using a bounded version-preparation writer and applicable verification. Windows build and manual checks remain pending.
