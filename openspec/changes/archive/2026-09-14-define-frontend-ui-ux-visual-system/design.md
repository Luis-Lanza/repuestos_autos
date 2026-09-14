# Design: Warm Industrial Operations Workspace

## Creative direction

**Build a warm, mineral, modern-industrial desktop workspace: precise enough for a parts counter, distinctive enough not to feel like generic enterprise software, and restrained enough that operational data remains louder than decoration.** Use mineral plum for identity and primary action, copper as the focus/accent signal, warm ivory and stone for the working environment, and semantic sage/amber/garnet only where status requires them.

The visual signature comes from disciplined alignment, warm materials, compact numeric typography, visible borders, and a simple typographic wordmark—not automotive clichés, gradients, chrome effects, blue-enterprise conventions, or neutral-only minimalism.

### Review first

1. Review the shell, Sales/POS, and palette pairings first.
2. Review Inventory, Catalog, and History for density and state continuity.
3. Review the frame inventory and accessibility annotations before authorizing any mockup work.
4. Stop after this document for human approval. This document does not authorize mockups, tasks, apply work, code, or assets.

## 1. Principles, mood, and boundaries

| Principle | Concrete design decision | Review failure |
| --- | --- | --- |
| Operational clarity first | Identity, SKU, stock, amount, state, and next action receive stable positions and aligned columns. | Decorative color or imagery competes with product and transaction facts. |
| Balanced density | Default rows are 48px; dense forms use 12–16px internal gaps; major regions use 20–24px gaps. | Spreadsheet-like 32px rows or oversized card stacks reduce scanning. |
| Warm mineral character | Ivory canvas, stone borders, plum navigation, copper focus/accent. | Enterprise blue becomes the identity or all accents are removed. |
| Uncommon but restrained | One identity color and one interaction accent dominate; semantic colors appear only for meaning. | Every card, heading, and icon receives a different accent. |
| Honest state | Loading, empty, stale, unavailable, warning, and error remain distinct in copy, icon, and structure. | Color alone communicates a state, or stale data appears current. |
| Stable desktop orientation | Persistent shell and active location remain visible across every existing top-level area. | Navigation disappears outside Sales or implies URL routes/accounts. |
| Historical integrity | Persisted sale facts and correction records look read-only and audit-like. | Current catalog values appear to overwrite sale-time facts. |

**Mood words:** workshop precision, mineral warmth, calm urgency, ledger clarity, durable tools.

**Avoid:** bright corporate blue, black-and-yellow hazard-strip decoration, skeuomorphic metal/carbon fiber, product photography, oversized dashboard metrics, pill-shaped everything, excessive shadows, novelty automotive icons, and invented workflows.

## 2. Color system

### 2.1 Semantic palette hypothesis

The values below assign the confirmed palette hypothesis to roles. They are **recommended design candidates, not measured contrast evidence**. Every foreground/background and meaningful non-text pairing must be checked in the editable design file with a recognized contrast tool. Any pair marked **Validate/tune** must not be presented as compliant until measured.

| Token | Candidate value | Role and usage boundary | Recommended pairing | Contrast target / status |
| --- | --- | --- | --- | --- |
| `color.canvas` | `#F4EFE6` warm ivory | App canvas and quiet empty space; not input fill. | Ink text. | 4.5:1 normal text; **validate**. |
| `color.surface.default` | `#FFFCF7` warm white | Main work panels, forms, tables, modal. Derived neutral, used sparingly. | Ink text; stone border. | **Validate** text and border. |
| `color.surface.subtle` | `#EEE7DC` | Secondary panels, table headers, read-only summaries. Derived tint between ivory/stone. | Ink text. | **Validate**. |
| `color.surface.elevated` | `#FFFFFF` | Modal and rare overlay only. | Ink text. | Expected strong pairing; still measure. |
| `color.border.default` | `#E4DDD2` light stone | Dividers, fields, table rows. Never the sole control boundary when contrast is insufficient. | Against white/ivory. | 3:1 meaningful control boundary; **likely needs darker companion**. |
| `color.border.strong` | `#A89D91` | Input boundary, selected row outline, strong dividers. Derived shade. | Against white/ivory. | 3:1 target; **validate/tune**. |
| `color.text.primary` | `#29252B` ink | Body, headings, values. | Ivory, white, subtle surfaces. | 4.5:1 normal text; **validate**. |
| `color.text.secondary` | `#625A63` | Secondary facts and helper text; never critical state alone. | White/ivory. | 4.5:1 target; **validate/tune**. |
| `color.text.inverse` | `#FFFFFF` | Text/icons on plum, dark sage, garnet. | Dark fills only. | 4.5:1 target; **validate each fill**. |
| `color.nav.background` | `#3D3042` mineral plum | Persistent navigation and app identity only. | White/ivory text and icons. | **Validate** normal text and icons. |
| `color.nav.active` | `#F4EFE6` | Active navigation inset/marker and text emphasis. | Plum background; use shape and `Actual` semantics. | 3:1 non-text and 4.5:1 text target. |
| `color.action.primary` | `#3D3042` mineral plum | One main action per work region: e.g. `Confirmar venta`. | White text. | 4.5:1 target; **validate**. |
| `color.action.primary.hover` | `#312535` | Primary hover, derived 12–15% shade. | White text. | **Validate**. |
| `color.action.secondary` | transparent / white | Secondary button surface. | Plum text + strong border. | Text 4.5:1; boundary 3:1 target. |
| `color.accent.focus` | `#C56845` copper terracotta | Focus ring, selected indicator, sparing active accent; not warning/error. | 3px ring outside a 2px canvas/surface gap. | 3:1 against adjacent colors; **validate/tune required**. |
| `color.accent.decorative` | `#C56845` | Wordmark dot/short rule, max one small accent per region. | No small white text on copper until measured. | Decorative use has no semantic burden. |
| `color.success.base` | `#71806A` industrial sage | Success icon, edge, or tinted banner—not body text on white. | Ink text on sage tint. | Raw sage/white pairing **requires validation**. |
| `color.success.strong` | `#52634D` | Success text/icon or filled status if validated. Derived shade. | White text or pale sage background. | **Validate/tune** to 4.5:1. |
| `color.success.surface` | `#E7ECE3` | Success banner/background, derived tint. | Ink or success-strong text + check icon. | **Validate**. |
| `color.warning.base` | `#C7923E` mineral amber | Advisory, low stock, stale preview/price. Never used as body text alone. | Ink text + triangle/clock icon. | Ink-on-amber and amber non-text **validate**. |
| `color.warning.surface` | `#F5E9D2` | Warning/stale background, derived tint. | Ink text + amber/dark icon. | **Validate**. |
| `color.error.base` | `#A33F46` garnet | Error/destructive border, icon, destructive action. | White on fill, or garnet text on pale error surface. | **Validate/tune** both pairings. |
| `color.error.hover` | `#873139` | Destructive hover, derived shade. | White text. | **Validate**. |
| `color.error.surface` | `#F5E2E2` | Error banner and invalid-field background hint. | Garnet/ink text + octagon/alert icon. | **Validate**. |
| `color.unavailable.surface` | `#E4DDD2` light stone | Archived/unavailable record and disabled context. | Ink/secondary text + explicit label/icon. | **Validate**. |
| `color.stale.surface` | `#F5E9D2` | Stale data, stale price, stale projection. | Ink + clock icon + `Desactualizado`. | Same as warning; stale remains a separately named token. |
| `color.disabled.foreground` | `#817982` | Disabled labels/icons only. | `#EEE7DC` disabled fill. | Disabled content is exempt only when truly inactive; still keep legible. |
| `color.disabled.background` | `#EEE7DC` | Disabled button/input fill. | Disabled foreground + disabled cursor/state. | Do not confuse with unavailable record state. |

