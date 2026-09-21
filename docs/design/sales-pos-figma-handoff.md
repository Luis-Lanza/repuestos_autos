# Sales/POS — Figma handoff

## Purpose

This document describes the existing Sales/POS screen as implemented. It is a design handoff, not a product-change proposal. Measurements are code-derived; no rendered screenshot measurements were taken.

## 1. Viewport, layout, and breakpoints

### Desktop (`min-width: 961px`)

- Sidebar: `208px`; padding: `24px`.
- Main content: `padding: 24px`, `max-block-size: 100vh`, `overflow: auto`.
- Sales grid:

  ```css
  grid-template-columns: minmax(0, 1fr) minmax(260px, 320px);
  grid-template-rows: minmax(0, 1fr);
  gap: 20px;
  ```

- The `Sales summary` panel is sticky at the block start.
- Panels: `20px` padding, `1px` border, `8px` radius.

### Checkout desktop (`min-width: 961px`)

- Dialog width: `min(1040px, 100%)`.
- Maximum height: `calc(100vh - 48px)`.
- Columns: `minmax(0, 1fr)` plus `minmax(280px, 320px)`, with `24px` gap.
- The cart has its own vertical scroll; the dialog uses `overflow: hidden`.

### Compact (`max-width: 960px`)

- Sidebar: `176px`; horizontal padding: `16px`.
- Page padding: `20px`; panel padding: `16px`.
- Sales becomes one column:

  ```css
  grid-template-columns: minmax(0, 1fr);
  grid-template-rows: minmax(0, 1fr) max-content;
  ```

- The sales summary is no longer sticky.
- Search and result rows become one column.
- Cart product and `Remove` remain two columns; the inner controls become one column.
- Checkout width: `min(520px, 100%)`; block layout with vertical overflow and `16px` backdrop/dialog padding.

There are no breakpoints between `961px` and `1200px`.

## 2. Visual order

### Persistent shell

1. Sidebar identity: `Repuestos Autos`.
2. Sidebar navigation, with `Sales` active.
3. `main[aria-labelledby="sale-heading"]`.

### Sales draft

1. `Sales` heading.
2. Product catalog:
   1. Search form.
   2. Search feedback.
   3. Product list.
   4. Pagination.
3. Sales summary:
   1. `Units`.
   2. `Total`.
   3. `Review and checkout`.

### Checkout

1. Backdrop.
2. `Review and checkout` dialog.
3. Cart lines.
4. Payment rail: cash, QR payment, current total, and draft discard action.
5. Dialog actions: `Back` and `Confirm sale`.

### Confirmed sale

1. `Sale confirmed` heading.
2. Sale identification.
3. Confirmed items.
4. Payment summary.
5. Persisted total.
6. `New sale` as the only action.

## 3. Controls and contracts

### Global navigation

| Control | Role | Result |
| --- | --- | --- |
| `Sales` | Navigation button; `aria-current="page"` when active | Opens or retains Sales. |
| Other sidebar destinations | Navigation buttons | Change screen. |

The current Sales shell has no visible icons.

### Product catalog

| Control | Role | States | Result |
| --- | --- | --- | --- |
| `Search the catalog` | `input[type="search"]`, `searchbox` | Editable; disabled while confirmation is pending | Retains query and triggers search. |
| `Category` | `select`, `combobox` | Editable; disabled while confirmation is pending | Filters products by category. |
| `Search` | Secondary button | Disabled while confirmation is pending | Searches and resets pagination to page 1. |
| `Add` per product | Secondary button | Disabled when out of stock, already in cart, or confirmation is pending | Adds a product line. |
| `Previous` | Tertiary button | Disabled on first page, during search, or when no previous page exists | Goes to prior page. |
| `Next` | Tertiary button | Disabled on last page, during search, or when no next page exists | Goes to next page. |

Each result shows product name, mono SKU, category, list price in `Bs`, and a textual stock badge:

