# Release v1.2.0

## Objective
Prepare and publish the Inventory and Windows-logo feature release from merged master `ff4aebd`, with a Windows NSIS installer and preserved customer data.

## Decisions and authority
- Maintainer approved version `1.2.0` (latest published release is `v1.1.1`). Never reuse or move an existing version tag.
- Maintainer chose no Authenticode signing; Windows security warnings are expected.
- Preserve identifier `com.repuestosautos.app`.
- Release preparation is authorized; obtain explicit commit/push/tag/publication authority before those actions.
- Do not uninstall or reset customer data. Before customer delivery, require verified backup on fixed NTFS and user-observed in-place update smoke over the previous installed version.

## Tasks
- [x] R1 Align application and Cargo versions to 1.2.0 on `chore/release-1.2.0` and verify static metadata consistency.
- [ ] R2 Generate and identify the Windows NSIS installer; verify backup and in-place smoke (launch, data, license, Catalog access, Sales, Inventory, Reports, icons).
- [ ] R3 After explicit delivery authority and successful smoke, create an immutable annotated tag and GitHub Release with the verified Windows installer attached.

## Evidence and limitations
- Existing remote master is `ff4aebd078269ad29fce8118bbc75962732b0e5f`; latest published/tagged release is `v1.1.1`.
- Inventory and logo acceptance were confirmed by the maintainer for development packaging; release-version build and update smoke are separate requirements.
- R1 static metadata verification: Tauri config, root Cargo package, and root Cargo.lock package report `1.2.0`; the application identifier remains `com.repuestosautos.app`. No dependency versions were changed. No Windows build was performed or claimed.
- Agent Linux desktop build prerequisites are unavailable; Windows packaging must be performed on Windows.
- No installer for 1.2.0, release tag, or published release exists yet.
