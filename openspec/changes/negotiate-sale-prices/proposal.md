# Negotiated sale prices with a list-price floor

## Intent

Adapt the offline POS to the normal Bolivian practice of agreeing a price at the counter without weakening sales, payment, stock, or historical-record integrity.

Each active product will have a **list price** and a **minimum sale price**. A draft sale line starts at the list price; the operator may set its final unit price above or below that reference, but never below the current minimum. The persisted sale records the final agreed price and the contemporaneous list/minimum facts.

This proposal is planning only. It does not authorize implementation.

## Current-state correction

The code already has partial but misleading terminology:

- `products.minimum_unit_price_centavos` currently holds the single price exposed as the catalog price.
- `sale_lines.negotiated_unit_price_centavos` and `minimum_unit_price_snapshot_centavos` already exist, but confirmation currently supplies the same backend-resolved amount for both.
- The React checkout shows that one price as read-only and the current confirmation contract treats price changes as stale-price acknowledgement.

The change separates these concepts rather than adding a parallel pricing path.

## Product and domain decisions

| Concept | Decision |
| --- | --- |
| List price | Positive monetary amount used as the default draft price and catalog/search reference. |
| Minimum sale price | Positive monetary amount that must be less than or equal to the product's list price. |
| Final agreed unit price | Positive operator-selected amount for one draft sale line. It may be below, equal to, or above the list price, but must be greater than or equal to the current minimum sale price. |
| Classification | Final price is a neutral business fact. No discount, surcharge, percentage, reason, approval, or operator attribution is added. |
| Historical facts | Confirmed lines retain the final agreed unit price plus list-price and minimum-price snapshots. Future catalog changes never alter them. |
| Monetary authority | Rust owns validation and totals using integer centavos. React provides input, immediate feedback, and display only. |

## Scope

### In scope

- Add a distinct list price to products while retaining a minimum sale price.
- Make product creation and catalog maintenance capture and validate both prices.
- Make catalog discovery expose list and minimum facts needed by checkout.
- Let an operator edit each draft line's **Precio de venta (Bs)** while the sale remains a draft; initialize it from the list price.
- Recalculate line subtotals, sale total, cash application, QR validation, and change from final agreed prices.
- At confirmation, reload the current product/list/minimum facts in the transaction and reject a final price below the current minimum.
- Persist immutable final/list/minimum snapshots with each confirmed sale line and return the final price in the persisted sale summary/history.
- Preserve request-ID idempotency: a same-ID retry returns the original sale; a changed price in a reused request is a conflict, not a reprice.
- Migrate existing products by setting their new list price equal to their existing stored price. Preserve existing sale-line values as their final and minimum snapshots; list snapshots for legacy historical lines must be explicitly represented as unavailable rather than fabricated.
- Amend `docs/PRD.md` from fixed-price checkout to the agreed-price model.
- Keep all new and modified operator-visible copy in Spanish and use `Bs` input/display formatting while preserving centavo IPC/domain values.

### UX contract

The Sales screen must extend the established industrial desktop visual system and the Sales mockups in the sibling worktree, especially the visible cart rows, monetary alignment, field treatment, feedback banners, focus outline, and 1200×800 / 960×640 layouts.

For every cart line, the operator can scan:

1. product identity and SKU;
2. **Precio de lista** as reference;
3. **Precio mínimo** as the non-negotiable limit;
4. editable **Precio de venta (Bs)**; and
5. calculated subtotal.

The amount field must be keyboard-operable, use comma-decimal `Bs` parsing, validate syntactic errors and below-minimum prices inline, associate the error with the field, and move focus to the first invalid price when confirmation is blocked. Pending confirmation locks price editing with the rest of the draft. No modal is required for ordinary price agreement.

A change to product pricing after a product entered the cart must not silently permit an invalid sale. Confirmation rechecks the current minimum. If it is now above the chosen final price, no record or stock change is persisted; the UI explains the new minimum and returns focus to that line's price field. A list-price-only change does not reclassify or overwrite a valid agreed price.

### Out of scope

- Reports, discount/recargo metrics, margins, purchase cost, suppliers, promotions, coupons, tax/invoicing, customer pricing, customer accounts, roles, approvals, or operator identification.
- Reasons or notes for prices below/above list.
- Editing a confirmed sale, retroactive repricing, or changing return/cancellation semantics.
- A separate price-negotiation screen, modal negotiation workflow, calculator, or external payment integration.
- Changing inventory quantities, stock movement rules, backup/restore behavior, multi-device behavior, or online features.

## Affected seams

| Seam | Required direction |
| --- | --- |
| SQLite schema/migrations | Add product list price and sale-line list-price snapshot. Backfill product list price from the current single stored price; define legacy sale-list snapshot availability explicitly. Preserve immutable confirmed lines. |
| Catalog domain/application | Validate positive centavo values and `minimum <= list`; stop naming a minimum as a catalog/list price. |
| Catalog command/TypeScript seam | Carry both prices in onboarding, maintenance, detail, and search contracts with strict runtime decoding. |
| Sales domain/application | Accept a proposed final unit price, verify it against the reloaded current minimum inside the owned transaction, calculate totals with it, and preserve idempotency. |
| Tauri command seam | Replace the fixed-price/stale-price confirmation contract with a bounded final-price/minimum-violation contract; return persisted snapshots. |
| React sales flow/screen | Keep final price draft state separate from product price snapshots; reset request identity on price edits; preserve stale-response, pending, and unmounted guards. |
| History/summary | Present the persisted final price as the charged historical amount. The proposal does not require exposing legacy-unavailable list snapshots as a new reporting view. |
| Tests | Cover migration, product-price validation, below-minimum rejection without partial effects, above/list/below-list valid cases, payment totals, idempotent retry/conflict, historical immutability, keyboard validation/focus, pending lock, and responsive rendered behavior. |

## Risks and mitigations

| Risk | Mitigation direction |
| --- | --- |
| Client bypasses a visible minimum. | Treat the Rust transaction as final authority; client checks are feedback only. |
| A catalog update invalidates an open agreement. | Re-read current minimum at confirmation and block atomically only when the chosen final amount is now below it. |
| Existing single-price data is misrepresented. | Backfill product list from the existing value; preserve legacy sale facts without inventing unavailable list snapshots. |
| Ambiguous naming causes incorrect future behavior. | Use list, minimum, and final agreed price consistently in documentation, contracts, tests, and UI. |
| Price edits break totals or payment validation. | Derive all UI display calculations from draft final prices; keep Rust authoritative for confirmed totals/payments. |
| Dense POS layout loses scanability at 960px. | Use the existing cart-row hierarchy and Field/Feedback patterns; verify keyboard order, focus, and reflow at both reference desktop sizes. |

## Success criteria

- A product cannot be created or updated with a non-positive price or a minimum above its list price.
- Adding a product to a draft uses its list price as final price by default.
- A seller can use a final price above, equal to, or below list, provided it is not below the current minimum.
- A below-minimum final price is visibly blocked in Spanish and is rejected again by the backend with no sale, payment, stock, or movement effects.
- Confirmed lines preserve final agreed, list, and minimum price facts; later catalog changes do not change historical sales.
- Cash, QR, and mixed payments remain exact against totals calculated from final agreed prices.
- Repeating an identical request ID returns the original persisted sale; reusing it for a changed final price fails safely.
- The POS remains keyboard-friendly, accessible, and visually consistent with the existing Sales mockups and visual system.

## Stop-before-implementation gate

This proposal requires human review. Do not create tasks, change source code, run migrations, or modify the PRD until the proposal is approved and an implementation scope is explicitly authorized.
