# Spanish PDF report readability

## Objective
Readable Spanish tables for all native PDFs: gross-profit operations and the inventory movement ledger (all six kinds). Display dates DD/MM/YYYY and timestamps DD/MM/YYYY HH:mm in the computer's timezone at each instant.

## Scope and protected invariants
- Branch: feat/spanish-pdf-reports, base04ad7b8749d12e51986bea2ddd69e79147c679dd (release branch chore/release-1.2.0).
- Rendering, presentation helpers and tests only; minimal pinned timezone dependencies.
- Preserve UTC/ISO storage, query/request bounds and gross-profit supplied local-date consistency validation, filtering, event order, historical snapshots, signs, integer money/quantity calculations and save/cancel/error handling.
- No changes to schema, migrations, backup/restore, license, IPC registration, release versions, logos or the already-built1.2.0 installer.
- No system installs, destructive Git operations, commit/push/PR/tag/publication without explicit authority. Code task outcomes below are verified. User explicitly authorized commit and push on 2026-10-03; P5 is now in progress. No PR/merge/tag/publication or release inclusion authorized.

## Tasks and routes
- [x] P1: Tested presentation-only local date formatting. Delegated worker (preparation/multi-file trigger), independent verifier. Checked IANA rules; honest errors instead of guessed UTC. Functional PASS; Windows runtime mapping checked separately in P4.
- [x] P2: Readable gross-profit PDF table. Delegated worker (multi-file trigger), independent verifier/correction recheck. Spanish copy, local timestamps,9pt landscape A4 body, word-aware/hard-line wrapping, numeric alignment, repeated headers/page numbers and complete continued rows. Functional PASS; visual review P4 pending.
- [x] P3: Readable Spanish movement ledger PDF. Delegated worker (multi-file trigger), independent verifier/correction recheck. All six kinds, local filter/bound display,9pt body/8pt wrapped headers, reason and note retained, signed numeric alignment and continuation. Functional PASS; visual review P4 pending.
- [ ] P4: Visual/manual PDF review and Windows local-time runtime check. Awaiting manual environment/evidence. Independent complete default Rust suite and focused checks PASSED. Desktop compilation not run on this host (missing Linux GTK/GIO/ATK/GDK libraries). No visual/render tool installed and no sample PDF artifact emitted; tests render in memory, not visual assurance.
- [ ] P5: Commit and push explicitly authorized work units. In progress. User selected single-pr for future review; no PR creation authorized. Plan: shared presentation helpers with tests, gross-profit rendering with tests, ledger rendering with tests, final evidence. Preserve unrelated untracked files. Record identities after observed commits and remote push.

## Acceptance and observed evidence
### P1
Compiling-stub RED: four valid-format tests failed; GREEN6 unit tests. Independent review found discarded IANA detection did not prevent Chrono Local silent UTC fallback and shape-only local assertions. Correction RED: invalid TZ case failed (4passed/1failed), GREEN5 subprocess tests. Final independent PASS:6 unit+5 integration, exact Los Angeles winter/summer, Kolkata, calendar boundaries and non-Z RFC3339 conversion through production adapter; child-only TZ environment, no process-global mutation. Unknown/empty/nonUTF8 override fails honestly; unset uses OS IANA resolver. POSIX TZ strings/custom files unsupported. Date-only calendar values are never timezone-shifted. Pinned chrono0.4.45 and iana-time-zone0.1.65 were already transitive; chrono-tz0.10.4 plus PHF added for historical IANA rules. No unrelated dependency upgrades.

### P2
RED ISO-period presentation then GREEN9 export tests. Independent review found É/Ç underestimated advances, middot/º overmeasure and lost valid hard newlines. Correction observed RED metric/hard-line/cell-bound/continuation tests, then GREEN3 table+11 gross-profit integration. Independent final PASS: Type1 Helvetica WinAnsi advances, unsupported '?'556-unit replacement verified against pinned serializer; independent exact widths,512-É token bounds, hard lines/blank paragraphs and256-chunk continuation order/repeated headers. Renderer permits up to512 wrapped lines/row,1000 pages,16MiB output;10000 events/name512/SKU128/timestamp64 source caps unchanged.

### P3 and final checks
RED missing Spanish title (2/6fail), GREEN7 ledger tests; all six kinds/filter/invalid-time/signed numeric and long content assertions. Independent full-suite run passed432 but found Existencia resultante8pt heading width73.808pt in23mm cell, causing overlap. Correction RED1failed/6passed then GREEN7; every heading now wraps within its cell, preserved8pt font and existing140..153mm header band. Problem heading baselines146.5/142.5mm; body starts132mm. Independent final parsed-PDF cell bounds, repeated headings and at least8mm header/body separation passed. Content/filter/query-bound/ordering and resource limits preserved by inspected diff and regressions.
Final independent commands on corrected candidate:
- cargo test --manifest-path src-tauri/Cargo.toml --test movement_ledger_export:7passed,0failed.
- cargo test --manifest-path src-tauri/Cargo.toml --test gross_profit_operations_export:11passed,0failed.
- cargo test --manifest-path src-tauri/Cargo.toml:432passed,0failed; six existing dead-code warnings.
- Scoped rustfmt --check --edition2021 on correction files:PASS.
- git diff --check:PASS.
Earlier independent shared-helper checks: pdf_table --lib3/3, pdf_format --lib6/6, --test pdf_format5/5. No failed automated check remains. Full desktop build and manual visual/Windows mapping are unobserved, not passed.

## Review/delivery workload
RDD remains off. Native ASSESS repeatedly unassessable because untracked scope requires declaration; returned high-risk fallback was followed through independent verification, without enabling RDD. Initial forecast300–550 lines was exceeded. Latest independent feature source/test/manifest count1889 authored lines (1594added+295deleted), before final header correction, excluding generated lock/task documents; includes formatting churn. Do not minify or remove tests. Delivery strategy:single-pr, explicitly chosen in the oversized menu on2026-10-03; no tracker/chain branches required. Keep coherent helper, gross-profit and ledger work-unit commits in the current branch. User separately authorized commit/push, but not PR/merge/publication. Larger future review couples rollback and increases review load; no repository-policy exception is inferred.

## Manual acceptance checklist (not observed)
- Run the feature candidate on Windows, not the older1.2.0 installer; verify local generated/event timestamps against known movements and Windows timezone.
- Export both PDFs, including a multipage report, long accented names and multiline details.
- Inspect normal zoom/printing for readable headings/body, cell fit, numeric alignment, correct Spanish, repeated headers/footers and uninterrupted row data.
- Provide evidence/screenshots and explicit acceptance before closing P4.

## Next step
Complete P5 authorized commits and push of feat/spanish-pdf-reports to origin. Then transfer that feature candidate to Windows and complete P4 manual verification; record only observed results. Pending1.2.0 release stays separate. All unrelated preexisting untracked task documents remain untouched.
