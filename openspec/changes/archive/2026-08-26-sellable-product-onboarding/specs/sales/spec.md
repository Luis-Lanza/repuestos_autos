# Delta for Sales

Retrospective delta: seeded-only removed; checkout/history unchanged.

## MODIFIED Requirements

### Requirement: Active Product Search and Cart

The system MUST globally search active catalog, including operator-created products, showing name, SKU, category, whole-unit stock, and price. Operators MUST add/remove available products without changing stock. Checkout MUST retain fixed-price/history rules.
(Previously: search/cart discovery were seeded-only.)

#### Scenario: Search and add an active product

- GIVEN the active catalog contains a seeded or operator-created product
- WHEN the operator searches by name, SKU, category, or configured field
- THEN it appears with price and can be added

#### Scenario: Archived or inactive products cannot be sold

- GIVEN a product is inactive
- WHEN the operator searches or confirms a cart containing it
- THEN it is unavailable; no sale/stock effect persists

#### Scenario: Discarding or removing a draft cart line

- GIVEN a product is in the draft cart
- WHEN its line is removed or the draft discarded
- THEN stock, sales, payments, and movements remain unchanged

#### Scenario: Sell an onboarded product under unchanged rules

- GIVEN an operator-created active product is in a valid cart
- WHEN the operator confirms the sale
- THEN checkout resolves price and history retains its snapshot

### Requirement: Confirm-Sale Scope Exclusions

Confirm-sale MUST remain active-catalog discovery, draft cart, confirmation, cash/QR recording, and atomic sales/stock persistence. It MUST NOT provide product management, stock entry/adjustment, returns/cancellation, reports, backup/restore, licensing, accounts/roles, customers, invoicing, gateways, barcodes, cloud, synchronization, multi-store support, or fractional quantities.
(Previously: confirm-sale discovery/catalog source were seeded-only.)

#### Scenario: Product-management workflows are separate

- GIVEN the operator uses confirm-sale
- WHEN product/category management is needed
- THEN no such operation exists; onboarding remains separate

#### Scenario: External and future workflows do not affect confirmation

- GIVEN the application is offline
- WHEN the operator confirms an in-scope cash, QR, or mixed-payment sale
- THEN confirmation needs no network, licensing, gateway, account, invoice, barcode, or synchronization
- AND returns, cancellations, adjustments, reports, and backup/restore are not performed