### 2.2 Role distinctions

- **Primary action:** plum fill; one per active work region. It means “advance/commit this routine workflow,” not success.
- **Focus:** copper outer ring plus a 2px light gap. Focus never uses warning amber or error garnet.
- **Success:** sage checkmark and pale sage surface after persisted/confirmed outcomes.
- **Warning/advisory:** amber triangle and explicit Spanish text such as `Precio actualizado; revisá antes de confirmar`.
- **Error/destructive:** garnet octagon/trash/ban icon and consequence language. Errors remain inline; irreversible confirmation uses a modal.
- **Stale:** amber clock/refresh icon and `Desactualizado`; never shown as success.
- **Unavailable:** stone surface, broken-link/ban icon, and `No disponible`; archived is separately labeled `Archivado`.
- **Navigation:** plum is identity/location, not an error or status color.

### 2.3 Tint, shade, and fallback strategy

Use raw hypothesis colors at `500`. Derive only documented `100` surfaces and `700` interaction shades; do not create a decorative rainbow of steps.

| Family | 100 use | 500 use | 700 use |
| --- | --- | --- | --- |
| Plum | Optional selected tint outside nav | Navigation/action | Hover/pressed |
| Copper | No large surface | Focus/decorative accent | Focus fallback if 500 misses 3:1 |
| Sage | Success surface | Icon/edge | Text/filled success |
| Amber | Warning surface | Icon/edge | Warning text if needed |
| Garnet | Error surface | Icon/destructive | Hover/strong text |

**Compact adjustment strategy:** keep semantic assignments fixed; first darken the relevant `500` toward its proposed `700`, then lighten only the background tint, then add a strong ink outline. If copper cannot achieve a 3:1 focus boundary on both ivory and plum, use a dual ring (`2px #FFFFFF` gap + `3px` tuned copper/darker copper). Do not replace the system with a blue theme. Record final measured ratios beside Figma color styles.

## 3. Typography, numerals, icons, and identity

### 3.1 Font recommendation

| Use | Family | License and language | Fallback |
| --- | --- | --- | --- |
| UI, headings, controls | **Barlow** | SIL Open Font License 1.1; Latin/Spanish glyph support must be included in the chosen files. Redistribution allowed under OFL; include license text. | `Segoe UI`, `Arial`, sans-serif |
| SKU, request IDs, optional compact machine values | **IBM Plex Mono** | SIL Open Font License 1.1; Spanish glyph support available. Include license text and source/version. | `Cascadia Mono`, `Consolas`, monospace |

Use local packaged font files in a future implementation rather than depending on network delivery; the application is offline-first. The designer must identify exact font files, weights, source URLs, versions, and OFL texts. If either family cannot be packaged with clear rights, use the Windows fallbacks and flag the substitution.

### 3.2 Type tokens

| Token | Size / line height / weight | Use |
| --- | --- | --- |
| `type.display` | 28px / 34px / 650 | Sale total and confirmed-sale outcome only. |
| `type.h1` | 24px / 30px / 650 | Screen title; one per main area. |
| `type.h2` | 18px / 24px / 650 | Region title. |
| `type.h3` | 16px / 22px / 600 | Subregion or correction record. |
| `type.body` | 15px / 22px / 400 | Default body and control text. |
| `type.body.strong` | 15px / 22px / 600 | Product names, selected facts. |
| `type.small` | 13px / 18px / 400 | Helper text and secondary metadata. |
| `type.label` | 13px / 18px / 600 | Field/table labels; sentence case. |
| `type.numeric` | 15px / 20px / 500 | Quantities, dates, prices; `font-variant-numeric: tabular-nums`. |
| `type.total` | 24px / 30px / 650 | `Bs 350,00`; tabular numerals. |
| `type.sku` | 13px / 18px / 500 IBM Plex Mono | `SKU: FIL-ACE-001`; never all-caps transform user data. |

Spanish sentence case is mandatory: `Historial de ventas`, not title case. Use a comma decimal in all proposed UI examples (`Bs 125,50`) and consistent aligned digits. Request IDs may truncate visually with full value available in accessible text/annotation; do not imply copy behavior unless already approved.

### 3.3 Icons

Recommend **Lucide** outline icons (ISC License) at 20px, 1.75px stroke, rounded joins; 16px only beside compact metadata and 24px in empty states. Keep a 44×44 hit area for icon actions. Include the Lucide license/version and list every selected glyph in handoff.

| Existing destination/action | Suggested icon | Spanish accessible name or visible label |
| --- | --- | --- |
| Ventas | `ShoppingCart` | `Ventas` |
| Inventario | `Boxes` | `Inventario` |
| Catálogo | `Tags` | `Catálogo` |
| Alta de productos | `PackagePlus` | `Alta de productos` |
| Historial de ventas | `History` | `Historial de ventas` |
| Copia y restauración | `DatabaseBackup` | `Copia y restauración` |
| Search/retry/stale | `Search`, `RefreshCw`, `Clock3` | Visible text or accessible name required. |
| Low/out stock | `TriangleAlert`, `CircleOff` | `Stock bajo`, `Sin stock`. |

Do not use an icon alone for primary, destructive, unfamiliar, or modal actions. Tooltips may supplement but never replace accessible names.

### 3.4 Wordmark

Use a text-only lockup: a 24px square containing a simple **RA** monogram built from type (not a bespoke production logo), followed by `Repuestos Autos` in Barlow 650. Add one short copper rule/dot as the sole identity accent. No gear, car silhouette, wrench, piston, or shield. This is a **direction for a future editable mockup**, not a produced brand asset. Ownership and final trademark review remain unresolved until human approval.

## 4. Spacing, geometry, and density

### 4.1 Tokens

| Token | Value | Use |
| --- | ---: | --- |
| `space.0` | 0 | Reset only. |
| `space.1` | 4px | Icon/text micro-gap. |
| `space.2` | 8px | Related inline items. |
| `space.3` | 12px | Compact field/row internals. |
| `space.4` | 16px | Default field and card gap. |
| `space.5` | 20px | Panel gap at minimum size. |
| `space.6` | 24px | Page regions and default page padding. |
| `space.8` | 32px | Major section separation. |
| `space.10` | 40px | Rare empty-state breathing room. |
| `space.12` | 48px | Shell identity height/large separation only. |

### 4.2 Geometry contract

