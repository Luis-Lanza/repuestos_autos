# Catalog Mockup Reference

## Objective
Preserve approved Catalog visual references as static repository assets, captured locally at exact viewport sizes without importing prototype behavior or dependencies.

## Approved sources
- Desktop Table: approved Round-1 source from `/tmp/catalogo1-2.zip`.
- Compact Gallery: approved Round-1 source from `/tmp/catalogo2-2.zip`.
- Category Management: approved visual direction from `/tmp/catalogo6-1.zip`.

## Decisions
- Capture desktop Table at 1440 × 900, compact Gallery at 960 × 800, and Category Management at 1440 × 900; DPR 1 and `fullPage: false`.
- Store PNG references and a README under `docs/design/catalog/`.
- Exclude ZIP files, rejected iterations, all `DESIGN.md` files, and prototype `code.html` files from repository preservation.
- `docs/design/catalog-figma-handoff.md` remains the behavioral and accessibility authority.

## Tasks
- [x] T1 Prepare isolated capture inputs/tooling.
- [x] T2 Capture approved frames at exact viewports.
- [x] T3 Preserve final references and independently verify the artifact set.

## Evidence
- Isolated Playwright 1.55 tooling was installed under `/tmp/catalog-reference-capture.RRz3qS`; Chromium is outside the repository.
- Captured at DPR 1 with `fullPage: false`: Table 1440×900, Gallery 960×800, Category Management 1440×900.
- Every capture matched its exact viewport and had no horizontal document overflow.
- Preserved the three PNGs and added `docs/design/catalog/README.md`; destination dimensions and SHA-256 hashes match the approved captures. The README limits them to static evidence and links the behavioral/accessibility authority.

## Constraints
- No production code or runtime dependency changes.
- Do not commit, push, open a PR, or merge without separate explicit authorization.
