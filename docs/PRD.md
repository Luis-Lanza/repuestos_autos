# Offline Inventory and Point-of-Sale — PRD

## 1. Product outcome

Deliver a Windows desktop application for one auto-parts store and one shared computer. It replaces separate Excel files with a single local source of truth for inventory, sales, stock alerts, and sales reporting.

The application is offline-first: its normal operation must not require internet access.

## Delivery status

This PRD remains the target product scope; its requirements are not all delivered.

| Area | Current status |
| --- | --- |
| Catalog onboarding and maintenance, fixed-price POS, and operational inventory | Delivered. |
| Backup and restore | Implemented and supported by Fedora evidence; Windows task 4.1 evidence remains deferred. |
| Sales history | Delivered as bounded, read-only calendar browsing with persisted sale details. |
| Returns, cancellations, and reports | Planned requirements; no current Rust commands/application layer or UI implementation. |

## 2. Problem

The store maintains inventory and sales in multiple category-specific Excel files. Staff must manually find and update the appropriate file after every sale. This prevents reliable global product search, makes stockouts easy to miss, and leaves sales history fragmented.

## 3. Users and operating context

| Topic | Requirement |
| --- | --- |
| Store | One physical branch. |
| Device | One shared Windows computer. |
| Users | Several people use the same computer; no user accounts, roles, or per-user attribution in v1. |
| Currency | Bolivian boliviano (`Bs`). |
| Connectivity | Core functionality works offline. |

## 4. Goals and success criteria

| Goal | Success criterion |
| --- | --- |
| Fast product discovery | Staff can find a known product from the global search in under 10 seconds. |
| Reliable stock | Every confirmed inventory event updates stock and leaves an audit record. |
| Prevent stockouts | Products at one unit or zero units appear immediately in the stock-alert view. |
| Accurate sales records | Each confirmed sale retains its products, catalog price at sale time, payment breakdown, total, date, and time. |

## 5. Functional requirements

### 5.1 Categories and configurable product data

- Create, edit, archive, and list categories.
- Each category can define its own additional product fields. Example: a `Belts` category can define length, width, and number of ribs.
- A category field has a label, data type (text, number, or predefined option), and required/optional setting.
- Category-specific field values are shown in the product form and included in global search.
- A product belongs to one category and has common fields: internal code/SKU, name, active status, current stock, catalog price, and category-specific fields.
- Initial categories, products, prices, and stock are entered manually. Excel import is out of scope.

### 5.2 Product search

- Provide one global search entry point across all active categories and products.
- Search by SKU, product name, category name, and configured category-specific field values.
- Results show product name, SKU, category, available stock, and current catalog price.
- Clearly distinguish low-stock and out-of-stock products.

### 5.3 Inventory and audit trail

- Record stock entries by quantity only; purchase costs and suppliers are out of scope.
- A stock adjustment is available for loss, damaged products, or physical-count differences.
- Each stock entry, sale, return, adjustment, and cancellation creates an immutable audit/movement record with date and time.
- Adjustments and cancellations require a reason. Audit-capable records reserve nullable `operator_id` metadata for future attribution, but v1 does not identify the person who performed the operation.
- Stock may not become negative.
- All inventory quantities are positive whole units. Fractional quantities and products sold by weight, length, or volume are out of scope.

### 5.4 Low-stock alerts

- All products use a minimum-stock threshold of **1 unit**.
- A product is low stock at 1 unit and out of stock at 0 units.
- Show an in-app/local alert counter and a dedicated list of low-stock and out-of-stock products.
- Recalculate alert status immediately after any inventory event.
- Remote notifications (email, WhatsApp, push) are out of scope because normal operation is offline.

### 5.5 Point of sale

- Create a draft sale, search and add products, and enter quantities. Each line displays the product's current catalog price; operators cannot edit sale-line prices.
- Catalog management can update a product's catalog price for future sales. A catalog-price update never changes a confirmed sale.
- At confirmation, the backend resolves the authoritative current catalog price for every product and stores that price as the historical sale-line snapshot.
- Confirming a sale stores the sale, its line items, sale-time catalog prices, total in Bs, date/time, and stock movements atomically.
- When confirmation begins, the UI creates and retains a UUID request ID for that sale intent. Every retry or repeated click uses that same ID; retrying it returns the already-created sale rather than creating a duplicate.
- A draft sale can be discarded without affecting stock.
- **Planned; not currently implemented:** a confirmed sale can be cancelled only with a required reason. Cancellation remains visible in history and creates reversing stock movements; it does not delete the original sale.

### 5.6 Payments

