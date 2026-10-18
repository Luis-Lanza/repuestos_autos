# Sales table thumbnail containment

## Objective and problem
Make Sales table thumbnails show complete images like Catalog. Chromium reproduced portrait images expanding the implicit grid row beyond the fixed clipping wrapper (48 x 192 image inside 48 x 48 wrapper).

## Scope and constraints
Only Sales table thumbnail CSS and its existing ProductBrowser regression-test seam. No Catalog/gallery behavior changes, responsive-column redesign, stored-image processing, IPC, Rust, database, backup or restore changes. User explicitly authorized committing and pushing this fix branch for Windows testing. No master merge or release authorized.

## Tasks
- [x] T1: Add a focused regression, observe RED, apply the validated Sales-only minimum-block-size fix, observe GREEN, and verify real-browser geometry and backup/restore surface isolation.

## Acceptance and checks
- Portrait, landscape and square thumbnail image boxes remain inside the Sales wrapper and match Catalog containment at desktop and narrow widths.
- Existing ProductBrowser and Sales mounted tests pass.
- Source diff contains only src/ui/styles.css and src/ui/catalog/product-browser.test.ts.
- Native review runs under the enabled user-owned RDD switch.

## Evidence and progress
- Baseline master a6bb43b; no tracked modifications; seven unrelated untracked task documents preserved.
- Prior Chromium verification independently validated min-block-size: 0 on the Sales table image for five aspect ratios at 1280 and 900px viewport widths, including a JPEG fixture.
- Durable CSS regression uses existing ProductBrowser stylesheet-contract tests; real browser geometry remains the functional evidence rather than JSDOM layout.
- Worker observed RED: 22 passed, 1 intended failure; GREEN: ProductBrowser 23/23 and Sales mounted 41/41.
- Independent verifier repeated ProductBrowser 23/23 and measured 24/24 actual-stylesheet image layouts in Chromium 151 at 1280px and 900px, including portrait JPEG. Sales and Catalog image boxes are 48 x 48; portrait content is fully contained; Catalog unchanged.
- git diff --check passed; tracked diff contains only the two allowed source/test files, 9 insertions and 1 deletion. No backup/restore, Rust, IPC, storage or dependency changes; no private-data operations.
- Native review review-1fb06bf32ebe609e approved and exact acknowledgement completed (authority burned).
- Native ASSESS unavailable due to untracked declaration; conservative independent-verifier requirement already satisfied by the completed verification.
- Native Windows/Tauri check and backup/restore runtime tests not run. Existing narrow-window 40px-column/48px-wrapper mismatch is outside scope and remains unchanged.
- Delivery: user authorized commit and push of this work unit for Windows testing. The work-unit commit contains this document, the scoped stylesheet change and regression test; identify it with git log on fix/sales-table-thumbnail-containment.

## Next step
Commit and push the authorized work unit on fix/sales-table-thumbnail-containment, then validate the corrected view in Windows. Native build/release and master integration are not part of this delivery.
