# Inventory — Figma handoff

## Purpose and authority

Design Inventory around **the next stock operation**: find a product, select it, then enter or count stock. Search, selection, and the operation dominate; keep the secondary, read-only stock-alert panel present throughout the flow. Its rows can misleadingly disappear after `Nueva operación` in the current implementation (Section 5); do not represent that empty state as verified stock status. Keep the existing combined browser → selected product → operation flow and minimal inline confirmation feedback.

This document is authoritative for behavior, data meaning, accessibility, and responsive constraints. Figma frames, screenshots, and optional HTML are **static visual references**, not permission to add functionality. If a mockup disagrees with this handoff, this document wins. No production change is authorized by this design-only deliverable.

## Quick design path

1. Draw Frame A (desktop browse) and Frame C (compact physical count) using the existing shell and visual system. Submit **A/C first** for shared-system approval.
2. After that approval, draw Frame B (desktop stock entry). Embed the state/edge annotations below in the handoff rather than drawing extra frames.
3. Check all three against the acceptance checklist; deliver ZIPs with `screen.png` and optional `code.html`.

## 1. Operation-first composition

Use one `Inventario` heading and the existing description `Operaciones de entrada de stock y conteo físico.` The primary region is `Operación de inventario`: while browsing it contains `Buscar producto`, `Categoría`, `Estado del stock`, `Buscar`, results and pagination; after selection it contains the selected identity, `Operación`, its fields, `Saldo proyectado`, and actions. Do not turn selection into a separate detail page or introduce an intermediate step. Keep the `Alertas de stock` panel visible as a secondary rail beside this region on desktop and as the subsequent section on compact screens. A loading or unavailable alert region remains present, not replaced by a new route.

| Step | Visible Spanish labels and treatment |
| --- | --- |
| Browse | `Buscar producto`, `Categoría` (`Todas las categorías`), `Estado del stock` (`Todo el stock`, `Disponible`, `Stock bajo`, `Sin stock`, `Alertas`), `Buscar`; result action `Seleccionar`; when applicable, `Página {page} de {total_pages}`, `Anterior`, `Siguiente`. Search and selection should be visually stronger than alert content. |
| Selected product | Product name, `SKU: {sku}`, `Stock actual: {available_quantity}`; `Operación` with `Entrada de stock` and `Conteo físico`. The browser gives way to the operation in the same primary panel. |
| Stock entry | `Cantidad (unidades enteras)`, `Nota (opcional)`, `Saldo proyectado: {value}`, `Confirmar operación`, `Nueva operación`. |
| Physical count | `Conteo físico (unidades enteras)`, `Motivo`, `Saldo proyectado: {value}`, the same two actions. |
| Alerts | `Alertas de stock`; status `Sin stock: 0` or `Stock bajo: 1` beside product name. No alert action. |

The existing selection button's **visible** copy can stay `Seleccionar`; each repeated button must gain a product-specific **accessible** name in a later approved presentation/accessibility implementation. Do not add a visible product action, column, or fact for this purpose.

## 2. Authoritative data and browse contract

Inventory browses the active product catalog, including only active products in active categories. The browse payload contains the following; payload presence is not display permission:

| Payload | Meaning and display boundary |
| --- | --- |
| `product_id`, `category_id`, `sku`, `name`, `category_name`, `available_quantity` | Identifiers, product and category identity, and current available units. Current Inventory result rows **display name, SKU, category name, list price, and textual stock status**. They do **not** display either ID or an additional numeric quantity column (the badge includes the stock quantity). The selected operation displays name, SKU, and current stock. Do not surface IDs as new visible browse/alert facts without separate approval. |
| `catalog_unit_price_centavos`, `list_price_centavos`, `minimum_sale_price_centavos`, `revision` | Catalog price is the compatibility alias of list price in this browse response; list price is shown in `Bs`, minimum sale price and revision are not shown. Do not surface alias, minimum price, revision, cost, or margin without separate approval. |
| `categories` | Active category ID/name options, ordered case-insensitively by name then ID. Category IDs are option values, not new visible row fields. |
| `page`, `page_size`, `total`, `total_pages` | Server paging metadata; Inventory requests **20** per page. `total` is the count matching browse filters, not an alert count. Show the existing page indicator and `Anterior`/`Siguiente` only when `total_pages > 1`; don't invent a new totals display or page-size selector. |

Search accepts a text query, optional category, and stock-state filter; activity is fixed to `active` on this screen, not a user control. Initial browse is an empty query, all categories, `all` stock, page 1. Query is trimmed when submitted; the server normalizes search for its catalog index. A query/filter edit resets the requested page to 1 and invalidates in-flight results; **`Buscar` submits** the chosen filters and starts page 1 (do not imply automatic fetch on every keystroke). `Anterior`/`Siguiente` submit the current query and filters for the requested page. A repeated submit is a new browse attempt. Results sort by product name case-insensitively, then product ID, **not relevance or stock severity**.

