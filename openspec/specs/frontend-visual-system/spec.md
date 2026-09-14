# Frontend Visual System Specification

## Purpose

This specification defines the review contract for a designer-ready visual-system brief for the existing React/Tauri desktop application. It covers the shared shell, operational screen compositions, reusable visual patterns, state treatments, accessibility annotations, and handoff evidence needed for future mockups. It authorizes no implementation, code, assets, behavior, IPC, Rust, or persistence change.

## Requirements

### Requirement: Preserve the product and architecture boundary

The designer-ready brief SHALL describe presentation and interaction states without inventing business behavior, navigation technology, command payloads, IPC contracts, Rust authority, persistence behavior, roles, authentication, or new workflows. It SHALL preserve every existing observable initial, loading, empty, populated, validation, pending, success, warning, failure, unavailable, stale, correction, acknowledgement, and confirmation state and safeguard.

#### Scenario: Future designer receives a bounded brief

- GIVEN the proposal and this specification are the only approved planning inputs
- WHEN a designer prepares future screens or component variants
- THEN the brief SHALL identify existing behavior and presentation guidance separately
- AND the designer SHALL have no implied authorization to implement or invent runtime behavior

#### Scenario: Existing safeguards remain visible

- GIVEN a workflow has stale-price review, restore acknowledgement, correction eligibility, persisted correction evidence, or retry/reload recovery
- WHEN that workflow is represented in the brief or a future mockup
- THEN the corresponding safeguard and observable state SHALL remain identifiable and visually distinct

### Requirement: Provide a persistent application shell and navigation model

The visual system SHALL define one persistent desktop shell with app identity, access to all existing top-level areas, clear grouping, a visible active location, and a consistent selected/focus treatment. The shell SHALL include a low-stock or stock-exception indication when that existing information is available, without implying permissions, accounts, or new alert behavior.

#### Scenario: Sales is the initial location

- GIVEN the application opens in Sales at 1200×800
- WHEN the shell is shown
- THEN the app identity, top-level navigation, Sales active location, and primary work area SHALL be visible without requiring a return action

#### Scenario: Another top-level area is active

- GIVEN Inventory, Catalog maintenance, Backup and restore, Product onboarding, or Sales history is active
- WHEN the shell is shown
- THEN the same navigation remains available, the active location is unambiguous, and the current work area is not presented as Sales

#### Scenario: Shell displays a stock exception cue

- GIVEN low-stock or out-of-stock information is present in the existing application state
- WHEN the shell or its navigation cue is represented
- THEN the cue SHALL expose a text or icon meaning in addition to any color treatment

### Requirement: Establish an uncommon, restrained modern-industrial visual direction

The brief SHALL define a modern-industrial direction that is distinctive but restrained for an auto-parts counter. It SHALL reject conventional enterprise blue as the primary identity and SHALL not resolve into a neutral-only palette. The initial palette candidates SHALL be documented as hypotheses subject to contrast and design validation, including mineral plum `#3D3042`, copper terracotta `#C56845`, industrial sage `#71806A`, warm ivory `#F4EFE6`, light stone `#E4DDD2`, ink `#29252B`, garnet `#A33F46`, and mineral amber `#C7923E`.

#### Scenario: Palette direction is reviewed

- GIVEN a reviewer compares the proposed palette direction with a generic enterprise-blue or neutral-only alternative
- WHEN the visual direction is assessed
- THEN the candidate direction SHALL read as modern industrial, uncommon, restrained, and operationally credible

#### Scenario: Palette candidates are validated rather than frozen

- GIVEN a candidate color fails contrast, legibility, or severity clarity in a representative state
- WHEN the designer validates the palette
- THEN the value or assignment MAY be tuned or rejected while the stated direction and semantic intent remain reviewable

#### Scenario: Accent use remains restrained

- GIVEN a populated operational screen contains navigation, data, actions, and status messages
- WHEN color roles are applied
- THEN accents SHALL establish hierarchy and state without coloring every surface or competing with primary operational facts

