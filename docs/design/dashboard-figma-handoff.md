# Dashboard — Figma handoff

## Purpose and authority

Design the Dashboard as an operational balance: **today and current-month sales metrics orient the operator first; stock alerts receive prominent operational treatment; top products, payment distribution, and recent sales provide supporting context.**

This repository-backed handoff is authoritative for behavior, data meaning, accessibility, responsive shell constraints, states, and non-goals. External Figma files, screenshots, and HTML mockups are static visual references only. If a mockup conflicts with this document, this document wins.

> **Implementation boundary:** this is a design specification, not authorization to change React, CSS, commands, Rust, SQLite, reporting rules, or navigation behavior. Do not infer new capabilities from visual treatments.

## Quick design path

1. Design Frame A (desktop typical) and Frame C (compact typical) first.
2. Preserve the hierarchy and shell constraints below while using the existing visual system.
3. Submit A and C for visual-system approval before producing B, D, E, F, and G.
4. Validate the complete set against the acceptance checklist.

## 1. Approved direction

The Dashboard should answer three questions in order:

1. **What happened today, and how is the month going?** Lead with the two metric groups and label both time scopes explicitly.
2. **What needs attention now?** Give stock alerts a prominent, actionable region with a single route to filtered Inventory.
3. **What explains the operating picture?** Use current-month top products and payments plus all-time recent sales as supporting context.

Prominence does not authorize new data. Use grouping, scale, spacing, typography, and status treatment to establish hierarchy; do not create trends, comparisons, totals, counts, or summaries that are absent from the payload.

## 2. Authoritative snapshot contract

The Dashboard loads one report through one command. All sections succeed together from one read transaction or fail together; they are not independently fetched. The client also validates the complete response atomically, so one malformed section makes the report unavailable.

Today and current-month boundaries are calculated from the desktop's local calendar and sent as half-open UTC ranges. The UI should describe the human scopes as `Hoy` and `Este mes`; it must not expose or reinterpret technical UTC bounds.

| Region | Scope and maximum | Exact facts |
| --- | --- | --- |
| Today metrics | Today | Effective sales count, persisted total, net units out, cancelled sales count. |
| Month metrics | Current month | Effective sales count, persisted total, net units out, cancelled sales count. |
| Top products | Current month; maximum 5 | Product ID, latest non-null persisted name, latest non-null persisted SKU, and net units out. Name and SKU are resolved independently and each falls back to the current catalog value when its persisted snapshot is absent. |
| Payment distribution | Current month | Applied amount for `cash` and/or `qr` only. |
| Recent sales | All time; maximum 8; newest first | Sale ID, persisted timestamp, derived status (`confirmed` or `cancelled`), and persisted total. |
| Stock alerts | Current active catalog; maximum 8 | Product ID, current name, current SKU, current quantity, and classification (`low_stock` or `out_of_stock`). Active product and category are required. Out of stock is quantity 0; the current fixed low-stock rule is quantity 1. |

### Cancellation and return semantics

- An **effective sale** is a persisted confirmed sale without a cancellation record.
- Effective sale count and persisted total exclude cancelled sales. The total remains the persisted sale total; it is not recalculated from current catalog data.
- Cancelled sales count counts confirmed sales whose confirmation timestamp falls within the period and that currently have a cancellation record.
- Net units out excludes cancelled sales and subtracts recorded returned quantities from their original sale-line quantities.
- Returns affect net units, not the persisted sale total or the applied-payment aggregation shown here.
- Top products use positive net units from non-cancelled current-month sales. Ranking is by net units; the report returns at most five.
- Payment distribution sums applied cash/QR amounts for non-cancelled current-month sales. It does not introduce refunds or other payment methods.
- Recent sales includes confirmed sale records whether effective or cancelled. Cancellation changes the displayed status, not the persisted total shown in that row.
- Cancellation restoration details and inventory movements are not separate Dashboard facts.

### Bounded ordering

- Top products: net units descending, then applicable product name case-insensitively, then product ID.
- Recent sales: confirmation timestamp descending, then sale ID descending.
- Stock alerts: quantity ascending, then current product name case-insensitively, then product ID. This naturally places quantity-zero records before quantity-one records.

