# Technical Design: Post-sale Lifecycle

## 1. Context and design goals

This change adds inventory-only post-sale corrections to the existing Rust/Tauri/SQLite application without changing original sale, sale-line, total, or payment facts. It follows the current architecture:

- pure business validation in `domain`;
- transaction orchestration and interfaces in `application`;
- parameterized SQLite statements in `infrastructure`;
- narrow tagged contracts in `commands` and TypeScript adapters;
- React as a presentation adapter only.

The design is additive. A cancelled sale remains persisted as a confirmed original sale; its observable lifecycle status is derived from an immutable cancellation header. This avoids rebuilding the referenced `sales` table solely to expand its v9 `CHECK (status IN ('pending', 'confirmed'))`, preserves the existing confirmation path, and keeps every original sale field unchanged.

## 2. Decisions

### 2.1 Corrections are additive facts

The authoritative facts are:

- `post_sale_requests`: global request identity and canonical payload identity;
- `sale_returns` and `sale_return_lines`: one atomic multi-line return;
- `sale_cancellations` and `sale_cancellation_lines`: one whole-sale cancellation and its per-original-line residual calculation;
- existing `inventory_movements`: one positive immutable movement for every positive correction line;
- existing `stock_balances`: current stock projection updated in the same transaction.

No correction updates or deletes original sale lines, snapshots, totals, or payment rows. No monetary correction table is introduced.

### 2.2 Lifecycle status is derived, not written into `sales.status`

`SaleLifecycleStatus` has observable values `confirmed` and `cancelled`:

- `cancelled` when a `sale_cancellations` header exists for the sale;
- otherwise `confirmed` for a persisted confirmed sale.

The base `sales.status` remains `confirmed`. Sales History readers project the lifecycle status with `CASE WHEN cancellation exists THEN 'cancelled' ELSE s.status END`. Returns are rejected once a cancellation exists. The existing partial index on confirmed sales remains usable and cancelled sales remain discoverable because the original sale status is not rewritten.

### 2.3 Request identity is global across both correction kinds

A UUID request ID is normalized with the existing `RequestId` type to lowercase hyphenated form. `post_sale_requests.request_id` is globally unique, so reusing an ID for a return and a cancellation is a conflict rather than two independent operations.

Each request stores:

- normalized request ID;
- operation kind (`return` or `cancellation`);
- sale ID;
- payload format version (`1`);
- canonical payload bytes;
- lowercase SHA-256 of those exact bytes;
- persisted creation time.

Both canonical bytes and hash are compared. The hash provides indexed/diagnostic identity; comparing the bytes prevents a hash collision from being treated as a valid replay.

### 2.4 Canonical payload format

Canonicalization happens after command-shape validation and before opening the transaction.

Return input rules:

- at least one line;
- each `sale_line_id` is positive and appears once; duplicate IDs are rejected, not merged;
- each quantity is a positive `i64` whole unit;
- line order is semantically irrelevant and is sorted by ascending sale-line ID.

Return canonical bytes are UTF-8:

```text
post-sale/v1\nreturn\n<sale_id>\n<line_count>\n<sale_line_id>:<quantity>\n...
```

Cancellation input rules:

- reason is trimmed at both ends;
- the trimmed reason must be non-empty;
- internal whitespace and case are preserved.

Cancellation canonical bytes are UTF-8:

```text
post-sale/v1\ncancellation\n<sale_id>\n<utf8-byte-length>:<trimmed-reason>
```

Length-prefixing makes arbitrary reason text unambiguous. The normalized reason is what is persisted and returned.

## 3. Schema v10

Add `0010_post_sale_lifecycle.sql` and advance `CURRENT_SCHEMA_VERSION` from 9 to 10.

### 3.1 Tables

