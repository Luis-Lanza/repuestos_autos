# Product search with repeated numeric attribute values

## Objective and problem
Keep one product search input across Sales, Inventory and Catalog. A query such as `28 28 58` must not reuse one attribute value `28` twice or use a partial SKU match to supply a missing measurement. Current FTS merges fields and does not preserve attribute multiplicity.

## Authorized scope
Branch: `fix/product-search-repeated-values`, baseline `78e2158447d50f19cd07b291f76de09ee6e9e387`.
User authorized implementation and confirmed public search/browse test seams using real in-memory SQLite, including results, totals and pagination. No UI selector, schema migration, IPC change, publication, push or unrelated cleanup. Preserve six pre-existing untracked task documents. User explicitly authorized local commits after verification and native review. No push, PR or merge authorized.

## Agreed search rules
- Classify the original query, not punctuation-split FTS tokens.
- A single complete finite numeric term may match a SKU using existing token-prefix lookup or an attribute whose complete value is numerically equal, not an attribute prefix such as `280` for `28`. Keep partial SKU matches (e.g. query `28`, SKU `39211-28GX0`) isolated from attributes.
- Two or more whitespace-separated complete numeric terms require distinct attribute rows for every occurrence. Order and attribute labels do not matter; SKU/name/category cannot fill missing terms.
- Whole numeric values stored as text or option values also participate. Mixed strings such as `model 28` do not participate.
- Retain exact whole-SKU access as an independent alternative; a full SKU match never fills attribute terms on another product. Preserve ordinary text and separator-bearing SKU search.
- Use existing finite numeric syntax (dot decimal, signed values, exponent), numeric equality, and original text for SKU matching. Do not infer units or comma-decimal conversion.
- Preserve existing activity/category/stock filters, stable order, safe projections, result limits and searchable persisted attributes. Apply attribute matching before count and pagination.

## Tasks
- [x] T1 — Implement and document shared search semantics through vertical RED/GREEN slices at public application search/browse seams. Status: implemented and verified; local work-unit commit authorized and being recorded. Include all projections and legacy search, duplicate numbers, text/option/numeric attributes, distinct-value permutations, numeric prefix rejection, decimal atomicity, SKU compatibility, filter/count/page consistency and realistic numeric-query performance evidence.
- [x] T2 — Independently verify regression, broader Rust checks, scope and native review under the enabled RDD switch. Status: verified; native approved acknowledgement completed. Report all failed/unavailable checks. A native review never substitutes for functional verification.
- [ ] T3 — Close evidence and obtain explicit authorization for a work-unit commit. Status: local commit authorized; evidence closure in progress. No push, PR or merge authorized.

## Allowed writer edit surfaces
- `src-tauri/src/application/catalog/mod.rs`
- `src-tauri/src/application/catalog/search.rs` (optional private module)
- `src-tauri/tests/catalog_search.rs`
- `src-tauri/tests/catalog_browse.rs`
- `docs/PRD.md`
- `src-tauri/Cargo.toml` (only add rusqlite `functions` feature if needed for strict whole-value numeric parsing; no version/dependency changes)

Prefer existing storage; no schema migrations or permissive SQL CAST of arbitrary text. If a scalar function is required for strict text parsing, keep it private, deterministic, safely registered on every search connection and tested through the agreed public seams. Return any required wider surfaces before edits. Roughly 400 changed lines is advisory; do not omit tests or compress code to meet it. Surface honest review workload.

## Checks
- Focused: `cargo test --offline --locked --manifest-path src-tauri/Cargo.toml --test catalog_search --test catalog_browse`
- Adjacent: `cargo test --offline --locked --manifest-path src-tauri/Cargo.toml --test catalog_access --test catalog_maintenance_commands`
- Text performance: `cargo test --offline --locked --manifest-path src-tauri/Cargo.toml --release --test catalog_search searches_twenty_thousand_catalog_products_within_the_release_target -- --exact`
- Relevant full Rust checks selected by independent verifier; keep pre-existing debt distinct from candidate regressions.
- Windows/manual desktop checks are pending unless actually observed. No UI files change; this does not establish Windows customer validation.
- RDD observed globally on; inspect authority after implementation and normalization, then follow provider-owned consent/capture only.