| Stock filter | Exact meaning for active browse products |
| --- | --- |
| `Todo el stock` | All nonnegative balances. |
| `Disponible` | Quantity **greater than 1**. |
| `Stock bajo` | Quantity **1**. |
| `Sin stock` | Quantity **0**. |
| `Alertas` | Quantity **0 or 1**. This is a browse filter, not the independently loaded alert panel. |

Current result badges read `Sin stock: 0`, `Stock bajo: 1`, or `Disponible: {quantity}`. Inventory allows selecting even an out-of-stock product for an operation. No sortable columns, activity selector, or search suggestion UI is approved.

## 3. Selected operation and confirmation

| Operation | Input rule | Preview and persistence |
| --- | --- | --- |
| `Entrada de stock` | Positive whole `Cantidad (unidades enteras)`; optional `Nota (opcional)`. | `Saldo proyectado` = selected current stock + entered quantity. Invalid or blank quantity cannot confirm. |
| `Conteo físico` | Whole `Conteo físico (unidades enteras)` **including zero**; required nonblank `Motivo`. | `Saldo proyectado` = count, not current stock plus count. Blank count is not zero. A count unchanged from the persisted current balance is rejected by the backend; don't show it as a successful adjustment. |

Keep the action `Confirmar operación` and reset action `Nueva operación` (returns to the browser while retaining its current query, filters, result page, and visible results); `Reintentar` belongs **only to a failed confirmation**, with a separate alert-panel `Reintentar` only when alerts are unavailable. There is no routine confirmation dialog. While confirmation is pending the operation is busy, fields/choices and actions are locked or disabled, duplicate confirmation is blocked, and the primary action says `Guardando…`. On success show only `Operación guardada. Stock actual: {resulting_quantity}.` Do not expose the richer persisted response (request ID, previous quantity, delta, timestamp, note) as new confirmation fields. The success stock value comes from the persisted result, not the preview. If persisted `previous_quantity` differs from the selected preview stock, also show `Saldo proyectado desactualizado. Revisá el stock actual.` Do not silently replace the preview with claimed live stock.

On generic confirmation failure: `No se pudo guardar la operación de inventario. Reintentá.` On request conflict: `El ID de solicitud ya fue usado con datos de inventario diferentes. Reintentá con los datos correctos.` Do not expose native storage diagnostics. The UI currently displays a generic confirmation error for backend validation errors such as unchanged count; changing that error wording or validation timing requires separate behavior approval. One unchanged confirmation payload is retried with its **same request ID** so persistence can replay it safely; editing product, operation, quantity, note, or reason or choosing `Nueva operación` resets that identity. Repeating a value without change does not reset it. A changed payload gets a fresh request ID on its next confirmation. Do not render request IDs or offer a manual retry before failure.

## 4. Alerts: context, not another workflow

The independent alert response contains **only** product ID, current product name, quantity, and classification. Only active products in active categories at quantity 0 (`out_of_stock` / `Sin stock: 0`) or 1 (`low_stock` / `Stock bajo: 1`) qualify. Ordering is **out of stock first**, then low stock; within each group, product name case-insensitively, then product ID. Inventory has **no alert list cap and no alert pagination**. Do not infer a total from what fits visually. The sidebar cue can report the returned alert count; this does not authorize an alert total in the panel or a `Ver todos` action. No alert rows are links, acknowledgements, or controls.

After a successful operation, Inventory asks App to refresh its sidebar cue and separately refreshes its own alert panel. App owns an independent alert request for the cue: its own refresh clears the cue while pending or on failure. A loading or failed Inventory alert-panel request does not by itself clear App's cue; the optional panel-to-cue callback is not wired by App. An alert-panel failure must not disable product browsing or confirmation.

## 5. State board — annotate, do not draw extra frames

| Area | Required annotated states and transitions |
| --- | --- |
| Browser | Initial `Seleccioná un producto para comenzar.`; initial automatic loading and subsequent browse resubmission `Buscando productos…` (keep this visible). During a subsequent load, the previous result list and pagination may remain visible alongside loading feedback; do not present them as newly fetched facts. Results include **sparse one-row** and **dense scrollable** pages; successful empty `No encontramos productos para “{query}”.`; failure `No se pudo buscar en el catálogo local. Reintentá.` with the existing search form available for resubmission. Empty is not loading or error. |
| Alerts | Independent `Cargando alertas de stock…`, ready rows or `No hay alertas de stock.`, and unavailable `Las alertas de stock no están disponibles. Reintentar`. Alert retry reloads only alerts. Browser and alert loading may overlap or finish in either order. |
| Confirmation | Idle/valid and invalid inputs; `Guardando…` pending lock; inline success, generic error + `Reintentar`, request conflict + `Reintentar`, and the stale-preview advisory after success. Retry reuses unchanged request identity; edit/reset creates another intent. |
| Async/navigation | Newer browser/alert requests win; old completions and completions after unmount do not update the screen. The sidebar has one `Inventario` button: when its alert cue is present, clicking anywhere on that button (including its text) opens Inventory with `Alertas` selected; without a cue it opens with `Todo el stock`. Dashboard `Ver en Inventario` also opens `Alertas`. There is no separate clickable cue. No duplicate initial browse or fabricated auto-refresh. |

