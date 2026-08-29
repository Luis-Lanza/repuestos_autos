# Sales History Specification

## Purpose

Provide a read-only, bounded view of persisted sales for calendar browsing and inspection.

## Requirements

### Requirement: Bounded Calendar-Filtered Summaries

The system MUST accept a calendar-date range and return no more than a defined maximum of summaries. The range MUST be half-open: start is inclusive at day start; end is exclusive at following day start. The response MUST indicate when further matches exist.

#### Scenario: Filter and bound results

- GIVEN persisted sales exist inside and outside a range
- WHEN the operator requests it
- THEN only sales inside its half-open interval are returned
- AND results do not exceed the maximum

#### Scenario: Include and exclude boundaries

- GIVEN one sale is at the from-date start and another at the day-after-to-date start
- WHEN the operator requests from through to
- THEN the first sale is included and the second is excluded

#### Scenario: Invalid range

- GIVEN the end date precedes the start date
- WHEN history is requested
- THEN an observable validation error is returned
- AND no historical record changes

#### Scenario: More matches than the bound

- GIVEN more sales match than the maximum
- WHEN history is requested
- THEN the response stays bounded
- AND the operator can determine that more matches exist

### Requirement: Deterministic Historical Summaries

Each summary MUST expose persisted sale identity, confirmation timestamp, status, total, and line/payment summary facts. Summaries MUST be newest-first by confirmation timestamp, with a stable identity tie-breaker. Historical values MUST use persisted sale-time snapshots, not current catalog values. For legacy lines missing SKU or product-name snapshots, each field MUST be explicitly unavailable, never filled from current catalog.

#### Scenario: Stable ordering and snapshots

- GIVEN matching sales share a confirmation timestamp and lines have persisted or missing snapshots
- WHEN summaries or detail are loaded after catalog edits
- THEN tied sales use the same identity ordering on every request
- AND persisted values remain exact while missing SKU or name fields show unavailable
- AND current catalog labels are never used

### Requirement: On-Demand Persisted Detail

Selecting a summary MUST load that sale's persisted lines, total, and payments on demand. Detail MUST show each line's persisted snapshot fields when present or explicit unavailable markers when absent, with positive whole-unit quantity. Each payment MUST show method and applied amount; cash MUST also show persisted tendered and change.

#### Scenario: Load existing detail

- GIVEN a summary identifies an existing sale
- WHEN the operator selects it
- THEN all persisted lines, total, and payments are shown
- AND monetary facts remain integer-centavo amounts formatted as Bs

#### Scenario: Unknown detail

- GIVEN the selected identity has no persisted sale
- WHEN detail is requested
- THEN an observable not-found/error state is shown
- AND no record is created or changed

### Requirement: Navigation and Retrieval States

The Sales vertical MUST provide history navigation, detail selection, and return to the list. While a retrieval is pending, a loading state MUST be observable; when no sales match, an empty state MUST be shown; and failures MUST show an error state without fabricated history.

#### Scenario: Navigate, empty, and loading

- GIVEN the operator is in Sales
- WHEN history is opened, a summary selected, and back chosen
- THEN list and detail are reachable and returning does not mutate the sale
- AND pending retrieval shows loading while a valid range with no matches shows empty

### Requirement: Read-Only Historical Access

History and detail operations MUST NOT create, update, delete, cancel, return, reprice, relabel, or otherwise mutate sales, lines, payments, catalog, stock, or inventory movements. Returns, cancellations, analytics, reports, and exports are outside this capability.

#### Scenario: Repeated browsing is side-effect free

- GIVEN persisted sales and related operational records
- WHEN the operator filters, opens, reloads, and revisits history
- THEN all persisted business facts remain unchanged
