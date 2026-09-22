# Implement Dashboard redesign

## Goal

Implement the approved operationally balanced Dashboard using the A/C references while preserving the existing atomic report contract, loading/error/empty behavior, stale-response protection, navigation, and inventory-alert action.

## Constraints

- Frontend presentation only.
- Do not change Rust, SQLite, Tauri IPC, `src/commands/dashboard.ts`, or `dashboard-flow.ts`.
- Preserve Dashboard as the initial route and the existing persistent shell.
- Preserve the 961px breakpoint and 208px/176px sidebar widths.
- Use the shell content as the sole vertical scrolling surface.
- Render only existing report facts and bounded collections.
- Keep `Ver en Inventario` as the sole Dashboard action outside the existing failure retry.
- Preserve loading, atomic failure with `Reintentar`, zero metrics, independent empty collections, stale completions, and unmount guards.
- Keep Catalog, Inventory, Sales, generic panels, and generic aligned-data presentation unchanged.

## Test seams

- Rendered and mounted behavior of `DashboardScreen`.
- Dashboard-scoped CSS contracts in `src/ui/styles.css`.

## Tasks

- [x] Specify and implement the approved Dashboard hierarchy and authoritative content.
- [x] Implement desktop/compact Dashboard-scoped styling and overflow resilience.
- [x] Run full automated verification.
- [ ] Validate desktop and compact behavior in the native Windows application.

## Evidence

- Behavioral authority: `docs/design/dashboard-figma-handoff.md`.
- Approved references: `docs/design/dashboard/desktop-typical.png` and `compact-typical.png`.
- Dashboard hierarchy mounted suite: 3/3 passed.
- Task-1 typecheck, diff check, and independent verification: passed.
- Dashboard styling mounted suite: 4/4 passed.
- Task-2 typecheck, build, diff check, and independent verification: passed.
- Focused Dashboard suites: 8/8 passed.
- Full frontend suite: 246/246 passed.
- `npm run typecheck:tests`, `npm run build`, and `git diff --check`: passed.
- Native Windows desktop/compact validation: pending.
- Implementation commit: `b6f8580` (`feat(dashboard): redesign operational overview`).
