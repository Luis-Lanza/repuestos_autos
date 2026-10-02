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
- [x] E3 Record user-observed Windows smoke: the original logo appears in the application window, desktop shortcut, and taskbar; the user confirmed the final appearance and requested documentation closure after being asked about taskbar and installer branding.

## Verification evidence
- Independent read-only verification confirmed exact source JPEG and retained installer ICO copies, six 32-bit ICO frames (16, 24, 32, 48, 64, 256 px), and previews retaining the complete artwork.
- Installed CLI 2.11.4 successfully read the configuration; installed schema and Rust config support `uninstallerIcon`. Windows codegen/build select the configured ICO for both the default window icon and executable resource.
- Formal JSON-Schema-engine validation was unavailable; no standalone validator is installed. No Windows build was run by the agents. Linux desktop prerequisites remain unavailable.
- E1/E2 are committed in `eb79641` (`feat(windows): use original logo for application icon`); prior structural evidence was pushed in `266e51b`. Native RDD is off (global setting); no native review ran.
- User-observed Windows evidence: the open application window showed the logo; the desktop shortcut initially showed a dark square, then the user confirmed it displayed correctly. The cause of the initial shortcut appearance was not established.
- The user subsequently confirmed the final taskbar/installer check and requested closure. No separate uninstaller appearance observation or Windows build log was supplied; uninstaller preservation is supported by the explicit configuration and byte-identical retained icon, not a claimed manual uninstall test.

## Acceptance criteria
- The application `.exe`, application window, and taskbar use the supplied original logo, proportionally rendered at the available Windows icon sizes.
- No logo artwork is cropped, redrawn, recolored, or simplified.
- The NSIS installer and uninstaller branding retain the existing icon behavior.
- The asset pipeline is reproducible and its output is structurally verified before Windows manual confirmation. Structural checks confirmed the canonical JPEG is an exact byte-for-byte copy of the supplied file and the generated Windows ICO contains multiple icon resolutions. E3 is closed on the user's Windows visual confirmation. Uninstaller appearance was not separately exercised; no uninstall was required or claimed.
