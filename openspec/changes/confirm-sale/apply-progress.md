# Apply Progress: Confirm Sale

## Work Unit 1 — Bootstrap and Searchable Seeded Catalog

- Completed: 1.1–1.6.
- Added a minimal React/TypeScript and Tauri/Rust manifest/configuration scaffold, a disposable SQLite bootstrap, migration, seeded active/inactive catalog, indexed active search, and focused frontend/Rust test entry points. No sale write path was introduced.
- After Rust provisioning, the first Rust run exposed two scaffold defects: Tauri's desktop dependency attempted to compile host GUI dependencies, and the catalog query returned a statement-borrowed temporary. The manifest now keeps the unused desktop dependency behind an opt-in `desktop` feature for this headless catalog test slice, and the query collects into a local result before the statement is dropped.
- Added an executable foreign-key assertion to the disposable catalog test and configured TypeScript's bundler module resolution so the installed React type dependencies resolve.

## TDD Cycle Evidence

| Phase | Evidence | Outcome |
| --- | --- | --- |
| RED | `src-tauri/tests/catalog_search.rs` was written before catalog implementation. The original `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_search` was blocked with `cargo: command not found` (exit 127). | Historical RED/bootstrap evidence retained. |
| GREEN | After provisioning, the same test command first failed to compile because the catalog query returned a temporary borrowing its statement. The minimal local-result correction compiled and passed. | PASS: 3 catalog tests. |
| TRIANGULATE | The disposable-database matrix searches name, SKU, category, and configured searchable field; asserts inactive exclusion, stock, minimum-price centavos, and foreign-key enablement. | PASS: 3 catalog tests. |
| REFACTOR | Bootstrap remains consolidated under `infrastructure/sqlite`; the catalog interface was not widened. The headless test profile keeps Tauri desktop dependencies opt-in until a command/desktop entry point exists. | PASS: `cargo check` without desktop feature. |

## Verification

- `npm test` — PASS: 1 Node frontend-harness test.
- `npx tsc --noEmit` — PASS.
- `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_search` — PASS: 3 passed, 0 failed.
- `cargo check --manifest-path src-tauri/Cargo.toml` — PASS.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check` — PASS after installing the stable `rustfmt` component.
- `cargo check --manifest-path src-tauri/Cargo.toml --features desktop` — UNAVAILABLE on this Linux host: Tauri desktop dependencies require missing GTK/GLib development packages (`gobject-2.0`, `glib-2.0`). Windows packaging was not run and remains unavailable for this Work Unit.

## Files Changed

`.gitignore`, `package.json`, `package-lock.json`, `tsconfig.json`, `vite.config.ts`, `src/main.ts`, `src/catalog.test.js`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `src-tauri/src/lib.rs`, `src-tauri/src/application/**`, `src-tauri/src/infrastructure/**`, `src-tauri/tests/catalog_search.rs`, `openspec/changes/confirm-sale/tasks.md`, and this file.

## Delivery Boundary

Feature-branch chain Work Unit 1 only. The bounded Work Unit source scaffold is 157 lines; the hand-authored delivery-hygiene changes are under 100 lines. Reproducibility lockfiles `package-lock.json` and `src-tauri/Cargo.lock` are retained (4,322 generated lines total); generated `node_modules/` and `src-tauri/target/` are ignored. No commit, push, PR, or Work Unit 2 changes.

## Remaining

Work Unit 1 is complete. Work Units 2–5 remain out of scope for this slice. Desktop GTK validation remains unavailable on this Linux host because required GTK/GLib development packages are missing; Windows packaging was not run. Rust formatting is installed and passes its final check.