Do not display a collection total when only a bounded list is available. Five top-product rows, eight recent-sale rows, or eight alert rows do not prove that no additional records exist.

## 3. Existing states and behavior to preserve

| State or behavior | Required outcome |
| --- | --- |
| Initial load | Dashboard is the application's initial route. Show a loading state for the whole report and loading treatment in each region; do not present loading as empty. |
| Atomic failure | No stale or partial report is shown. Present a Dashboard-level failure with `Reintentar`, and keep each region visibly unavailable rather than empty. Retry reloads the entire snapshot. |
| Successful zero metrics | Render all four metric values as numeric zero for both periods. Zero is valid data, not an empty state. |
| Independent empty collections | After a successful snapshot, top products, payment distribution, recent sales, and stock alerts may each be empty independently. Show the matching contained empty message only for that collection. |
| Async safety | Ignore stale completions after a newer load and ignore completion after unmount. A late response must not replace the current state. |
| Stock action | `Ver en Inventario` opens Inventory with the stock-state filter set to alerts and refreshes the sidebar inventory cue. It is not a Dashboard filter or drilldown. |
| Focus on entry/load | Preserve current behavior: entering Dashboard, loading, success, and failure do not programmatically move focus. |

Existing Spanish state copy may be retained:

- Whole load: `Cargando dashboard…`
- Atomic failure: `No se pudo cargar el dashboard. Reintentar`
- Collection empties: `No hay productos vendidos en este período.`, `No hay pagos registrados en este período.`, `No hay ventas recientes.`, and `No hay alertas de stock.`

The designer may improve layout and visual emphasis, but must not imply that a failed report contains valid zero or empty data.

## 4. Non-negotiable shell constraints

| Contract | Requirement |
| --- | --- |
| Persistent navigation | Sidebar remains visible at all supported widths. Dashboard remains the active destination. |
| Desktop | `min-width: 961px`; sidebar is exactly `208px`. |
| Compact | `max-width: 960px`; sidebar is exactly `176px`. |
| Breakpoint | The transition is between 961px and 960px. Do not add an intermediate shell mode. |
| Scrolling | Shell content owns vertical scrolling and keeps visible scrollbars. Dashboard regions must not create nested vertical scroll areas. |
| Chrome | Do not add a topbar, hamburger control, overlay drawer, collapsed rail, or mobile replacement navigation. |

The current page padding is 24px on desktop and 20px on compact. A visual proposal should work within those bounds.

## 5. Required composition

### Page hierarchy

Use this logical and DOM order at both breakpoints:

1. Page header: `Dashboard` and `Resumen operativo de ventas e inventario.`
2. Period orientation: `Hoy` and `Este mes`.
3. Operational attention: `Alertas de stock` and `Ver en Inventario`.
4. Supporting context: `Productos más vendidos`, `Distribución de pagos`, and `Ventas recientes`.

Do not change semantic order only to achieve a desktop grid. A desktop composition may place the stock-alert panel beside or immediately below the period metrics, provided keyboard and reading order remain the sequence above.

### Period orientation

- Keep `Hoy` and `Este mes` as visible panel headings; never rely on proximity alone to convey scope.
- Each period contains the same four labels: `Ventas efectivas`, `Total efectivo`, `Unidades netas`, and `Ventas canceladas`.
- Treat numbers as the strongest content within each period. Use tabular numerals for counts and money.
- Do not add arrows, deltas, percentages, sparklines, targets, or comparison copy.

### Operational stock treatment

- Stock alerts should be the most prominent region after period orientation.
- Every record must preserve product ID, current name, SKU, current quantity, and a textual `Sin stock` or `Stock bajo` status.
- Keep `Ver en Inventario` visually attached to this region as its only action.
- Use urgency without turning the whole page into an alarm surface. Status cannot rely on color alone.
- Do not display “8 alerts” or another total based on row count; the payload is capped.

### Supporting context

- Label scopes in visible copy: `Productos más vendidos · Este mes`, `Distribución de pagos · Este mes`, and `Ventas recientes · Todas las fechas` (or equally explicit Spanish wording).
- Top products must keep product ID, applicable name, SKU, and net units.
- Payments must show only `Efectivo` and/or `QR` when present, with applied amounts in `Bs`.
- Recent sales must keep sale ID, persisted timestamp, textual status, and persisted total.
- Use semantic tables where comparison across columns matters. On compact layouts, tables may become labeled records while preserving every field and the same reading order.