### Requirement: Define semantic color roles with contrast and non-color cues

The brief SHALL document semantic roles independently from raw color values, including at least canvas, surface, elevated surface, primary text, muted text, border, focus, action, success, advisory, error/destructive, unavailable, and stale data. Each role SHALL include intended usage, contrast validation evidence or target, and a non-color cue where the role communicates state.

#### Scenario: Text and controls meet contrast targets

- GIVEN normal text, large text, borders, focus indicators, or actionable controls are placed on their intended surfaces
- WHEN the palette is reviewed
- THEN normal text SHALL target at least 4.5:1 contrast, large text SHALL target at least 3:1, and meaningful non-text control or focus indicators SHALL target at least 3:1 against adjacent colors

#### Scenario: State is understandable without color

- GIVEN an item is low stock, out of stock, archived, stale, invalid, unavailable, successful, advisory, or destructive
- WHEN the item is viewed in grayscale or by a user who cannot distinguish the chosen colors
- THEN text, iconography, shape, label, pattern, position, or explicit affordance SHALL communicate the state

#### Scenario: Error and destructive roles are distinguishable

- GIVEN an ordinary advisory and a destructive confirmation appear in the same workflow
- WHEN their roles are compared
- THEN their labels, icon or severity treatment, and action language SHALL distinguish them without relying on hue alone

### Requirement: Prepare light-first tokens for future theme extension

The visual-system brief SHALL define a light-first token vocabulary for color roles, typography, spacing, layout, surfaces, borders, elevation, controls, and focus. Tokens SHALL express semantic roles rather than hard-coding screen-specific hues, and SHALL include enough separation between role and value for a future dark theme without requiring dark-theme behavior or dark mockups in this change.

#### Scenario: First delivery is light-first

- GIVEN the first future mockup package is reviewed
- WHEN its theme is assessed
- THEN it SHALL show a coherent light-first theme and SHALL not require a dark-mode frame or dark-mode interaction design

#### Scenario: Future theme readiness is inspected

- GIVEN a reviewer inspects the token documentation
- WHEN a future dark theme is considered
- THEN the reviewer SHALL be able to identify semantic replacements for surfaces, text, borders, actions, states, and focus without changing screen meaning

### Requirement: Use Spanish operator copy and preserve monetary contracts

The visual-system brief and future mockups SHALL use Spanish interface copy, realistic labels, helper text, status text, and action text. Operator-facing monetary values SHALL be labeled and displayed or entered in `Bs`, with examples such as `Bs 125,50`; centavos SHALL not be exposed as the operator-facing unit. Presentation formatting SHALL not change integer-centavo Rust or IPC contracts, introduce floating-point authority, or alter persisted sale-time prices.

#### Scenario: Sales price and total are shown to an operator

- GIVEN a product price, payment amount, or sale total is represented
- WHEN the operator reads the value
- THEN it SHALL be presented as a `Bs` amount with a clear label and locale-appropriate decimal example rather than raw centavos

#### Scenario: Monetary input remains presentation-only

- GIVEN an operator enters a cash or other monetary amount
- WHEN the field is annotated for handoff
- THEN its Spanish label, `Bs` affordance, and example SHALL be documented while the underlying centavo contract remains unchanged and unspecified by the visual layer

#### Scenario: Whole-unit stock remains unambiguous

- GIVEN inventory quantity or correction quantity is represented
- WHEN the field or table is reviewed
- THEN it SHALL communicate positive whole units and SHALL not imply fractional stock

### Requirement: Balance density, typography, and operational alignment

The visual system SHALL define balanced default density for rapid counter scanning: enough visible operational facts without turning every workflow into a compact spreadsheet. It SHALL document a readable type hierarchy, tabular treatment for quantities, dates, prices, totals, and other comparable numbers, and a distinct scannable treatment for SKU values. It SHALL document a consistent spacing grid, layout rhythm, surfaces, borders, and restrained elevation.

