# Change Proposal: Confirm Sale

## Intent

Deliver the first usable end-to-end point-of-sale slice for the offline Windows desktop application. A store operator will be able to find seeded active products, build a cart, record cash and/or QR payment, and confirm a sale through the complete React → Tauri → Rust → SQLite path.

This slice establishes the architectural and transactional foundation for later inventory and sales workflows while replacing no product-management workflow yet.

## Problem and Outcome

The current repository documents the product and architecture but does not yet provide an executable sales flow. The first slice must prove that the selected local modular-monolith architecture can preserve the core sales invariants across every layer without relying on network services.

The intended outcome is a keyboard-friendly sales flow that:

- searches all seeded active products from one entry point;
- builds a cart using positive whole-unit quantities;
- prefills each line with the product's configured minimum sale price while allowing a higher negotiated price;
- records cash, QR, or mixed payments;
- confirms the sale exactly once, atomically updates stock, and returns the persisted result; and
- remains fully functional offline.

## Scope

### In Scope

- Establish the minimum React, Tauri, Rust, and SQLite structure required for this vertical slice.
- Seed active categories/products, configured minimum sale prices, and stock balances so the flow is usable without product-management screens.
- Provide global product search across the seeded active catalog, including the product information needed to select an item and assess available stock and minimum price.
- Provide a draft cart with add/remove behavior, positive whole-unit quantities, and editable negotiated unit prices prefilled from each product's configured minimum.
- Generate and retain one UUID `request_id` when confirmation begins; repeated clicks or retries for the same sale intent reuse that identifier.
- Support payment lines for:
  - cash, including amount applied, amount tendered, and change given;
  - QR, including amount applied; and
  - mixed cash and QR payments.
- Validate authoritative sales rules in Rust, including active products, positive whole-unit quantities, negotiated price at or above the current minimum, sufficient stock, payment equality, and cash tender/change consistency.
- Persist the sale header, sale lines, payment lines, stock balance decrements, and immutable inventory movements in one SQLite transaction owned by `ConfirmSaleUseCase`.
- Enforce request-id uniqueness in SQLite and return the already-persisted sale when the same request ID is retried, without duplicating lines, payments, movements, or stock deductions.
- Return and display a persisted sale summary containing the sale identity, request ID, status, timestamp, line items, negotiated prices, quantities, payment breakdown, total in Bs, and resulting confirmation outcome.
- Store monetary values as integer centavos and inventory quantities as integers.
- Define verification for the observable happy paths and rejection paths, while acknowledging that executable test runners are not currently configured.

### Out of Scope

- Category or product creation, editing, archiving, and other product-management UI.
- Stock entry, manual adjustment, returns, sale cancellation, reports, backup, and restore workflows.
- Licensing, subscription validation, expiry behavior, or any write-operation enforcement based on license state.
- User accounts, roles, operator attribution, customer accounts, invoicing, payment gateways, barcode hardware, cloud services, synchronization, or multi-store support.
- Excel import or migration of real store data; this slice uses seeded data only.
- Fractional quantities or products sold by weight, length, or volume.

## Business and Integrity Rules

- Only active products can be added and confirmed.
- Every sale quantity is a positive whole number, and stock must never become negative.
- A negotiated unit price cannot be below the product's current configured minimum; the confirmed line retains both the negotiated value and the relevant minimum-price snapshot.
- The sum of payment amounts applied must equal the sale total exactly.
- Cash values are non-negative and must satisfy `amount_tendered - amount_applied = change_given`.
- A sale, its lines, payments, stock deductions, and immutable stock movements are committed together or not at all.
- Retrying the same retained request ID returns the original persisted sale and does not apply any inventory or payment effect again.
- Rust application/domain code and SQLite constraints are authoritative; React validation provides feedback but does not define the business rules.

## Affected Areas

| Area | Proposed impact |
| --- | --- |
| React + TypeScript UI | Global search, product results, draft cart, negotiated-price editing, payment entry, request-ID retention, confirmation states, and persisted summary presentation. |
| Tauri command boundary | Typed search and confirm-sale request/response contracts with stable error mapping. |
| Rust application layer | Catalog search orchestration and `ConfirmSaleUseCase`, including transaction ownership and idempotent retry behavior. |
| Rust domain layer | Money, quantity, sale line, payment, price-floor, stock, and confirmation invariants without UI or SQLite dependencies. |
| SQLite infrastructure | Initial schema/migrations, seed data, indexed catalog search support, repositories, unique `sales.request_id`, stock constraints, and atomic persistence. |
| Verification | Layer-level and end-to-end evidence for search, pricing, payments, stock, atomic rollback, idempotency, and persisted summary behavior once runners exist. |
| OpenSpec planning | Later specs, design, and tasks must preserve this boundary and forecast work against the 350-line review budget. |

