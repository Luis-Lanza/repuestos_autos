# Sales confirmed result — Figma handoff

## Purpose

Design the post-checkout `Venta confirmada` state as a centered, receipt-like completion page inside the existing Sales shell.

The target should have the visual concentration of a modal without using modal behavior. It remains a page: the sidebar stays visible, the document owns scrolling, and there is no backdrop, focus trap, close button, or hidden completed draft underneath.

This handoff defines presentation and interaction only. It does not authorize changes to pricing, stock, payments, persistence, idempotency, IPC, Rust, SQLite, or historical data projection.

## 1. Product decision

| Topic | Decision |
| --- | --- |
| Pattern | Centered receipt-style confirmation page, not a dialog. |
| Context | Keep the persistent application shell and `Ventas` navigation state visible. |
| Data | Render only authoritative persisted sale facts. Never read current catalog data to complete the result. |
| Mutability | Entire result is read-only. |
| Actions | `Nueva venta` is the only action. |
| Scrolling | Normal page scrolling; no nested vertical scroll regions. |
| Large sales | The result grows naturally and must support an unbounded number of lines. |
| Next-sale focus | After `Nueva venta`, move focus to the catalog search field. |

## 2. Design goals

1. Confirm unmistakably that the transaction was saved.
2. Let the operator verify the persisted items, payments, change, and total quickly.
3. Reduce the excessive whitespace and administrative-table feeling of the current view.
4. Keep the primary continuation, `Nueva venta`, obvious and close to the total.
5. Support repeated counter sales without adding receipt, print, refund, or customer workflows.
6. Preserve complete historical facts at desktop and compact widths.

## 3. Required Figma frames

### Frame A — Desktop typical

- Viewport: at least `1280px` wide.
- Three or four confirmed lines.
- Mixed cash and QR payment.
- Non-zero change.
- No unnecessary full-viewport-height panels.

### Frame B — Desktop long sale

- At least twelve confirmed lines.
- Long product names and SKUs.
- Large quantities and monetary values.
- Natural document scrolling.
- Payment rail remains easy to locate; it may be sticky at the block start on desktop.

### Frame C — Compact typical

- Viewport: `960px` or narrower.
- One-column result.
- Three or four lines shown as labeled stacked records.
- Payments, total, and action follow the lines in DOM order.

### Frame D — Compact long sale

- Many confirmed lines.
- Long names and SKUs.
- No horizontal scroll or truncated historical facts.
- One document scroll surface only.

### Frame E — Edge facts

Include examples of:

- `Cambio: Bs 0,00`;
- cash plus QR;
- QR-only payment;
- `Fecha no disponible`;
- long names and SKUs;
- large totals;
- an unavailable or empty payment collection represented truthfully, without inventing a payment method.

## 4. Information architecture

Use the same logical order at every breakpoint.

1. Success header.
2. Sale identity.
3. Confirmed items.
4. Confirmed payments.
5. Persisted total.
6. `Nueva venta`.

Do not rearrange the DOM merely to create desktop columns.

### Success header

Required copy:

- Title: `Venta confirmada`
- Description: `La venta quedó guardada y estos datos son de solo lectura.`

Requirements:

- Include a success indicator that does not rely on color alone.
- Keep the title as the page `h1`.
- Avoid a tall banner that spans the entire viewport width.
- The header belongs inside the centered receipt surface.

### Sale identity

Required labels and facts:

- `Venta` → `Venta #N`
- `Fecha y hora` → persisted formatted value

Present identity as a compact metadata row or small inset block. It must not become a separate full-width panel with excessive vertical space.

### Confirmed items

Every line must preserve:

- item ID;
- persisted product name;
- persisted SKU;
- quantity;
- charged unit price;
- persisted subtotal.

#### Desktop recommendation

Use a dense but readable receipt/table treatment. To reduce six-column pressure:

- product name is the primary line identity;
- item ID and SKU may appear as secondary metadata under the name;
- quantity, charged unit price, and subtotal remain aligned numeric facts;
- subtotal is the strongest value in each row.

Do not hide item ID or SKU. Combining them visually with the product cell is allowed; removing them is not.

#### Compact recommendation

Convert each line into a labeled record:

- product name;
- `ID artículo`;
- `SKU`;
- `Cantidad`;
- `Precio unitario`;
- `Subtotal`.

Names may wrap. SKU and item ID must remain complete. Monetary values should remain non-wrapping when possible.

### Confirmed payments

Preserve the distinct persisted facts:

- `Efectivo aplicado`;
- `Efectivo recibido`;
- `Cambio`;
- `Pago QR`.

Do not collapse these into a generic `Efectivo` row. `Cambio: Bs 0,00` remains meaningful and must not be hidden.

