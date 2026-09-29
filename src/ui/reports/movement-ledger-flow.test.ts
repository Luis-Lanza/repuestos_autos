import assert from "node:assert/strict";
import test from "node:test";
import { createMovementLedgerFlow, initialLedgerState, currentMonthFilters } from "./movement-ledger-flow.ts";
import type { MovementLedgerRow } from "../../commands/movement-ledger.ts";
const row: MovementLedgerRow = { movement_id: 1, occurred_at: "2025-01-02 10:00:00", product_id: 2, product_name: "Filtro", product_sku: "FLT", movement_type: "stock_entry", quantity_delta: 2, resulting_quantity: null, reason: null, note: null, sale_id: null, sale_line_id: null };
const success = (rows = [row], has_more = false) => ({ kind: "success" as const, rows, page: 1, page_size: 50, has_more });
test("default is local calendar month and flow ignores stale list/export responses", () => {
 const initial = initialLedgerState(new Date(2025, 2, 20));
 assert.deepEqual(currentMonthFilters(new Date(2025, 2, 20)), { from: "2025-03-01", to: "2025-03-31", product_id: null, movement_type: null });
 let state = createMovementLedgerFlow(initial, { type: "filters_applied", request_id: 1 });
 state = createMovementLedgerFlow(state, { type: "filters_applied", request_id: 2 });
 state = createMovementLedgerFlow(state, { type: "list_succeeded", request_id: 1, response: success() });
 assert.equal(state.status, "loading");
 state = createMovementLedgerFlow(state, { type: "list_succeeded", request_id: 2, response: success([], false) });
 assert.equal(state.status, "empty");
 state = createMovementLedgerFlow(state, { type: "export_started", request_id: 3 });
 state = createMovementLedgerFlow(state, { type: "export_started", request_id: 4 });
 state = createMovementLedgerFlow(state, { type: "export_finished", request_id: 3, response: { kind: "success" } });
 assert.equal(state.export_status, "pending");
});
test("filters, paging, and retry retain applied intent and ignore stale failures", () => {
 let state = initialLedgerState(new Date(2025, 0, 1));
 state = createMovementLedgerFlow(state, { type: "draft_changed", filters: { ...state.draft, product_id: 9, movement_type: "sale" } });
 state = createMovementLedgerFlow(state, { type: "filters_applied", request_id: 1 });
 assert.equal(state.applied.product_id, 9);
 assert.equal(state.applied.movement_type, "sale");
 state = createMovementLedgerFlow(state, { type: "list_succeeded", request_id: 1, response: success([row], true) });
 state = createMovementLedgerFlow(state, { type: "page_requested", request_id: 2, page: 2 });
 state = createMovementLedgerFlow(state, { type: "list_failed", request_id: 1 });
 assert.equal(state.status, "loading");
 assert.equal(state.applied.product_id, 9);
 assert.equal(state.page, 2);
});
test("product options include archived products as read-only selector values", () => {
 let state = initialLedgerState();
 state = createMovementLedgerFlow(state, { type: "products_started", request_id: 1 });
 state = createMovementLedgerFlow(state, { type: "products_finished", request_id: 1, response: { kind: "success", products: [{ product_id: 3, product_name: "Archivado", product_sku: "ARC", active: false }] } });
 assert.equal(state.products[0].active, false);
});
