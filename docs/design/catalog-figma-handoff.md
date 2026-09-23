# Catalog — Figma handoff

## Purpose and authority

Design Catalog as a product-first workspace: **Products is the default surface**, with compact Table/Gallery presentation controls and a dedicated category-management subview. This handoff records approved product decisions and repository-backed behavior; it does not authorize implementation or fill gaps with imagined behavior. Figma frames, screenshots, and optional static HTML are visual references only. If a mockup conflicts with this document, this document wins.

> **Implementation boundary:** no production change is authorized by this design-only deliverable. Image storage, backup/restore, IPC, and database behavior require a separately authorized vertical implementation scope. A visual mockup is not a promise that those capabilities exist or are implemented.

Evidence below is a focused inspection of named seams and tests, not a claim that all repository code was inspected. Approved decisions in this handoff take precedence over current UI behavior where they intentionally change it (notably archive confirmation and the new presentations/category subview).

## Quick design path

1. Design **Frame A — desktop Table** and **Frame C — compact Gallery** as the initial review package.
2. Preserve the persistent shell, observable states, and data constraints below; do not fill unknowns with prototype behavior.
3. Submit A and C for review. Create other frames only after approval: desktop Gallery, Categories desktop/compact, image-edit states, and confirmation/state/accessibility boards as needed.
4. Validate the approved frame set against the acceptance checklist before implementation is separately considered.

## 1. Approved product direction

- Products is the default Catalog surface. `Manage categories` opens a dedicated Catalog subview; it is not a category filter or a separate application destination.
- The search row includes two compact, keyboard-accessible `Table` and `Gallery` toggles, with visible selected state. The user's last mode persists across app restarts.
- Switching presentation changes presentation only. Retain the query, submitted filters, pagination, and any already-open editor/detail. It does not create a selected-row/card state. The explicit `Edit` control remains the entry to authoritative product detail/edit; do not imply a new search.
- Table preserves the existing dense browser behavior, augmented by an optional small product-image thumbnail.
- Gallery uses **5 columns at normal desktop width** and **2 columns at compact width**. Each card shows one optional product image or a non-misleading placeholder and has an explicit `Edit` button. Clicking the card itself does not enter editing.
- A product has exactly one optional local image. No product image URL, remote image, or cloud scope is approved. Product imagery is part of backup/restore; its persistence and backup implementation are future engineering work.
- Category Management supports search, existing categories, name, active-product count, state, and `Edit`, `Archive`, and `Reactivate` actions only. Adding category creation is not approved for this redesign and remains in onboarding.
- Archiving a product or category requires confirmation. Reactivation is direct. An active category with active products cannot be archived.

Do not invent category sorting, category-search submission behavior, count refresh timing, image capture/pick/crop behavior, image removal confirmation, dialog copy or dimensions, or persistence details. Where not specified below, mark these in Figma as pending product/engineering clarification rather than prototype an answer.

## 2. Existing contracts and presentation data

### Product browser

The current ProductBrowser is a paged product browser. The inspected Catalog screen initializes its product-browser flow and requests pages of 20. The shared browser supports query, category, stock-state and activity inputs; Catalog currently passes query/category/activity and page size 20. Keep the approved Catalog filter surface as established by the existing design/product direction; do not add sort controls or infer filters not approved for this handoff.

The current browse response contains product ID, category ID, SKU, name, category name, available quantity, catalog/list/minimum prices, revision, categories and page metadata. Existing Catalog rows visibly show name, SKU, category, list price, textual stock status, and an `Editar` action. IDs, revision, minimum price, and the price compatibility alias are not established visible facts for the redesign. Do not add them as visible columns or card content absent separate approval. In particular, images are new approved optional presentation content, not an existing browse payload field.

