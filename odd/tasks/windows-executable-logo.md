# Windows executable logo

## Objective
Use the supplied original Advance Auto Parts logo as the Windows application executable icon, with only proportional rescaling. Preserve the existing NSIS installer branding.

## Decisions
- The supplied `/tmp/logo-repuestos.jpeg` is the canonical artwork. Do not crop, redraw, simplify, recolor, or otherwise alter its composition.
- Windows icon files may re-encode and proportionally resize that artwork into required bitmap resolutions; that is not a visual redesign.
- The installed application's executable/window icon changes. NSIS setup and uninstall branding must retain their current appearance.
- This work is separate from the pending Inventory Windows smoke task.

## Tasks
- [x] E1 Preserve the canonical original artwork at `src-tauri/icons/advance-auto-original.jpeg` and generate the Windows multi-resolution application icon at `src-tauri/icons/windows-executable/icon.ico` using Tauri CLI 2.11.4 (`node_modules/.bin/tauri icon src-tauri/icons/advance-auto-original.jpeg --output src-tauri/icons/windows-executable`). The existing `icon.ico` is retained byte-for-byte as `src-tauri/icons/icon-installer.ico` for installer branding.
- [x] E2 Configure `bundle.icon` to use the generated executable icon and set both `bundle.windows.nsis.installerIcon` and `uninstallerIcon` explicitly to the retained prior icon.
- [ ] E3 Perform a Windows packaged-app smoke check for executable versus installer branding; required before closing this task.

## Verification evidence
- Independent read-only verification confirmed exact source JPEG and retained installer ICO copies, six 32-bit ICO frames (16, 24, 32, 48, 64, 256 px), and previews retaining the complete artwork.
- Installed CLI 2.11.4 successfully read the configuration; installed schema and Rust config support `uninstallerIcon`. Windows codegen/build select the configured ICO for both the default window icon and executable resource.
- Formal JSON-Schema-engine validation was unavailable; no standalone validator is installed. No Windows build or appearance smoke was run. Linux desktop prerequisites remain unavailable.
- Changes remain uncommitted and unpushed; Windows verification requires transferring these changes first.

## Acceptance criteria
- The application `.exe`, application window, and taskbar use the supplied original logo, proportionally rendered at the available Windows icon sizes.
- No logo artwork is cropped, redrawn, recolored, or simplified.
- The NSIS installer and uninstaller branding retain the existing icon behavior.
- The asset pipeline is reproducible and its output is structurally verified before Windows manual confirmation. Structural checks confirmed the canonical JPEG is an exact byte-for-byte copy of the supplied file and the generated Windows ICO contains multiple icon resolutions. Windows packaged behavior and installer/uninstaller appearance remain unverified until E3.