## 6. Required Figma frames

All example values below are illustrative payload-valid content. They are not new product facts.

### Frame A — Desktop typical

**Viewport and shell:** 1440 × 900; 208px persistent sidebar; 24px content padding; shell content is the scroll owner; Dashboard active.

**Example content:**

- `Hoy`: 3 effective sales; `Bs 18.750,00`; 7 net units; 1 cancelled sale.
- `Este mes`: 42 effective sales; `Bs 286.430,00`; 96 net units; 3 cancelled sales.
- Stock: product #31 `Correa de distribución reforzada`, SKU `COR-DIST-031`, quantity 0, `Sin stock`; product #8 `Filtro de aire`, SKU `FLT-AIR-008`, quantity 1, `Stock bajo`.
- Top products: three rows, led by product #8 with 18 net units.
- Payments: `Efectivo — Bs 180.000,00`; `QR — Bs 106.430,00`.
- Recent sales: four rows with both `Confirmada` and `Cancelada` statuses.

**Proves:** approved hierarchy, explicit scopes, typical density, prominent stock treatment, and the one stock action without adding controls.

### Frame B — Desktop dense and bounded maximums

**Viewport and shell:** 1280 × 800; 208px sidebar; 24px content padding; show the visible scrollbar and enough page length to demonstrate shell-content scrolling.

**Example content:** five top products, both payment methods, eight recent sales, and eight stock alerts. Include long names/SKUs such as product #104 `Kit completo de distribución para motor diésel`, SKU `KIT-DISTR-DIESEL-2024-LARGO`; use large but valid amounts such as `Bs 9.876.543,21`. Include quantity-zero and quantity-one alerts and confirmed/cancelled recent rows.

**Proves:** all payload maximums remain readable, bounded rows are not mislabeled as totals, long content wraps, tables do not overlap, and no panel creates a nested vertical scrollbar.

### Frame C — Compact typical

**Viewport and shell:** 960 × 800; 176px persistent sidebar; 20px content padding; one-column Dashboard content; shell content owns scrolling.

**Example content:** use Frame A's data. Convert dense tables to labeled records where needed. Keep `Hoy` before `Este mes`, then stock alerts, top products, payments, and recent sales. Keep product IDs, SKUs, statuses, and amounts complete.

**Proves:** the compact breakpoint preserves the sidebar, logical order, full facts, 44px action target, wrapping, and one scroll surface without introducing a drawer or topbar.

### Frame D — Successful empty state

**Viewport and shell:** 1440 × 900 desktop shell. An optional compact inset may supplement, not replace, the desktop frame.

**Example content:** both metric panels show `0`, `Bs 0,00`, `0`, and `0`. All four collections show their independent successful empty messages. Keep section headings and explicit scopes visible. The stock panel has no invented alert count; show the empty treatment without implying a command failure.

**Proves:** zero metrics remain successful data, each collection owns its empty state, and successful emptiness is visually distinct from loading and failure.

### Frame E — Loading and atomic failure state board

**Viewport and shell:** two side-by-side 1280 × 800 desktop states, each with a 208px sidebar; include a compact annotation showing that the same behavior stacks at 960px.

**Loading example:** `Cargando dashboard…` plus contained loading treatments in every region. No zero values or empty messages.

**Failure example:** `No se pudo cargar el dashboard.` with a 44px-minimum `Reintentar` control. Every region is visibly unavailable; do not leave plausible data behind and do not present collection empty messages.

**Proves:** loading is not empty, failure is atomic, retry reloads the report, and no section suggests independent freshness.

### Frame F — Stock-alert action and navigation handoff

**Viewport and shell:** annotate a 1440 × 900 Dashboard state and the resulting Inventory state in the same persistent 208px shell.

**Example content:** Dashboard stock records include product #31 at quantity 0 and product #8 at quantity 1. Show keyboard focus on `Ver en Inventario`, then the resulting Inventory screen with the existing stock-state control set to the alerts filter and Inventory active in the sidebar.

