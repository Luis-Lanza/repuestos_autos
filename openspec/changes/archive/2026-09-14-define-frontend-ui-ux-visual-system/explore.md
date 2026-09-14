# Exploration: designer-ready UI/UX visual system brief

## Outcome

The current frontend is a functional, mostly unstyled React/Tauri desktop interface with six top-level screens and two important nested outcomes: confirmed-sale summary and sales-history detail/corrections. A visual brief should preserve all existing behavior and Tauri command seams while giving an external designer a coherent Windows desktop shell, high-density operational layouts, reusable feedback/state patterns, and accessible keyboard/focus behavior.

This exploration is planning-only. It does not select a final visual direction, palette, typeface, icon set, or implementation approach.

## Product and runtime context

- Offline-first inventory and point-of-sale application for one auto-parts store on one shared Windows computer.
- Current app navigation is React state, not URL routing. It always starts on Sales.
- Tauri config opens one 1200×800 window with minimum 960×640; no mobile or multi-window behavior is defined.
- The PRD prioritizes known-product discovery under 10 seconds, search results within one second at up to 20,000 products, reliable stock, and keyboard-friendly counter operation.
- Currency is Bolivian boliviano (`Bs`), but several editing forms expose raw centavos.
- Business behavior, flow transitions, Tauri IPC, Rust authority, persistence, and command payloads are out of scope for the visual work.

## Screen, route, and navigation inventory

There are no URL routes. `src/ui/app.ts` switches among stateful screens.

| User-visible area | Entry/navigation | Purpose | Primary UI modules | Flow and command adapters |
|---|---|---|---|---|
| Sales / POS (default) | App start; return target from all other top-level areas | Search active catalog, add/remove cart lines, set quantities, enter cash/QR payment, acknowledge stale catalog price, confirm or discard sale | `sales/sale-screen.ts`, `sales/catalog-result.ts` | `sales/sale-flow.ts`; `commands/catalog.ts`, `commands/confirm-sale.ts` |
| Sale confirmed summary | Replaces POS after successful confirmation | Show persisted sale identity, timestamp, products, payments, total, and begin a new sale | `sales/sale-screen.ts`, `sales/persisted-summary.ts` | `sales/sale-flow.ts`; persisted response from `commands/confirm-sale.ts` |
| Product onboarding | “Onboard product” in Sales navigation | Create configurable categories/fields, then create an active product with opening stock and category attributes | `onboarding/onboarding-screen.ts`, `onboarding/onboarding-form.ts` | Local screen state; `commands/onboarding.ts` |
| Inventory | “Inventory” in Sales navigation | Search/select product, record stock entry or physical count, preview resulting balance, and monitor low/out-of-stock alerts | `inventory/inventory-screen.ts` | `inventory/inventory-flow.ts`; `commands/catalog.ts`, `commands/inventory.ts` |
| Backup and restore | “Backup and restore” in Sales navigation | Select a destination and create a backup; validate a selected backup and explicitly confirm destructive restore | `backup/backup-screen.ts` | `backup/backup-flow.ts`; `commands/backup.ts` plus native file dialogs |
| Catalog maintenance | “Catalog maintenance” in Sales navigation | List category/product records, inspect metadata, edit category/product fields, archive, reactivate, and recover stale/unavailable records | `catalog/catalog-maintenance-screen.ts` | `catalog/catalog-maintenance-flow.ts`; `commands/catalog.ts` |
| Sales history list | “Sales history” in Sales navigation; auto-loads today | Filter by inclusive date range, show bounded results, disclose overflow, and open a persisted sale | `sales/history-screen.ts` | `sales/history-flow.ts`; `commands/sales-history.ts` |
| Sales history detail and corrections | Select a sale from history | Inspect original items/payment facts and correction history; begin item return or sale cancellation when eligible | `sales/history-screen.ts` | `sales/history-flow.ts`; `commands/sales-history.ts`, `commands/post-sale.ts` |
| Return workflow (inline in detail) | “Begin item return” | Select eligible original lines and quantities, validate, submit, recover, and reload persisted detail | `sales/history-screen.ts` | `sales/history-flow.ts`; `commands/post-sale.ts` |
| Cancellation workflow (inline in detail) | “Begin sale cancellation” | Enter reason, explicitly confirm inventory correction, submit, recover, and reload detail | `sales/history-screen.ts` | `sales/history-flow.ts`; `commands/post-sale.ts` |

