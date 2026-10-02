# Rename Metrics presentation

## Goal
Rename the visible Dashboard presentation to `Métricas` in the sidebar and screen heading without changing navigation actions, internal screen identifiers, dashboard commands, or domain behavior.

## Tasks
- [x] M1 Map visible Dashboard labels and navigation accessibility coverage.
- [x] M2 Rename visible sidebar and heading labels while preserving screen/action contracts.
- [x] M3 Verify navigation, active state, heading accessibility, and existing dashboard behavior. Evidence: `c633b42 feat(ui): rename dashboard metrics`; 14 focused mounted tests and `git diff --check` passed.

## Acceptance criteria
- The sidebar displays `Métricas`.
- The dashboard screen heading displays `Métricas`.
- Navigation remains keyboard accessible and retains active-state behavior.
- Internal `dashboard` identifiers and backend command contracts remain unchanged.
