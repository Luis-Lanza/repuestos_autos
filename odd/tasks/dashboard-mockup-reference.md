# Preserve approved Dashboard mockup references

## Goal

Generate exact viewport screenshots from the corrected Dashboard A/C HTML prototypes and preserve the approved static references in the repository without adding browser tooling or prototype dependencies to the project.

## Scope

- Use isolated temporary browser tooling outside the repository.
- Capture Frame A at exactly 1440 × 900 with device pixel ratio 1.
- Capture Frame C at exactly 960 × 800 with device pixel ratio 1.
- Verify viewport composition, shell-content scrolling, and absence of visible technical footer prose.
- Preserve final HTML and PNG references under `docs/design/dashboard/` with a reference-only README.
- Exclude designer `DESIGN.md` files and temporary capture tooling.

## Tasks

- [x] Generate and audit exact A/C viewport screenshots.
- [x] Preserve approved A/C static references and verify repository boundaries.

## Evidence

- Corrected sources: `/tmp/dashboard-round1-approval-review/A/` and `/tmp/dashboard-round1-approval-review/C/`.
- Behavioral authority: `docs/design/dashboard-figma-handoff.md`.
- A capture: 1440 × 900, DPR 1, shell-content scroll verified.
- C capture: 960 × 800, DPR 1, shell-content scroll verified.
- Independent Round-1 A/C approval: passed.
- Four preserved artifacts match approved sources byte-for-byte.
- Repository boundary, exact-file-set, and `git diff --check` verification: passed.
- Commit: pending explicit user authorization.