| Data | Display boundary for this handoff |
| --- | --- |
| Product identity | Keep name and SKU complete and associated. Use category only where it belongs to the existing browser treatment; do not infer new visible facts from opaque IDs. |
| Price | Preserve the existing list-price presentation in `Bs`. Do not substitute minimum sale price, cost, margin, or derived pricing. |
| Stock | Preserve the existing textual stock labels and their meaning: `Sin stock: 0`, `Stock bajo: 1`, or `Disponible: {quantity}`. Status must not rely on color alone. |
| Pagination | Preserve existing result-page behavior and current page when changing presentation. Do not imply a changed page size, sort, or “load more” behavior. |
| Image | Exactly zero or one local product image. Use a clearly neutral placeholder when absent; do not imply a remote fetch, image URL, multiple gallery photos, or an image with misleading product identity. |

The reviewed frontend decoder requires positive safe integer product/category IDs and prices, nonnegative stock/revision values, validates response shape, and requires minimum price not to exceed list price. Those are transport/domain constraints, not a reason to expose IDs, revisions, or minimum price in the design.

### Product edit and domain constraints

The existing product editor presents name, SKU, list price, minimum sale price, and category-defined attributes. The current form validates nonempty name/SKU, positive prices, minimum sale price not above list price, required attributes, and numeric/option attribute types. Preserve these constraints; do not invent free-form attributes or turn required/type-constrained values into optional decorative metadata.

The domain/application/repository paths inspected support normalized uniqueness checks for product SKU/name and category name, typed attribute validation, lifecycle checks, stale revision handling, and catalog refresh/recovery. Product reactivation requires an active category and valid attribute values. Preserve category-defined required/type-constrained attributes and the active-category prerequisite. Keep exact error wording and any unobserved timing unspecified; do not imply inline validation or focus behavior beyond what is evidenced in Section 4.

### Categories subview

The category view must show the approved fields: search, name, active-product count, state, and lifecycle actions. Do not add category description, product previews, attribute-definition summaries, totals beyond each category's active-product count, or sorting controls. The count is active products only, not all products. Existing data seams expose maintenance records and category details, but the approved dedicated view and its search/count presentation are a design addition; loading, paging, sorting, and search commit semantics for that view are not established. Mark those details pending rather than presenting them as existing behavior.

## 3. Existing observable behavior to preserve

| Area | Evidence-backed behavior to preserve or clearly annotate |
| --- | --- |
| Initial Catalog load | The current screen loads category maintenance records and then browses products. It distinguishes loading and unavailable states. Existing copy includes `Cargando registros del catálogo…` and `El catálogo no está disponible. Reintentar catálogo`. Do not present failure as an empty catalog. |
| Browser query and page | Search is submitted through `Buscar`; changing query/filter resets the requested page and invalidates in-flight requests. Catalog browses 20 products per request. Keep the current query, submitted filters, and page when switching Table/Gallery; preserve any already-open editor/detail. Mode switching does not create a selected-row/card state or submit a browse request. The explicit `Edit` control remains the entry to authoritative detail/edit. |
| Browser result states | Preserve initial, loading, results, empty, and error distinctions. The shared flow rejects stale browse responses. Existing Catalog failure copy is `La navegación paginada de productos no está disponible en esta versión.` Do not design a stale response as current results. |
| Product editing | In current Catalog, choosing a product opens its named edit dialog after loading authoritative detail; the dialog focuses the name input when detail becomes ready. Form validation focuses the first invalid field. Escape closes the dialog and returns focus to its opener. Do not imply that clicking a Gallery card edits; only its explicit `Edit` button does. |
| Save/lifecycle pending | Existing edits and lifecycle mutations lock incompatible controls while pending, avoid duplicate mutation, and show feedback. Preserve pending, validation, stale-record, unavailable, and recovery states. |
| Stale/recovery | Stale detail or mutation results may require explicit reload; recovery reloads list/browser and authoritative detail where needed. Preserve recovery lock and feedback rather than showing outdated data as authoritative. Older async results must not replace a newer request or update after unmount. |
| Lifecycle change | Current product/category maintenance action is immediate, with no confirmation dialog; success refreshes catalog records and browser. The redesign intentionally changes archive to require confirmation while keeping reactivation direct. Preserve the existing category-archive block when active products remain. |
| Category maintenance list | Existing Catalog currently has a combined category/product maintenance list, not the approved dedicated Categories subview. Do not treat its current list layout as authority for the new subview. |

### Confirmation behavior not yet implemented