**Proves:** the action opens filtered Inventory rather than filtering Dashboard, preserves one shell, and has a clear keyboard path. Annotate the Inventory focus destination as **pending product/engineering approval**; current behavior does not define or move focus to a destination after this action.

### Frame G — Accessibility and edge annotation board

**Viewport and shell:** annotated desktop and compact fragments; no standalone alternate product concept.

**Examples and annotations:**

- Exact focus ring: solid 3px `#C56845` with 2px offset.
- A 44 × 44px minimum `Reintentar` and `Ver en Inventario` target.
- `Sin stock`, `Stock bajo`, `Confirmada`, and `Cancelada` shown with text/icon or shape in addition to color.
- Long product name, SKU, timestamp, and money value wrapping without ellipsis or hidden content.
- Forced-colors mapping and reduced-motion note.
- Visible horizontal overflow only if a desktop semantic table cannot fit; prefer compact labeled records and never hide scrollbars.
- Reading/tab order annotation matching the hierarchy in Section 5.
- Live-region annotation limited to restrained loading/error feedback; repeated panels must not create a chorus of redundant announcements.

**Proves:** focus, targets, semantics, non-color status, wrapping, assistive-technology behavior, forced colors, reduced motion, and overflow are specified rather than left to visual inference.

## 7. Accessibility and focus contract

- Use one page `h1`, `Dashboard`, and associate the main region with it.
- Use named semantic sections/panels. Use semantic tables for columnar comparison or complete labeled records when compact.
- Preserve table captions or equivalent accessible collection names.
- Controls require accessible names and at least 44 × 44px targets.
- Visible keyboard focus is exactly a solid 3px `#C56845` outline with a 2px offset. In forced-colors mode, allow the system highlight color.
- Status meaning must include text, not color alone.
- Respect `prefers-reduced-motion`; do not require animation to understand loading or navigation.
- Support forced colors with visible borders and focus.
- Keep scrollbars visible. Do not make wheel, touch, or pointer use the only way to reach content.
- Allow full wrapping for authoritative names, SKUs, timestamps, labels, and values. Do not silently truncate with ellipsis.
- Keep tab order aligned with DOM/reading order: sidebar navigation, page content, retry when present, and the stock action in its semantic position. Read-only tables/records do not become tab stops merely for styling.
- Use live regions sparingly. Loading and failure feedback may announce state, but avoid redundant simultaneous announcements from every visual placeholder.
- Do not move focus on Dashboard entry, load start, success, or failure; current behavior does not do so.
- Any proposed Inventory focus destination after `Ver en Inventario` must be annotated **pending**, not represented as existing behavior.

## 8. Prohibited or invented capabilities

Do not show, imply, or reserve controls for:

- filters, custom date ranges, or period selectors;
- trends, comparisons, deltas, percentages, forecasts, goals, or targets;
- profit, margin, cost, tax, discount, or average-ticket reporting;
- customers, operators, users, branches, stores, or channels;
- configurable stock thresholds;
- manual refresh, auto refresh, “last updated,” freshness timestamps, or auto polling;
- drilldowns, clickable metric cards, clickable table rows, or product/sale detail links;
- export, print, download, or share;
- payment methods beyond cash and QR;
- cloud, sync, connectivity, or service status;
- totals or counts inferred from bounded collection lengths;
- pagination, “view all,” carousels, or hidden overflow for the bounded collections;
- search fields or user-controlled sorting;
- product editing, stock editing, or inventory adjustment from Dashboard;
- sale cancellation, return, or modification actions;
- controls to dismiss, confirm, acknowledge, or otherwise mutate stock alerts.

## 9. Prototype boundary

Figma prototypes and optional static HTML are disposable visual references. They must not become runtime dependencies.

Do not use or introduce in React/Tauri production:

- Tailwind CDN;
- Google Fonts;
- Material Symbols or any icon/font CDN;
- inline global event handlers;
- hidden scrollbars;
- runtime prototype scripts;
- imports, embeds, assets, or package dependencies required only by a mockup.

Use the repository's existing system fonts and visual tokens when implementation is separately authorized. This handoff contains no implementation code.