Browse and alerts are independent requests, not one atomic snapshot. Pagination does not authorize alert pagination. `Nueva operación` clears the selected operation, but the existing ProductBrowser state and its visible results remain; do not depict a fresh initial browse or an automatic reload. Current limitation: this reset also clears the Inventory panel's alert rows without reloading them, leaving its ready state to display `No hay alertas de stock.` even if App's independent sidebar cue still reports alerts. Do not treat that misleading empty panel as authoritative stock status: it can persist until another alert load occurs. Do not add a new refresh behavior in a presentation-only change; fixing the reset requires separate behavior approval.

## 6. Shell, responsive layout, and scroll ownership

| Constraint | Design requirement |
| --- | --- |
| Persistent shell | Sidebar stays present: **208px at min-width 961px**, **176px at max-width 960px**. `Inventario` is active. Existing page padding is 24px desktop, 20px compact. No topbar, drawer, collapsed rail, new shell navigation, dialog, or focus trap. |
| Vertical scrolling | Shell content owns primary page scrolling. The **dense shared ProductBrowser result list additionally has its own tested nested vertical scroll area**; make its bounds and visible scrollbar discoverable, never clip or hide it. Its search form and pagination remain outside that list. The single-result desktop sparse case expands without a nested list scrollbar. Do not mistakenly apply Dashboard's single-scroll-panel rule here. |
| Compact | Stack the primary operation panel before the alerts panel. Keep the persistent 176px sidebar; allow page and, where dense, list scrolling without concealing alert access. Annotate which region scrolls. |

## 7. Three minimal Figma frames

All example content is illustrative and constrained to supported fields; it is not a new domain fact.

### A — Desktop browse + alerts (first review package)

1440 × 900, 208px sidebar. Show several active results, e.g. `Filtro de aire` / `FLT-AIR-008` / `Filtros` / `Bs 25,00` / `Stock bajo: 1`, `Correa de distribución` / `COR-DIST-031` / `Transmisión` / `Bs 84,00` / `Sin stock: 0`, each with `Seleccionar`. Show `Página 1 de 2`, `Anterior` disabled, `Siguiente` available; the example assumes at least 21 matching products even if only a few rows fit the image. Alerts show those two classifications and product names, read-only. Mark the list viewport, scrollbar, form, pagination, shell scroll, and secondary rail. **Proves:** browse hierarchy, multiple-result density, paging, visible alert context, and both scroll owners without suggesting alert pagination.

### C — Compact selected physical count (first review package)

960 × 800, 176px sidebar. Selected `Filtro de aire`, `SKU: FLT-AIR-008`, `Stock actual: 1`; `Operación` = `Conteo físico`, count `0`, nonblank `Motivo` (e.g. `Recuento de depósito`), `Saldo proyectado: 0`, enabled `Confirmar operación`, and `Nueva operación`; alerts below, including a zero and a one record. Annotate **shell content as primary scroll owner** and **dense browser list as a separate, only-when-browsing nested scroll viewport**; this selected screen has no browser list. Indicate that alerts are reached by scrolling the shell, not via a drawer. **Proves:** zero is valid, selected-first hierarchy, full labels, compact stacking, and explicit scroll ownership.

### B — Desktop selected stock entry (after A/C approval)

1440 × 900, 208px sidebar. Selected `Filtro de aire`, `SKU: FLT-AIR-008`, `Stock actual: 1`; `Operación` = `Entrada de stock`, quantity `3`, optional note, `Saldo proyectado: 4`, `Confirmar operación` and `Nueva operación`. Keep the same read-only alerts rail; no summary dialog. **Proves:** the browser-to-operation replacement, positive whole quantity, optional note, preview, minimal confirmation path, and alert continuity.

No separate loading, failure, detail, modal, or accessibility frames. Put Section 5 state annotations and Section 8 accessibility notes directly in the handoff/frame annotation board without making more mockups.

## 8. Accessibility and presentation acceptance items