| Element | Decision |
| --- | --- |
| Field/button height | 44px default; 48px for Sales search and `Confirmar venta`; compact rows still preserve a 44px hit area. |
| Checkbox | 20px visual box inside a minimum 44px label target. |
| Quantity stepper | 44px input and 44×44 decrement/increment targets only if existing behavior supports steppers; otherwise render the numeric input alone. |
| Radius | 6px fields/buttons; 8px panels; 10px modal. Avoid full pills except short status badges. |
| Border | 1px default; 2px selected/invalid; 3px focus ring outside 2px gap. |
| Elevation | Level 0 flat canvas; level 1 subtle `0 1px 2px rgba(41,37,43,.10)` for sticky bars; level 2 `0 12px 32px rgba(41,37,43,.18)` for modal only. Values remain visual hypotheses. |
| Content width | Shell fills window. Form reading column max 720px; detail text max 840px; data regions may use all remaining width. |
| Page padding | 24px at 1200×800; 20px at 960×640. |
| Table row | 48px default, 56px when two metadata lines are necessary. Header 40px. |
| Panel padding | 20px default; 16px at 960×640. |
| Target size | Minimum 44×44 CSS px for every interactive target. |

### 4.3 Density rules

- Show one primary and up to three supporting facts per result row before wrapping to a second metadata line.
- Do not place each fact in its own card. Use panels for work regions and aligned rows inside them.
- Money and quantities align right; names/SKU align left; states and actions occupy stable trailing columns.
- At 960px, reduce gaps before reducing type. Never drop below the documented body/label sizes.
- Horizontal scrolling is not acceptable for the shell or primary action region. A wide data table may become labeled rows/stacked detail rather than hide columns.

## 5. Persistent shell

### 5.1 Information architecture

No URLs, routes, roles, account controls, or permissions are introduced. Navigation changes the same six existing top-level React screen states.

```text
┌──────── Persistent navigation ────────┬──────────── Main work area ────────────┐
│ [RA] Repuestos Autos                  │ Screen header: title + status/action   │
│                                      ├─────────────────────────────────────────┤
│ OPERACIÓN                            │ Inline page feedback, when present      │
│  Ventas                              ├─────────────────────────────────────────┤
│  Inventario  [3 alertas]*            │ Screen-specific panels / data           │
│                                      │                                         │
│ GESTIÓN                              │ Only this region scrolls by default      │
│  Catálogo                            │                                         │
│  Alta de productos                   │                                         │
│                                      │                                         │
│ REGISTROS Y SISTEMA                  │                                         │
│  Historial de ventas                 │                                         │
│  Copia y restauración                │                                         │
└──────────────────────────────────────┴─────────────────────────────────────────┘
* Show the existing alert count only when available; do not invent polling/notifications.
```

### 5.2 Exact composition by reference size

| Property | 1200×800 | 960×640 |
| --- | --- | --- |
| Navigation | 208px fixed width; 24px horizontal padding; full wordmark and labels. | 176px fixed width; 16px padding; wordmark wraps to two lines if needed; labels remain visible. No icon-only rail. |
| Main width | 992px; page padding 24px; usable inner width 944px. | 784px; page padding 20px; usable inner width 744px. |
| Identity block | 72px high, top aligned. | 64px high. |
| Nav item | 44px high, 8px radius, 12px gap icon-to-label. | 44px high, 8px gap; 13px label if needed. |
| Content header | 64px minimum; title left, existing contextual action/status right. | 56px minimum; title first row; status/action may wrap beneath within header. |
| Scroll | Nav stays fixed; main work viewport scrolls. | Same. Do not create nested page scroll unless a master-detail pane explicitly needs it. |
| Sticky | Content header may remain sticky with level-1 shadow after scroll; Sales total/action bar stays sticky at the bottom of its panel. | Content header and Sales total/action bar remain sticky; detail identity bar is sticky inside long History/Catalog detail. |

**Active treatment:** 4px copper left marker, ivory-tinted inset surface at 10% opacity on plum, 600-weight white text, icon and visible `Actual` text only in annotation/accessibility metadata—not an extra UI badge unless space permits. Inactive items use warm-white at lower emphasis but must still meet contrast. Focus uses the dual copper ring and is distinct from active state.

**Stock cue:** Inventory row may show `3 alertas` in a compact badge with `TriangleAlert`; zero has either no badge or explicit existing-state treatment chosen by the designer, but no new notification behavior. At minimum size use `3` plus accessible/tooltip annotation `3 alertas de stock` only if label space is constrained.

**Content header:** screen title, one-line purpose or persisted identity, and only existing contextual actions. Do not add global search, user profile, settings, breadcrumbs, shortcuts, or draft preservation.

**Focus order:** navigation landmark in DOM order, then page header/context action, inline page feedback when programmatically focused after failure, then main work controls. Visual placement must match this logical order.

## 6. Shared primitives and state contracts

### 6.1 Buttons

| Variant | Appearance | Existing examples | States |
| --- | --- | --- | --- |
| Primary | Plum fill, white label, optional leading icon. | `Confirmar venta`, `Confirmar operación`, `Guardar metadatos del catálogo`. | Hover darkens; pressed inset; focus dual ring; pending retains width and uses spinner + `Confirmando…`; disabled stone. |
| Secondary | White/transparent fill, strong plum border/text. | `Nueva venta`, `Seleccionar`, `Recargar detalle de venta`. | Same interaction states; no fill competition with primary. |
| Tertiary | Text/icon on transparent surface. | `Volver al historial`, `Descartar borrador`, `Nueva operación`. | Underline or subtle surface on hover; visible focus. |
| Destructive | Garnet fill or garnet outline when not the final modal action. | `Archivar`; final `Cancelar venta` / `Restaurar datos`. | Never visually primary outside confirmation; consequence label must be explicit. |
| Icon action | 44×44, 20px icon, accessible name. | Search/retry/remove only where the visible context is unambiguous. | Tooltip optional; focus and pressed states required. |

`Quitar` a draft cart line is tertiary, not destructive: it changes only the current draft. `Descartar borrador` remains immediate because the existing behavior is immediate and it does not alter stock. `Archivar` should be garnet outline with `Archive` icon and `Archivar` text; this design does not add a new confirmation behavior. Truly irreversible restore and sale cancellation use the modal pattern below.

### 6.2 Fields

| Primitive | Exact treatment and Spanish example |
| --- | --- |
| Text | Label above, 8px gap, 44px field. Example `Nombre del producto`; helper below. |
| Search | 48px field with leading Search icon, label `Buscar en el catálogo`, placeholder `SKU, nombre, categoría o atributo`; adjacent `Buscar`. Do not imply instant search. |
| Date | Label `Desde` / `Hasta`, native date affordance retained; no invented date picker behavior. |
| Select | 44px, visible chevron, label above. Example `Operación` with `Entrada de stock` and `Conteo físico`. |
| Checkbox | Checkbox left, full sentence label right, 44px row; grouping uses fieldset/legend. |
| Quantity | Right-aligned tabular number, suffix/helper `unidades`, integer step and limits in annotation. Example `Cantidad` = `2`. Do not imply fractional units. |
| Money | Prefix box `Bs`, operator value `125,50`, right-aligned tabular numerals. Annotation: presentation parses/formats to integer centavos at the existing seam; no floating-point authority. |
| SKU | Standard text field when editable; displayed in mono with `SKU` label. Example `FIL-ACE-001`. |

**Field states:** default strong boundary; hover darker boundary; focus dual copper ring; invalid 2px garnet boundary + error icon + adjacent Spanish correction; disabled stone fill; pending retains values and sets busy/disabled intent where existing flow does; read-only persisted facts use a definition-list/table value, not a disabled input.