If both cash and QR exist, show all applicable facts in one payment summary. Do not infer or recalculate values in the design.

### Persisted total

Required label:

- `Total persistido`

Requirements:

- It is the strongest monetary value on the page.
- Use tabular numerals.
- Visually associate it with the payment summary and `Nueva venta`.
- Do not rename it to tax-inclusive, estimated, current, or draft total.

## 5. Desktop composition (`min-width: 961px`)

Keep the existing application shell:

- sidebar: `208px`;
- Sales remains the active destination;
- main content padding: `24px`.

Receipt surface:

- centered horizontally;
- inline size: `min(1040px, 100%)`;
- natural block size;
- elevated white surface;
- `1px` border;
- `10px` radius;
- `24px` padding;
- `20–24px` primary gaps.

Recommended internal grid:

```css
grid-template-columns: minmax(0, 1fr) minmax(280px, 320px);
gap: 24px;
```

- Main column: identity and confirmed items.
- Rail: payments, persisted total, and `Nueva venta`.
- The rail may be sticky at the block start for long receipts.
- The page scrolls; the item list must not introduce a second vertical scrollbar.

For short sales, the surface should end shortly after its content instead of stretching to the viewport bottom.

## 6. Compact composition (`max-width: 960px`)

Keep existing shell contracts:

- sidebar: `176px`;
- page padding: `20px`;
- receipt padding: `16px`.

Composition:

```css
grid-template-columns: minmax(0, 1fr);
```

Order:

1. success header;
2. sale identity;
3. confirmed item records;
4. confirmed payments;
5. persisted total;
6. `Nueva venta`.

Requirements:

- no horizontal table scroll;
- no sticky payment rail;
- no inner item-list scrollbar;
- no fixed-height receipt surface;
- long content uses the main document scroll.

## 7. Controls and interaction

| Control | Role | Result |
| --- | --- | --- |
| `Nueva venta` | Primary button, at least `48px` high | Clears the completed flow, starts a fresh Sales draft, reloads active catalog page 1, and focuses catalog search. |
| Sidebar destinations | Existing navigation buttons | Navigate normally and unmount the current Sales result. |

There are no other controls in the confirmed result.

Do not include:

- close `×`;
- `Volver`;
- editable fields;
- item removal;
- print;
- share;
- receipt actions;
- refund or return;
- customer assignment;
- discounts;
- taxes;
- request UUID;
- redundant `status` or `outcome` labels;
- historical minimum/list price snapshots unless separately approved.

## 8. Focus and accessibility

### Entry focus

When the confirmed page replaces checkout:

- focus the `Venta confirmada` heading or a static result container with `tabindex="-1"`;
- announce the success title before the detailed receipt;
- do not attempt to restore focus to the removed `Revisar y cobrar` trigger.

### New-sale focus

After `Nueva venta`:

- reset the prior draft, payments, errors, persisted summary, and request identity;
- load active catalog page 1;
- focus `Buscar en el catálogo` once the fresh Sales screen is ready.

### General contracts

- One page `h1`.
- Success is communicated by text and iconography, not color alone.
- Minimum target size: `44px`.
- Primary `Nueva venta` height: at least `48px`.
- Visible focus: solid `3px` copper outline with `2px` offset.
- Preserve reduced-motion and forced-colors support.
- Use semantic tables or labeled records, not visually aligned anonymous `div` collections.
- Long names wrap; no silent ellipsis for authoritative facts.
- SKU uses the mono type style.
- Money uses tabular numerals.

Because this is a page, do not add `role="dialog"`, `aria-modal`, a focus trap, Escape handling, or a backdrop.

## 9. Observable and edge states

### Typical success

- Valid sale number and date/time.
- One or more confirmed lines.
- One or more persisted payment facts.
- Positive persisted total.

### Date fallback

Use the existing truthful fallback:

- `Fecha no disponible`

Do not invent or localize a missing timestamp from the current machine clock.

### Payment facts unavailable

If an impossible or legacy payload reaches presentation without payment rows:

- show a contained truthful empty state such as `Sin datos de pago` only if product/engineering approves that fallback;
- never infer cash, QR, or change from the total;
- the preferred contract remains a persisted non-empty payment collection.

### Many lines

- Do not assume a maximum transaction size.
- Allow natural page growth.
- Keep the payment/total rail locatable on desktop.
- Preserve every confirmed line and historical fact.

### Repeat sales

`Nueva venta` must never reuse:

- request ID;
- prior lines;
- prior payment values;
- prior errors;
- persisted confirmation details.

Late responses from the previous flow must not reopen or replace the new sale.

## 10. Visual tokens

Use the existing Sales visual system.

### Color