#### Scenario: Numeric facts are compared quickly

- GIVEN a list or table contains SKU, quantity, price, date, and total values
- WHEN an operator scans multiple rows
- THEN comparable values SHALL align consistently, preserve meaningful precision, and be visually distinguishable from prose labels

#### Scenario: Density is reviewed at both reference sizes

- GIVEN the same populated operational workflow is shown at 1200×800 and 960×640
- WHEN reviewers assess the composition
- THEN primary facts and actions SHALL remain discoverable without clipping, unexplained overlap, or an unnecessarily compressed spreadsheet treatment

#### Scenario: Surfaces communicate hierarchy

- GIVEN a screen contains a work area, secondary information, a focused editor, and an urgent status
- WHEN surfaces, borders, and elevation are applied
- THEN hierarchy SHALL be communicated with a documented, restrained combination of spacing, surface, border, or elevation rather than decoration alone

### Requirement: Define a complete button and action hierarchy

The visual system SHALL define primary, secondary, tertiary, destructive, and icon action variants with intended priority, Spanish label guidance, icon-use guidance, and behavior for hover, focus-visible, pressed, disabled, and pending states. Every interactive target SHALL provide a hit area of at least 44×44 CSS pixels, and an icon-only action SHALL have an accessible name and a documented meaning that does not depend on the icon alone.

#### Scenario: Action priority is legible

- GIVEN a screen contains a main confirmation, a secondary navigation or alternative action, a tertiary utility action, and a destructive action
- WHEN the buttons are viewed together
- THEN their hierarchy and severity SHALL be immediately distinguishable and SHALL not make a destructive action appear primary by default

#### Scenario: Button states are reviewable

- GIVEN a button is idle, hovered, focused, pressed, disabled, or pending
- WHEN its state variant is shown in the component/state sheet
- THEN the state SHALL have a visible, accessible treatment, SHALL preserve the action label or an understandable pending label, and SHALL communicate when duplicate activation is unavailable

#### Scenario: Icon actions remain operable

- GIVEN an icon-only action is used for search, navigation, editing, retry, or another existing action
- WHEN it is reached by keyboard or assistive technology
- THEN its accessible name, focus treatment, target size, and purpose SHALL be available independently of the icon shape

### Requirement: Define form controls and field feedback

The visual system SHALL cover text, numeric, search, date, select, checkbox, and quantity controls, including labels, required or optional indication, help text, units, grouping, disabled and pending treatment, and accessible error association. It SHALL define field-level validation and focus recovery without changing existing validation rules.

#### Scenario: A valid form is scanned

- GIVEN a form contains required fields, optional fields, dynamic attributes, and a quantity or monetary field
- WHEN an operator reviews the form before submission
- THEN each control SHALL have an adjacent Spanish label, a clear unit or example where applicable, and a visible relationship to its group and action

#### Scenario: A field is invalid

- GIVEN a required, malformed, out-of-range, or otherwise invalid field is submitted
- WHEN validation feedback is shown
- THEN the field SHALL be visually marked, the error SHALL be associated with that field, the message SHALL explain the correction in Spanish, and the intended focus destination SHALL be annotated

#### Scenario: A form is pending or unavailable

- GIVEN a form submission or dependent record load is pending or unavailable
- WHEN the state is represented
- THEN the affected controls SHALL expose disabled or busy intent, prevent ambiguous duplicate action, retain relevant entered context where the existing flow does so, and expose a retry or recovery path when one exists

### Requirement: Define data, table, and master-detail patterns for scanning

The visual system SHALL define aligned table-like or data-list patterns for products, stock, sales, dates, amounts, lifecycle state, and correction facts. It SHALL define a master-detail or equivalent hierarchy for Catalog and History that supports selection, context retention, and a logical reading order independent of visual placement. Patterns SHALL support scanning without implying unapproved sorting, editing, pagination, bulk operations, or other behavior.

#### Scenario: Product and sales records are scanned