**Validation example:** `Ingresá una cantidad entera entre 1 y 3.` Associate with the field and annotate focus to the first invalid field. General command errors remain at form level after field errors.

### 6.3 Data lists, tables, master-detail, and panels

- Use semantic table intent only for truly tabular facts. Product search may be an aligned data list because each row includes an action.
- Header labels are visible and sentence case. Do not display sort arrows unless sorting exists.
- Selected rows have a 2px plum outline, copper 4px edge, and `Seleccionado` accessible annotation; color is not the sole cue.
- Catalog master list uses 36–40% width at 1200 and stacks above detail at 960. History list/detail are distinct existing views, not a simultaneous invented route.
- Sticky table headers are allowed only with an annotated scroll container and logical reading order.
- Panels use surface default, 1px border, 8px radius, and no shadow unless sticky/elevated.
- Summary strips use aligned definition facts, not dashboard cards.

### 6.4 Badges

| Meaning | Copy | Cue |
| --- | --- | --- |
| Available | `Disponible: 8` | Neutral outlined badge. |
| Low stock | `Stock bajo: 1` | Amber triangle + text. |
| Out of stock | `Sin stock: 0` | Garnet/ink CircleOff + text. |
| Active | `Activo` | Sage check + text. |
| Archived | `Archivado` | Stone badge + Archive icon. |
| Stale | `Desactualizado` | Amber clock + text. |
| Unavailable historical data | `No disponible` | Stone broken-link/ban icon + text. |
| Confirmed/cancelled | `Confirmada` / `Cancelada` | Check / ban icon plus text; persisted fact, not an editable control. |

### 6.5 Feedback and recovery

| State | Pattern | Spanish copy example | Semantic/live intent |
| --- | --- | --- | --- |
| Loading | Inline progress row or restrained skeleton that preserves layout. | `Cargando historial de ventas…` | Affected region `busy`; polite status once. |
| Initial | Quiet instruction with contextual icon. | `Buscá un producto para comenzar.` | No alert. |
| Empty collection | Dashed/outlined quiet panel. | `Todavía no hay registros para mostrar.` | Polite status if result of load. |
| No results | Search term retained and Search icon. | `No encontramos productos para “filtro aceite”.` | Polite status. |
| Success | Pale sage inline banner with check. | `Operación guardada. Stock actual: 8.` | Polite status; persists according to existing screen behavior. |
| Advisory | Amber inline banner. | `El saldo proyectado quedó desactualizado. Revisá el valor actual.` | Polite status unless urgent. |
| Validation | Field-level message and summary only when needed. | `Ingresá un monto válido en Bs.` | Field description; focus first invalid control. |
| Error | Pale garnet banner in affected region. | `No se pudo buscar en el catálogo local.` | Assertive alert once; keep entered context. |
| Unavailable/retry | Bordered recovery panel. | `El catálogo no está disponible.` + `Reintentar` | Alert on failure, then status on retry result. |
| Stale | Amber clock strip next to the affected record/fact. | `El precio cambió de Bs 120,00 a Bs 125,50.` + `Aceptar precio actual` | Status plus required acknowledgement remains. |
| Pending | Button spinner and stable pending label; affected form busy. | `Confirmando…` | Busy; duplicate activation unavailable. |

Do not add toasts as the primary feedback mechanism. Inline placement preserves the affected-work relationship and offline recovery context.

### 6.6 Destructive modal

Reserved for **restore over current local data** and **irreversible sale cancellation** as required by the specification. It is not used for search, selection, routine save, retry, return submission, cart-line removal, or draft discard.

```text
┌──────────────────────────────────────────────┐
│ [!] Cancelar venta #184                     │
│ Esta acción cancela la venta y corrige el   │
│ inventario. Los pagos originales no cambian.│
│                                              │
│ Motivo: Producto cargado por error           │
│ ☑ Entiendo la corrección de inventario       │
│                                              │
│ [Volver]                  [Cancelar venta]   │
└──────────────────────────────────────────────┘
```

Modal width 520px (max `calc(100vw - 48px)`), 24px padding, 10px radius, garnet icon/title accent, and dimmed ink overlay. Initial focus goes to `Volver` unless required reason/acknowledgement is invalid, then to that correction target. Tab is contained. Escape and `Volver` dismiss safely. On dismissal focus returns to `Iniciar cancelación de venta` or `Elegir archivo de respaldo`; on completion focus moves to the refreshed persisted detail/success heading. Pending keeps the modal open, disables duplicate submission, and announces status. Restore preserves the existing acknowledgement inside the modal.

## 7. Screen descriptions for future mockups

All content below is illustrative Spanish copy for design review. IDs, dates, SKUs, and amounts are realistic sample data, not new fixtures or behavior.

### 7.1 Sales/POS — hero composition

**1200×800 regions:** within the 944px inner width, use a 560px discovery/cart panel and a 364px payment/summary panel separated by 20px. The content header reads `Ventas` and helper `Nueva venta`. Discovery contains Search, results (top), and cart (below). Payment/summary contains payment fields, total, stale/error feedback, and sticky actions.

**960×640 transformation:** one-column logical order: Search/results, cart, payment, total/actions. Search results may scroll within a max 176px region while the page remains the primary scroll container. The total/action bar sticks to the bottom of the main viewport without covering focused fields. No horizontal scroll.

**Default populated content:**

- Search: `filtro aceite`.
- Results row 1: `Filtro de aceite ACDelco`, `SKU FIL-ACE-001`, `Filtros`, `Disponible: 8`, `Bs 85,50`, action `Agregar`.
- Results row 2: `Filtro de aceite Premium`, `SKU FIL-PRE-014`, `Stock bajo: 1`, `Bs 125,50`, action `Agregar`.
- Cart: two aligned lines with product/SKU, immutable unit price, quantity field, line subtotal, and tertiary `Quitar`.
- Payment group `Pago`: `Efectivo recibido (Bs)` = `200,00`; `Pago QR (Bs)` = `150,00`; derived persisted/payment facts may be shown only as current behavior supports. Total: `Bs 350,00`.

**Actions to render:**

- Primary: `Confirmar venta` (`Confirmando…` pending).
- Secondary result-row action: `Agregar`.
- Tertiary: `Quitar`, `Descartar borrador`.
- Stale safeguard: `Aceptar precio actual` after explicit `El precio de Filtro de aceite Premium cambió de Bs 120,00 a Bs 125,50.`
- There is no destructive action and no confirmation modal in draft Sales.

**Required states:** initial (`Buscá un producto para comenzar`), search loading, no results, populated, invalid quantity/payment, stale price with disabled confirmation, command failure, confirmation pending/disabled, and focused controls. Preserve query/cart context where the existing state does. Do not invent autocomplete, barcode, customer, discount, tax, print, or receipt behavior.

**Focus order:** Search field → `Buscar` → result rows/actions → cart quantity/removal by line → cash → QR → stale acknowledgement if present → `Confirmar venta` → `Descartar borrador`. On invalid payment, focus the first invalid payment field; stale response focuses the stale banner heading/action.

### 7.2 Confirmed sale summary