### Current navigation shape

- The five cross-area navigation buttons exist only above the Sales screen.
- Inventory, Backup, Catalog, and Sales History get a separate plain “Sales” button from the app shell; Onboarding owns a “Back to sales” control.
- History detail has its own “Back to history” control.
- There is no persistent app identity, active-location indication, breadcrumb, global command area, alert badge, or navigation grouping.
- Moving between top-level screens unmounts the prior screen, so in-progress local state is not preserved.

## Existing visual system and primitives

| Topic | Current evidence |
|---|---|
| Shared styling | No CSS/SCSS files, design tokens, theme provider, classes, or shared visual primitive library. Browser defaults control nearly all layout and appearance. |
| Typography | No font asset or font declaration; platform/browser defaults apply. Monetary and tabular figures have no specialized treatment. |
| Assets and identity | No `public/` directory. Only Tauri `icon.ico` and `icon.png` exist; no validated logo, wordmark, illustrations, product imagery, or brand asset set. |
| Icons | No UI icon library and no visible icon usage. Actions are text-only. |
| Forms | Native labels, inputs, number/date controls, selects, checkboxes, fieldsets, legends, and buttons. Catalog editing has the strongest field-level error semantics. |
| Data display | Lists, paragraphs, definition lists, and nested sections only. There are no tables, data grids, cards, pagination controls, summary bars, or sticky regions. |
| Modals | None. Restore confirmation and post-sale corrections are inline; file selection uses native Tauri dialogs. |
| Notifications | Inline `role=status` and `role=alert` text. No toast/notification center, consistent banner treatment, persistence policy, or shared severity model. |
| Focus | Most controls rely on browser focus. Correction validation explicitly moves focus to an invalid return quantity/selection or cancellation field. No designed focus appearance is defined. |
| Target sizing | Only history correction controls explicitly set 44×44 minimum dimensions. Other controls use browser defaults. |
| Responsive assumptions | Desktop viewport meta exists, but no responsive CSS or breakpoint behavior exists. The practical contract is one resizable Windows window from 960×640 upward, initially 1200×800. |

## Observable state map