- `Available: N`
- `Low stock: 1`
- `Out of stock: 0`

Results use `ul` / `li`, not a table.

### Sales summary

| Control or data | Role | States | Result |
| --- | --- | --- | --- |
| `Review and checkout` | Primary button | Disabled with no lines or during confirmation | Opens checkout. |
| `Units` | Derived read-only text | — | Shows draft units. |
| `Total` | Derived read-only numeric text | — | Shows calculated draft total. |

`Review and checkout` exposes `aria-controls="checkout-dialog"` and `aria-expanded`.

### Checkout

| Control | Role | States | Result |
| --- | --- | --- | --- |
| `Back` | Secondary button | Disabled while confirmation is pending | Closes dialog and preserves draft. |
| `Quantity of {product}` | `input[type="number"]`, `spinbutton` | Disabled while pending; positive integer | Changes quantity. |
| `Sale price (Bs)` | Textbox | Disabled while pending; monetary validation | Changes final line price. |
| `Remove {product}` | Tertiary button with `aria-label` | Disabled while pending | Removes a draft line. |
| `Cash received` | Monetary textbox | Disabled while pending; supports `Bs` comma decimals | Sets cash amount. |
| `QR payment` | Monetary textbox | Disabled while pending; supports `Bs` comma decimals | Sets QR amount. |
| `Discard draft` | Tertiary button | Disabled while pending | Discards draft and restarts search. |
| `Confirm sale` | Primary button | Disabled with invalid or empty cart, and while pending | Submits sale once. |
| `New sale` | Primary button | Available only after confirmation | Restarts flow. |

## 4. Visual tokens

### Color

| Token | Value |
| --- | --- |
| Canvas | `#F4EFE6` |
| Surface | `#FFFCF7` |
| Surface subtle | `#EEE7DC` |
| Surface elevated | `#FFFFFF` |
| Text | `#29252B` |
| Muted text | `#625A63` |
| Border | `#E4DDD2` |
| Strong border | `#8D8177` |
| Primary action | `#3D3042` |
| Primary hover | `#312535` |
| Focus/accent | `#C56845` |
| Success | `#52634D` |
| Warning edge | `#C7923E` |
| Danger | `#A33F46` |
| Danger hover | `#873139` |
| Disabled text | `#817982` |

### Typography

- UI: `"Segoe UI", Arial, sans-serif`.
- SKU: `"Cascadia Mono", Consolas, monospace`.
- Body: `15px / 22px / 400`.
- H1: `24px / 30px / 650`.
- H2: `18px / 24px / 650`.
- H3: `16px / 22px / 600`.
- Label: `13px / 18px / 600`.
- Caption: `13px / 18px`.
- Money: `15px / 20px`, tabular numerals.
- Total: `24px / 30px / 650`, tabular numerals.
- SKU: mono, `13px / 18px`.

### Geometry and spacing

- Standard control: minimum `44px × 44px`.
- Prominent control: minimum `48px` height.
- Field/button radius: `6px`.
- Panel radius: `8px`.
- Modal radius: `10px`.
- Main page gap: `20px`.
- Form gap: `12px`.
- Sales summary gap: `16px`.
- Cart row gap and vertical padding: `12px`.
- Result row gap: `12px`; vertical padding: `8px`.
- Desktop dialog padding: `24px`.
- Dialog action gap: `12px`.
- Motion: `140ms`.
- Modal elevation: `0 12px 32px rgb(41 37 43 / 18%)`.

Money fields display a `Bs` prefix on a subtle surface and align values right. They remain presentation in `Bs`; native commands retain integer cents.

## 5. Observable states

### Search and catalog

- Initial: `Search for a product to begin.`
- Loading: `Searching products…`
- Empty: `No products found for “{query}”.`
- Error: `Could not search the local catalog.` The query remains visible.
- Pagination: `Page X of Y`.