```sql
CREATE TABLE post_sale_requests (
    id INTEGER PRIMARY KEY,
    request_id TEXT NOT NULL UNIQUE CHECK (trim(request_id) <> ''),
    operation_kind TEXT NOT NULL CHECK (operation_kind IN ('return', 'cancellation')),
    sale_id INTEGER NOT NULL REFERENCES sales(id),
    payload_version INTEGER NOT NULL CHECK (payload_version = 1),
    canonical_payload BLOB NOT NULL,
    payload_sha256 TEXT NOT NULL CHECK (length(payload_sha256) = 64),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (id, sale_id, operation_kind)
);

CREATE TABLE sale_returns (
    id INTEGER PRIMARY KEY,
    sale_id INTEGER NOT NULL,
    operation_kind TEXT NOT NULL DEFAULT 'return' CHECK (operation_kind = 'return'),
    occurred_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (id, sale_id),
    FOREIGN KEY (id, sale_id, operation_kind)
      REFERENCES post_sale_requests(id, sale_id, operation_kind)
);

CREATE TABLE sale_return_lines (
    return_id INTEGER NOT NULL,
    sale_id INTEGER NOT NULL,
    sale_line_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    movement_id INTEGER NOT NULL UNIQUE REFERENCES inventory_movements(id),
    PRIMARY KEY (return_id, sale_line_id),
    FOREIGN KEY (return_id, sale_id) REFERENCES sale_returns(id, sale_id),
    FOREIGN KEY (sale_line_id, sale_id, product_id)
      REFERENCES sale_lines(id, sale_id, product_id)
);

CREATE TABLE sale_cancellations (
    id INTEGER PRIMARY KEY,
    sale_id INTEGER NOT NULL UNIQUE,
    operation_kind TEXT NOT NULL DEFAULT 'cancellation' CHECK (operation_kind = 'cancellation'),
    reason TEXT NOT NULL CHECK (trim(reason) <> ''),
    occurred_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (id, sale_id),
    FOREIGN KEY (id, sale_id, operation_kind)
      REFERENCES post_sale_requests(id, sale_id, operation_kind)
);

CREATE TABLE sale_cancellation_lines (
    cancellation_id INTEGER NOT NULL,
    sale_id INTEGER NOT NULL,
    sale_line_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    restored_quantity INTEGER NOT NULL CHECK (restored_quantity >= 0),
    movement_id INTEGER UNIQUE REFERENCES inventory_movements(id),
    PRIMARY KEY (cancellation_id, sale_line_id),
    FOREIGN KEY (cancellation_id, sale_id)
      REFERENCES sale_cancellations(id, sale_id),
    FOREIGN KEY (sale_line_id, sale_id, product_id)
      REFERENCES sale_lines(id, sale_id, product_id),
    CHECK ((restored_quantity = 0 AND movement_id IS NULL)
        OR (restored_quantity > 0 AND movement_id IS NOT NULL))
);
```

A cancellation line is recorded for every original sale line, including zero residuals. Therefore a fully returned sale has a cancellation header and zero-valued audit lines but no cancellation movements.

### 3.2 Indexes and immutability

Add indexes for:

- `post_sale_requests(sale_id, created_at, id)`;
- `sale_return_lines(sale_line_id, return_id)`;
- `sale_cancellation_lines(sale_line_id)`.

The unique request ID and unique cancellation sale ID cover replay and cancel-once checks. Add update/delete rejection triggers to all five new fact tables. Application code never edits a committed correction.

Add insert-validation triggers on return and cancellation lines as a second line of defense. They verify that:

- the linked movement has the same sale, original sale line, product, movement type, and positive quantity;
- a return line plus prior return lines and any cancellation line cannot exceed the original sold quantity;
- a cancellation line plus prior returns cannot exceed the original sold quantity;
- a zero cancellation line has no movement.

Every application statement containing values uses `?N` parameters and `rusqlite::params!`. SQL identifiers are fixed constants; no user-controlled value is interpolated.

### 3.3 Forward migration from v9

The v9-to-v10 migration is one SQLite transaction:

1. validate the v9 schema and foreign keys;
2. create only the new tables, indexes, and triggers;
3. run v10 structural and cross-fact validation;
4. set `PRAGMA user_version = 10`;
5. commit.

No v9 row is copied, rewritten, or deleted. Existing v9 `return` or `cancellation` movement rows remain valid legacy movement facts and are not retroactively attached to new correction headers. V10 validation requires every new correction line to have a valid movement, but does not require every pre-existing movement to have a new correction line.

