# Repair Inventory mockup captures

## Goal

Correct the approved designer's static A/C artifacts in `/tmp` for visual review, without changing production or adopting prototype HTML as source code.

## Scope and constraints

- A: export a faithful 1440×900 DPR-1 PNG, preserving the existing operation-first browse/alert composition.
- C: align the 960×800 prototype to the viewport origin, use 20px compact padding, capture at 960×800 DPR 1, and make alerts reachable by shell scrolling.
- Supply a separate state, scroll, and accessibility annotation board derived from `docs/design/inventory-figma-handoff.md`; no annotation UI inside the app frame.
- Work only on copies under `/tmp/inventory-review/corrected/`; do not edit React/Tauri/repository design references, run untrusted prototype scripts, or commit. `DESIGN.md` is not authority.
- Preserve user-provided ZIPs and original mockups untouched.

## Tasks

- [x] T1 Inspect local capture tooling, prototype inputs, and constraints. Route: inline for environment check, delegated mapping if more than three files needed. Check: identify a runnable local Chromium/Playwright pair and known risks with remote assets.
- [x] T2 Crop A and correct C on temporary copies, then capture exact screenshots. Route: inline one-file mechanical crop for A and one-file targeted CSS correction for C; rendered PNGs are generated outputs. The `/tmp`-only output constraint prevents the repository-relative edit surface required by the generic bounded writer. Check: PNG dimensions, viewport layout, visible alert context, faithful approved facts and shell scroll; disable prototype interaction scripts in the capture.
- [x] T3 Prepare a separate annotation board and verify deliverables. Route: delegated writer/verify as applicable. Check: Sections 5–8 states, focus, scroll ownership, no new UI facts; screenshot/readback and static checks.

## Evidence and progress

- Inputs: `/tmp/inventory1-3.zip`, `/tmp/inventory2-3.zip` (both contain `screen.png`, optional `code.html`, non-authoritative `DESIGN.md`).
- Existing PNGs: A 1440×1024 and C 960×800; C screenshot centered vertically and clips alert rows. Local Chromium is installed in `~/.cache/ms-playwright`; Playwright is available in another local workspace. A 960×800 DPR-1 headless capture context was exercised successfully with Chromium. Prototype interaction scripts will be stripped for rendering; the Tailwind CDN remains an external styling dependency of this disposable reference.
- TDD: not applicable to static reference artifacts; functional verification will be screenshot/readback and file/HTML checks. Runner: local Playwright Chromium (if usable).
- T2 observed: `/tmp/inventory-review/corrected/A/screen.png` is 1440×900 cropped from the original without losing UI. `/tmp/inventory-review/corrected/C/code.html` removes body centering and uses 20px compact padding; the prototype interaction script was removed. Its capture is 960×800 DPR 1 with frame at (0,0), shell scrollHeight 902 > clientHeight 800; `/tmp/inventory-review/corrected/C/scroll-proof.png` shows both alert rows after scrolling 102px. Rendering still uses the original external Tailwind styling CDN and fonts only for the disposable static reference; no local prototype interaction script ran. The bundled FFmpeg could not decode the source PNG; a separate read-only incident diagnosis led to safe in-memory PNG cropping with installed Playwright.
- T3 observed: `/tmp/inventory-review/corrected/ANNOTATIONS.md` covers state, scroll, focus and authority boundaries. Independent read-only verification confirmed PNG dimensions, C layout and 20px padding, scroll proof showing both alerts, and truthful annotation limits. Original prototypes were not executed; the corrected C HTML retains Tailwind CDN/config styling scripts but removes its own interaction script. DPR-1 and 102px shell scroll were observed by the local capture command, not independently rerun. `/tmp/inventory-review/inventory-AC-corrected.zip` contains exactly four artifacts (A/screen.png, C/screen.png, C/scroll-proof.png, ANNOTATIONS.md); Python zipfile testzip passed. `zip` CLI was absent, so Python standard library packed the archive without installation.
- Forecast: completed bounded temporary artifacts; no production diff or PR delivery.
- Commits: explicitly excluded by user authorization for this prototype-only request.

## Next step

Present the corrected reference ZIP for human A/C approval; request B only after approval. No production implementation is authorized.
