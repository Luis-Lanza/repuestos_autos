# Inventory A/C reference annotations

Read alongside [`../inventory-figma-handoff.md`](../inventory-figma-handoff.md), which takes precedence for behavior, data, accessibility, and responsive constraints. The PNGs are static visual references; neither PNG nor optional HTML proves runtime behavior. These are annotations **outside** the application UI, not new screen copy or controls. Designer `DESIGN.md` is excluded and non-authoritative.

## Frame A — desktop browse, 1440 × 900

- Shell: persistent 208px sidebar; page content is the primary scroll owner. The `Inventario` label and sidebar alert cue are one navigation control. When a cue exists, clicking any part of that button opens the `Alertas` browse filter; without a cue, ordinary navigation opens `Todo el stock`.
- Catalog: search is submitted with `Buscar` or Enter, not fetched per keystroke. Category and stock filters take effect on submit; activity is fixed to active. Results come in pages of 20, ordered case-insensitively by product name then ID. `Página 1 de 2` requires at least 21 matching products, even though the illustration shows only a few visible rows. `Anterior` is disabled; `Siguiente` requests the next page.
- Scroll: the dense result list has its own nested vertical scroll viewport and visible scrollbar; search form and pagination remain outside it. The shell content may also scroll. Sparse one-result desktop lists do not need a nested scrollbar. Long names and SKUs wrap instead of being cut off.
- Alerts: a separate read-only request. Rows carry only product name, quantity/classification (out-of-stock before low-stock), not SKU, thresholds, or actions. Sidebar cue is loaded independently by App; the illustrated count of two is not a synchronized runtime guarantee or an alert total in the panel.

## Frame C — compact selected physical count, 960 × 800

- Shell: persistent 176px sidebar, 20px page padding. Selected operation comes before the alert panel. The page content, not a drawer or the sidebar, scrolls vertically. [`compact-physical-count-scrolled.png`](compact-physical-count-scrolled.png) is a supplementary capture after scrolling the shell 102px; both alert rows become visible. This is capture evidence for the static layout, not a keyboard or production-behavior test. No nested catalog list exists while a product is selected; the nested list applies only when browsing densely.
- The selected product displays name, `SKU: FLT-AIR-008`, and `Stock actual: 1`. `Conteo físico` takes a nonnegative whole **absolute count**; 0 is valid. Blank is **not** 0. `Motivo` is required and nonblank; `Saldo proyectado: 0` is a preview, not a persisted fact.
- `Confirmar operación` saves only after a valid input; `Nueva operación` clears the operation but retains ProductBrowser state. Current limitation: the reset also clears the panel alert rows without reloading them and can misleadingly display `No hay alertas de stock.` until another alert load; the independent sidebar cue may still report alerts. Do not depict this false empty as authoritative stock status or silently repair the reset in a presentation-only implementation.

## State board — annotations, not extra frames

| Area | States and transitions to preserve |
| --- | --- |
| Browser | Initial `Seleccioná un producto para comenzar.`; automatic first load and subsequent `Buscando productos…`; sparse and dense results; successful empty `No encontramos productos para “{query}”.`; error `No se pudo buscar en el catálogo local. Reintentá.` and resubmit through the existing search form. A subsequent loading state can retain old rows and pagination; these are not freshly fetched facts. |
| Alerts | Independent `Cargando alertas de stock…`, ready rows or `No hay alertas de stock.`, unavailable `Las alertas de stock no están disponibles. Reintentar`. That retry reloads only the panel, not the browser or App cue. |
| Operation | Stock entry: positive whole quantity, optional note, projected balance = selected stock + quantity. Physical count: nonnegative whole absolute count and required reason; blank is not zero; unchanged persisted count is rejected by backend. The existing UI does not proactively change validation timing. |
| Confirmation | Invalid/valid idle; `Guardando…` disables fields and choices and blocks duplicate confirmation. Success shows only `Operación guardada. Stock actual: {resulting_quantity}.`; if persisted previous stock differs from preview, also show `Saldo proyectado desactualizado. Revisá el stock actual.`. Generic failure and request conflict show their existing message plus `Reintentar` **only after error**. Unchanged retry reuses the request ID; editing intent or `Nueva operación` resets it. No routine confirmation dialog. |
| Async | Newer browser/alert responses win; completions after unmount do not update the screen. App cue uses its own request and refreshes after a successful operation; Inventory panel refreshes separately. A panel failure alone does not clear the global cue. |

## Accessibility review notes

- One `Inventario` h1; named operation and alert regions; native labeled form controls. Maintain reading/tab order: sidebar → page heading → browse/results/pagination **or** selected operation and inline feedback → alerts and its retry if unavailable. Alert rows are not focusable actions.
- Every repeated `Seleccionar` button needs a product-specific accessible name while retaining visible `Seleccionar`; preserve Catalog/Sales behavior when later implemented. No programmatic focus move is currently defined on selection, save, reset, or navigation; any new focus behavior needs a separate decision.
- Visible focus outline 3px `#C56845` with 2px offset (system Highlight in forced colors); interactive targets at least 44 × 44px. Preserve textual statuses, wrapping, discoverable scrollbar, reduced-motion handling, and restrained live feedback without duplicate announcements.
- The prototype's optional CDN/fonts/icons and scripts must not be copied to React/Tauri. Screenshots and HTML do not prove keyboard, focus, forced-colors, runtime alert synchronization, or persistence. Validate those separately if implementation is authorized.

## Review boundary

Approved A/C/B images are visual references only. The handoff governs implementation; the scroll proof is supplemental capture evidence, not an additional frame.