Rollback before release may remove the unreleased migration with the feature. Once v10 data exists, rollback means disabling commands/UI while retaining v10 tables and facts; no down migration deletes accepted corrections.

## 4. Domain and application design

### 4.1 Domain module

Add `domain/sales/post_sale.rs` with value types and pure planning functions:

- `RequestedReturnLine { sale_line_id, quantity }`;
- `OriginalSaleLine { sale_line_id, product_id, sold_quantity, returned_quantity }`;
- `ReturnPlan` and `ReturnPlanLine`;
- `CancellationPlan` and `CancellationPlanLine`;
- `PostSaleDomainError`.

`plan_return` rejects empty requests, duplicate line IDs, unknown/wrong-sale lines, non-positive quantities, cancelled sales, and quantities above remaining returnable quantity. It returns one plan line per requested original line.

`plan_cancellation` requires a normalized non-blank reason, rejects non-confirmed/already-cancelled sales, and emits one plan line per original line with `sold - returned`. Checked `i64` arithmetic is mandatory.

The domain module receives persisted facts; it does not query SQLite, hash payloads, generate IDs, or calculate UI state.

### 4.2 Application module and seam

Add `application/sales/post_sale.rs` containing:

- application request/result DTOs;
- `PostSaleError`;
- `PersistedRequest` replay metadata;
- the `PostSaleRepository` interface;
- `PostSaleLifecycleUseCase`.

The external interface is two methods:

```text
create_return(request) -> Result<ReturnResult, PostSaleError>
cancel_sale(request) -> Result<CancellationResult, PostSaleError>
```

The repository interface is the seam for persisted request lookup, sale/correction fact loading, result replay loading, and persistence of a validated plan inside a supplied transaction. SQLite is the production adapter; focused tests use a fake adapter only where a pure domain test is insufficient.

### 4.3 Exact transaction boundary

Each use-case call owns exactly one `rusqlite::TransactionBehavior::Immediate` transaction. The boundary includes request identity lookup, eligibility reads, aggregate reads, header/line writes, stock updates, movement writes, and result reload.

Algorithm for both operations:

1. Parse and canonicalize the request before transaction start.
2. Begin `IMMEDIATE`; this obtains SQLite's reserved writer lock before eligibility is read.
3. Lookup normalized request ID.
4. If found, compare operation kind, sale ID, payload version, canonical bytes, and SHA-256:
   - exact match: load and return the persisted result without writes;
   - any mismatch: return `RequestConflict` without writes.
5. Load the confirmed sale, original lines, accepted return aggregates, and cancellation state in the transaction.
6. Build the domain plan.
7. Insert request header and operation header.
8. For each positive plan line:
   - update exactly one `stock_balances` row with checked addition;
   - insert one `inventory_movements` row using `return` or `cancellation`, the original sale/line/product IDs, and the positive quantity;
   - insert the correction line linked to that movement.
9. For cancellation zero residuals, insert the zero cancellation line with no movement.
10. Reload the persisted result from the same transaction.
11. Commit once. On any error, explicitly roll back; rollback failure maps to persistence failure.

Cancellation movements store the normalized cancellation reason in existing `inventory_movements.reason`. Return movements keep `reason` null. `source_reference` is a deterministic internal value based on correction header and original line IDs. Existing movement `request_id` remains null because its unique v6 index permits only one movement per ID and a correction request can contain multiple lines; authoritative request identity is `post_sale_requests`.

SQLite serializes competing writers. The process-level `DatabaseState` mutex already serializes commands in one app instance, while `IMMEDIATE` protects correctness for any additional connection. A losing overlapping return re-evaluates remaining quantity after the winner commits. A return losing to cancellation returns `sale_cancelled`; an over-returning loser returns `quantity_exceeds_remaining`; an exact same-ID retry replays; different payload reuse returns `request_conflict`. `SQLITE_BUSY`/timeout and unexpected storage errors map to `persistence_failure` with no partial effects.

## 5. Infrastructure adapter

