# Define a designer-ready frontend visual system

## Intent

Create the proposal, specification, and design documents that together form a reviewable, designer-ready UI/UX visual-system brief for the existing React/Tauri desktop application. The completed planning brief will give an external designer enough product, workflow, visual, state, accessibility, and handoff context to produce future mockups for a coherent light-first operational interface without changing application behavior or authorizing implementation.

The proposed direction is a modern industrial workspace that feels distinctive but restrained, supports rapid scanning at an auto-parts counter, and avoids both conventional enterprise-blue styling and generic neutral-only palettes.

## Problem

The application supports core store operations, but its React UI is mostly browser-default. Navigation is only fully visible from Sales, screen hierarchy varies by area, dense operational facts are rendered as prose-like lists, and there is no shared visual language for actions, forms, data, stock state, feedback, focus, or destructive operations.

This creates avoidable scanning and orientation cost during time-sensitive work. Counter staff must distinguish products, stock, prices, payment facts, corrections, and failures quickly, while inventory and maintenance workflows must expose dense records without obscuring risk or status. Independent page styling would not solve the underlying inconsistency; the designer needs one shared system and one persistent shell.

## Users and operating context

- Several store operators share one Windows computer in one physical auto-parts store; v1 has no accounts, roles, or per-user attribution.
- Counter operators need fast, keyboard-friendly product discovery and checkout, including known-product discovery in under 10 seconds.
- Inventory and catalog operators need to scan stock exceptions, product records, and editable facts accurately.
- Owners or operators reviewing sales history need persisted transaction and correction facts to remain clear and auditable.
- The application is offline-first and opens at 1200×800 with a supported minimum window of 960×640.
- The interface and mockups use Spanish. Monetary values are displayed and entered in `Bs`; existing Rust and IPC money contracts remain integer centavos.

## Goals

1. Define a persistent desktop shell with clear identity, active location, and consistent access to existing top-level workflows.
2. Establish a reusable visual system for operational data, controls, action priority, feedback, stock/lifecycle states, focus, and destructive actions.
3. Make Sales/POS, Inventory, Catalog maintenance, and Sales History/detail/corrections easier to scan at balanced default density in both supported reference window sizes.
4. Complete the designer-ready brief through `design.md`, including a concrete future mockup inventory, state contract, palette hypothesis, constraints, and measurable review criteria.
5. Preserve all existing workflows, observable state transitions, accessibility semantics, and frontend-to-native architecture seams.

## Authorized planning sequence

1. This proposal establishes intent, scope, constraints, risks, and success criteria.
2. The next authorized phase is the OpenSpec specification, which will make the visual-system and designer-handoff requirements explicit and verifiable.
3. The following authorized phase is `design.md`, which will complete the designer-ready brief by resolving the planning structure, interaction/state guidance, visual-system decisions, and future mockup handoff contract.
4. A human reviews the completed proposal, specification, and design brief before any external mockup production or implementation planning proceeds.

Specification and design are authorized planning artifacts in this change. They do not authorize tasks, apply work, frontend implementation, code, or assets.

## Non-goals

- No React, CSS, asset, Tauri, Rust, database, or persistence implementation.
- No task plan, apply phase, frontend implementation, production asset creation, or code changes as part of this authorized planning sequence.
- No change to business rules, flow transitions, command payloads, Tauri IPC, Rust authority, transaction behavior, or SQLite persistence.
- No URL routing, multi-window behavior, draft-preservation policy, authentication, roles, or operator attribution.
- No product images, barcode workflow, customer accounts, cloud or multi-device features, printing, tax invoicing, payment-gateway integration, or remote notifications.
- No dark-theme mockups in the first delivery. Tokens should be structured so a future dark theme is possible, but dark-mode behavior is not designed or approved here.
- No requirement to retain the current Tauri icon. A designer may propose a simple wordmark, but broader brand-identity work is outside scope.

## Scope

### 1. Shared visual-system brief

The brief will define a coherent, light-first visual language covering:

- typography hierarchy and tabular treatment for SKU, quantities, dates, and money;
- color roles, spacing, layout grid, surfaces, borders, elevation, and icon-use guidance;
- persistent navigation, active-location treatment, app identity/optional wordmark, and low-stock visibility;
- buttons and action hierarchy; text, numeric, search, date, select, checkbox, and quantity controls;
- data-list, table-like, master-detail, summary, badge, banner, inline feedback, and confirmation patterns;
- initial, loading, empty, populated, validation, pending/disabled, success, warning/advisory, failure, unavailable/retry, stale-data, and focus-visible states;
- keyboard and focus behavior, target sizing, contrast, zoom/reflow, reduced motion, and non-color state cues;
- responsive desktop behavior between the 1200×800 starting window and 960×640 minimum.

The initial palette hypothesis is:

