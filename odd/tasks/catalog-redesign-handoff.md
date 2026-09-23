# Catalog Redesign Figma Handoff

## Objective
Create an evidence-based, designer-ready handoff for redesigning Catalog product browsing, category management, and optional local product imagery without changing existing domain constraints.

## Decisions
- Products are the default Catalog surface.
- Product presentation is user-configurable and persisted across restarts: Table or Gallery.
- The compact toolbar contains accessible Table and Gallery toggle buttons in the search-bar row.
- Gallery uses five columns on normal desktop and two columns on compact layouts.
- Each product has at most one optional local image; images are included in backup and restore.
- Gallery cards expose an explicit Edit button.
- Table retains its current dense operational structure and adds an image thumbnail.
- Category Management is a dedicated Catalog subview with search, name, active-product count, state, and lifecycle actions.
- Archive requires confirmation; reactivation remains direct.

## Tasks
- [x] T1 Map source evidence and existing handoff conventions.
- [x] T2 Draft the Catalog Figma handoff.
- [x] T3 Independently verify the handoff against approved decisions and current contracts.

## Evidence
- Completed T2 in `docs/design/catalog-figma-handoff.md`, following Dashboard/Inventory authority, frame, state, accessibility, non-goal, acceptance, and future-boundary conventions.
- Drafting evidence was checked against focused Catalog screen/flow/browser, edit-dialog, command-adapter, UI-test, and Rust command/application/domain/repository seams; this was not a full repository audit.
- Independent read-only verification approved the handoff after corrections: it preserves the established ProductBrowser dense-results nested scroll, retains only query/submitted filters/page/open editor-detail across view changes, and excludes category creation from this redesign.

## Constraints
- Preserve SKU/name uniqueness, price and attribute validation, category archive blocking while active products remain, and stale async recovery.
- Do not treat this documentation task as implementation authorization.