Add `SqlitePostSaleRepository` in `infrastructure/sqlite/post_sale_repository.rs`.

Responsibilities:

- execute only fixed, parameterized SQL;
- map persisted rows into domain/application types and reject malformed facts;
- aggregate returns by exact `sale_line_id`, never product ID;
- persist the plan in the caller-owned transaction;
- verify affected row counts for stock updates and status/identity reads;
- use checked integer bounds in stock updates;
- reconstruct replay results from immutable headers and lines rather than recomputing from current stock.

The adapter does not decide business eligibility. Database constraints/triggers are defensive and adapter errors become typed application errors.

## 6. Tauri and TypeScript contracts

### 6.1 Rust commands

Add `commands/post_sale.rs` and register both commands in test and desktop `generate_handler!` lists:

```text
create_sale_return_command({ request })
cancel_sale_command({ request })
```

Rust requests use `#[serde(deny_unknown_fields)]` and owned values:

```text
CreateSaleReturnRequest {
  request_id: String,
  sale_id: i64,
  lines: Vec<{ sale_line_id: i64, quantity: i64 }>
}

CancelSaleRequest {
  request_id: String,
  sale_id: i64,
  reason: String
}
```

Responses are tagged unions:

```text
{ kind: "success", result: ReturnResult }
{ kind: "success", result: CancellationResult }
{ kind: "error", code, message }
```

Persisted success results always contain normalized request ID, correction header ID, sale ID, lifecycle status, occurred-at timestamp, and exact original-line IDs/quantities. Return results contain accepted quantities; cancellation results contain every original line's residual restored quantity. Replays reconstruct the same `result` bytes semantically and create no replay-specific persistence.

Stable error codes:

- `invalid_request`;
- `invalid_quantity`;
- `duplicate_sale_line`;
- `sale_not_found`;
- `sale_not_confirmed`;
- `sale_cancelled`;
- `sale_line_not_found`;
- `quantity_exceeds_remaining`;
- `cancellation_reason_required`;
- `cancellation_already_recorded`;
- `request_conflict`;
- `persistence_failure`.

Messages describe inventory correction only and do not expose SQL details or imply refund, reimbursement, reversal, credit, or settlement.

### 6.2 TypeScript adapter

Add `src/commands/post-sale.ts` with matching discriminated unions, runtime shape guards, and a thin `createPostSaleCommands(invoke)` adapter. It sends snake_case request fields under `{ request }` and converts malformed IPC values/rejections to `persistence_failure`. It performs no remaining-quantity or residual calculation.

The browser-generated UUID is created once when an intent form opens and retained across retries. A new UUID is generated only after success or when the operator abandons that intent.

## 7. Sales History and UI extension

### 7.1 Read model

Extend Sales History projections:

- summaries include derived `status` and `has_corrections`;
- `HistoricalLine` includes `sale_line_id`, `returned_quantity`, `cancellation_restored_quantity`, and `remaining_returnable_quantity` loaded from persisted correction facts;
- detail includes ordered `returns[]` with header ID, request ID, timestamp, and lines;
- detail includes optional `cancellation` with header ID, request ID, timestamp, normalized reason, and all per-line restored quantities;
- original payments and money fields remain unchanged.

Readers include base sales with `s.status = 'confirmed'`; they do not filter on derived lifecycle status. All filters, IDs, and limits remain parameterized.

### 7.2 Interaction design

Extend the existing Sales History detail rather than adding a new top-level route.

- Show a text status badge (`Confirmed` or `Cancelled`) so status is not conveyed by color alone.
- Keep `Original sale items` and `Original payment facts` in distinct read-only sections.
- Add `Inventory correction history` listing accepted return quantities and cancellation-restored quantities. Copy explicitly states that payment facts were not changed.
- For a confirmed sale with remaining quantities, `Record item return` opens a labelled form. Each original line has a checkbox, a whole-number input with persisted remaining quantity as the maximum, and inline errors. Submission is disabled while pending.
- `Cancel sale` is visually separated as a destructive lifecycle action, requires a visible reason label, and uses a confirmation step summarizing that only inventory residuals are restored and payments are unchanged.
- Cancelled sales expose neither action. Fully returned confirmed sales hide/disable return but still allow cancellation.
- On success, reload detail from Rust; React never adjusts quantities optimistically.
- On conflict/stale eligibility, keep the form values, announce the error with `role="alert"`, and offer reload/retry.
- Buttons and fields remain keyboard operable, focus moves to the first invalid field, pending states use `aria-busy`, and touch targets remain at least 44px.

