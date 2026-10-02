import assert from "node:assert/strict";
import test from "node:test";
import { createGrossProfitOperationsCommands, decodeGrossProfitOperations } from "./gross-profit-operations.ts";
const row = { occurred_at: "2024-03-10T11:30:00Z", operation_kind: "venta", sale_id: 7, return_id: null, product_name: "Filtro", sku: "FIL-1", signed_quantity: 2, negotiated_unit_price_centavos: 1250, unit_cost_snapshot_centavos: null, cost_state: "unknown", signed_gross_profit_centavos: null };
const page = (rows = [row], current = 1, total = 1) => ({ kind: "success", report: { rows, page: current, page_size: 20, total, total_pages: Math.ceil(total / 20) } });
test("requests bounded pages using inclusive local dates and decodes explicit unknown cost", async () => {
 const calls: unknown[] = []; const commands = createGrossProfitOperationsCommands(async (name, payload) => { calls.push([name, payload]); return page(); });
 assert.deepEqual(await commands.load("2024-03-10", "2024-03-10", 1), page());
 const [name, payload] = calls[0] as [string, { request: Record<string, unknown> }];
 assert.equal(name, "gross_profit_operations_command"); assert.equal(payload.request.page, 1); assert.equal(payload.request.page_size, 20);
 assert.equal(payload.request.from_utc, new Date(2024, 2, 10).toISOString()); assert.equal(payload.request.to_exclusive_utc, new Date(2024, 2, 11).toISOString());
});
test("rejects unknown fields, malformed costs, unsafe profit arithmetic, and unbounded or incomplete pages", () => {
 assert.equal(decodeGrossProfitOperations({ ...page(), extra: true }).kind, "error");
 assert.equal(decodeGrossProfitOperations(page([{ ...row, mystery: true }])).kind, "error");
 assert.equal(decodeGrossProfitOperations(page([{ ...row, unit_cost_snapshot_centavos: 0 }])).kind, "error");
 assert.equal(decodeGrossProfitOperations(page([{ ...row, signed_quantity: Number.MAX_SAFE_INTEGER + 1 }])).kind, "error");
 const known = { ...row, unit_cost_snapshot_centavos: 500, cost_state: "known", signed_gross_profit_centavos: 1499 } as const;
 assert.equal(decodeGrossProfitOperations(page([known])).kind, "error", "profit must match signed quantity × (price − cost)");
 assert.equal(decodeGrossProfitOperations(page([{ ...known, signed_quantity: Number.MAX_SAFE_INTEGER, signed_gross_profit_centavos: 1 }])).kind, "error", "unsafe intermediate arithmetic must not be rounded");
 assert.equal(decodeGrossProfitOperations(page([], 1, 1)).kind, "error", "a positive total cannot have an empty first page");
 assert.equal(decodeGrossProfitOperations(page([row], 1, 21)).kind, "error", "a non-final page must be full");
 assert.equal(decodeGrossProfitOperations(page([row, row], 2, 21)).kind, "error", "the final page must have its exact remainder");
 assert.equal(decodeGrossProfitOperations({ kind: "success", report: { ...page().report, page_size: 101 } }).kind, "error");
 assert.equal(decodeGrossProfitOperations({ kind: "error", code: "internal", message: "unsafe" }).code, "persistence_failure");
});
test("accepts exact signed known profit, negative returns and a distinct empty page", () => {
 assert.equal(decodeGrossProfitOperations(page([{ ...row, unit_cost_snapshot_centavos: 500, cost_state: "known", signed_gross_profit_centavos: 1500 }])).kind, "success");
 assert.equal(decodeGrossProfitOperations(page([{ ...row, operation_kind: "devolución", return_id: 8, signed_quantity: -1, unit_cost_snapshot_centavos: 500, cost_state: "known", signed_gross_profit_centavos: -750 }])).kind, "success");
 assert.deepEqual(decodeGrossProfitOperations(page([], 1, 0)), page([], 1, 0));
 const finalPage = { ...page([row], 2, 21), report: { ...page([row], 2, 21).report, rows: [row] } };
 assert.deepEqual(decodeGrossProfitOperations(finalPage), finalPage);
 assert.equal(decodeGrossProfitOperations({ kind: "success", report: { ...page([], 1, 0).report, page: 2 } }).kind, "error", "empty totals permit only canonical page metadata");
});
