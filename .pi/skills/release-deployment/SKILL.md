---
name: release-deployment
description: "Trigger: release, publish release, deploy update, customer update, NSIS installer. Guide authorized versioning, Windows installer validation, and customer delivery for Repuestos Autos."
license: Apache-2.0
metadata:
  author: gentleman-programming
  version: "1.0"
---

## Activation Contract

Activate for release, tag, deployment, installer, or customer-update work. Do not activate for ordinary development-only changes. Treat this as an operational contract; keep user authorization and data safety ahead of speed.

## Hard Rules

- Before mutating anything, inspect current Git state, previous tags and releases, app versions in `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`, and `docs/DEPLOYMENT_AND_LICENSING.md`.
- Before any Windows installer update, align both app versions to a human-approved, monotonically newer version. Never move or reuse an existing tag. Keep the identifier `com.repuestosautos.app` stable despite Tauri's `.app` warning.
- Do not commit, push, tag, publish, or deliver without explicit user authorization for each external action. Do not assume Authenticode signing; follow the user's explicit signing preference.
- Build the Windows NSIS installer with `npm run tauri -- build --features desktop --bundles nsis`. Expect the artifact under `src-tauri\\target\\release\\bundle\\nsis\\`.
- Before customer delivery, require a verified backup on fixed NTFS and a manual in-place update smoke test over the previous version. Do not uninstall. Verify the app opens, data, license, device-local Catalog access, Sales, Inventory, and Reports.
- A same-PC update does not require a new license. Never include license configuration in SQLite backup expectations.

## Decision Gates

Stop for human approval if the target version, signing preference, backup, smoke results, or any external action is unclear or unauthorized. Do not deliver if the backup or any required smoke check fails. After successful smoke, use an immutable annotated version tag and GitHub Release only when separately authorized; state whether the Windows installer was actually attached.

## Execution Steps

1. Complete the pre-mutation inspection and report relevant state or conflicts.
2. Obtain approval for a newer version, then align both version declarations while preserving the stable identifier.
3. Build the NSIS bundle; identify and verify the produced installer.
4. Confirm fixed-NTFS backup, then manually update the previous installation in place and perform every required smoke check.
5. Request/confirm separate authorization before each commit, tag, push, publication, or delivery action. Never force, move, or reuse a tag.

## Output Contract

Report version, build/artifact location, backup and smoke outcomes, authorization/action status, and whether the installer was attached to the GitHub Release. Distinguish completed actions from planned or unavailable ones; never imply customer delivery without authorization.

## References

- `docs/DEPLOYMENT_AND_LICENSING.md`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`