| Area | Initial / loading / empty | Populated / editing | Validation / focus | Pending | Success | Failure / unavailable | Destructive confirmation |
|---|---|---|---|---|---|---|---|
| Sales | Empty query, results, cart, and payment. Search has no visible loading state or explicit empty-result message. | Results list, cart lines, quantities, cash/QR fields. | Invalid quantity becomes a general alert. Stale price requires review/acknowledgement. Focus movement is not defined. | Confirm button disables and reads “Confirming…”. Search remains unguarded/undisclosed. | Full persisted confirmation summary, then “New sale”. | Search and confirmation errors share the alert area. No explicit catalog-unavailable layout. | “Discard draft” is immediate with no confirmation. Sale confirmation itself has no separate review/confirmation step. |
| Onboarding | Categories load after mount; no loading indicator and an empty category list leaves product creation unusable without explanation. | Category-field builder, pending field list, category form, dynamic product form. | Native required/min constraints plus one general feedback message; no field-level errors or focus routing. | Create category/product controls do not expose pending or prevent duplicate submission. | General status text after category/product creation. | Category load/create failures appear in the same status region, including raw error codes. | None. |
| Inventory | Alerts load silently; search has no loading or explicit empty state; no product selected initially. | Product results, selected-product operation, stock entry/physical count, projected balance, alert list. | Physical-count reason gates submit; malformed values largely reach general failure. No field-level focus treatment. | Confirm disables and reads “Saving…”. | Saved delta/balance status and refreshed alerts; advisory if preview became stale. | Search and operation failures use alert text; alert-load failure is silent. | Physical count can materially alter stock but has no explicit confirmation beyond “Confirm operation”. “New operation” immediately discards local intent. |
| Backup/restore | Idle explanatory page. Native picker cancellation is a state. | Backup summary or validated restore-candidate summary. | Restore checkbox is required before confirm. | Separate backup/restore pending labels and disabled controls. | Backup detail/status; restore-completed status. | Inline errors for storage, destination, invalid schema/file, expired token, database, or restore failure. | Restore clearly states replacement, requires acknowledgement checkbox, then confirm. It is inline rather than modal. |
| Catalog maintenance | Starts loading. Empty loaded record set has no explicit empty message. | Record list, detail editor, archived notice, dynamic attributes. | Field-level errors use `aria-invalid`, descriptions, and alerts; no explicit focus movement. | Record controls and form disable; form uses `aria-busy`; saving label changes. | Status notice after edit; lifecycle mutation uses feedback text. | Unavailable/retry and stale-record reload recovery are explicit. | Archive is immediate and visually equivalent to normal actions; no confirmation. Reactivation is also immediate. |
| Sales history list | Auto-loads today with visible loading. Explicit empty result state. | Date filters, sale results, bounded-result notice. | Browser date input only; invalid range surfaces as general load error. | Load button disables and main is busy. | Ready list of sale summaries. | Alert plus Retry. | None. |
| History detail | Visible loading; error state can leave only Back control and alert. | Persisted identity, lifecycle, total, lines, payments, returns, cancellation. “Unavailable” substitutes missing historical SKU/name. | Return and cancellation have deterministic validation, alerts, 44×44 controls, and focus routing to correction targets. | Correction form is busy and submit disables. | Successful correction triggers detail reload; resulting persisted history is the success evidence. | Inline correction error can offer “Reload sale detail”. | Cancellation requires reason and acknowledgement checkbox. Return requires explicit line selection but no final review dialog. |

### State-system gaps for the visual brief

The designer should define reusable visual patterns for loading, empty, success, warning/advisory, validation, error, unavailable/retry, pending/disabled, stale data, destructive confirmation, and keyboard focus. These states exist inconsistently today and must remain semantically distinguishable without relying on color alone.

## Usability, consistency, and accessibility findings

### Highest-impact operational issues

1. **Sales lacks glanceable transaction structure.** Search results, cart, payment, errors, and final action are sequential unstyled lists; totals are not surfaced in the draft UI, and prices/stock are embedded in long text strings.
2. **Navigation is inconsistent and location is unclear.** The full menu disappears outside Sales, return labels vary, and there is no active destination or persistent low-stock signal.
3. **Catalog and onboarding are dense mixed-purpose forms.** Category creation, custom-field construction, and product creation share one long onboarding page. Catalog records, lifecycle actions, and dynamic editing are another long list with weak hierarchy.
4. **History detail is exceptionally dense.** Original facts, correction history, return controls, and cancellation controls all expand inline; irreversible and ordinary actions compete visually.
5. **Inventory alerts and operations compete.** Alerts are always appended below the operation instead of acting as a navigable priority queue or contextual summary.
6. **Raw operational units burden staff.** Several screens ask for centavos while other views show formatted Bs, increasing avoidable conversion and entry risk.

### Visual inconsistency

- Headings and native controls provide the only hierarchy; spacing, alignment, widths, action priority, status severity, and selected states are undefined.
- The same concepts use different labels and patterns: “Sales” versus “Back to sales”; general feedback can be status or alert; success may replace an entire screen or appear as one paragraph.
- Archive, restore, discard, cancellation, confirmation, and retry actions have no shared visual semantics.
- Dense records are rendered as prose lists instead of aligned columns, making SKU, stock, price, date, status, quantity, and totals hard to scan.

### Accessibility risks and strengths

**Strengths to preserve:** semantic `main`, headings, forms, labels, fieldsets, legends, buttons, busy states in key areas, live status/alert roles, catalog field error relationships, correction focus recovery, and 44×44 correction targets.