- One page `h1` and named main/primary-operation/alert regions. Use properly labeled search and operation form controls, semantic result lists or tables where columns actually compare; compact results may be fully labeled records. Preserve meaningful labels and reading order rather than inventing clickable cards.
- Reading/tab order follows persistent sidebar → heading → browse form/results and their `Seleccionar` controls/pagination **or** selected-product controls and inline feedback → alert panel and its retry if unavailable. Noninteractive alert records need no tab stop. No programmatic focus move on entry, selection, load, confirmation, or navigation is claimed; proposing one is a **pending behavior decision**, not an approved fact.
- Visible focus: solid **3px `#C56845` outline, 2px offset**; system Highlight in forced colors. Minimum **44 × 44px** targets. Maintain visible borders/focus in forced-colors mode, respect reduced motion, keep scrollbars visible, and allow long names, SKUs, labels, money and statuses to wrap without ellipsis or overlap. Status must have text, never color alone.
- Keep live announcements restrained: announce meaningful loading, error, and confirmation outcomes, not every repeated badge/placeholder or a duplicate chorus from the alert cue and both panels. Do not hide visible loading copy behind an accessible-only label.
- **Evidence-backed future acceptance fixes (presentation/accessibility only):** shared ProductBrowser currently gives repeated `Seleccionar` buttons identical accessible names; make each name product-specific while leaving visible copy `Seleccionar` and preserving Catalog/Sales uses. Retain the current visible `Buscando productos…` during loads. Avoid redundant live announcements across browser, alerts, and sidebar. Make the existing dense nested-list scroll visibly discoverable and keyboard-reachable without hidden scrollbars. Check these in rendered interaction tests when implementation is separately approved.
- **Not preapproved:** changing validation timing (including detecting unchanged physical count before submit), introducing focus movement, or adding new status behavior. Seek a behavior decision first.

## 9. Prohibited capabilities and prototype boundary

Do not show, imply, or reserve controls for:

- inventory history, product detail, drilldown, linked alert rows, or another screen in this flow;
- extra persisted confirmation facts (request ID, delta, previous balance, note, timestamp), signed stock adjustments, thresholds, or bulk operations;
- suppliers, cost/margin, batches, lots, barcodes, transfers, import/export, print, product edit/archive/delete;
- alert actions, alert pagination, `Ver todos`, invented alert totals, manual/auto refresh or freshness indicators;
- autosave, undo, extra operations, routine confirmation modal, or new shell navigation.

Figma and optional HTML are disposable visual references. No Tailwind CDN, external fonts or icons, runtime prototype scripts, inline global handlers, hidden scrollbars, or prototype-only dependencies may enter production. Do not import or embed mockup assets in the app as a shortcut.

## 10. Acceptance checklist

- [ ] Operation-first primary panel preserves browser → selection → stock entry/physical count; alerts remain visible, read-only secondary context at both widths.
- [ ] Browse shows only the current approved fields; paging is 20 per request, filters and ordering match Section 2, and no alert count is inferred from browse metadata.
- [ ] Selected product, positive entry and zero-valid count, required reason, previews, backend unchanged-count rejection, and minimal persisted-success feedback match Section 3.
- [ ] Pending lock, error-only confirmation retry, exact request-ID reuse/replacement, stale advisory, alert retry, navigation filters, cue refresh, and stale/unmounted guards are not contradicted.
- [ ] Independent browser and alert state annotations cover initial/loading/results/empty/error, alert loading/ready/unavailable, sparse/dense, pending/success/error/conflict/stale.
- [ ] 208px/176px sidebar, 961px/960px breakpoint, shell scroll and discoverable dense-list nested scroll are annotated without new chrome or hidden scrollbars.
- [ ] Semantic structure, product-specific accessible selection names, visible loading copy, restrained announcements, exact focus, 44px targets, wrapping, forced colors and reduced motion are covered; no focus movement or new validation timing is claimed.
- [ ] Exactly A/C first, then B after approval; no prohibited capabilities or production prototype dependencies.

## 11. Delivery and later implementation boundary

Deliver ZIP packages for **A and C first**, and **B after visual-system approval**. Each frame ZIP contains required `screen.png` and optionally static `code.html`. `DESIGN.md` is excluded and non-authoritative; keep behavioral/accessibility annotations in this handoff and on the frames, not a competing document.

A future **separately authorized** visual implementation should stay in the Inventory screen and its rendered tests (`src/ui/inventory/inventory-screen.ts`, `src/ui/inventory/inventory-screen.mounted.test.ts`, and relevant screen tests), plus Inventory-scoped rules in `src/ui/styles.css`. Shared `src/ui/catalog/product-browser.ts` and its tests may change **only** for an approved generic accessibility fix that preserves Catalog and Sales behavior. Inventory flow/tests, `src/commands/`, Rust application/domain/commands, SQLite, and shell/navigation are unchanged for visual work. New data, validation timing, focus behavior, or operations require a separate decision before expanding these surfaces.