### Cart and checkout

- Empty cart: `The cart is empty.` Confirmation stays disabled.
- Cart lines show product, SKU, list price, minimum price, quantity, final price, and subtotal.
- Invalid quantity: `Enter a whole quantity greater than zero.`
- Invalid sale price: `Enter a valid sale price in Bs, with up to two decimals.`
- Non-positive sale price: `The sale price must be greater than zero.`
- Invalid payment: `Enter a valid amount in Bs, with up to two decimals.`
- Payment error focus goes to Cash received first, then QR payment if cash is valid.

### Confirmation

- Pending: button label changes to `Confirming…`; main and related controls expose `aria-busy`; search, editing, removal, and duplicate submission are blocked.
- Normalized errors cover invalid request, invalid quantity/payment, missing or inactive product, insufficient stock, request-ID conflict, invalid minimum price, and persistence failure.
- Fallback error: `Could not confirm the sale. Try again.`
- Success replaces the draft with `Sale confirmed`.

### Confirmed summary

Read-only historical data: sale identity, date/time, product, SKU, quantity, historical unit price, subtotal, payments, change, and total. Only `New sale` is available.

## 6. Dialog and accessibility contracts

Checkout uses `CheckoutDialog` and `ConfirmationDialog`:

- `role="dialog"`, `aria-modal="true"`, `aria-labelledby`, and `aria-describedby`.
- Initial focus: first available final-price field.
- Focus trap for `Tab` and `Shift+Tab`.
- `Escape` and `Back` close only while not pending.
- Focus returns to the opener after close.
- Dialog remains open while confirmation is pending.

Other non-negotiable behavior:

- One `h1`: `Sales`; main is named with `aria-labelledby`.
- Panels are semantic sections with associated headings.
- Every field has a visible label. `Field` provides `id`, `aria-describedby`, and `aria-invalid`.
- Feedback uses `role="status"`; errors use `role="alert"`.
- Visible focus: solid `3px` copper outline with `2px` offset.
- Minimum interaction targets are `44px`.
- Reduced-motion and forced-colors rules are supported.
- Stock, errors, and success never rely only on color.
- Enter submits search.
- Late search or confirmation responses are ignored after a newer request, draft discard, or unmount.
- Do not change logical DOM order merely to redesign columns.
- The confirmed summary must keep persisted historical facts, not current catalog data.
- Do not add product capabilities such as printing, sharing, refunds, customers, discounts, or taxes without a separate product decision.

## 7. Implementation seams and evidence

| Seam | Purpose |
| --- | --- |
| `src/ui/visual-system/controls.ts` | `Action`, `Field`, `Feedback`, `Badge`, `IconAction` |
| `src/ui/visual-system/structure.ts` | `Panel`, `AlignedData` |
| `src/ui/visual-system/confirmation-dialog.ts` | Modal dialog, focus trap, Escape, focus restoration |
| `src/ui/visual-system/checkout-dialog.ts` | Routine checkout dialog variant |
| `src/ui/catalog/product-browser.ts` | Search, filter, result, state, and pagination UI |
| `src/ui/styles.css` | Tokens, controls, feedback, badges, panels, dialogs, and Sales layout |
| `data-ui-*` attributes | Styling contract between markup and CSS |
| `src/ui/sales/sale-flow.ts` | Validation, totals, prices, request identity, and state |
| `src/ui/sales/persisted-summary.ts` | Read-only persisted result projection |

Primary evidence paths:

- `src/ui/sales/sale-screen.ts`
- `src/ui/catalog/product-browser.ts`
- `src/ui/sales/persisted-summary.ts`
- `src/ui/sales/sale-flow.ts`
- `src/ui/app-shell.ts`
- `src/ui/styles.css`
- `src/ui/sales/sale-screen.mounted.test.ts`
- `openspec/changes/archive/2026-09-14-define-frontend-ui-ux-visual-system/design.md`