No copy uses refund, reimbursement, payment reversal, credit, settlement, or money-return language.

## 8. Backup and restore impact

No new backup workflow is added. Existing SQLite backup copies the additive tables automatically. Compatibility changes are limited to schema validation:

- `stage_and_validate` accepts versions 1 through 10, migrates staged v1-v9 databases to v10, and leaves the selected source unchanged;
- `validate_restored_database` calls new `validate_version_ten_schema` rather than stopping at v6 validation;
- v10 validation checks required tables/columns/indexes/triggers, foreign keys, request-to-header consistency, correction-line-to-movement consistency, cumulative restored quantities, and complete cancellation line coverage;
- validation allows legacy unlinked v9 correction movement types;
- backup/restore tests create a v10 database containing returns and cancellation, restore it, and compare original sale facts, correction facts, balances, and movement links.

`OperationalFacts` need not become a reporting surface for corrections. Existing counts remain unchanged in shape; integrity validation proves preservation of the new facts.

## 9. Test seams and evidence

### Domain tests

- multi-line returns with repeated product identity but distinct sale-line IDs;
- zero/negative/duplicate/unknown/over-remaining rejection;
- cancellation residual planning before/after returns and fully returned zero residuals;
- checked arithmetic and non-blank normalized reason.

### SQLite/application integration tests

- one transaction commits headers, lines, balance increments, and exactly one movement per positive line;
- injected failure after the first line rolls back the entire request;
- same ID/same canonical payload replays with byte-equivalent persisted result;
- reordered return lines replay because canonical order is stable;
- same ID/different quantity, sale, reason, or operation conflicts without writes;
- two connections with `IMMEDIATE` transactions cannot over-return or double-restore;
- overlapping return/cancellation produces one valid serialization;
- original sale lines, snapshots, totals, and payments compare equal before/after;
- fully returned cancellation creates header and zero lines without movement/stock change;
- all SQL user values remain bound parameters, including hostile reason text.

A narrow failure-injection adapter/seam is used only in tests to fail at a named persistence step; production behavior is unchanged.

### Command and frontend tests

- Rust request parsing, tagged success/error mapping, stable codes, and no database detail leakage;
- Tauri registration for both commands and removal of their old excluded-command assertions;
- TypeScript payload casing, runtime guards, malformed response handling, and stable request ID reuse;
- Sales History includes cancelled sales and complete correction detail while preserving payment facts;
- reducer/interaction tests cover loading, validation, success reload, retry, cancellation confirmation, disabled states, accessibility labels, and inventory-only copy.

### Migration and backup tests

- v9 fixture upgrades to v10 with byte-for-byte logical equality of original rows;
- fresh database reaches v10 and reopen is idempotent;
- invalid v9 preflight does not advance version;
- v10 tables, triggers, links, and constraints reject malformed correction facts;
- backup staging supports every version through v10;
- restored v10 lifecycle history and stock/movements remain consistent.

## 10. Expected file change map

### New source files

- `src-tauri/src/infrastructure/sqlite/migrations/0010_post_sale_lifecycle.sql` — additive v10 schema, indexes, and triggers.
- `src-tauri/src/domain/sales/post_sale.rs` — pure correction plans and domain errors.
- `src-tauri/src/application/sales/post_sale.rs` — use case, repository interface, canonical identity, application DTOs/errors.
- `src-tauri/src/infrastructure/sqlite/post_sale_repository.rs` — parameterized SQLite adapter and replay reconstruction.
- `src-tauri/src/commands/post_sale.rs` — Rust/Tauri request and tagged response contracts.
- `src/commands/post-sale.ts` — TypeScript contracts, guards, and invoke adapter.
- `src-tauri/tests/post_sale_lifecycle.rs` — transaction, idempotency, concurrency, immutability, and rollback integration tests.
- `src-tauri/tests/post_sale_commands.rs` — Rust command contract tests.
- `src/commands/post-sale.test.ts` — TypeScript command adapter tests.