This replaces the editable Sales draft and retains the shell with Ventas active. It must feel persisted and receipt-like without implying printing.

**Regions:** success header with check and `Venta confirmada`; identity strip (`Venta #184`, `Confirmada el 14/08/2026, 10:42`, optional request ID as secondary audit text); products table (`Producto`, `SKU`, `Cant.`, `Precio unitario`, `Subtotal`); payment facts (`Efectivo aplicado`, `Efectivo recibido`, `Cambio`, `QR`); total; one action.

**Default content:** `Filtro de aceite ACDelco`, quantity `2`, `Bs 85,50`; `Correa auxiliar 6PK`, quantity `1`, `Bs 179,00`; total `Bs 350,00`. Historical values use read-only surfaces and tabular numerals.

**Primary action:** `Nueva venta`. No edit, print, share, refund, or navigation shortcut is invented.

**960×640:** identity strip wraps; product table becomes labeled rows while keeping quantity/price/subtotal aligned; `Nueva venta` remains visible after the total. Main region scrolls, header remains sticky.

### 7.3 Inventory

**1200×800 regions:** header `Inventario`; 600px operation workspace on the left and 324px stock-alert panel on the right. Search sits above selected product inside the workspace. Selected-product strip shows name, SKU, and current stock. Operation panel uses a select for `Entrada de stock` / `Conteo físico`.

**Default populated physical-count content:** selected `Filtro de aceite Premium`, `SKU FIL-PRE-014`, `Stock actual: 1`; operation `Conteo físico`; `Conteo físico` = `0`; required `Motivo` = `Producto dañado`; projected strip `Saldo proyectado: 0 · Sin stock`. Alert panel groups `Sin stock (2)` before `Stock bajo (3)` with text/icon cues.

**Stock-entry annotated variant:** `Cantidad` = `6`, optional `Nota` = `Ingreso de depósito`, `Saldo proyectado: 7`. Do not invent supplier/cost fields.

**Actions to render:**

- Search: `Buscar`.
- Result row secondary: `Seleccionar`.
- Primary: `Confirmar operación` (`Guardando…`).
- Tertiary: `Nueva operación`.
- Recovery where existing: `Reintentar`.
- No new modal. Physical count risk is expressed with projection/reason and existing confirmation action, not an invented additional safeguard.

**960×640:** operation workspace first; alerts stack below as a full-width section. A compact header cue shows `5 alertas de stock` and links visually to the existing Inventory destination without inventing new navigation behavior. Selected identity and primary action remain sticky within the page only if annotations preserve reading order. Alert list remains reachable after operation result.

**States:** no selection, search loading/no results, selected operation variants, reason validation, pending, success with resulting balance, stale-projection advisory, command failure, low stock = exactly 1, out of stock = exactly 0. Whole units only.

### 7.4 Catalog maintenance

**1200×800:** 336px master list + 588px detail/editor with 20px gap. Header `Catálogo` and helper `Editá metadatos o cambiá el estado de categorías y productos.` Master list filter/search is not drawn unless existing behavior supports it; the current record list is simply scannable. Each row shows target type, name/SKU when available, and `Activo`/`Archivado`. Selected row is structurally marked.

**Default selected product:** `Filtro de aceite Premium`, `SKU FIL-PRE-014`, `Producto`, `Activo`, revision as secondary metadata only if surfaced by current behavior. Editor groups `Datos generales` and `Atributos de Filtros`. Fields: `Nombre del producto`, `SKU`, `Precio actual del catálogo (Bs)` = `125,50`; helper `Afecta solo ventas futuras. Las ventas confirmadas no cambian.` Dynamic examples `Marca`, `Rosca`, `Altura (mm)`.

**Actions to render:**

- Row: `Ver detalles`/whole-row selection treatment.
- Primary: `Guardar metadatos` (`Guardando metadatos…`).
- Destructive outline lifecycle action: `Archivar` for active records.
- Secondary lifecycle action: `Reactivar` for archived records.
- Recovery: `Reintentar catálogo`, `Recargar registros del catálogo`.

`Archivar` remains the existing immediate lifecycle command; this brief does not add a modal or alter behavior. The treatment communicates severity without pretending it deletes historical data.

**960×640:** stack master over detail. Master list is a 176–208px-high scroll region; selected identity (`Filtro de aceite Premium · Activo`) becomes a sticky detail identity bar. Detail form follows in logical order. Preserve selection while visually stacking; no side-by-side squeeze. Actions remain at the end of the relevant detail region, with save optionally sticky only when it does not cover field errors.

**States:** initial loading, explicit no-record empty treatment, active/archived comparison, field invalid with focus destination, pending/busy, success status, unavailable/retry, stale record with `Registro desactualizado` and `Recargar registros del catálogo`. Unavailable data is not editable.

### 7.5 Sales history list

**Regions:** header `Historial de ventas`; compact date filter row (`Desde`, `Hasta`, `Cargar historial`); bounded-result notice; table/data list with `Venta`, `Fecha y hora`, `Estado`, `Líneas`, `Pagos`, `Total`, action/selection.

**Default populated content:** today’s inclusive range; rows such as `Venta #184`, `14/08/2026, 10:42`, `Confirmada`, `2 líneas`, `2 pagos`, `Bs 350,00`. Use `Hay más ventas. Reducí el rango de fechas.` for bounded overflow. Do not show pagination, reports, sorting, or export.

**Actions:** primary within filter `Cargar historial`; secondary row `Ver detalle`; recovery `Reintentar historial`.

**960×640:** filters wrap into two date fields and full-row action; each sale becomes a labeled two-line row, keeping identity/date/total visible. Main page scrolls. Loading, empty (`No hay ventas confirmadas en este rango.`), populated, overflow notice, error, and retry are annotated variants.

### 7.6 History detail and return

**Regions in order:** tertiary `Volver al historial`; sticky persisted identity strip; `Artículos originales`; `Pagos originales`; `Historial de correcciones de inventario`; eligible correction actions. Original facts use neutral read-only surfaces. Corrections use separate dated records and never overwrite originals.

**Default facts:** `Venta #184`, `Confirmada`, `14/08/2026, 10:42`, `Total original: Bs 350,00`. Missing snapshots read `SKU no disponible` / `Producto no disponible`, not substituted current data.

**Actions:**

- Tertiary: `Volver al historial`.
- Secondary routine correction start: `Iniciar devolución de artículos`.
- Primary within return form: `Registrar devolución`.
- Recovery: `Recargar detalle de venta`.
- Destructive entry: `Iniciar cancelación de venta`; final destructive modal action `Cancelar venta`.

**Open return:** inline panel after original items. Each eligible line has checkbox `Incluir este artículo`, persisted remaining quantity, and integer `Cantidad a devolver`. Ineligible lines remain visible with `Sin unidades disponibles para devolver`. Error example: `Seleccioná al menos un artículo.` or `Ingresá una cantidad entre 1 y 2.` Focus moves to first invalid selection/quantity. Pending disables duplicate submission and says `Registrando devolución…`; failure keeps selections and shows existing reload recovery; success is evidenced by refreshed persisted correction history.

**960×640:** all sections stack. Persisted identity bar remains sticky; tables become labeled rows. Only one correction work area is visually open. Reading and focus order follow DOM order; sticky placement does not reorder controls.

