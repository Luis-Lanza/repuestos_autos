import assert from "node:assert/strict";
import test from "node:test";

import { createInventoryCommands } from "./inventory.ts";

test("projects all valid alerts with the public success kind", async () => {
  const calls: unknown[] = [];
  const commands = createInventoryCommands(async (command, payload) => { calls.push({ command, payload }); return { kind: "alerts", alerts: [{ product_id: 1, product_name: "Filter", quantity: 0, classification: "out_of_stock", hidden: true }, { product_id: 2, product_name: "Belt", quantity: 2, classification: "low_stock", hidden: true }], hidden: true }; });
  assert.deepEqual(await commands.listAlerts(), { kind: "success", alerts: [{ product_id: 1, product_name: "Filter", quantity: 0, classification: "out_of_stock" }, { product_id: 2, product_name: "Belt", quantity: 2, classification: "low_stock" }] });
  assert.deepEqual(calls, [{ command: "list_inventory_alerts_command", payload: {} }]);
});

test("rejects malformed alerts atomically and checks safe integers", async () => {
  const valid = { product_id: 1, product_name: "Filter", quantity: 0, classification: "out_of_stock" };
  for (const alert of [{ ...valid, quantity: 1.5 }, { ...valid, quantity: Number.MAX_SAFE_INTEGER + 1 }, { ...valid, classification: "unknown" }, { ...valid, product_name: 1 }, { ...valid, product_id: undefined }]) {
    const commands = createInventoryCommands(async () => ({ kind: "alerts", alerts: [valid, alert] }));
    assert.deepEqual(await commands.listAlerts(), { kind: "error", code: "persistence_failure", message: "The inventory operation could not be completed." });
  }
});

test("allowlists inventory payloads and maps malformed responses to opaque errors", async () => {
  const calls: unknown[] = [];
  const commands = createInventoryCommands(async (command, payload) => {
    calls.push({ command, payload });
    return { kind: "success", request_id: "550e8400-e29b-41d4-a716-446655440211", product_id: 1, previous_quantity: 8, quantity_delta: 2, resulting_quantity: 10, occurred_at: "now", note: "delivery", internal: "never expose" };
  });
  const result = await commands.confirmStockEntry({ request_id: "550e8400-e29b-41d4-a716-446655440211", product_id: 1, quantity: 2, unit_purchase_price_centavos: 1800, sale_price_centavos: 2500, minimum_sale_price_centavos: 2000, note: "delivery", ignored: true } as never);
  assert.deepEqual(result, { kind: "success", request_id: "550e8400-e29b-41d4-a716-446655440211", product_id: 1, previous_quantity: 8, quantity_delta: 2, resulting_quantity: 10, occurred_at: "now", note: "delivery" });
  assert.deepEqual(calls, [{ command: "confirm_stock_entry_command", payload: { request: { request_id: "550e8400-e29b-41d4-a716-446655440211", product_id: 1, quantity: 2, unit_purchase_price_centavos: 1800, sale_price_centavos: 2500, minimum_sale_price_centavos: 2000, note: "delivery" } } }]);
});

test("requires positive safe prices and validates optional minimum against sale before IPC", async () => {
  let calls = 0;
  const commands = createInventoryCommands(async () => { calls += 1; return { kind: "success" }; });
  const base = { request_id: "550e8400-e29b-41d4-a716-446655440215", product_id: 1, quantity: 2, unit_purchase_price_centavos: 100, note: null };
  for (const request of [
    { ...base, unit_purchase_price_centavos: 0 },
    { ...base, unit_purchase_price_centavos: Number.MAX_SAFE_INTEGER + 1 },
    { ...base, sale_price_centavos: -1 },
    { ...base, minimum_sale_price_centavos: Number.MAX_SAFE_INTEGER + 1 },
    { ...base, sale_price_centavos: 100, minimum_sale_price_centavos: 101 },
  ]) assert.deepEqual(await commands.confirmStockEntry(request), { kind: "error", code: "invalid_price", message: "The inventory operation could not be completed." });
  assert.equal(calls, 0);
  assert.equal((await commands.confirmStockEntry(base)).kind, "error");
  assert.equal(calls, 1);
});

test("maps invoke failures and backend errors to stable inventory errors", async () => {
  const commands = createInventoryCommands(async () => { throw new Error("sqlite details"); });
  assert.deepEqual(await commands.listAlerts(), { kind: "error", code: "persistence_failure", message: "The inventory operation could not be completed." });
});

test("bounds alert error variants and rejects unknown top-level kinds", async () => {
  const known = createInventoryCommands(async () => ({ kind: "error", code: "missing_product", message: "SQL /panic native text" }));
  assert.deepEqual(await known.listAlerts(), { kind: "error", code: "missing_product", message: "The inventory operation could not be completed." });
  for (const response of [{ kind: "error", code: "unknown", message: "native" }, { kind: "alerts", alerts: "wrong" }, { kind: "unexpected" }]) {
    const commands = createInventoryCommands(async () => response);
    assert.deepEqual(await commands.listAlerts(), { kind: "error", code: "persistence_failure", message: "The inventory operation could not be completed." });
  }
});

test("rejects fractional inventory quantities before invoking IPC", async () => {
  const commands = createInventoryCommands(async () => ({ kind: "success" }));
  assert.deepEqual(await commands.confirmStockEntry({ request_id: "550e8400-e29b-41d4-a716-446655440212", product_id: 1, quantity: 1.5, unit_purchase_price_centavos: 100, note: null }), { kind: "error", code: "invalid_quantity", message: "The inventory operation could not be completed." });
  assert.deepEqual(await commands.confirmPhysicalCount({ request_id: "550e8400-e29b-41d4-a716-446655440213", product_id: 1, count: 1.5, reason: "counted" }), { kind: "error", code: "invalid_count", message: "The inventory operation could not be completed." });
});

test("preserves the stable request conflict code from inventory IPC", async () => {
  const commands = createInventoryCommands(async () => ({ kind: "error", code: "request_conflict", message: "Native storage details" }));
  assert.deepEqual(await commands.confirmStockEntry({ request_id: "550e8400-e29b-41d4-a716-446655440214", product_id: 1, quantity: 2, unit_purchase_price_centavos: 1900, note: null }), { kind: "error", code: "request_conflict", message: "The inventory operation could not be completed." });
});