| Role candidate | Color |
| --- | --- |
| Mineral plum | `#3D3042` |
| Copper terracotta | `#C56845` |
| Industrial sage | `#71806A` |
| Warm ivory | `#F4EFE6` |
| Light stone | `#E4DDD2` |
| Ink | `#29252B` |
| Garnet | `#A33F46` |
| Mineral amber | `#C7923E` |

These colors are inputs to design validation, not pre-approved semantic assignments. The designer may tune or reject individual values while preserving the modern-industrial, uncommon-but-restrained direction and the prohibition on typical enterprise blues or a neutral-only result.

### 2. Future external-designer mockup deliverable

The specification and `design.md` will complete the brief for a future external-designer review package. After the planning brief passes human review and separate mockup production is authorized, that future package should show the persistent shell and each of these workflow groups at both 1200×800 and 960×640:

1. **Sales/POS:** product search results, stock and price facts, populated cart, quantities, cash/QR payment, total, and primary/secondary actions.
2. **Inventory:** selected product, stock-entry or physical-count operation, projected balance, and low/out-of-stock priorities.
3. **Catalog maintenance:** scannable record selection, product/category facts, editable dynamic fields, lifecycle state, and action hierarchy.
4. **Sales History:** bounded list and persisted sale detail, including correction history and an open return or cancellation state.

The future package should also include a component/state sheet or annotated variants demonstrating how the system handles the observable states relevant to those workflows. A separate dark mockup is not required. No external mockups are assumed to exist, and mockup availability is not a prerequisite for completing the specification or `design.md`.

### 3. Interaction and confirmation contract

- Routine validation, pending state, success, advisory, retry, and recoverable failure remain inline and associated with the relevant work area.
- Modal confirmation is reserved for truly destructive or irreversible actions. It must not be added to routine actions merely to create visual emphasis.
- Existing acknowledgements and safeguards remain visible, including stale-price review, restore acknowledgement, correction eligibility and validation, persisted correction evidence, and retry/reload recovery.
- Visual simplification must not erase or merge semantically distinct states, and color must never be the sole signifier.

### 4. Designer handoff annotations

The completed `design.md` brief will require the future external-designer package to identify:

- reusable patterns versus screen-specific compositions;
- intended action priority and destructive severity;
- Spanish labels and realistic `Bs` formatting/input examples;
- focus order, focus destination after validation, keyboard-operable controls, and live status/error intent;
- reflow decisions at each reference size, including which regions scroll or remain visible;
- token candidates and semantic color roles suitable for later implementation and future theme extension;
- any font, icon, or asset dependencies and their licensing assumptions.

## Constraints and invariants

- Preserve React screen flows and all user-observable initial, loading, empty, populated, validation, pending, success, failure, unavailable, stale, correction, and confirmation states.
- Preserve semantic landmarks, headings, forms, labels, fieldsets, live status/alert intent, busy states, error associations, keyboard operation, correction focus recovery, and accessible control names.
- Preserve the command adapter seam under `src/commands/`, Tauri IPC contracts, Rust business-rule authority, and SQLite persistence.
- Treat persisted sale details and sale-time prices as historical facts; do not substitute current catalog data or imply editable history.
- Keep stock and payment authority out of the visual layer. Inventory quantities remain positive whole units, and the visual system must not imply fractional stock.
- Display and input money as `Bs` without changing centavo-based Rust contracts or introducing floating-point authority.
- Keep the default density balanced: enough visible operational information for fast scanning without turning every workflow into a compact spreadsheet.
- Support the existing single-window desktop range; mobile, touch-first, and multi-window layouts are not required.

## Measurable acceptance

### Planning-brief acceptance before human handoff

The authorized proposal/specification/design sequence is accepted for human review when reviewers can verify all of the following:

- [ ] `proposal.md`, the visual-system specification, and `design.md` together define the persistent shell plus Sales/POS, Inventory, Catalog maintenance, and Sales History/detail/corrections at both 1200×800 and 960×640.
- [ ] The completed brief specifies Spanish interface copy and `Bs` display/input examples while preserving integer-centavo Rust and IPC contracts.
- [ ] The brief defines the modern-industrial direction, the palette as a hypothesis requiring design validation, semantic token candidates, balanced density, responsive behavior, and future dark-theme extensibility without requiring dark mockups.
- [ ] The brief distinguishes reusable visual-system patterns from screen compositions and covers loading, empty, validation, pending/disabled, success, warning/advisory, error, unavailable/retry, stale data, destructive confirmation, and focus-visible treatment.
- [ ] The brief preserves all existing business rules, safeguards, observable states, accessibility semantics, command seams, native authority, and persisted historical facts.
- [ ] The future mockup inventory, required annotations, licensing assumptions, review criteria, and handoff boundaries are sufficiently explicit for an external designer to begin after human approval without product behavior being invented.
- [ ] The planning artifacts do not contain tasks, apply instructions, frontend code, production assets, or implied implementation authority.

### Future mockup acceptance after separate authorization

Future external-designer mockups are not an output or prerequisite of the current proposal/specification/design sequence. When separately produced, they will be accepted only when reviewers can verify all of the following:

