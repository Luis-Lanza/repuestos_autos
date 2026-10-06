# Action outcome notifications

## Objective and problem
Make accepted action outcomes visible in a shared top-right notification host independent of screen scrolling. Customers miss the previous screen-local result messages.

## Authorization and scope
User authorized all-screen exploration and implementation on `feat/action-notifications-preview` from `89ad0f9`, not just inventory. User explicitly authorized commits and push ONLY this feature branch for Windows preview. No master edits/merge, version changes, release or destructive operations authorized. Behavior was committed as one coherent cross-screen work unit to avoid inconsistent intermediate app-integration expectations. Six unrelated untracked ODD documents remain untouched.

Preserve Spanish UI conventions. No dependencies, command contracts, Rust or database changes. One narrow presentation classification correction in backup-flow.ts was authorized after a proven regression. Keep field/server validation, loading/empty/read-only states, stale/partial/durability recovery, safety dialogs and receipts inline. No blanket Feedback conversion or message-text classification.

## Behavior and boundaries
One publication per accepted operation completion; duplicate, stale and unmounted completions suppressed. Repeated legitimate identical actions receive separate notices. Provider above activation and shell; fixed top-right host escapes shell clipping. Success/info expire after six seconds with hover/focus pause; warnings/errors require dismissal. Three FIFO cards visible; overflow retained in session memory without eviction, queued expiry starts only on display. Queue is unbounded in memory. Stable live regions/portal move into active modal without stealing focus. Closely batched same-channel live updates may coalesce; actual assistive-technology delivery is unverified. Dismissal never clears fields, receipts, retry identities or recovery requirements.

## Tasks
- [x] N1 (implementation_verified; committed f2e3e42): Shared infrastructure/app host and inventory. Independent 42/42, test types/build/diff PASS.
- [x] N2 (implementation_verified; committed f2e3e42): Sales/onboarding/catalog access/password/recovery/metadata/categories/images/locations. Independent 125/125 plus N1 42/42, test types/build/diff PASS.
- [x] N3 (implementation_verified; committed f2e3e42; baseline type debt disclosed): Backup/restore, history corrections, exports and licensing; integration corrections and full independent verification. Final 473/473 tests, test types/build/diff PASS; zero feature-added production diagnostics.
- [ ] N4 (in_progress; awaiting visual preview): Real desktop/WebView appearance, scroll/modal clipping, contrast, screen-reader behavior and user acceptance; final keep/discard decision pending; feature-only commit/push authorized. No application/database launch yet.

## Acceptance and evidence
Test-first at public React host and mounted action seams. Writer-observed RED then GREEN: N1 host/integration/classification cases; N2 baseline98, initial seven new REDs and two metadata-validation regression REDs, final125; N3 five provider outcome REDs plus history modal-close guard RED, final52. App shell integration adapted only after reproduced RED7/9 then GREEN9/9, preserving safe recovery navigation, disabled restore/zero restore IPC, one activation success owner and durability guidance after dismissal. No failed tests waived.

Independent final review initially found a candidate-caused P1: prepare_restore token_invalid became toast-only, unlike feature-start89ad0f9. Bounded fix uses typed token_invalid->invalid classification; preparation and confirmation regression tests retain inline guidance and gate obsolete candidates. Writer RED17/20 then GREEN20/20; independent20/20. Stale-page export probe passed suppression/pending reset/subsequent export; no deterministic history recovery-gate blocker found. History mutation success precedes detail refresh, with repeat correction gated until persisted detail verified. Activation success belongs solely to App across screen unmount. Partial saves/failed refresh retain corrective guidance.

Four feature-added TS2769 Feedback calls in catalog/history corrected by explicit children props, without API/config widening. Independent exact production `npx tsc --noEmit` comparison against isolated89ad0f9 source snapshot with same dependencies used path/code/full-message multisets: baseline94, current93, retained93, feature-added0, removed1 existing unreachable location_success diagnostic. Production typecheck still FAIL(exit2); existing93 are not waived or fixed here.

Final independent commands:
- Focused backup screen/flow:20/20 PASS.
- `npm test`:473/473 PASS; zero failed/skipped/cancelled.
- `npm run typecheck:tests`:PASS.
- `npm run build`:PASS,76 modules; only authorized ignored dist generated, non-ignored hashes unchanged.
- `git diff --check`:PASS.
- `npx tsc --noEmit`:FAIL93 retained baseline diagnostics, zero added.
- W9 evidence audit intentionally skipped; native/database tests not run because no corresponding changes.
- Real WebView/desktop geometry, contrast and screen-reader checks NOT run. Nonfatal React/localStorage and mixed Tauri import warnings remain.

Native RDD observed globally OFF. ASSESS unassessable due undeclared untracked paths required independent verifier fallback, fulfilled per writer unit and final corrections; no native review transaction started. Feature scope includes 28 tracked changed files (+1290/-124 at final status) plus new host/tests/docs/task document; it is not PR/release approved. Protect future review with coherent slices; do not minify or omit tests to meet advisory400-line units.

## Commit evidence
`f2e3e42`: feat(ui): show shared action outcome notifications across screens; N1/N2/N3 coherent behavior, tests and docs together. 33 files, +1667/-124 including new files. User explicitly authorized commit/feature-branch push. Tracking evidence commit `7cdec44`: docs(odd): record notification preview verification and commit evidence. Authorized feature-only push succeeded; remote refs/heads/feat/action-notifications-preview verified equal to 7cdec445ac366e26f925273e28f622f901cc005b at first transfer. No master merge/version change/release.

## Next step
User evaluates Windows visual preview before keep/discard decision. Branch transferred to origin; no actual Windows runtime observation yet. Prefer separate Windows account for disposable data. APPDATA override is not proven desktop isolation. On existing profile close installed app and create/verify in-app backup first; avoid destructive/business-data mutations. SQLite/license files share app-data directory; separate account does not isolate machine-bound identity. Existing production type debt requires separate disposition. Documentation/manual checklist: `docs/action-notifications.md`.