Archive confirmation is an approved behavior change, but exact dialog copy, whether the affected item/count appears in the confirmation, initial focus, focus trap/return mechanics, cancel key behavior, and error/retry presentation have not been established for this proposed confirmation. Annotate them as **pending product/engineering specification**. Do not borrow undocumented behavior from another dialog as if it were already the archive contract. Reactivation remains direct, with pending/success/error/recovery feedback consistent with existing lifecycle behavior.

## 4. Focus, dialog, and accessibility contract

- Table/Gallery controls are compact, keyboard accessible, have clear accessible names, and expose a visible selected state. Use a semantic button/toggle pattern; exact ARIA state convention may follow the implementation's accessible pattern, but do not make the mode discoverable by color alone.
- Each Gallery `Edit` button is a distinct, keyboard-operable control with a product-specific accessible name. The card container itself is not an edit control and must not be clickable as a whole.
- Use one page heading, named main regions, a semantic results list/table appropriate to the presentation, and explicit labels for search and filters. Do not add tab stops to static text merely for styling.
- Keep reading and tab order aligned with the visible hierarchy: persistent sidebar, Catalog heading/toolbar, browser controls, results, then pagination; in Categories, heading/search/list and each record's actions. Presentation switching must not silently change query, submitted filters, or pagination, and must preserve any already-open editor/detail without creating a selected-row/card state.
- Existing product-edit behavior focuses the name input after authoritative detail loads, focuses the first validation error, and returns focus to the opener on Escape close. Preserve these observed details where the existing edit dialog remains in use.
- New archive confirmation focus behavior is **pending**. Do not claim focus placement, focus trap, Escape handling, or return destination until specified. No programmatic focus behavior for mode switching or category search is evidenced. Category creation is not approved in this redesign and remains in onboarding.
- Give every image/placeholder an appropriate accessible treatment: meaningful product image alternative text derived from product identity where appropriate; decorative placeholder treatment must not pretend to be an actual photograph or convey unsupported product facts. Exact image alt strategy is pending implementation/design review.
- Visible keyboard focus must remain discernible. Follow existing app visual-system focus treatment and supported contrast behavior; do not invent a new global focus style in this handoff. Meet at least 44px control targets where practical, especially for compact actions, and annotate any dense control exception for approval.
- Do not use color alone for selected mode, activity state, stock status, validation, pending, or error. Respect reduced motion and forced-colors settings. Keep all authoritative text readable without overlap or silent truncation.
- Announce meaningful loading, validation, success, and failure feedback without redundant announcements from every row or placeholder. Exact live-region behavior for the new category search and image controls is pending.

## 5. Shell, viewport, and scroll ownership

The existing Dashboard and Inventory handoffs establish these application-shell constraints, which apply to Catalog design:

| Constraint | Requirement |
| --- | --- |
| Persistent sidebar | Keep the sidebar visible at every supported width: **208px at min-width 961px**, **176px at max-width 960px**. No overlay, drawer, collapsed rail, hamburger, or replacement navigation. |
| Content padding | **24px desktop**, **20px compact**. |
| Breakpoint | Use only the existing transition between 961px and 960px. Do not add an intermediate or extra shell breakpoint. |
| Shell chrome | Do not add a topbar or new shell chrome. |
| Scrolling | Shell content remains the primary page-level vertical scroll owner; keep scrollbars visible. Catalog preserves the existing shared ProductBrowser dense-results nested vertical scroll (`overflow-y: auto`). Do not imply its removal or add other nested vertical scrollers for gallery cards or category results. If a horizontal table overflow is unavoidable, keep it discoverable and annotate for review rather than hiding content/scrollbars. |

The shared ProductBrowser dense-results nested vertical scroll (`overflow-y: auto`) is an established behavior to preserve within the shell; shell content remains the primary page scroll owner.

## 6. Required Figma frames

All sample values below are illustrative only. They must not be mistaken for current stored data or additional product decisions.

### Frame A — Desktop Table (initial review package)

**Viewport/shell:** 1440 × 900; persistent 208px sidebar; 24px page padding; no new chrome; Catalog active. Show Products as the default surface.