### 7.7 Cancellation

The cancellation intent belongs to History detail. Reason entry and the required inventory-correction acknowledgement remain explicit. Final confirmation uses the shared modal.

**Inline preparation:** heading `Cancelar venta`, field `Motivo de cancelación`, checkbox `Confirmo esta corrección de inventario. Los pagos originales no cambian.`, safe `Volver`, destructive `Continuar con la cancelación`. The latter opens the final modal only after existing validation is represented; no new eligibility rule is invented.

**Modal:** title `Cancelar venta #184`; consequence `Se cancelará la venta y se restaurarán únicamente las unidades todavía no devueltas. La venta y los pagos originales seguirán visibles en el historial.` Actions `Volver` and `Cancelar venta`. Pending `Cancelando venta…`; failure remains in modal or affected detail with `Recargar detalle de venta`; success returns to refreshed persisted detail showing `Cancelada` and cancellation record.

### 7.8 Screen/state summary

| Screen | Primary action | Secondary/tertiary actions | Destructive action |
| --- | --- | --- | --- |
| Sales | `Confirmar venta` | `Buscar`, `Agregar`, `Quitar`, `Aceptar precio actual`, `Descartar borrador` | None |
| Confirmed summary | `Nueva venta` | None | None |
| Inventory | `Confirmar operación` | `Buscar`, `Seleccionar`, `Nueva operación`, `Reintentar` | None |
| Catalog | `Guardar metadatos` | `Ver detalles`, `Reactivar`, retry/reload | `Archivar` outline, existing immediate behavior |
| History list | `Cargar historial` | `Ver detalle`, `Reintentar historial` | None |
| History return | `Registrar devolución` | `Volver`, select lines, `Recargar detalle` | None |
| History cancellation | `Continuar con la cancelación` | `Volver`, reload | Modal `Cancelar venta` |
| Backup/restore | `Crear copia` / preparation action | file selection/recovery | Modal `Restaurar datos` |

## 8. Continuity: onboarding and backup/restore

These areas use the same shell, fields, panels, statuses, and accessibility contract, but they are **not first-delivery hero full frames**.

### Product onboarding

Use header `Alta de productos`. Separate the long page into two sequential visual panels without changing behavior: `Crear categoría` (category name, field builder, pending field list) and `Crear producto activo` (category, SKU, name, `Precio de catálogo (Bs)`, opening whole-unit stock, dynamic attributes). Shared examples: `Agregar campo`, `Crear categoría`, `Crear producto`. Show category loading, no-category explanation (`Creá una categoría para habilitar el alta de productos.`), validation, pending, success, and failure as annotated component compositions. Do not invent a wizard, saved draft, bulk import, or centavo-facing field.

### Backup and restore

Use header `Copia y restauración` and two bordered regions. Backup: `Elegir destino de la copia`, pending `Creando copia…`, persisted summary with path/date/size/schema as secondary facts. Restore: `Elegir archivo de respaldo`, prepared candidate summary, invalid/expired/unavailable/error recovery, and native-file-dialog handoff annotation. Native dialog appearance is not redesigned. Final restore uses the shared destructive modal titled `Restaurar datos locales`, consequence `Esta acción reemplazará los datos locales actuales`, existing acknowledgement, `Volver`, and destructive `Restaurar datos`. Do not imply cloud backup or automatic scheduling.

## 9. Future mockup/Figma frame inventory

No frames are produced by this change. After separate human authorization, the first designer delivery is bounded as follows.

### 9.1 Required full frames

| ID | Size | Full frame | Required content/state |
| --- | --- | --- | --- |
| F01 | 1200×800 | Sales/POS | Populated hero, persistent shell, search results, cart, payment split, total, actions. |
| F02 | 960×640 | Sales/POS | Same populated facts with one-column reflow and sticky total/action annotation. |
| F03 | 1200×800 | Confirmed sale | Persisted identity, items, payments, total, `Nueva venta`. |
| F04 | 960×640 | Confirmed sale | Labeled-row reflow and scroll behavior. |
| F05 | 1200×800 | Inventory | Selected product, physical count, projection, low/out alerts. |
| F06 | 960×640 | Inventory | Stacked operation/alerts with minimum-size behavior. |
| F07 | 1200×800 | Catalog | Active/archived master list and populated product editor. |
| F08 | 960×640 | Catalog | Stacked master/detail and sticky identity annotation. |
| F09 | 1200×800 | History list | Date filter, populated bounded list, overflow notice. |
| F10 | 960×640 | History list | Responsive labeled rows and wrapped filters. |
| F11 | 1200×800 | History detail/return | Persisted original facts, correction history, open return. |
| F12 | 960×640 | History detail/return | Stacked detail, unavailable historical text, open return. |
| F13 | 1200×800 | History cancellation | Detail context plus final destructive cancellation modal. |
| F14 | 960×640 | History cancellation | Minimum-size modal, focus/scroll annotation. |
| F15 | 1600×1200 minimum canvas | Component/state/token sheet | Shared primitives and annotated variants below; canvas may extend vertically. |

The shell is not a separate full frame because F01–F14 demonstrate it in every active context. F01/F02 are the shell reference frames.

### 9.2 Annotated variants on F15, not additional full frames

- Buttons: all five variants × idle/hover/focus/pressed/disabled/pending.
- Fields: text/search/date/select/checkbox/quantity/money × default/focus/invalid/disabled/pending.
- Data row: default/hover/focus/selected/archived/unavailable/stale.
- Badges: available/low/out/active/archived/stale/unavailable/confirmed/cancelled.
- Feedback: loading/initial/empty/no-results/success/advisory/validation/error/unavailable-retry/stale/pending.
- Sales: empty search, search loading, no results, invalid payment/quantity, stale price, command failure, pending confirmation.
- Inventory: stock-entry toggle variant, no selection, validation, pending, success, stale preview, failure.
- Catalog: loading, empty, field validation, pending, success, unavailable/retry, stale recovery.
- History: list loading/empty/error; return validation/pending/failure/reload; cancellation preparation/pending/failure/success evidence.
- Onboarding continuity: category loading/empty and dynamic field group.
- Backup continuity: prepared restore candidate and invalid/expired/unavailable feedback; modal appearance reuses F13/F14.
- Typography, spacing, grid, elevations, icons, wordmark direction, token values, and measured contrast annotations.

This split controls workload while preserving full responsive evidence for the four primary workflow groups and confirmed sale.

## 10. Accessibility and handoff contract

### 10.1 Keyboard, focus, and recovery

- Preserve landmarks, one H1 per main view, ordered headings, labels, fieldsets/legends, busy states, alerts, status messages, and field error associations.
- Tab order follows navigation, header, main task from top-left to bottom-right, then supporting/recovery actions. Do not make every static table cell focusable.
- A selected master row is a focusable existing selection control; detail content follows it logically.
- Focus ring: 3px copper (or tuned darker copper) outside a 2px surface gap, target 3:1 against adjacent colors. Never remove the platform outline unless this replacement is present.
- Validation focuses the first invalid control. Return selection/quantity and cancellation reason/acknowledgement preserve the existing correction focus recovery.
- Loading completion does not steal focus. Error recovery focuses the alert heading only when the submission/load failure requires immediate correction; retry completion leaves focus on retry or moves to the loaded region heading by annotation.
- Modal focus is contained; Escape/`Volver` dismisses; focus returns to the invoking control. Completion moves to refreshed persisted evidence.

