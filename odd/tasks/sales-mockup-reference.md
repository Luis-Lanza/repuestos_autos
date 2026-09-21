# Sales mockup reference

## Objective
Track the approved Sales visual references in the repository so implementation and future design work use a stable, reviewable baseline.

## Approved source set
- Desktop checkout: `/tmp/sales-v11-review/base/screen.png`.
- Compact checkout: `/tmp/compact-corrected11-review/one/screen.png`.
- Compact active draft: `/tmp/compact-corrected11-review/two/screen.png`.
- Compact empty draft: `/tmp/compact-corrected11-review/three/screen.png`.

## Scope
- Add the four approved PNG mockups under `docs/design/sales-pos/`.
- Add a concise index identifying each frame as a visual reference and pointing to the behavioral handoff.
- Preserve the approved mockup HTML sources and one shared `DESIGN.md` under `docs/design/sales-pos/reference-source/` as implementation documentation.

## Non-goals
- Do not version discarded iterations, prototype HTML, ZIP archives, CDN dependencies, or stale DESIGN.md files.
- Do not implement the UI or change behavior.
- No PR or merge is authorized.

## Tasks
- [x] T1 Copy the four approved reference frames and add their index.
- [x] T2 Verify tracked scope, integrity, and index links.
- [x] T3 Copy the approved mockup HTML sources and shared design reference.
- [x] T4 Verify documentation sources and README guidance.

## Progress
- T1: completed by delegated writer; four PNGs and `README.md` added under `docs/design/sales-pos/`.
- T2: independently verified: exactly four PNGs, a valid README handoff link, and no ZIP/prototype artifacts in the reference directory.
- 2026-09-21: User authorized retaining approved mockup HTML and design documentation in the repository; these remain reference-only and never runtime dependencies.
- T3: completed by delegated writer; four HTML visual references and the shared `DESIGN.md` copied byte-for-byte into `reference-source/`.
- T4: independently verified: exactly five source documentation files, README restrictions and authority link are present, and no ZIP artifacts were added.
- 2026-09-21: User authorized commit and push to the feature branch for Windows validation.
- Work-unit commit: `0344b88` (`docs(design): track approved Sales mockups`).

## Acceptance criteria
- Four approved PNGs have clear, stable names under `docs/design/sales-pos/`.
- The index points to `docs/design/sales-pos-figma-handoff.md` for interaction constraints.
- No discarded mockup or prototype artifact is tracked.
