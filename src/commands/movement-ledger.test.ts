import assert from "node:assert/strict";
import test from "node:test";
import { createMovementLedgerCommands, type MovementLedgerFilters } from "./movement-ledger.ts";
const filters: MovementLedgerFilters = { from: "2025-03-01", to: "2025-03-31", product_id: 7, movement_type: "sale" };
const row = { movement_id: 1, occurred_at: "2025-03-02 10:00:00", product_id: 7, product_name: "Filtro", product_sku: "F-1", movement_type: "sale", quantity_delta: -2, resulting_quantity: null, reason: null, note: "nota", sale_id: 4, sale_line_id: 2 };
test("adapter decodes ledger results and sends explicit inclusive date range and applied filters", async () => {
 const calls: Array<[string, Record<string, unknown> | undefined]> = [];
 const commands = createMovementLedgerCommands(async (name, payload) => { calls.push([name, payload]); return { kind: "success", rows: [row], page: 2, page_size: 50, has_more: true }; });
 const response = await commands.list(filters, 2);
 assert.equal(response.kind, "success"); if (response.kind !== "success") return;
 assert.equal(response.rows[0].resulting_quantity, null); assert.equal(response.page, 2); assert.equal(response.has_more, true);
 const request = calls[0][1]?.request as Record<string, unknown>;
 assert.equal(request.product_id, 7); assert.equal(request.movement_type, "sale"); assert.equal(request.page, 2);
 assert.equal(request.from_utc, new Date(2025, 2, 1).toISOString());
 assert.equal(request.to_exclusive_utc, new Date(2025, 3, 1).toISOString());
});
test("product option adapter sends bounded query/page inputs, accepts archived entries, and rejects oversized responses", async () => {
 const calls: Array<[string, Record<string, unknown> | undefined]> = [];
 const commands = createMovementLedgerCommands(async (name, payload) => { calls.push([name, payload]); return { kind: "success", products: [{ product_id: 2, product_name: "Archivado", product_sku: "ARC", active: false }], page: 2, page_size: 20, has_more: true }; });
 const response = await commands.productOptions(" filtro ", 2);
 assert.equal(response.kind, "success"); if (response.kind === "success") { assert.equal(response.products[0].active, false); assert.equal(response.has_more, true); }
 assert.deepEqual(calls, [["list_movement_ledger_product_options_command", { request: { query: " filtro ", page: 2, page_size: 20 } }]]);
 const oversized = createMovementLedgerCommands(async () => ({ kind: "success", products: Array.from({ length: 21 }, (_, index) => ({ product_id: index + 1, product_name: "P", product_sku: "S", active: true })), page: 1, page_size: 20, has_more: false }));
 assert.equal((await oversized.productOptions("x")).kind, "error");
 const malformed = createMovementLedgerCommands(async () => ({ kind: "success", rows: [{ ...row, quantity_delta: "-2" }], page: 1, page_size: 50, has_more: false }));
 assert.equal((await malformed.list(filters, 1)).kind, "error");
});
test("export decodes success, cancellation, and malformed values", async () => {
 for (const value of [{ kind: "success" }, { kind: "cancelled" }, { kind: "unexpected" }]) {
  const commands = createMovementLedgerCommands(async () => value);
  const response = await commands.export(filters);
  assert.equal(response.kind, value.kind === "unexpected" ? "error" : value.kind);
 }
});