- GIVEN a populated Catalog, Inventory, or Sales history list
- WHEN an operator scans records
- THEN identity, SKU or sale identity, relevant date, quantity, amount, lifecycle or stock state, and available action SHALL occupy consistently named and aligned regions

#### Scenario: Master-detail is shown at the minimum size

- GIVEN Catalog or History is represented at 960×640
- WHEN the list and selected detail are both needed
- THEN the composition SHALL preserve a clear selected record and detail context through documented stacking, resizing, or scrolling, without hiding the selected record's identity

#### Scenario: Reading order differs from visual placement

- GIVEN a split pane, sticky region, or visually reordered data pattern is proposed
- WHEN keyboard or assistive-technology reading order is reviewed
- THEN the handoff annotation SHALL state a single logical order that reaches the same information and actions

### Requirement: Define reusable status, loading, empty, and recovery patterns

The visual system SHALL define distinct patterns for initial, loading, empty, populated, validation, pending/disabled, success, warning/advisory, error, unavailable/retry, stale-data, and focus-visible states. Each pattern SHALL specify its message intent, severity, associated work area, persistence or dismissal expectation where already observable, and non-color cue.

#### Scenario: A collection is loading or empty

- GIVEN a search, list, or dependent record area is loading, has no records, or has no search matches
- WHEN the state is represented
- THEN the area SHALL identify whether it is loading, empty, or no-results, SHALL avoid presenting an empty state as a failure, and SHALL retain the relevant context or recovery action

#### Scenario: An operation succeeds or needs advisory attention

- GIVEN an operation succeeds or returns an advisory such as a stale preview or stale price
- WHEN feedback is shown
- THEN the message SHALL remain associated with the operation, distinguish success from advisory, and preserve any required acknowledgement or next action

#### Scenario: An operation fails or becomes unavailable

- GIVEN a command, search, record, or persisted detail fails or is unavailable
- WHEN the failure state is represented
- THEN the message SHALL identify the affected work, use Spanish recovery guidance, and expose Retry, Reload, or the existing recovery action when available

#### Scenario: A record is stale

- GIVEN displayed data or a projected result is stale relative to the existing flow
- WHEN the operator can act on it
- THEN stale status SHALL be explicit, SHALL not be confused with a fresh success state, and SHALL preserve the existing review or reload safeguard

### Requirement: Apply modal-only destructive confirmation and inline routine feedback

The visual system SHALL keep routine validation, pending, success, advisory, retry, and recoverable failure feedback inline and adjacent to the affected work. A truly destructive or irreversible action SHALL use a modal confirmation pattern only, with explicit Spanish consequence text, a distinct destructive action, a safe dismissal path, focus containment and return guidance, and preservation of any existing acknowledgement. Modal treatment SHALL not be added merely to emphasize a routine action.

#### Scenario: Routine feedback stays inline

- GIVEN a quantity validation error, save pending state, successful save, advisory, or recoverable load failure
- WHEN feedback is shown
- THEN it SHALL remain inline with the relevant field, form, list, or work area and SHALL not interrupt the operator with a modal

#### Scenario: Restore or cancellation requires destructive confirmation

- GIVEN the operator is about to restore over current data or cancel a sale irreversibly
- WHEN the confirmation state is represented
- THEN it SHALL be modal-only, state the consequence in Spanish, preserve the required acknowledgement or reason, distinguish Cancel from the destructive action, and document focus return after dismissal or completion

#### Scenario: Reversible or ordinary actions are not over-confirmed

- GIVEN the operator performs search, selection, routine save, navigation, retry, or another non-destructive action
- WHEN its visual treatment is reviewed
- THEN it SHALL not acquire a modal confirmation solely for emphasis

### Requirement: Specify responsive desktop behavior at both supported sizes

The brief SHALL provide explicit annotated compositions for 1200×800 and 960×640. It SHALL identify what remains visible, what scrolls, what becomes sticky, and how panels, navigation, tables, forms, actions, and master-detail areas reflow at each size. Mobile, touch-first, and multi-window behavior SHALL remain out of scope.

