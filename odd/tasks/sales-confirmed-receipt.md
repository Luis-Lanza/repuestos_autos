# Implement persisted confirmed-sale receipt

## Goal

Replace the current confirmed-sale presentation with the approved centered receipt page while preserving the existing persisted-data contract, confirmation flow, request identity, stale-response guards, and new-sale reset behavior.

## Constraints

- Presentation-only frontend change.
- Do not change Rust, SQLite, Tauri IPC, pricing, stock, payment, idempotency, or `sale-flow.ts`.
- Render only authoritative persisted facts.
- Keep the sidebar visible and preserve the 961px breakpoint with 208px/176px sidebar widths.
- Use the shell content as the single scrolling surface; no nested receipt or item-list vertical scrolling.
- Keep `Nueva venta` as the sole result action.
- On result entry, focus `Venta confirmada`; after `Nueva venta`, focus `Buscar en el catálogo`.
- For an authoritative empty payment collection, render `Sin datos de pago`.

## Test seams

- Public rendered output of `PersistedSaleSummaryView`.
- Mounted interaction behavior of `SaleScreen`.

## Tasks

- [x] Specify and implement authoritative receipt content and semantics.
- [x] Specify and implement result-entry and new-sale focus transitions.
- [x] Implement responsive receipt styling and complete automated verification.
- [x] Validate desktop and compact behavior in the native Windows application.

## Evidence

- Approved handoff: `docs/design/sales-confirmed-figma-handoff.md`.
- Approved static references: `docs/design/sales-confirmed/`.
- Receipt semantics focused tests: 7/7 passed.
- Receipt semantics independent verification: passed.
- Mounted focus suite: 24/24 passed.
- Focus-transition independent verification: passed.
- Focused receipt suites: 32/32 passed.
- Full frontend suite: 245/245 passed.
- `npm run typecheck:tests`: passed.
- `npm run build`: passed with the existing mixed static/dynamic Tauri `core.js` warning.
- `git diff --check`: passed.
- Complete independent implementation verification: passed.
- Native Windows desktop/compact visual and interaction validation: approved by the user.
- Approved issue: `#176`.
- Implementation work-unit commit: `b4a05c0` (`feat(sales): add persisted confirmation receipt`).
