# Prepare v1.1.1 Windows installer update

## Objective
Create a patch-release candidate whose application manifests advertise a version newer than the existing 0.1.0 customer installer, so the NSIS installer can update it in place.

## Decisions
- Publish this packaging correction as v1.1.1; existing v1.1.0 tag remains immutable.
- Update the version consistently in Tauri and Cargo manifests.
- Keep `com.repuestosautos.app` unchanged so the installer targets the existing application.
- The customer runs the newer NSIS installer over the existing installation; no uninstall and no license reissue on the same PC.
- Authenticode signing is intentionally out of scope by user decision.

## Tasks
- [x] V1 Align application versions to 1.1.1 without changing product identity. Evidence: `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml` both declare `1.1.1`; identifier remains `com.repuestosautos.app`.
- [x] V2 Verify manifest consistency and prepare the Windows NSIS build command/artifact path. Evidence: independent metadata verification passed; build command is `npm run tauri -- build --features desktop --bundles nsis`, producing the installer under `src-tauri\\target\\release\\bundle\\nsis\\`.
- [x] V3 Build and smoke-test upgrade from 0.1.0 on Windows before customer delivery. Evidence: Windows built `Repuestos Autos_1.1.1_x64-setup.exe`; user installed it in place over 0.1.0 and confirmed the new version works correctly.

## Acceptance criteria
- Tauri and Cargo both declare 1.1.1.
- Product identifier remains unchanged.
- A Windows NSIS installer can be produced with a versioned 1.1.1 filename.
- Upgrade validation preserves application data, license, and device-local Catalog access.