#### Scenario: Starting window remains operational

- GIVEN a populated Sales, Inventory, Catalog, or History composition at 1200×800
- WHEN the frame is reviewed
- THEN the shell, active location, primary work facts, primary action, and relevant status SHALL remain visible without requiring unexplained horizontal scrolling

#### Scenario: Minimum window remains usable

- GIVEN the same composition at 960×640
- WHEN the frame is reviewed
- THEN the operator SHALL be able to identify the current location, reach the main work controls, read critical facts, and reach recovery or destructive actions through documented reflow or scrolling

#### Scenario: Scroll and sticky decisions are explicit

- GIVEN a screen contains long records, tables, forms, or a persistent action area
- WHEN the handoff is reviewed
- THEN annotations SHALL state which region scrolls, which header or action remains sticky if any, where focus moves after reflow, and how the logical reading order is preserved

### Requirement: Define accessibility and inclusive interaction annotations

The visual-system brief SHALL annotate keyboard operation, focus order, focus-visible appearance, focus recovery after validation and modal dismissal, semantic naming intent, live-region intent, contrast targets, zoom and reflow behavior, reduced-motion behavior, and non-color state cues. It SHALL preserve existing landmarks, headings, labels, fieldsets, busy states, alerts, status messages, error associations, and correction focus recovery.

#### Scenario: Operator uses the keyboard

- GIVEN an operator navigates a shell, form, table-like list, master-detail view, or modal without a pointer
- WHEN the interaction order is reviewed
- THEN every action and control SHALL be reachable in a predictable order, the active target SHALL be visible, and no visual placement SHALL make an otherwise reachable action inaccessible

#### Scenario: Validation or modal completion restores focus

- GIVEN validation identifies a correction target or a modal is dismissed or completed
- WHEN focus recovery is represented
- THEN the destination SHALL be named in the annotation and SHALL return the operator to the next useful control or the affected content without trapping focus in the page

#### Scenario: Async feedback is announced appropriately

- GIVEN a load, pending action, success, advisory, error, unavailable state, or retry result changes
- WHEN its feedback is represented
- THEN the brief SHALL identify whether it is status, alert, busy, or field error content and SHALL keep it associated with the affected work rather than relying on visual placement alone

#### Scenario: Zoom, motion, and contrast are reviewed

- GIVEN the interface is enlarged, viewed with reduced motion, or inspected for contrast
- WHEN the design is evaluated
- THEN content and controls SHALL remain usable through reflow, essential meaning SHALL not depend on animation, and documented contrast targets SHALL continue to apply

### Requirement: Compose the Sales/POS screen and shell states

The future designer brief SHALL specify Sales/POS composition and state variants within the persistent shell. The populated composition SHALL expose product identity, SKU, search/result context, stock, price, cart lines, quantities, cash and QR payment amounts, total, primary confirmation, secondary actions, and stale-price review as separately scannable facts. It SHALL also specify initial/empty search, loading, no results, invalid input, pending confirmation, failure, and persisted confirmed-sale summary states without changing their behavior.

#### Scenario: Populated Sales is reviewed

- GIVEN Sales contains search results and a populated cart
- WHEN the composition is reviewed at either reference size
- THEN product discovery, stock and price facts, cart quantities, payment split, total, and the primary confirmation action SHALL be findable as distinct regions in a keyboard-friendly order

#### Scenario: Sales has a stale price or invalid payment

- GIVEN a stale catalog price or invalid quantity/payment is present
- WHEN the state is represented
- THEN the affected fact or field SHALL expose the existing review or validation requirement inline, with an explicit Spanish message and no removed safeguard

#### Scenario: Sale confirmation succeeds

- GIVEN the existing confirmation flow returns a persisted sale summary
- WHEN the success composition is reviewed
- THEN sale identity, timestamp, persisted products, payment facts, total, and the existing New sale action SHALL be visually distinct from the editable draft state