### 10.2 Contrast, zoom, motion, and targets

- Normal text target: WCAG AA 4.5:1. Large text target: 3:1. Meaningful control boundaries, state icons, and focus indicators target 3:1 against adjacent colors.
- The designer must measure and annotate every semantic pair; this brief intentionally does not claim unmeasured compliance.
- Review at 100%, 200%, and browser/OS text enlargement. At 200% equivalent, content reflows into a usable single column; no two-dimensional page scroll for core tasks.
- Minimum target is 44×44 CSS px, including checkboxes and icon actions.
- Motion is optional and non-essential: 120–160ms color/opacity transitions only. With reduced motion, remove transforms/slides and show state immediately. No spinner is the sole loading cue; pair it with text.
- Do not rely on color: pair state with explicit Spanish text, icon, border/shape, and position.
- Windows high-contrast/forced-colors implementation is not designed here, but handoff annotations must avoid erased boundaries and state that native focus/control semantics remain necessary.

### 10.3 Semantic and live intent

| Change | Intent |
| --- | --- |
| Search/list loading | Affected region busy + polite `status`; no repeated announcements. |
| Empty/no results | Polite status after a user-triggered load/search; static instruction initially. |
| Save/confirm pending | Form/region busy; pending button label; duplicate activation unavailable. |
| Success/advisory | Polite status associated with operation. |
| Command failure/unavailable | Assertive alert once, followed by explicit recovery action. |
| Field validation | Error described by and associated with invalid field; focus first correction. |
| Stale price/data | Polite status plus explicit acknowledgement/reload; confirmation remains unavailable where existing flow requires it. |
| Modal | Accessible dialog name/description, modal semantics, contained focus, return focus. |

### 10.4 Editable handoff package annotations

Each full frame must annotate:

- reusable pattern name and semantic token names, not raw ad hoc values;
- logical focus order and focus destination after validation, retry, modal dismissal, and completion;
- scroll container, sticky region, and reading order;
- live intent (`status`, `alert`, `busy`, field error, dialog description);
- existing action priority and pending/disabled behavior;
- responsive transformation between paired frames;
- realistic Spanish UI copy and `Bs` examples;
- non-color cue for every status;
- font/icon source, version, exact files, license, redistribution/attribution, and replacement status;
- any unresolved contrast or licensing item marked `Requiere validación`, never silently approved.

### 10.5 Token naming and asset ownership

Use category/role/state naming such as `color.action.primary`, `color.feedback.error.surface`, `space.4`, `radius.control`, `size.control.default`, `type.numeric`, `elevation.modal`. Do not use screen-specific names such as `salesPurple` or value names such as `orange500` at the semantic layer. Raw palette aliases may exist beneath semantic styles for future theme replacement.

No font, icon, wordmark, or other visual dependency becomes a production dependency through this document. Future editable files must preserve license notices and identify who owns any newly drawn identity work. Unclear rights are a blocking handoff issue.

### 10.6 Designer acceptance checklist

- [ ] F01–F15 are present, editable, and named exactly or cross-referenced clearly.
- [ ] Paired 1200×800 and 960×640 frames preserve location, facts, action, and recovery.
- [ ] All visible UI labels and messages are Spanish; money uses `Bs` and comma decimals.
- [ ] Sales exposes product/SKU/stock/price/cart/payment/total/action distinctly.
- [ ] Inventory states `Stock bajo: 1` and `Sin stock: 0` without color-only meaning.
- [ ] Catalog and History remain scannable at 960×640 with logical reading order.
- [ ] Original sale facts, sale-time prices, correction history, and unavailable snapshots remain distinct.
- [ ] All required state families appear as full-frame context or F15 variants.
- [ ] Primary, secondary, tertiary, destructive, and icon actions are distinguishable in every state.
- [ ] Routine feedback is inline; restore and sale cancellation use the modal pattern only.
- [ ] Focus order/recovery, live intent, busy/disabled behavior, target sizes, reduced motion, and reflow are annotated.
- [ ] Every color pairing has a recorded measured ratio or an explicit unresolved/tuning label.
- [ ] Every font/icon/asset has source, version, rights, and replacement notes.
- [ ] No frame invents routes, roles, draft preservation, sorting, reports, printing, barcode, customer, cloud, or other behavior.

## 11. Decision log, impact, validation, and rollout boundary

### 11.1 Decision log

| Decision | Rationale | Rejected alternative |
| --- | --- | --- |
| Warm industrial, light-first direction | Fits workshop context while keeping long sessions readable. | Blue enterprise theme; neutral-only utility theme; dark-first workshop aesthetic. |
| Persistent 208/176px side navigation | Keeps all existing destinations and labels visible at both sizes. | Icon-only rail, hidden hamburger, top tabs. |
| Plum identity/action + copper focus | Distinguishes brand hierarchy from semantic states. | Using copper for every primary action or amber for focus. |
| Balanced 48px rows and 44px controls | Supports quick scanning and accessible targets. | Dense spreadsheet rows or oversized cards. |
| Inline routine feedback | Keeps context and avoids interruption. | Toast-first or modal-heavy feedback. |
| Modal only for restore/cancellation | Matches destructive consequence contract. | Confirmation dialogs for routine saves, returns, archive, or draft discard. |
| `Bs` presentation, centavo authority preserved | Removes operator conversion burden without changing backend contracts. | Raw centavo fields or floating-point business authority. |
| Barlow + IBM Plex Mono | Warm technical character and clear SKU/numeric distinction with open licenses. | Proprietary fonts or novelty display faces. |
| Full responsive primary frames; state variants on one sheet | Provides review evidence while bounding designer workload. | A separate full frame for every state. |

### 11.2 Data and interaction flow preserved

```text
Operator input in Spanish / Bs presentation
        ↓
Existing React screen and flow state
        ↓
Existing command adapter seam under src/commands
        ↓
Existing Tauri IPC → Rust authority → SQLite persistence
        ↓
Decoded observable success / advisory / stale / error state
        ↓
Inline visual pattern or persisted summary/detail
```

The visual layer formats `Bs` for entry/display but does not own money, stock, payment, eligibility, idempotency, or historical truth. Integer centavos remain authoritative below the presentation seam. Whole-unit quantities remain authoritative. Existing request IDs, retries, stale acknowledgements, and persisted evidence remain intact.

### 11.3 Artifact and file impact

| Surface | Impact in this authorized phase |
| --- | --- |
| `openspec/changes/define-frontend-ui-ux-visual-system/design.md` | This designer-ready planning brief. |
| Engram topic `sdd/define-frontend-ui-ux-visual-system/design` | Mirrored design artifact. |
| Frontend, command adapters, Tauri, Rust, database, tests, assets | Read-only and unchanged. |

No implementation module, interface, adapter, or seam is changed. A later visual-system module could provide leverage and locality across screens, but its implementation shape is intentionally not planned or authorized here.

### 11.4 Review validation—not implementation tests