## Dependencies and Constraints

- The implementation must follow the local modular-monolith dependency direction documented in `docs/ARCHITECTURE.md`.
- Normal operation must not require internet access.
- SQLite foreign keys must be enabled for every connection, with database constraints acting as a second line of defense.
- Repository transaction methods receive the application-owned transaction context and must not independently commit.
- The repository currently has no executable frontend, Rust, or end-to-end test runner configuration. Planning must describe intended evidence without claiming tests are currently runnable.
- Delivery uses `ask-on-risk`; the tasks phase must surface a decision before apply if the forecast exceeds the 350 changed-line review budget or otherwise recommends chained delivery.

## Risks and Mitigations

| Risk | Impact | Mitigation |
| --- | --- | --- |
| The first vertical slice also requires project scaffolding and may exceed the 350-line review budget. | Review quality may degrade or apply may be blocked pending a delivery decision. | Keep scaffolding minimal, split work into reviewable vertical units, forecast changed lines in tasks, and use the configured ask-on-risk gate before apply. |
| UI retries accidentally generate new request IDs. | Duplicate sales and stock deductions. | Create the UUID once per sale intent, retain it through pending/error states, and verify retries use the same value. |
| Application-only idempotency checks race or are bypassed. | Duplicate persisted effects. | Protect `sales.request_id` with a SQLite unique constraint and resolve conflicts by returning the existing persisted sale inside the transaction flow. |
| Stock is checked before, rather than decremented conditionally inside, the transaction. | Overselling or inconsistent balances. | Use a conditional atomic decrement and roll back the entire transaction when any line cannot be fulfilled. |
| Business rules drift into React or repository code. | Different behavior across entry points and weak testability. | Keep rule authority in Rust domain/application code; use UI checks only for immediate feedback and SQLite constraints for defense in depth. |
| Money is represented as floating point across the command boundary. | Rounding errors can violate payment equality. | Use integer centavos end to end and format Bs only for display. |
| Retrying after an ambiguous commit returns an incomplete summary. | The operator cannot tell whether confirmation succeeded. | Persist all summary source data atomically and reconstruct the response from persisted records for both new and duplicate requests. |
| Seed assumptions leak into future catalog behavior. | Later product-management work becomes constrained by fixture-specific design. | Treat seeds as bootstrap data through normal catalog tables and repositories, not as hard-coded UI products. |

## Rollback

- Before implementation, rollback consists of removing the `confirm-sale` OpenSpec change directory.
- During development, revert the slice as one bounded change and recreate the disposable local development database if unreleased migrations or seed data must be removed.
- After any real sale data exists, do not delete or rewrite persisted sales as rollback. Disable the new confirmation entry point, preserve the database, and use a forward migration or corrective release that maintains sales and inventory audit records.
- A failed confirmation must always roll back its SQLite transaction automatically; operational rollback must not be needed to repair partial sale effects.

## Success Criteria

1. An operator can search the seeded active catalog globally and add an available product to a cart.
2. Cart quantities accept positive whole units only, and each new line starts at the configured minimum sale price.
3. The operator can edit a negotiated price, but confirmation is rejected below the current minimum without persisting any sale or stock effect.
4. Cash-only, QR-only, and valid mixed-payment sales can be confirmed; applied payment amounts must exactly equal the sale total, and cash tender/change values must be consistent.
5. A sale with unavailable stock is rejected without persisting the sale, lines, payments, movements, or any partial stock deduction.
6. A successful multi-line confirmation atomically persists the sale, its lines and payments, one immutable negative inventory movement per line, and all stock balance decrements.
7. Repeating confirmation with the retained UUID request ID returns the same persisted sale and leaves sale counts and stock unchanged after the first success.
8. The UI displays a summary reconstructed from persisted data, including sale identity, request ID, timestamp, products, whole-unit quantities, negotiated prices, payment breakdown, and total in Bs.
9. The complete flow operates locally through React → Tauri → Rust → SQLite without licensing checks or network access.
10. Planning for implementation explicitly reports whether the slice fits the 350-line review budget and invokes the configured ask-on-risk decision when required.

## Source Alignment

- `docs/PRD.md`: product outcome, POS behavior, price floor, payments, whole-unit stock, auditability, and idempotency.
- `docs/ARCHITECTURE.md`: modular-monolith boundaries, `ConfirmSaleUseCase` transaction ownership, SQLite model, conditional stock decrement, and request-ID conflict behavior.
- `docs/TECHNICAL_RECOMMENDATION.md`: Tauri 2 + React + TypeScript + SQLite stack, integer money, transactional integrity, and offline operation.
- `docs/DEPLOYMENT_AND_LICENSING.md`: Windows/offline context and the explicit exclusion of licensing enforcement from v1.