## Progress and verification evidence
- Read-only scout mapped typed attribute storage and all backend search seams. User confirmed numeric text participation and public integration-test seams.
- Branch and preserved unrelated files checked before tracking.
- T1 writer `mv8lkmut-4-lyj6` settled: 26 search/browse and 15 access/maintenance tests passed; four behavior slices reported intended RED exit 101 then GREEN exit 0. One browse fixture setup error was corrected before the intended RED assertion. Numeric 20,000-product debug check reported 230.375868 ms before binding-only cleanup. Six allowed implementation/test/doc files changed; no commit.
- Parent readback found and corrected single-number partial-SKU compatibility. Correction writer `mv8lzn63-5-sk59` settled: query `28` against SKU `39211-28GX0` produced intended RED exit 101 then GREEN exit 0. Final focused suites passed 27 search/browse tests and 15 access/maintenance tests; `git diff --check` passed. The decimal fixture initially used SKU `FLT-001` which legitimately matched `0`; corrected to `ATOMIC`, then full suite passed. No unresolved writer failure is waived.
- Shared strict finite numeric parser and isolated SKU token-prefix parser are private deterministic SQLite functions; only rusqlite's `functions` feature was added. Approximately 445 authored lines including new module, tests and docs form one coherent backend unit (above the advisory 400-line heuristic, no unrelated areas).
- Initial native ASSESS was unassessable until intended untracked declaration and returned the independent-verifier high-risk plan, fulfilled above. INSPECT later selected only new `search.rs`; six unrelated documents and this progress tracker were excluded. Native START created `review-a79bf16c03e3dfa0` for exactly six candidate paths, tier medium, 452 authored lines, one consolidated `review-reliability` lens. Host resolved consent; one-run forecast was relayed. Native capture closed approved, then exact acknowledgement returned `native-approved-acknowledgement-completed`, authority `burned`, target `sha256:0ee50c82dfcbb5b3a51a4cd6a0154e294092d094fe0356f097bbf8f98323afa8`, consumed revision `sha256:66d4e3048cf12e6f197641cef5152f046abcfd871154fa4374ad2118b9212167`. No lifecycle STATUS after that burn. Review never authorizes commit/push.
- Independent verifier `mv8m4wsp-6-m9bg` confirmed focused command PASS: 42 tests (4 access, 11 browse, 11 maintenance, 16 search). First full-suite invocation hit harness timeout at 120 seconds during existing schema-upgrade tests; rerun with 600-second limit completed successfully, total count pending final report. Prior child absence was not checked before the rerun because it was already underway when the parent follow-up arrived; do not claim that check occurred.
- Settled independent verifier confirmed full Rust retry exit 0: 465 passed, zero failed/ignored, zero doctests. Release text search over 20,000 products passed at 21.189361 ms; numeric count + second page + legacy search passed at 83.587679 ms (one-second budget). All candidate/source/lockfile hashes remained unchanged during verification; baseline Cargo.lock SHA-256 `db1bcbf64f36dd3b80916943498dd23519b641d58f15c08ca1a914e2ff4ee97e`.
- New-module rustfmt initially failed exit 1 at lines 32, 50 and 96. After settlement, parent normalized only these three locations; the flat_map normalization also removed redundant closure braces and added a trailing comma, without behavior change. Follow-up verifier `mv8mir0v-7-26t9` settled: rustfmt exit 0, focused/adjacent 42 tests exit 0, diff check exit 0. Other five candidate hashes and Cargo.lock unchanged; new module SHA-256 `cb299d2e724cbbbc713237bf9f812100401507391b7f349baf456b5b407a66e5`. Full suite and release timings retained, not rerun after this formatting-only change.
- User explicitly authorized the local commit after verified implementation; commit identity pending execution. Evidence-only closure may record the resulting work-unit hash without changing reviewed source.

## Next step
Create the authorized local Conventional Commit containing the six reviewed behavior/test/doc files and this feature tracker, then record its identity in a passive evidence-only closure. No push, PR or merge authorized. Windows customer/manual acceptance remains unrun and must be performed before customer delivery. Preserve all six unrelated untracked documents.