- Record payment lines in cash and/or QR.
- A sale can be paid entirely in cash, entirely by QR, or with a mixed payment.
- For cash payments, the operator enters the tendered cash; the system derives the amount applied to the sale and change from the cart total after any QR amount.
- Each payment persists its applied amount. Cash payments additionally persist tendered amount and derived change, so the amount received may exceed the amount applied.
- The derived cash amount applied plus QR amounts applied must equal the sale total before confirmation.
- Payment recording is informational only; QR payment-gateway integration and invoicing are out of scope.

### 5.7 Returns

**Planned; not currently implemented.**

- Register a return only against its original confirmed sale while that sale has not been cancelled.
- For each return line, accepted quantity must be a positive whole number and must not exceed the remaining returnable quantity: original sold quantity minus quantities already returned for that same sale line.
- A return records the product quantities returned, date/time, and the resulting stock movement.
- A return restores only its accepted quantity to inventory and does not alter or delete the original sale.
- If a sale with prior returns is cancelled, cancellation restores only the still-unreturned quantity for each sale line. Returns and cancellation must never restore the same sold unit twice.

### 5.8 Sales history and reports

**Sales history is implemented. Reports remain planned.**

- Provide a bounded, read-only sales history filtered by an inclusive calendar-date range. The list shows persisted date/time, status, total, line count, payment count, and payment methods, newest first.
- Selecting a sale loads its persisted product, quantity, sale-time catalog-price, payment-breakdown, and total details without using current catalog values to fill missing historical snapshots.
- **Planned; not currently implemented:** provide reports for a selected date range, by product, and by category.
- Planned report values use Bs and exclude cancelled sales from effective sales totals while preserving them in audit/history views.

### 5.9 Backup and restore

**Implemented with Fedora evidence; Windows task 4.1 evidence remains deferred.**

- Allow the operator to create a local backup to a USB drive or external disk.
- Allow restoration from a selected backup file after explicit confirmation.
- Validate the selected backup before overwriting local data.

## 6. Business rules

| Rule | Expected behavior |
| --- | --- |
| Fixed sale price | A sale line uses the product's authoritative current catalog price resolved by the backend at confirmation; an operator cannot negotiate or edit that price in checkout. |
| Historical price | A sale line retains its catalog price at sale time even if catalog management changes the product price later. |
| Payment integrity | The system derives cash applied and change from the cart total, tendered cash, and any QR amount. The derived cash amount plus QR amounts applied exactly equals the sale total. |
| Stock integrity | Confirming a multi-line sale applies every stock change or none. |
| Corrections | Returns, adjustments, and cancellations create compensating records; a return is capped at each sale line’s remaining returnable quantity, is prohibited for cancelled sales, and cancellation restores only still-unreturned units. Historical records are never silently edited or deleted. |
| Product availability | Archived products remain in history but cannot be sold. |

## 7. Non-functional requirements

- **Offline reliability:** Catalog, search, inventory, POS, history, reports, and backup work without internet.
- **Performance:** Initial global-search results appear within 1 second for up to 20,000 products on the target computer.
- **Data integrity:** A shutdown or failure cannot leave a partially confirmed sale or inconsistent stock.
- **Usability:** Primary sales operations are keyboard-friendly for counter use.
- **Distribution:** The application is installed and runs on Windows.

## 8. Explicitly out of scope for v1

- Tax invoices, fiscal integration, accounting, payment-terminal integration, and QR gateway integration.
- User accounts, roles, and identifying the actor in audit entries.
- Supplier management, purchase orders, and purchase-cost tracking.
- Excel import, multi-store operation, multi-device synchronization, cloud collaboration, and remote notifications.
- Customer accounts, credit, product images, barcode hardware integration, and advanced reporting beyond the three defined report dimensions.
- Subscription billing, license validation, and blocking the app due to non-payment.

## 9. Acceptance checklist

- [ ] Categories can define required and optional product fields, and their values are searchable.
- [ ] Staff can create products manually and search them across all categories.
- [ ] A sale cannot confirm with negative stock; its backend-resolved catalog price is stored as the sale-line snapshot.
- [ ] Cash, QR, and mixed payments—including operator-entered cash tendered and system-derived applied amount and change—are validated and stored correctly.
- [ ] Retrying a sale confirmation with the same request ID returns the existing sale and does not deduct stock again.
- [ ] Stock entries, adjustments, sales, returns, and cancellations are timestamped and auditable.
- [ ] A return cannot exceed the original sale-line quantity less prior returns, cannot target a cancelled sale, and cannot cause stock to be restored twice when a sale is cancelled.
- [ ] Low-stock status appears at one unit and out-of-stock status at zero.
- [ ] Sales history is available by calendar-date range with persisted sale details.
- [ ] Reports by date range, product, and category are available.
- [ ] Backups can be created and restored locally with validation and confirmation.
