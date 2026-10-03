# Release v1.2.1

## Objective and decision
Prepare version1.2.1 from integrated masterb21a2ec01d2bf85416410a81853a15b6576fdde3, including the Spanish PDF fixes from mergedPR225. User explicitly selected1.2.1 rather than reuse the prior1.2.0 installer identity.

## Scope and authority
- Branch: chore/release-1.2.1, baseb21a2ec01d2bf85416410a81853a15b6576fdde3.
- Align only app/package versions in src-tauri/Cargo.toml, Cargo.lock and tauri.conf.json. No dependency updates.
- Preserve identifier com.repuestosautos.app, unsigned installer preference, data/schema/backup/license/IPC/logo/PDF behavior.
- User explicitly authorized this preparation's commit and push after independent verification. Scope: three metadata files plus this task document only. No system installs, destructive Git operations, PR/merge/tag/publication without separate explicit authority.
- Preserve the existing modified odd/tasks/spanish-pdf-reports.md bookkeeping and six unrelated untracked task files. Do not include them in a version work-unit commit.
- GitHub matching tags v1.2 returned no matches before preparation. Never move or reuse existing tags.

## Tasks
- [ ] R1 Align and verify1.2.1 metadata. IMPLEMENTED AND INDEPENDENTLY VERIFIED; work-unit closure awaits explicit commit authority. Bounded worker completed exactlythree root version fields1.2.0->1.2.1; parsedfullmetadata comparisonagainstHEAD PASS(allothervalues/dependencies/identifier unchanged), offlinecargometadataPASS1.2.1,gitdiffwhitespacePASS. Parent inspected the three single-linehunks anddiffwhitespacePASS. NativeASSESSunassessable because of untrackedscope,RDDoff; required independent verifier musr1bpo-1g-bec8 PASS: fullparsedHEADcomparison and exactsingle-line version replacements inall3files, unchangeddependencies/workspacepackages/identifier, offlinecargometadatareports1.2.1, gitdiffwhitespacePASS. Parent final spotcheckdiffwhitespacePASS. Tests/build skipped as not applicable to version-only structural validation; no Windowsartifact acceptance inferred. Passive version metadata has no meaningful behavior-test RED; use ordinary structural verification. Work-unit commit and push explicitly authorized; execution pending.
- [ ] R2 Build and identify NEW Windows NSIS1.2.1 installer from final integrated commit, record SHA256 and verify fixed-NTFS backup plus in-place update without uninstalling. Smoke: launch, data, license, Catalog, Sales, Inventory, Reports/PDFs, icons; full multipage/local-time acceptance pending.
- [ ] R3 After successful artifact/update checks and separate human authority, create immutable annotated v1.2.1 tag and GitHub Release with exact verified installer attached. No tag/publication now.

## Evidence and limitations
- Integrated masterb21a2ec contains source candidate770beed: independent436defaultRust/22focusedPDFtests passed before merge. No post-merge suite or Windows build claimed.
- GitHub latest published releasev1.1.1; previous1.2.0 installer was built earlier from04ad7b8 and does not contain the final PDF feature/fixes. Actual distribution details are not independently established; the human version choice is explicit.
- Linux desktop build dependencies unavailable; do not install them. Windows build/update/raster acceptance remains manual.
- Deployment guide recommends signing; human explicitly selected unsigned installers. Preserve that choice and disclose expected Windows warnings.

## Next step
R1metadata checks passed independently; perform explicitly authorized scoped commit/push, then identify the exact Windows build candidate. Do not build/tag/publish an old1.2.0 artifact as1.2.1.