- Visual review at exact 1200×800 and 960×640 frames.
- Palette contrast measurements for every documented pair and state.
- Keyboard-order walkthrough using numbered annotations.
- 200% reflow review and reduced-motion annotation review.
- Grayscale/non-color-cue review for every badge/banner/selection.
- Spanish terminology and `Bs` formatting review by the store stakeholder.
- Invariant review against current flows, safeguards, and persisted historical facts.
- Licensing review for fonts, icons, and identity ownership.

These are acceptance checks for a future design package, not test or implementation instructions.

### 11.5 Risks and mitigations

| Risk | Mitigation / approval question |
| --- | --- |
| Candidate colors miss contrast | Tune within the same role/family and record measured ratios; do not claim compliance now. |
| Side navigation consumes too much width at 960 | Review F02/F06/F08/F10/F12/F14 with 176px nav; reduce gaps before hiding labels. |
| Sticky bars obscure focused content | Annotate scroll padding and verify focused controls remain visible. |
| Master-detail stacking loses selection | Preserve selected identity in a sticky detail bar and logical order. |
| Modal adaptation changes behavior | Restrict it to specification-approved restore/cancellation confirmation and preserve acknowledgement/reason. |
| `Bs` fields imply contract changes | Explicitly annotate presentation-only formatting and centavo authority. |
| Open fonts/icons are packaged incorrectly | Require exact file/version/license inventory and offline packaging approval later. |
| Spanish copy drifts from local usage | Human reviewer confirms terminology before mockup authorization. |

### 11.6 Out of scope and stop gate

- No code, CSS, React change, Tauri change, Rust change, persistence change, or tests.
- No tasks, apply phase, implementation plan, production assets, mockups, prototype, or exported images.
- No URL routes, navigation persistence, accounts, roles, permissions, multi-window, or mobile behavior.
- No dark-theme frames or dark-theme behavior; only semantic token readiness.
- No reports, product images, barcode, customer accounts, printing, invoicing, discounts, tax, cloud, remote notifications, or payment gateway.
- No invented sorting, pagination, bulk editing, draft preservation, keyboard shortcuts, or alert polling.
- No changes to stale-price review, restore acknowledgement, correction eligibility, correction focus recovery, retry/reload recovery, historical facts, idempotency, whole-unit stock, or cent-based backend contracts.

**Stop status: DESIGN COMPLETE FOR HUMAN REVIEW.** Human approval is required before mockups, tasks, apply work, code, or assets.

## 12. Requirement and scenario coverage

Scenario IDs below follow specification order and cover all **22 requirements and 61 scenarios**. The cited design sections are the review evidence.

| Req. | Requirement | Scenarios covered (exact specification titles) | Design evidence |
| --- | --- | --- | --- |
| R1 | Preserve product and architecture boundary | S01 `Future designer receives a bounded brief`; S02 `Existing safeguards remain visible` | §§1, 6, 11.2, 11.6 |
| R2 | Persistent shell and navigation | S03 `Sales is the initial location`; S04 `Another top-level area is active`; S05 `Shell displays a stock exception cue` | §5, F01–F14 |
| R3 | Modern-industrial direction | S06 `Palette direction is reviewed`; S07 `Palette candidates are validated rather than frozen`; S08 `Accent use remains restrained` | §§1–2 |
| R4 | Semantic color and contrast | S09 `Text and controls meet contrast targets`; S10 `State is understandable without color`; S11 `Error and destructive roles are distinguishable` | §§2.1–2.3, 6.4–6.6, 10.2 |
| R5 | Light-first future-theme tokens | S12 `First delivery is light-first`; S13 `Future theme readiness is inspected` | §§2, 10.5; F15 |
| R6 | Spanish copy and money contracts | S14 `Sales price and total are shown to an operator`; S15 `Monetary input remains presentation-only`; S16 `Whole-unit stock remains unambiguous` | §§3.2, 6.2, 7, 11.2 |
| R7 | Density, typography, alignment | S17 `Numeric facts are compared quickly`; S18 `Density is reviewed at both reference sizes`; S19 `Surfaces communicate hierarchy` | §§3–4, 7; paired frames |
| R8 | Button/action hierarchy | S20 `Action priority is legible`; S21 `Button states are reviewable`; S22 `Icon actions remain operable` | §§3.3, 6.1, 7.8; F15 |
| R9 | Form controls and feedback | S23 `A valid form is scanned`; S24 `A field is invalid`; S25 `A form is pending or unavailable` | §§6.2, 6.5, 7, 10.1 |
| R10 | Data/table/master-detail | S26 `Product and sales records are scanned`; S27 `Master-detail is shown at the minimum size`; S28 `Reading order differs from visual placement` | §§6.3, 7.4–7.6, 10.1 |
| R11 | Reusable status/recovery | S29 `A collection is loading or empty`; S30 `An operation succeeds or needs advisory attention`; S31 `An operation fails or becomes unavailable`; S32 `A record is stale` | §6.5; state variants on F15 |
| R12 | Modal-only destructive confirmation | S33 `Routine feedback stays inline`; S34 `Restore or cancellation requires destructive confirmation`; S35 `Reversible or ordinary actions are not over-confirmed` | §§6.1, 6.5–6.6, 7.7, 8 |
| R13 | Responsive desktop behavior | S36 `Starting window remains operational`; S37 `Minimum window remains usable`; S38 `Scroll and sticky decisions are explicit` | §§5.2, 7, 9.1, 10.4 |
| R14 | Accessibility annotations | S39 `Operator uses the keyboard`; S40 `Validation or modal completion restores focus`; S41 `Async feedback is announced appropriately`; S42 `Zoom, motion, and contrast are reviewed` | §§5.2, 6, 10 |
| R15 | Sales/POS and shell states | S43 `Populated Sales is reviewed`; S44 `Sales has a stale price or invalid payment`; S45 `Sale confirmation succeeds` | §§7.1–7.2; F01–F04; F15 |
| R16 | Inventory | S46 `Inventory operation is reviewed`; S47 `Inventory exceptions are prioritized` | §7.3; F05–F06; F15 |
| R17 | Catalog maintenance | S48 `Active and archived records are compared`; S49 `Dynamic form validation or stale recovery occurs` | §7.4; F07–F08; F15 |
| R18 | History/detail/corrections | S50 `History list and detail are scanned`; S51 `Return correction is open`; S52 `Cancellation correction is open` | §§7.5–7.7; F09–F14; F15 |
| R19 | Onboarding and backup continuity | S53 `Product onboarding uses shared form patterns`; S54 `Backup and restore uses shared recovery patterns` | §8; F15 |
| R20 | Component/state/token sheet | S55 `Reviewer traces a screen to reusable patterns`; S56 `State coverage is checked`; S57 `Token intent is checked` | §§2–6, 9.2; F15 |
| R21 | Typography/icons/wordmark/licensing | S58 `External designer can prepare an editable package`; S59 `Unlicensed dependency is proposed` | §3, §§10.4–10.5 |
| R22 | Testable design review contract | S60 `Human reviewer evaluates planning completeness`; S61 `Implementation authority is checked` | §§9–12, stop gate |

**Coverage result:** 22/22 requirements and 61/61 scenarios are explicitly mapped. Note that S29–S61 numbering above follows uninterrupted specification order; no scenario is omitted or duplicated.