**Risks to address in the brief:**

- No authored focus-visible style, contrast system, zoom/reflow behavior, reduced-motion policy, or high-contrast/theme behavior.
- Most controls lack an intentional minimum target size; long inline records and action buttons may be difficult to scan and operate quickly.
- Some labels wrap multiple sibling inputs (Inventory quantity plus unlabeled-looking note/reason), weakening clear field grouping visually.
- Status and error semantics are inconsistent; Onboarding may announce failures as status, while Sales may treat non-error acknowledgement text as alert.
- Search has no announced pending/empty state, and several asynchronous actions do not disable duplicate input.
- Color cannot become the sole signifier for stock, lifecycle, validation, or destructive severity.
- A redesigned data table must preserve keyboard navigation, accessible names, logical reading order, and reflow at the 960×640 minimum.

## Demo-mockup priorities

| Priority | Screen/state | Why it demonstrates progress |
|---|---|---|
| P0 | Sales populated cart with search results, stock/price facts, payment, total, and primary confirmation | Core counter workflow and clearest test of density, hierarchy, keyboard speed, and visual identity. |
| P0 | Persistent desktop shell/navigation at 1200×800 and 960×640 | Establishes brand placement, information architecture, active location, alert visibility, and window behavior across all screens. |
| P0 | Inventory with selected product, physical-count variant, projected balance, and low/out-of-stock alerts | Shows operational states, risk hierarchy, numeric entry, and inventory priorities. |
| P1 | Sales history detail with correction history and an open return or cancellation flow | Tests dense audit information, destructive actions, validation, and persisted facts. |
| P1 | Catalog maintenance with populated record list and product editor | Tests master-detail structure, dynamic forms, archived status, and high-volume scanning. |
| P1 | Sale confirmed summary | Gives a credible success moment and receipt-like persisted record without inventing printing behavior. |
| P2 | Product onboarding with dynamic category fields | Validates complex form organization after the operational core is settled. |
| P2 | Backup/restore with prepared destructive confirmation | Validates system/maintenance tone and critical warning treatment. |

## High-level visual-direction candidates

These are intentionally distinct starting positions, not inferred preferences or final palettes.

### Candidate A — Industrial operations console

- **Character:** utilitarian, rugged, compact, high-contrast, workshop-adjacent without decorative imitation.
- **Structure:** persistent side rail, dense master-detail work areas, strong status strips, aligned numeric columns, prominent stock and transaction facts.
- **Best fit:** frequent staff use, keyboard operation, large catalogs, and quick exception scanning.
- **Tradeoffs:** can feel austere or technical; density and strong borders require disciplined hierarchy to avoid visual fatigue.

### Candidate B — Calm retail workspace

- **Character:** approachable, trustworthy, clean, with moderate density and clearer whitespace between task stages.
- **Structure:** persistent top/side navigation, task cards or panels, staged POS flow, readable summaries, restrained status accents.
- **Best fit:** mixed-experience staff, learnability, demos, and reducing anxiety around complex maintenance tasks.
- **Tradeoffs:** may consume more vertical space and slow expert scanning unless compact variants and keyboard pathways are explicitly designed.

### Candidate C — Data-first command center

- **Character:** information-dense, precise, spreadsheet-familiar but modernized, optimized for scanning and bulk-like operational review.
- **Structure:** compact navigation, sortable-looking data-grid patterns, split panes, sticky headers/actions, monospace or tabular treatment for SKU and money.
- **Best fit:** migration from Excel mental models, 20,000-product discovery, audit/history comparison, and inventory exceptions.
- **Tradeoffs:** higher cognitive load for occasional users; risks making checkout feel like administration and demands careful accessibility/reflow design.

## Product/design decisions required before proposal

The parent should ask these questions without presuming answers:

