# Exact SKU search preference

## Objective and authorization
Implement user-authorized exact SKU preference: if the original query exactly matches a SKU eligible for the selected category/activity scope, return only that SKU; otherwise preserve current broad/numeric search. User confirmed that an exact match excluded by stock filters must produce no results, not substitute similar references. All existing numeric fallback behavior remains when no exact SKU exists.

## Safety constraints
Customer database is nearly complete. Change read-only application queries only; no migrations, schema changes, stored-data edits, FTS rebuilds, SKU normalization, backup/restore changes, customer database access or installation. Synthetic test databases only. User subsequently explicitly authorized commit and push of this feature for Windows pull. No version bump, publication, merge or customer installation authorized. Preserve seven unrelated untracked task documents.

## Branch and baseline
- Feature branch: fix/exact-sku-search-preference
- Baseline: 4a9643d (release/v1.4.2); executable release source 6100c9c.
- Work-unit commit: 4769042120b7c85418909e084570121348ab7b95 — fix(search): prefer scoped exact SKU matches. Four source/test paths exactly match approved candidate b0071915904ba100d698d870fab0eecf7c486ce4. User explicitly authorized commit/push for Windows pull.

## Tasks
- [x] S1 (verified; work-unit commit 4769042): Add deterministic public application search/browse regression coverage, observe RED, implement minimal scope-aware exact preference, then observe GREEN and focused existing search regressions.
- [x] S2 (verified against work-unit commit 4769042): Independently verify read-only search, numeric/filter/count/pagination compatibility and synthetic backup preservation; complete native review if consented, inspect final diff and record remaining delivery gates.

## Edit surfaces
Production: src-tauri/src/application/catalog/search.rs and src-tauri/src/application/catalog/mod.rs.
Tests: src-tauri/tests/catalog_search.rs and src-tauri/tests/catalog_browse.rs.
Parent-owned evidence: odd/tasks/exact-sku-search-preference.md.

## Acceptance and verification
- NGK BKR6E-11 hides BKR6EIX-11 when exact eligible SKU exists, across both legacy searches and Catalog/Inventory/Sales browse.
- Without an eligible exact match, existing prefix and numeric search remain unchanged; exact numeric SKU takes precedence on collisions.
- Respect existing category/activity scopes, including archived/all Catalog/Inventory. Stock-filter exclusion of exact must not trigger fallback.
- Exact selection precedes pagination; counts and page results agree, including an empty second page.
- Preserve blank/punctuation/case/whitespace/equality semantics; propagate lookup errors and bind SQL inputs.
- Search fixture snapshots and total_changes() prove no data mutation; no persistence/backup/schema diff.
- RED: cargo test --locked --manifest-path src-tauri/Cargo.toml --test catalog_search --test catalog_browse exact_sku_preference
- GREEN: cargo test --locked --manifest-path src-tauri/Cargo.toml --test catalog_search --test catalog_browse
- Performance: cargo test --locked --release --manifest-path src-tauri/Cargo.toml --test catalog_search
- Synthetic backup tests: backup_restore targets stages_each_supported_schema_without_mutating_the_selected_source; rejects_v24_name_index_without_downgrading_or_mutating_source; rejects_invalid_and_unsupported_candidates_without_source_mutation.
- Run applicable Rust suite and formatting/diff checks; separate environmental failures from regressions.

## Evidence and progress
- Read-only scout mvhakyeh-2-vjf2 confirms exact SKU OR FTS fallback shared by five public projections; no migration/storage change needed.
- RDD global on; functional checks and native review completed below.
- Writer mvhap8li-3-r1wl implemented four authorized files (307 additions, 11 deletions). RED observed NGK result [9701,1] instead of [1]; GREEN 19 browse + 17 search tests passed. Release search 17/17 including 20,000-product checks passed. Query-only fixture snapshots and total_changes unchanged. No customer data touched.
- Formatting follow-up mvhavcmk-4-km58 complete: 36 focused tests passed again; zero introduced formatting differences by baseline comparison; baseline hunks retained unchanged (search 0, mod 30, search tests 38, browse tests 33). Whole-file rustfmt still reports baseline-only differences. git diff --check passed. Final diff 557 additions/11 deletions, largely expanded readable test coverage; same four edit surfaces. S1 outcome verified; later committed as 4769042 after explicit user request.
- ASSESS returned unassessable because untracked declaration is required; conservative independent verifier plan followed, not a risk waiver.
- Independent verifier mvhayopg-5-ix7c: all six Cargo commands passed. Focused 19 browse +17 search; three synthetic source-preservation tests passed; full default Rust suite 474 passed/0 failed (log /tmp/exact-sku-s2-full-rust.log); release repeat17 passed including both20,000-product checks. Query-only execution/snapshots/total_changes and synthetic source bytes unchanged. No blocking behavior/test-adequacy defect; lookup-error test propagates public errors without isolating only existence lookup. Diff remains four authorized paths +557/-11; git diff --check passed before/after. Baseline-only format differences and unused helper warnings remain. Native review and Windows installer/update acceptance pending.

- Native review review-77a6fa513cfd920e approved on the exact four-file candidate b0071915904ba100d698d870fab0eecf7c486ce4; exact acknowledgement succeeded, authority burned at revision sha256:0ce65fdebc6afa03f3bbbc1466a9400c6e957be905f10090e74739af7fcd870f. No source corrections required. Untracked evidence documents excluded from frozen candidate.
- Native preflight managed_assets_outdated resolved via exact offered sync (no files changed). Initial acknowledgement input rejected with no mutation; provider-required lineage-only resubmission closed successfully. No post-burn STATUS.

## Next step
Implementation and automated validation complete. User authorized commit/push for Windows pull; source/tests committed as 4769042. Push this feature branch with evidence, then verify exact remote identity. Before customer delivery, prepare a separate Windows build/update rehearsal with a verified backup in a test profile. No version bump, installer, publication or customer installation performed.