| Token | Value |
| --- | --- |
| Canvas | `#F4EFE6` |
| Surface | `#FFFCF7` |
| Elevated receipt | `#FFFFFF` |
| Subtle surface | `#EEE7DC` |
| Text | `#29252B` |
| Muted text | `#625A63` |
| Border | `#E4DDD2` |
| Strong border | `#8D8177` |
| Primary action | `#3D3042` |
| Primary hover | `#312535` |
| Focus | `#C56845` |
| Success | `#52634D` |

### Typography

- UI: `"Segoe UI", Arial, sans-serif`.
- SKU: `"Cascadia Mono", Consolas, monospace`.
- Body: `15px / 22px / 400`.
- H1: `24px / 30px / 650`.
- H2: `18px / 24px / 650`.
- Label: `13px / 18px / 600`.
- Caption: `13px / 18px`.
- Money: tabular numerals.
- Persisted total: `24px / 30px / 650` or stronger only if the hierarchy remains balanced.

### Geometry

- Receipt radius: `10px`.
- Receipt shadow: `0 12px 32px rgb(41 37 43 / 18%)`.
- Desktop padding: `24px`.
- Compact padding: `16px`.
- Standard gap: `12px`.
- Section gap: `20–24px`.
- Minimum target: `44px`.
- Primary action: `48px` minimum height.

## 11. Acceptance checklist for design review

### Content

- [ ] Sale number and persisted date/time are visible.
- [ ] Every item keeps ID, name, SKU, quantity, charged unit price, and subtotal.
- [ ] Cash applied, tendered, change, and QR remain distinct when present.
- [ ] Persisted total is visually dominant.
- [ ] `Nueva venta` is the only result action.

### Layout

- [ ] Short desktop sales do not create a viewport-height empty panel.
- [ ] Long desktop sales use normal page scrolling.
- [ ] Compact has one column and no horizontal scrolling.
- [ ] Long names, SKUs, and monetary values do not overlap.
- [ ] Payment summary and total remain easy to locate.

### Interaction and accessibility

- [ ] The result is a page, not a modal.
- [ ] No backdrop, close icon, Back button, or focus trap is present.
- [ ] Success is not communicated by color alone.
- [ ] Focus entry and next-sale focus are represented in prototype notes.
- [ ] Targets and visible focus meet the existing system contracts.

### Scope

- [ ] No new product capabilities are implied.
- [ ] No mutable draft or current catalog data appears in the persisted result.
- [ ] Desktop and compact preserve the same logical information order.

## 12. Implementation seams and evidence

| Seam | Responsibility |
| --- | --- |
| `src/ui/sales/persisted-summary.ts` | Immutable authoritative response-to-presentation projection and confirmed-result view. |
| `src/ui/sales/sale-screen.ts` | Success transition, persisted snapshot ownership, `Nueva venta`, fresh browse, and focus orchestration. |
| `src/ui/sales/sale-flow.ts` | Confirmation state, request identity, and draft reset behavior; visual redesign must not alter its business rules. |
| `src/ui/visual-system/structure.ts` | Semantic panel and aligned-data primitives. |
| `src/ui/styles.css` | Sales-scoped responsive page and receipt styling. |
| `src/commands/confirm-sale.ts` | Persisted confirmation transport contract. |
| `data-ui-*` attributes | Stable presentation hooks for focused styling/tests. |

Primary evidence paths:

- `src/ui/sales/persisted-summary.ts`
- `src/ui/sales/persisted-summary.test.ts`
- `src/ui/sales/sale-screen.ts`
- `src/ui/sales/sale-screen.mounted.test.ts`
- `src/ui/sales/sale-flow.ts`
- `src/commands/confirm-sale.ts`
- `src-tauri/src/commands/confirm_sale.rs`
- `src-tauri/src/application/sales/application_contract.rs`
- `src-tauri/src/infrastructure/sqlite/sale_repository.rs`
- `docs/design/sales-pos-figma-handoff.md`

## 13. External UX references

- GOV.UK Design System, confirmation pages: use a confirmation page at the end of a transaction, include a reference, and explain the next step.
  - <https://design-system.service.gov.uk/patterns/confirmation-pages/>
- U.S. Web Design System, modal: modals disable page content and intentionally interrupt workflow; use them sparingly.
  - <https://designsystem.digital.gov/components/modal/>
- WAI-ARIA modal dialog pattern: modal semantics require focus containment and defined focus entry/return behavior, which this page pattern intentionally avoids.
  - <https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/>
- Shopify POS receipt management: completed payments transition into a dedicated success/result surface with explicit next actions.
  - <https://help.shopify.com/en/manual/sell-in-person/shopify-pos/receipt-management/managing-receipts>