### Requirement: Compose the Inventory screen and stock-priority states

The brief SHALL specify Inventory with product search/selection, selected-product identity, stock-entry and physical-count operation variants, positive whole-unit quantity treatment, projected balance, low-stock at one unit, out-of-stock at zero, reason or note context, pending, success/advisory, validation, failure, and retry/recovery states.

#### Scenario: Inventory operation is reviewed

- GIVEN a product is selected for a stock entry or physical count
- WHEN the composition is reviewed
- THEN current stock, operation inputs, reason or note requirements, projected balance, and the operation action SHALL be separately identifiable and shall not imply fractional quantities

#### Scenario: Inventory exceptions are prioritized

- GIVEN low-stock and out-of-stock alerts coexist with an operation
- WHEN the screen is reviewed in populated and minimum-size states
- THEN the alert area SHALL distinguish one unit from zero with text or iconography, expose priority without color alone, and remain reachable without obscuring the operation result

### Requirement: Compose Catalog maintenance as scannable master-detail

The brief SHALL specify Catalog maintenance with a scannable category/product record list, selected record detail, product and category facts, dynamic editable fields, active and archived lifecycle states, action hierarchy, field validation, pending/disabled, success, unavailable/retry, and stale-record recovery states. It SHALL keep lifecycle and editing semantics distinct without adding unapproved record operations.

#### Scenario: Active and archived records are compared

- GIVEN the catalog contains active and archived category or product records
- WHEN an operator scans and selects a record
- THEN record identity, lifecycle state, relevant facts, available lifecycle action, and editable versus read-only information SHALL be visually distinct and non-color-coded

#### Scenario: Dynamic form validation or stale recovery occurs

- GIVEN a dynamic attribute is invalid, a save is pending, or the selected record is stale or unavailable
- WHEN the state is represented
- THEN field feedback, busy treatment, recovery action, and intended focus destination SHALL remain associated with the selected record and SHALL preserve existing recovery semantics

### Requirement: Compose Sales History, persisted detail, and corrections

The brief SHALL specify Sales history list and detail compositions with bounded date filtering, loading/empty/populated/error/retry states, persisted sale identity and timestamps, original line and payment facts, sale-time prices, correction history, eligibility, unavailable historical product text, open return workflow, and open cancellation workflow. It SHALL keep original persisted facts visually distinct from current catalog facts and correction outcomes.

#### Scenario: History list and detail are scanned

- GIVEN the operator loads a bounded history list and opens a sale
- WHEN the list and detail are reviewed
- THEN dates, sale identity, totals, original items, payments, correction history, and unavailable historical product information SHALL have a clear hierarchy and remain readable at 960×640

#### Scenario: Return correction is open

- GIVEN eligible original lines are available for return
- WHEN the return workflow is represented
- THEN line selection, whole-unit quantities, validation, pending state, failure/reload recovery, and persisted correction evidence SHALL be visually distinct and remain inline unless the action qualifies as truly destructive

#### Scenario: Cancellation correction is open

- GIVEN an eligible sale cancellation requires a reason and explicit inventory correction acknowledgement
- WHEN the cancellation workflow is represented
- THEN reason entry, acknowledgement, destructive consequence, modal confirmation, pending, failure/reload recovery, and resulting persisted history SHALL each remain identifiable

### Requirement: Preserve continuity for onboarding and backup/restore states

Although Sales, Inventory, Catalog, and History are the primary composition review, the visual-system brief SHALL provide reusable state guidance sufficient for Product onboarding and Backup and restore. It SHALL cover dynamic category-field and product forms, empty-category and loading states, validation, pending, success, failure, native file-dialog handoff, prepared restore candidate, acknowledgement, destructive confirmation, invalid or expired backup, unavailable storage, and recovery.

#### Scenario: Product onboarding uses shared form patterns