### Existing source files to modify

- `src-tauri/src/infrastructure/sqlite/mod.rs` — register adapter, migrate 9→10, set current version, add v10 validation.
- `src-tauri/src/infrastructure/sqlite/backup.rs` — validate the current v10 schema and lifecycle consistency.
- `src-tauri/src/domain/sales/mod.rs` — expose post-sale domain types.
- `src-tauri/src/application/sales/mod.rs` — expose post-sale use case/contracts.
- `src-tauri/src/application/sales/history.rs` — extend lifecycle/correction read models.
- `src-tauri/src/infrastructure/sqlite/sale_history_repository.rs` — include cancelled projections and additive correction detail with parameterized SQL.
- `src-tauri/src/commands/mod.rs` — register the post-sale command module.
- `src-tauri/src/commands/sales_history.rs` — serialize extended history contracts without changing error leakage policy.
- `src-tauri/src/lib.rs` — add two Tauri wrappers/registrations and update command-surface expectations.
- `src/commands/sales-history.ts` — extend guarded history types for line identity and correction details.
- `src/ui/sales/history-flow.ts` — add return/cancellation intent, pending, error, and reload actions.
- `src/ui/sales/history-screen.ts` — render lifecycle/correction history and accessible correction forms.

No capability change is expected because these are ordinary registered Tauri commands using existing managed state and no new plugin.

### Existing tests to modify

- `src-tauri/tests/sqlite_migrations.rs` — include v10 migration and preservation/constraint evidence.
- `src-tauri/tests/backup_restore.rs` — include migration 0010, version 10, and lifecycle restore preservation.
- `src-tauri/tests/sales_history.rs` — cancelled visibility, exact line identity, correction projections, immutable originals.
- `src-tauri/tests/sales_history_commands.rs` — extended tagged detail/list contracts.
- `src/commands/sales-history.test.ts` — extended IPC shape and correction detail guards.
- `src/ui/sales/history-flow.test.ts` — correction UI flow, accessibility, retry, and no-refund-language evidence.
- inline tests in `src-tauri/src/lib.rs` — register new command seam and remove `create_return_command`/`cancel_sale_command` from excluded operations (the implemented return command is deliberately named `create_sale_return_command`).

No `Cargo.toml` change is expected: `sha2`, `uuid`, `serde`, and `rusqlite` are already production dependencies. No report, export, payment, catalog, or backup UI file changes are expected.

## 11. Rollout and observability

1. Ship schema v10 and application support together.
2. Migration is automatic on open and atomic; failure leaves the database at v9.
3. Before any correction is accepted, an unreleased build can be rolled back with its migration.
4. After corrections exist, retain v10 data and disable UI/commands if rollback is required.
5. Log internal persistence failures with operation kind and opaque request/correction IDs only; do not log reason text or raw canonical payload.
6. Any future compensating stock action must be a new explicit inventory correction, never an update/delete of lifecycle facts.

## 12. Risks and mitigations

- **Review size exceeds 400 lines:** this cross-stack change is expected to exceed the configured review budget. Task planning should split bounded vertical slices and trigger `ask-on-risk` before apply.
- **Concurrent double restoration:** `IMMEDIATE` transaction plus in-transaction eligibility and database triggers serialize writers and cap cumulative restoration.
- **Ambiguous retries:** global request identity plus versioned canonical bytes and hash distinguishes replay from conflict.
- **Migration fragility:** additive tables avoid rebuilding `sales` or `inventory_movements`; v9 facts remain untouched.
- **History regressions:** lifecycle status is derived while the existing confirmed base status/index remains intact.
- **Accounting implication:** contracts and UI expose quantities only; original payment facts are labelled informational and unchanged.
- **Corrupt restored corrections:** v10 backup validation checks all forward correction links and cumulative quantities before accepting a candidate.