## 10. Acceptance checklist

### Facts and scope

- [ ] Today and current-month metrics show exactly the four contracted facts.
- [ ] Top products show product ID, applicable name/SKU, and net units; no more than five examples are implied.
- [ ] Payments show current-month applied cash/QR amounts only.
- [ ] Recent sales preserve all-time scope, sale ID, timestamp, status, and persisted total; no more than eight are implied.
- [ ] Stock alerts preserve active-catalog identity, quantity, and textual low/out-of-stock status; no more than eight are implied.
- [ ] Cancellation and return semantics are not contradicted by labels or visual summaries.
- [ ] Every temporal scope is visible in UI copy.

### Hierarchy and states

- [ ] Today/month orientation comes first, stock alerts receive prominent operational treatment, and contextual sections remain secondary.
- [ ] Loading is not presented as zero or empty.
- [ ] Atomic failure removes plausible report data and includes `Reintentar`.
- [ ] Successful zero metrics remain visible.
- [ ] Each successful empty collection has its own contained state.
- [ ] The design does not imply independent section fetching or freshness.

### Desktop, compact, and scrolling

- [ ] Desktop uses the persistent 208px sidebar at 961px and wider.
- [ ] Compact uses the persistent 176px sidebar at 960px and narrower.
- [ ] No topbar, drawer, collapsed navigation, or intermediate breakpoint is introduced.
- [ ] Shell content owns vertical scrolling; panels do not add nested vertical scrollbars.
- [ ] Long names, SKUs, timestamps, and values wrap without loss or overlap.
- [ ] Scrollbars remain visible.

### Accessibility and action integrity

- [ ] One `h1`, named panels, and semantic tables or labeled records are specified.
- [ ] Focus is a 3px `#C56845` outline with a 2px offset.
- [ ] Controls meet the 44px minimum target.
- [ ] Statuses do not rely on color alone.
- [ ] Reduced motion and forced colors are annotated.
- [ ] Reading and tab order match the semantic hierarchy.
- [ ] Live-region use is restrained.
- [ ] No Dashboard entry/load focus movement is invented.
- [ ] `Ver en Inventario` opens filtered Inventory; any destination focus proposal is marked pending.

### Non-goals and prototype hygiene

- [ ] No prohibited capability from Section 8 appears or is implied.
- [ ] No counts are inferred from capped list lengths.
- [ ] No mockup-only library, CDN, script, font, icon system, or hidden-scrollbar behavior is proposed for production.
- [ ] External mockups are treated as visual references, not behavioral authority.

## 11. Designer delivery instructions

Deliver in two review packages:

1. **Visual-system approval:** Frames A and C.
2. **Complete state coverage after approval:** Frames B, D, E, F, and G.

Each frame package must contain:

- `screen.png` — required;
- `code.html` — optional, static reference only.

`DESIGN.md` is not authoritative and must not be required for review or implementation. Put behavioral and accessibility annotations in the Figma frame/board and keep them consistent with this handoff.

## 12. Future implementation candidate surfaces — not authorized now

A later, separately approved visual implementation should remain bounded to Dashboard presentation:

| Candidate surface | Allowed future responsibility |
| --- | --- |
| `src/ui/dashboard/dashboard-screen.ts` | Dashboard composition, semantic presentation, state rendering, and existing stock-action placement. |
| `src/ui/dashboard/dashboard-screen.mounted.test.ts` | Rendered hierarchy, state, action, and accessibility behavior affected by the approved design. |
| `src/ui/dashboard/dashboard-flow.ts` and `src/ui/dashboard/dashboard-flow.test.ts` | Only if a later approved behavior change truly affects the existing load state machine or stale-response contract; a visual redesign alone should not touch them. |
| `src/ui/styles.css` | Dashboard-scoped selectors only, such as `[data-ui-dashboard]`, its named descendants, and breakpoint-specific Dashboard presentation. Do not use or modify Sales-scoped styling for this work. |

No Rust, SQLite, Tauri command, response decoder, reporting scope, shell/navigation flow, Inventory flow, or Sales flow change is part of this handoff. If a later approved design requires new behavior or data, stop and obtain a separate product and implementation decision before expanding those surfaces.