1. **Audience:** Who operates each area day to day—counter sellers, owner/manager, inventory staff, or the same people—and what is their computer proficiency?
2. **Brand personality:** Should the product feel industrial/rugged, calm/retail-friendly, data/technical, or something else? Which candidate is closest and what must be avoided?
3. **Density:** Should the default optimize expert speed and maximum visible rows, easier learning with moderate spacing, or offer compact/comfortable density modes?
4. **Theme:** Is light-only acceptable, is dark mode required, or should the system support both? Are Windows high-contrast expectations part of the deliverable?
5. **Screen priority:** Which 3–5 screens/states must appear in the first progress demo, and which single workflow is the visual “hero”?
6. **Logo/assets:** Is there an approved logo, wordmark, color reference, storefront signage, or existing brand guide? May the designer create identity assets, or only UI visuals? Is the current Tauri icon temporary?
7. **Display size:** What is the real shared computer’s Windows resolution, scaling percentage, monitor size, input devices, and typical maximized/windowed usage? Must 960×640 remain fully supported?
8. **Language:** Should the delivered interface remain English, switch to Spanish, or support both? Which terminology and locale formatting should the mockups use for Bs, dates, and numbers?
9. **Designer deliverable format:** Figma or another tool; editable source versus static screens; required component library/tokens; prototype depth; annotation/handoff detail; export formats; and ownership of fonts/icons/assets.
10. **Workflow specifics:** Should navigation preserve drafts when switching areas, should destructive actions use modal or inline confirmation, and may visual design format monetary input as Bs while preserving existing centavo-based business contracts internally?

## Minimum external-designer mock inventory

The minimum useful package covers these **responsive desktop screen groups plus a state/component sheet**:

1. Shared app shell at **1200×800**, Sales active, with persistent navigation and alert/location treatment.
2. Shared shell reflow at **960×640** with the same populated Sales content.
3. Sales/POS populated state: search results, low/out-of-stock distinctions, cart quantities/prices, cash+QR payment, total, primary/secondary actions.
4. Sales state variants: initial/empty search, search loading, no results, invalid quantity/payment, confirmation pending, stale-price acknowledgement, command failure, and disabled/focus-visible controls.
5. Sale confirmed summary with persisted products, payment breakdown, total, date/time, and New sale.
6. Inventory populated state with selected product and stock entry/physical count variants, projected balance, alerts, pending, success/advisory, validation, and failure.
7. Catalog maintenance master-detail state with active/archived records, dynamic product fields, field validation, pending, success, unavailable/retry, and stale-record recovery.
8. Sales history list plus detail/correction variants: loading, empty, populated, bounded notice, unavailable historical product text, return selection/validation/pending, cancellation confirmation, and correction failure/reload.
9. Backup/restore destructive state: prepared candidate, acknowledgement, confirm, pending, success, invalid/expired/unavailable failures, and native-dialog handoff annotation.
10. Product onboarding: category-field builder and dynamic product form with empty-category, loading, validation, pending, success, and failure variants.
11. Component/state sheet: typography hierarchy; spacing/layout grid; navigation; buttons/action priority; inputs/selects/date/checkboxes; table/list patterns; money/SKU/numeric alignment; stock/lifecycle badges; banners/live feedback; empty/loading skeleton or indicator; modal/inline confirmation; focus/disabled/hover/pressed/error/success states; and icon usage rules.

If budget forces a smaller first review, frames 1, 3, 4, 6, 7, and 8 are the minimum progress-demo set; Backup and Onboarding can follow after visual direction approval.

## Constraints for later phases

- Preserve the existing business outcomes and user-observable state transitions; visual simplification must not remove stale-price acknowledgement, idempotent pending behavior, restore acknowledgement, correction eligibility, persisted historical facts, retry/reload recovery, or focus restoration.
- Preserve the command adapter seam under `src/commands/` and do not redesign Tauri IPC as part of this change.
- Do not invent product images, barcode workflows, customer accounts, cloud state, printing, tax invoicing, or remote notifications.
- Treat the visual system as a shared interface across screens rather than independent page styling, so severity, action hierarchy, fields, tables, and feedback remain consistent.
- Stop before tasks/apply until a human selects or revises the visual direction and answers the decision gaps above.
