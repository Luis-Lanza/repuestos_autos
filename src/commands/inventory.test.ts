import assert from "node:assert/strict";
import test from "node:test";

import { createInventoryCommands } from "./inventory.ts";

test("allowlists inventory payloads and maps malformed responses to opaque errors", async () => {
  const calls: unknown[] = [];
  const commands = createInventoryCommands(async (command, payload) => {
    calls.push({ command, payload });
    return { kind: "success", request_id: "550e8400-e29b-41d4-a716-446655440211", product_id: 1, previous_quantity: 8, quantity_delta: 2, resulting_quantity: 10, occurred_at: "now", internal: "never expose" };
  });
  const result = await commands.confirmStockEntry({ request_id: "550e8400-e29b-41d4-a716-446655440211", product_id: 1, quantity: 2, note: "delivery", ignored: true } as never);
  assert.deepEqual(result, { kind: "success", request_id: "550e8400-e29b-41d4-a716-446655440211", product_id: 1, previous_quantity: 8, quantity_delta: 2, resulting_quantity: 10, occurred_at: "now" });
  assert.deepEqual(calls, [{ command: "confirm_stock_entry_command", payload: { request: { request_id: "550e8400-e29b-41d4-a716-446655440211", product_id: 1, quantity: 2, note: "delivery" } } }]);
});

test("maps invoke failures and backend errors to stable inventory errors", async () => {
  const commands = createInventoryCommands(async () => { throw new Error("sqlite details"); });
  assert.deepEqual(await commands.listAlerts(), { kind: "error", code: "persistence_failure", message: "The inventory operation could not be completed." });
});
