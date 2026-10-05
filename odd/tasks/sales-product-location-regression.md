# Restore Sales Product Location

## Objective and problem
Restore assigned storage location in Sales browse/checkout details. v1.0.0 displayed it; 63f7cb7 removed it when separating Sales from locked Catalog.

## Scope and constraints
- Branch: fix/sales-product-location; base ffa1992.
- Add nullable primary_location_code to existing read-only Sales query/DTO, strict decoder and shared details.
- Preserve Catalog lock, purchase-cost/revision/maintenance exclusions, rows, stock, confirmation payload and assignments.
- No migration, dependency/version change, installation, release, push or PR.
- User explicitly authorized scoped work-unit commit and push to fix/sales-product-location for Windows testing. No PR, merge, release or client installation authorization.
- Preserve six unrelated untracked task documents. No prior content-hash baseline exists: exact paths and no observed mutations verified, not retrospective byte preservation.
- UI follows existing Spanish convention. Delivery strategy ask-on-risk; forecast150–250 authored lines. Actual tracked code/test diff160 additions/23 deletions across11 files.

## Tasks
- [x] T1 Restore location and align regression contracts. Delegated gentle-ai-worker (mandatory multi-file trigger); test-first RED/GREEN. Includes narrow follow-ups for stale exact-key assertions in catalog_search.rs and command_seam.rs. Full default Rust439/439 PASS; no remaining stale runtime assertions found. Commit/push authorized; execution pending.
- [ ] T2 Complete verification/delivery qualification. IN PROGRESS / BLOCKED on unresolved W9 policy failures and pending desktop/Windows checks. Delegated gentle-ai-verify. Independent functional checks passed as recorded below; no all-checks-green or customer-delivery claim. Native risk assessment unassessable because untracked scope; RDDoff, independent verifier executed. Commit/push authorized; execution pending.

## Acceptance and verification evidence
- Assigned/unassigned location visible in Sales table/gallery details and checkout; fallback Sin ubicación asignada. Rows unchanged.
- Strict decoder accepts string/null, rejects malformed values; exact field allowlists retain sensitive exclusions.
- Primary-key LEFT JOIN preserves product cardinality/count/stock; no persistence mutation added.
- Focused frontend87/87 PASS, full frontend415/415 PASS, typecheck:tests PASS, frontend build PASS (independent verifier).
- Writer full default Rust suite439/439 PASS (98lib+341integration); doctests/bootstrap0 tests. Final independent focused Rust36/36 PASS (12command_seam,10catalog_search,9catalog_browse,5product_locations).
- Earlier full Rust attempts failed at stale catalog_search then command_seam assertions and one180s timeout; corrected with observed RED/GREEN. Final full suite writer PASS supersedes functional failures.
- Parent structural spotchecks and repeated git diff --check PASS; exact11 tracked code/test paths +160/-23, seven untracked docs observed, no unexpected mutation during verification.
- W9 last executed24/30, FAIL, not rerun after test-only follow-ups. Five historical candidate/hash failures include pre-existing committed task-byte mismatch. Live protected-path guard rejects product_locations.rs; subsequent historical registration allowance also rejects bounded test-only lib.rs changes. No observed production pricing/auth drift. W9 unchanged; no hashes refreshed/admissions broadened. Separate audit-policy maintenance requires authorization.
- Desktop-gated native IPC tests updated but not executed. Windows installed-app smoke unavailable and pending. No desktop dependencies installed.
- Existing dead-code and experimental localStorage warnings observed.
- RDDoff by user-owned global switch; no native review started.

## Relevant files
src-tauri/src/application/catalog/mod.rs; src-tauri/src/lib.rs; src-tauri/tests/catalog_browse.rs; src-tauri/tests/catalog_search.rs; src-tauri/tests/command_seam.rs; src-tauri/tests/product_locations.rs; src/commands/catalog.ts; src/commands/catalog.test.ts; src/ui/catalog/product-browser.ts; src/ui/catalog/product-browser.test.ts; src/ui/sales/sale-screen.mounted.test.ts.

## Next step
Report local functional fix with qualified verification. User authorized commit and push for local Windows testing; execute scoped delivery and report exact revision. Before customer installation, build/test on Windows; decide separately whether to authorize W9 evidence-policy maintenance. No published installer exists for this correction.