**Composition:** page heading, search/filter row, compact Table/Gallery toggles with Table visibly selected, dense product table, optional small thumbnail column, and existing pagination when the illustrative result set spans pages. Preserve complete names, SKUs, category, list price and textual stock treatment supported by the existing browser. Use the existing `Edit` action per product; do not make rows or images click-to-edit. The optional image may be absent and represented by a neutral placeholder.

**State annotation:** show the current query and submitted filter context, current page, and any already-open editor/detail as retained state, not as an effect of changing mode. Mode switching creates no selected-row/card state; the explicit `Edit` control enters authoritative detail/edit. Annotate loading, empty, error, stale-response rejection, edit detail loading, validation, pending save, stale recovery, and lifecycle feedback in a sideboard rather than implying the example screenshot covers every state.

**Proves:** default Products surface, dense Table behavior, unobtrusive optional image, accessible presentation switch, explicit edit action, existing data hierarchy, and desktop shell.

### Frame C — Compact Gallery (initial review package)

**Viewport/shell:** 960 × 800; persistent 176px sidebar; 20px page padding; no new chrome; Catalog active.

**Composition:** same Products-first hierarchy and search-row controls; Gallery visibly selected; two card columns. Each card has exactly one optional image or clearly neutral placeholder and an explicit `Edit` button. The card surface itself is not an edit action. Show enough cards to demonstrate wrapping without clipping names/SKUs; preserve the shared ProductBrowser dense-results nested vertical scroll (`overflow-y: auto`) while shell content remains the primary page scroll owner. Keep controls reachable in the semantic order.

**State annotation:** indicate that Table/Gallery switching preserves query, submitted filters, pagination, and any already-open editor/detail; it does not create a selected-row/card state. The explicit `Edit` control enters authoritative detail/edit. Show image-free placeholder behavior without suggesting all products have photos. Annotate that the selected mode persists across application restarts; no persistence UI, sync indicator, or settings screen is implied.

**Proves:** compact Gallery density, persistent shell, safe card interaction, optional image representation, and visible selected mode.

### Additional frames — only after A/C approval

Produce only the additional material approved after initial review:

- **Desktop Gallery:** five columns at normal desktop width, with the same card content/action contract as C.
- **Categories desktop and/or compact:** dedicated Catalog subview supporting search, existing categories, name, active-product count, state, and Edit/Archive/Reactivate actions only. Category creation is not approved for this redesign and remains in onboarding. Search behavior and result loading/error states remain annotated pending where not yet specified.
- **Image edit states:** only if needed to communicate the single optional local image and neutral placeholder. Exact add/replace/remove, picker, crop, preview, errors, and focus behavior are pending; do not show a local/remote path field, URL input, multi-image carousel, or cloud control.
- **Confirmation/state/accessibility board:** as needed for archive confirmation, pending/failure/stale/recovery, keyboard focus, forced colors, wrapping and responsive behavior. Unknown confirmation mechanics must be explicitly marked pending.

Do not generate additional frames before approval merely to visualize every state. An annotation board can accompany the first two frames without becoming another product concept.

## 7. Lifecycle, image, and state annotations

### Archive and reactivation

- Archiving either a product or category presents a confirmation step before mutation. Exact copy and mechanics are pending as described above.
- A category with active products cannot be archived. Make the blocked result understandable without implying that archiving the category also archives its products.
- Reactivation is direct (no confirmation). A product can reactivate only when its category is active and its required/type-constrained attribute values remain valid.
- Preserve existing mutation-pending lock, stale-record response, validation/failure feedback, and recovery/reload behavior. Do not allow repeated activation of a pending action to suggest duplicate writes.

### Product images

- Exactly one optional local image per product, incorporated into product backup/restore. Use a non-misleading absent-image placeholder.
- No image URL, cloud image, externally hosted media, multiple photos, or remote fallback.
- Figma may illustrate an image slot and a neutral missing-image state only. File selection, image formats/size limits, storage location, crop/resize, replacement/removal semantics, failed backup/restore behavior, and image-related dialog focus/error states are **pending implementation/product specification**.
- Do not imply the current browse/detail/command contract already returns or saves an image.