- [ ] One review package contains the persistent shell plus Sales/POS, Inventory, Catalog maintenance, and Sales History/detail/corrections at both 1200×800 and 960×640.
- [ ] Every frame uses Spanish interface copy and formats or accepts monetary values in `Bs`, with no mockup exposing centavos as the operator-facing unit.
- [ ] The visual direction is recognizably modern industrial, uses more than a neutral-only palette, and does not use conventional enterprise blue as the primary identity.
- [ ] The package includes reusable tokens or token candidates for color roles, typography, spacing, layout, surfaces, controls, and focus, prepared for future dark-theme extension without requiring dark mockups.
- [ ] Sales exposes product identity, SKU, stock, price, cart quantity, payment split, total, and the primary confirmation action as separately scannable facts.
- [ ] Inventory distinguishes low stock at one unit from out of stock at zero using text or iconography in addition to color.
- [ ] Catalog and History demonstrate aligned, scannable dense data and usable master-detail or equivalent hierarchy at the minimum window size.
- [ ] History detail keeps original persisted facts, correction history, eligibility, and an open return or cancellation workflow visually distinct.
- [ ] The state sheet or annotations cover loading, empty, validation, pending/disabled, success, warning/advisory, error, unavailable/retry, stale data, destructive confirmation, and focus-visible treatment.
- [ ] Focus visibility, keyboard operation, accessible naming intent, target sizing, contrast, zoom/reflow, and non-color cues are annotated sufficiently for later implementation review.
- [ ] Routine validation and status feedback are inline; modal treatment appears only where the represented action is truly destructive or irreversible.
- [ ] No frame introduces new business behavior, removes an existing safeguard, changes architecture contracts, or implies a runtime implementation has been approved.

## Affected areas and impact

| Area | Proposal impact |
| --- | --- |
| Store operations | Establishes clearer navigation, hierarchy, and status recognition for shared-computer workflows without changing policy or permissions. |
| Sales/POS | Defines the visual contract for rapid search, cart scanning, payment entry, total, confirmation, stale-price handling, and persisted success. |
| Inventory | Separates operation entry, projected outcomes, and stock exceptions while preserving whole-unit and audit expectations. |
| Catalog | Defines a consistent dense-data and dynamic-form hierarchy for active, archived, validation, pending, and recovery states. |
| Sales history/corrections | Organizes persisted facts, correction history, eligibility, validation, and destructive severity without weakening audit visibility. |
| Accessibility | Makes existing semantic strengths and missing focus, contrast, reflow, target, and state treatments explicit design inputs. |
| Frontend architecture | Provides a future shared visual-system interface across screens; this proposal does not alter current React modules or navigation state. |
| Native/backend layers | No impact. Command adapters, IPC, Rust authority, transactions, and persistence remain unchanged. |
| External design workflow | The completed specification and `design.md` will replace an open-ended styling request with a bounded future frame inventory, state matrix, visual hypothesis, and acceptance checklist. |

## Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| A distinctive palette reduces contrast or overuses accent colors. | Validate semantic assignments and contrast; retain text/icon/state-shape cues and allow palette tuning before approval. |
| Balanced density becomes too sparse for experts or too dense at 960×640. | Review the same populated workflows at both required sizes and require explicit reflow/scroll annotations. |
| Visual redesign accidentally invents behavior or hides safeguards. | Treat existing flows and observable states as invariants in the specification and design brief, then review every future mock against that state contract. |
| Tables or split panes harm keyboard order, zoom, or screen-reader reading order. | Require accessibility annotations and a logical single reading order independent of visual placement. |
| A modal-heavy design slows routine operation. | Restrict modal confirmation to truly destructive or irreversible actions and keep ordinary status/validation inline. |
| `Bs`-friendly fields imply a backend money-contract change. | Annotate display/input formatting as presentation only; retain integer-centavo Rust and IPC contracts. |
| A future dark theme is overbuilt during the first delivery. | Prepare semantic tokens only; explicitly exclude dark mockups and dark-mode implementation. |
| Optional wordmark work expands into a full rebrand. | Limit identity exploration to a simple app wordmark; treat the current icon as non-binding and defer broader branding. |

## Rollback

This change produces proposal, specification, and design-planning artifacts only, so it has no runtime or data rollback. If the completed brief fails human review, revise or discard the unapproved planning direction and retain the current application unchanged. If future external mockups later fail review, revise or discard those mockups independently. The palette, shell, and component/state concepts remain hypotheses until human approval; none may be treated as implementation authority before that approval.

## Stop-before-implementation gate

**Continue now through the authorized specification and `design.md`, then stop for human review.** The designer-ready visual-system brief must be complete through design before handoff. Do not create tasks, invoke apply, edit frontend code, or create production assets. External designer mockups are a future deliverable produced from the human-reviewed brief, not an existing input and not a prerequisite for specification or design. Any mockup production, task planning, implementation, code, or asset work requires separate explicit authorization after the design-stage human review.
