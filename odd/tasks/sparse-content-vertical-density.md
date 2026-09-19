# Sparse Content Vertical Density

## Objective
Keep operational content compact and top-aligned when a desktop screen or dialog has little data, while retaining bounded internal scrolling for dense lists.

## Problem
Windows screenshots show Backup/Restore cards, empty Sales History filters, a one-result Inventory browser, and a one-item checkout cart distributing spare viewport height between related controls. This makes routine states look disconnected and wastes vertical space.

## Decision
Use explicit, screen-owned sparse-density markers and scoped CSS overrides. Sparse states size content tracks automatically and top-align related content. Dense states retain the existing flexible tracks and local scrolling.

## Scope
- Compact Backup/Restore and empty Sales History vertical layout.
- Compact one-result Inventory layout without changing dense result scrolling.
- Compact one-item Sales checkout cart without changing multi-item scrolling.
- Add focused rendered-output and CSS-contract regression coverage.

## Non-goals
- No Tauri, command, domain, persistence, or business-flow changes.
- No global removal of flexible layout rules.
- No changes to dense list/dialog behavior beyond preserving it.

## Acceptance criteria
- Sparse Backup/Restore and empty History keep their primary controls/results directly below their headings.
- A single Inventory result occupies content height rather than stretching its panel.
- A single checkout line keeps cart facts, payment summary, and actions visually contiguous.
- Inventory with multiple results and checkout with multiple lines retain bounded local scrolling.
- Existing semantics, keyboard behavior, feedback, validation, and pending states remain intact.

## TDD and delivery
- TDD mode: not configured; use focused regression checks after each task.
- Delivery strategy: ask-on-risk; forecast under 400 authored changed lines.
- Commit/push: not authorized by the user.

## Tasks
- [x] T1 Add sparse-state markers at Backup, History, Inventory, and checkout screen seams. Route: delegated writer (multi-file UI change).
- [x] T2 Add scoped responsive CSS that compacts sparse states and preserves dense scrolling. Route: delegated writer (multi-file UI change).
- [x] T3 Add and run focused mounted/style regression checks. Route: delegated writer plus independent verification as required.

## Progress
- 2026-09-19: User supplied `/tmp/bug7.png`, `/tmp/bug8.png`, and `/tmp/bug9.png`, then confirmed the same issue in the one-item checkout review dialog. Mapping attributes the defect to flex-growing shell/content tracks and sparse lists inheriting dense viewport rules.
- 2026-09-19: Read-only mapping completed by `gentle-ai-explore`; exact surfaces and density-marker approach established.
- Implemented screen-owned sparse markers for empty History, one-result Inventory, and one-item checkout; Backup uses its existing root selector.
- Added desktop-only scoped compact overrides while leaving dense Inventory and checkout scrolling rules unchanged.

## Verification evidence
- `./node_modules/.bin/tsx --test --import ./test/react-dom.ts src/ui/backup/backup-screen.mounted.test.ts src/ui/sales/history-screen.mounted.test.ts src/ui/inventory/inventory-screen.mounted.test.ts src/ui/sales/sale-screen.mounted.test.ts src/ui/app-shell.mounted.test.ts`: 48 tests passed.
- `npm run typecheck:tests`: passed.
- `npm test`: 214 tests passed.
- Independent verifier: `git diff --check` passed; focused mounted suite passed 42/42; `npm run typecheck:tests` passed. No code-level regressions found. Actual Windows geometry/scrolling remains a manual visual check because JSDOM does not compute layout.
- Native risk assessment was unavailable (empty native output), so the change was independently verified as a high-risk fallback.

## Commit evidence
- `fix(ui): compact sparse desktop layouts` (work-unit commit on `fix/sparse-content-vertical-density`).

## Next step
Run Windows visual validation at 1200×800 and 960×640 with empty History, Backup/Restore, one-result Inventory, and one-item/multi-item checkout. Commit/push only with explicit user authorization.