### State coverage board

Annotate only states supported by existing seams or explicitly approved change:

| Surface | Required states |
| --- | --- |
| Product browser | Initial/loading, successful results, successful empty, unavailable/error, and stale request ignored. Retain current submitted query/filter/page behavior. Do not equate empty with unavailable. |
| Product edit | Detail loading, detail unavailable/retry, ready form, field validation/focus, pending save, success, stale record/recovery, and failed refresh recovery. Keep error/failure wording unspecified unless existing copy is shown from evidence. |
| Product/category lifecycle | Pending, success, lifecycle blocked, stale conflict and recovery. Add confirmation before archive; leave its unapproved focus/copy/mechanics pending. Reactivation stays direct. |
| Categories subview | The approved list/search/count/state/actions are required, but initial/loading/empty/error/pagination and search timing are not evidenced for the new view. Mark the states and interaction mechanics pending rather than inventing them. |
| Image slot | Present/absent visual only. All upload, replace, remove, processing, storage, and restore failure behavior remains pending implementation scope. |

## 8. Non-goals and prototype boundary

Do not show, imply, or reserve controls for:

- product image URLs, cloud storage/sync, remote fallback, multiple product images, image galleries, or externally hosted imagery;
- extra product attributes, unbounded custom metadata, unsupported price fields, cost, margin, or computed product metrics;
- card-wide click-to-edit, row-wide editing, inline edit behavior, or a selected-row/card state created by changing Table/Gallery mode;
- new filters, sorting, search-as-you-type/submission changes, page size changes, or different query/page semantics;
- category bulk operations, category-to-product cascading archive, or archive when active products remain;
- category sorting, pagination, count semantics other than active products, or unapproved actions;
- undocumented focus movement, confirmation behavior, dialog mechanics, image selection/storage behavior, or background sync;
- new topbar, drawer, sidebar mode, breakpoint, additional nested scroll surface beyond the established shared ProductBrowser dense-results vertical scroll, or hidden scrollbar.

Figma and optional HTML are disposable visual references. No CDN fonts/icons, runtime prototype scripts, inline global handlers, hidden scrollbars, or prototype-only dependencies may enter production. Do not embed mockup assets in the app as a substitute for separately scoped image support.

## 9. Acceptance checklist

### Direction and behavior

- [ ] Products is the default Catalog surface; `Manage categories` opens the dedicated Catalog subview.
- [ ] Table and Gallery are compact, keyboard-accessible, visibly selected controls in the search row.
- [ ] The last mode persists across restarts, and switching mode changes presentation only while retaining query, submitted filters, pagination, and any already-open editor/detail; it creates no selected-row/card state. The explicit `Edit` control enters authoritative detail/edit.
- [ ] Table retains dense browser behavior and adds only an optional small thumbnail.
- [ ] Gallery uses five columns at normal desktop and two at compact width; cards show one optional image/neutral placeholder and explicit `Edit`; cards themselves are not edit controls.
- [ ] Product image scope is exactly one optional local image; no URL/cloud/multiple-photo behavior is shown. Backup/restore inclusion is documented as future implementation scope, not as an existing UI capability.
- [ ] Category Management supports search, existing categories, name, active-product count, state, and Edit/Archive/Reactivate only. Adding category creation is not approved for this redesign and remains in onboarding.
- [ ] Archive confirmation is represented for both product and category; reactivation is direct. Active category with active products remains blocked.

### Data and domain integrity

- [ ] Existing SKU/name uniqueness, positive prices, minimum price ≤ list price, and category-defined required/type-constrained attribute rules are not contradicted.
- [ ] Product reactivation requires an active category and valid attributes.
- [ ] Existing browser fields and textual stock meanings are preserved; IDs/revision/minimum price are not surfaced as new facts.
- [ ] Empty, loading, unavailable/error, pending, stale, and recovery states are distinct where evidenced; unknown new-view/image/confirmation behavior is explicitly pending.
- [ ] No image behavior absent from current payload/commands is presented as implemented.

### Shell, accessibility, and prototype boundary