- GIVEN onboarding has loading categories, an empty category list, dynamic fields, or a product form
- WHEN its future composition is reviewed
- THEN it SHALL reuse the documented form labels, grouping, validation, pending, success, and failure patterns without introducing new product behavior

#### Scenario: Backup and restore uses shared recovery patterns

- GIVEN backup creation or restore is idle, prepared, pending, successful, invalid, expired, unavailable, or failed
- WHEN its future composition is reviewed
- THEN native dialog handoff, restore replacement consequence, required acknowledgement, destructive confirmation, and recovery feedback SHALL remain explicit and consistent with the shared system

### Requirement: Deliver a component, state, and token sheet

The designer-ready handoff SHALL include a component/state sheet or equivalent annotated inventory covering shell/navigation, typography, spacing/layout grid, buttons, icon actions, form controls, table/list and master-detail patterns, SKU and numeric treatment, stock/lifecycle badges, banners and inline live feedback, loading/empty indicators, modal/inline confirmation, focus, hover, pressed, disabled, pending, error, warning, success, unavailable, stale, and destructive variants. It SHALL mark each pattern as reusable or screen-specific.

#### Scenario: Reviewer traces a screen to reusable patterns

- GIVEN a reviewer inspects Sales, Inventory, Catalog, or History
- WHEN each visible pattern is compared with the sheet
- THEN the reviewer SHALL be able to identify its reusable system pattern, state variant, semantic role, and any screen-specific composition decision

#### Scenario: State coverage is checked

- GIVEN a state named in the existing observable state map is required for review
- WHEN the component/state sheet is checked
- THEN the corresponding visual treatment, Spanish message intent, action priority, accessibility annotation, and non-color cue SHALL be present or explicitly marked as screen-specific

#### Scenario: Token intent is checked

- GIVEN a reviewer inspects a token candidate
- WHEN its documentation is read
- THEN its semantic purpose, light-first value or hypothesis, contrast evidence or target, usage boundary, and future-theme replacement intent SHALL be clear

### Requirement: Define typography, icon, wordmark, and licensing handoff

The handoff brief SHALL identify proposed or placeholder font, icon, wordmark, and other asset dependencies, their intended use, source, license, redistribution rights, attribution requirements, and ownership or replacement assumptions. Wordmark exploration SHALL remain limited to a simple application identity; the existing Tauri icon SHALL not be treated as binding, and no production asset SHALL be created by this change.

#### Scenario: External designer can prepare an editable package

- GIVEN a future designer selects a font, icon set, wordmark direction, or asset reference
- WHEN the handoff package is reviewed
- THEN each dependency SHALL have enough licensing and ownership information for the reviewer to approve, replace, or exclude it before implementation

#### Scenario: Unlicensed dependency is proposed

- GIVEN a font, icon, or wordmark source lacks clear rights for the intended package
- WHEN the handoff is reviewed
- THEN it SHALL be flagged as unresolved and SHALL not be treated as an approved production dependency

### Requirement: Make the future design review contract testable

The specification and subsequent design brief SHALL state the review evidence required at both reference sizes: persistent shell and active location; Sales/POS; Inventory; Catalog; History/detail/corrections; reusable component/state sheet; Spanish copy and `Bs` examples; semantic tokens; accessibility annotations; explicit reflow and scrolling decisions; licensing assumptions; and invariant checks. The current phase SHALL stop at planning artifacts and SHALL not contain tasks, apply instructions, implementation code, production assets, or mockups.

#### Scenario: Human reviewer evaluates planning completeness

- GIVEN proposal, specification, and design brief are presented for review
- WHEN the reviewer checks the planning package
- THEN each required screen group, state family, handoff annotation, invariant, and review size SHALL be traceable to a requirement or scenario

#### Scenario: Implementation authority is checked

- GIVEN a reader asks what may be changed after this specification phase
- WHEN the boundary is inspected
- THEN the artifacts SHALL authorize only the proposal/specification/design planning sequence and SHALL require separate authorization before mockups, tasks, apply work, code, or assets
