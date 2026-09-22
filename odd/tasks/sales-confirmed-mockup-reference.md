# Preserve confirmed-sale mockup references

## Goal

Preserve the approved confirmed-sale visual and HTML references in the repository without importing prototype dependencies or excluded designer documentation into production guidance.

## Scope

- Add approved Frames A–E under `docs/design/sales-confirmed/`.
- Preserve each approved `screen.png` and `code.html` with descriptive repository filenames.
- Add a README that identifies each frame and establishes the static-reference boundary.
- Exclude every designer-provided `DESIGN.md`.

## Tasks

- [x] Preserve approved screenshots and HTML sources.
- [x] Verify file mapping, documentation, and repository diff.

## Evidence

- Source review paths:
  - Frame A: `/tmp/confirm-sales-targeted-review/A/`
  - Frame B: `/tmp/confirm-sales-targeted-review/B/`
  - Frame C: `/tmp/confirm-sales-final-review/C/`
  - Frame D: `/tmp/confirm-sales-targeted-review/D/`
  - Frame E: `/tmp/confirm-sales-frameE-final/`
- Exact-byte comparison: passed for all five PNGs and five HTML files.
- Artifact inventory and `DESIGN.md` exclusion: passed.
- README authority and production-boundary review: passed.
- `git diff --check`: passed.
- Independent verification: passed.
- Approved issue: `#177`.
- Work-unit commit: `d4ec398` (`docs(sales): preserve confirmed-sale design references`).