- [ ] Sidebar is 208px at ≥961px and 176px at ≤960px; page padding is 24px/20px respectively.
- [ ] No new shell chrome or breakpoint; shell content remains the primary page scroll owner and Catalog preserves the shared ProductBrowser dense-results nested vertical scroll (`overflow-y: auto`); scrollbars remain visible.
- [ ] Toggle controls, search, actions and result reading order are keyboard accessible with visible focus and selected state.
- [ ] Product cards are not click-to-edit; every card Edit action has an accessible name tied to its product.
- [ ] Names/SKUs/status/price wrap without overlap or silent loss; state is not conveyed by color alone.
- [ ] Reduced motion and forced-colors treatment are considered; live feedback is restrained.
- [ ] Unknown focus/dialog mechanics are marked pending, not asserted as existing behavior.
- [ ] Mockups do not introduce unsupported capability or production-only dependency.

## 10. Evidence reviewed and future implementation boundary

### Focused repository evidence

The following specific seams and tests were inspected to ground this handoff:

- `src/ui/catalog/catalog-maintenance-screen.ts` — current combined category/product maintenance shell, category list and ProductBrowser wiring; browse request size 20; detail/edit/lifecycle orchestration, mutation lock, stale request guard and refresh/recovery paths.
- `src/ui/catalog/catalog-maintenance-flow.ts` — loading/ready/pending/unavailable state transitions, validation, stale-record recovery, lifecycle feedback and success state.
- `src/ui/catalog/product-browser.ts` — current shared browser query/category/activity/stock controls, result rendering, stock labels, page navigation and product-specific selection naming support. It has no image or Table/Gallery presentation behavior.
- `src/ui/visual-system/catalog-edit-dialog.ts` — current routine edit dialog, authoritative detail loading treatment, name-input focus, pending controls, lifecycle action, inline feedback and recovery retry. Current archive action is immediate; no archive confirmation exists here.
- `src/commands/catalog.ts` — runtime decoding and frontend contracts for browse, metadata detail, edit and lifecycle commands; no image field or image command is present in the inspected surface.
- `src/ui/catalog/catalog-maintenance-flow.test.ts`, `src/ui/catalog/catalog-maintenance-screen.mounted.test.ts`, `src/ui/catalog/product-browser.test.ts`, and `src/ui/visual-system/catalog-edit-dialog.mounted.test.ts` — focused coverage for validation, async ordering/stale browse, editing, lifecycle, recovery, keyboard Escape, validation focus, and direct reactivation.
- `src-tauri/src/commands/catalog.rs`, `src-tauri/src/application/catalog/`, `src-tauri/src/domain/catalog.rs`, and `src-tauri/src/infrastructure/sqlite/catalog_repository.rs` — command/application/domain/repository seams for browse, maintenance, uniqueness/typed validation and lifecycle persistence. The domain blocks category archival with active products and product reactivation when category or values are invalid; repositories check normalized identities and current revision.

This is a focused evidence list only, not an exhaustive repository audit. The approved Gallery, Categories subview, mode persistence, and local image capability are design decisions, not observed current features.

### Future implementation candidate areas — not authorized now

Any later production work requires separate authorization and a scoped vertical plan. In particular:

- Presentation work may involve `src/ui/catalog/catalog-maintenance-screen.ts`, `src/ui/catalog/catalog-maintenance-flow.ts`, `src/ui/catalog/product-browser.ts`, their focused tests, and Catalog-scoped styling. Preserve shared ProductBrowser behavior for its other consumers.
- Categories search/count/subview and archive confirmation require explicit behavior, focus, dialog, and error/recovery contracts before implementation surfaces are expanded.
- Persisting the Table/Gallery preference across restarts requires an explicitly chosen storage boundary and migration/compatibility plan if persistence affects stored application data.
- Local image storage and inclusion in backup/restore cross UI, command/IPC, Rust application/repository, database/schema and backup/restore boundaries. **These are implementation tasks requiring separately authorized vertical scope, not a visual mockup promise.** Do not implement only a UI affordance or silently widen this visual handoff into backend work.
- No source, test, migration, command, database, backup, or runtime file change is authorized by this documentation task.
