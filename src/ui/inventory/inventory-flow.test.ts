import assert from "node:assert/strict";
import test from "node:test";

import { createInventoryFlow, initialInventoryState, projectedBalance } from "./inventory-flow.ts";

test("retains one request through failures, reports stale projections, refreshes alerts, and resets intents", () => {
  const selected = createInventoryFlow(initialInventoryState, { type: "product_selected", product: { product_id: 1, name: "Filter", available_quantity: 8 } });
  const pending = createInventoryFlow(selected, { type: "confirmation_started", request_id: "550e8400-e29b-41d4-a716-446655440221" });
  const retry = createInventoryFlow(createInventoryFlow(pending, { type: "confirmation_failed", message: "Retry." }), { type: "confirmation_started", request_id: "550e8400-e29b-41d4-a716-446655440222" });
  const success = createInventoryFlow(retry, { type: "confirmation_succeeded", result: { request_id: pending.request_id!, previous_quantity: 10, resulting_quantity: 12 } });
  const refreshed = createInventoryFlow(success, { type: "alerts_refreshed", alerts: [{ product_id: 2, product_name: "Spark plug", quantity: 0, classification: "out_of_stock" }] });
  assert.equal(retry.request_id, pending.request_id);
  assert.equal(success.advisory_notice, "Stock changed after the preview.");
  assert.equal(refreshed.alerts[0].classification, "out_of_stock");
  assert.deepEqual(createInventoryFlow(refreshed, { type: "discard" }), initialInventoryState);
});

test("projects an explicit physical zero but not a blank count", () => {
  const selected = createInventoryFlow(initialInventoryState, { type: "product_selected", product: { product_id: 1, name: "Filter", available_quantity: 8 } });
  const physical = createInventoryFlow(selected, { type: "operation_changed", operation: "physical_count" });
  assert.equal(projectedBalance(physical), null);
  assert.equal(projectedBalance(createInventoryFlow(physical, { type: "physical_count_changed", value: "0" })), 0);
});

test("keeps the failed request identity when an inventory action repeats the same value", () => {
  const product = { product_id: 1, name: "Filter", available_quantity: 8 };
  let state = createInventoryFlow(initialInventoryState, { type: "product_selected", product });
  state = createInventoryFlow(state, { type: "entry_quantity_changed", value: "4" });
  state = createInventoryFlow(state, { type: "note_changed", value: "Conteo de depósito" });
  state = createInventoryFlow(state, { type: "confirmation_started", request_id: "inventory-request-exact" });
  state = createInventoryFlow(state, { type: "confirmation_failed", message: "Retry." });

  for (const action of [
    { type: "product_selected", product },
    { type: "operation_changed", operation: "stock_entry" as const },
    { type: "entry_quantity_changed", value: "4" },
    { type: "physical_count_changed", value: "" },
    { type: "note_changed", value: "Conteo de depósito" },
    { type: "reason_changed", value: "" },
  ] as const) {
    const repeated = createInventoryFlow(state, action);
    assert.equal(repeated, state, action.type);
    assert.equal(repeated.request_id, "inventory-request-exact");
  }
});

test("replaces request identity for every changed inventory payload", () => {
  const product = { product_id: 1, name: "Filter", available_quantity: 8 };
  const otherProduct = { product_id: 2, name: "Oil filter", available_quantity: 3 };
  let state = createInventoryFlow(initialInventoryState, { type: "product_selected", product });
  const fail = (current: typeof initialInventoryState, request_id: string) => createInventoryFlow(
    createInventoryFlow(current, { type: "confirmation_started", request_id }),
    { type: "confirmation_failed", message: "Retry." },
  );

  state = fail(state, "inventory-request-1");
  state = createInventoryFlow(state, { type: "product_selected", product: otherProduct });
  assert.equal(state.request_id, null);
  state = fail(state, "inventory-request-2");
  state = createInventoryFlow(state, { type: "operation_changed", operation: "physical_count" });
  assert.equal(state.request_id, null);
  state = fail(state, "inventory-request-3");
  state = createInventoryFlow(state, { type: "entry_quantity_changed", value: "4" });
  assert.equal(state.request_id, null);
  state = fail(state, "inventory-request-4");
  state = createInventoryFlow(state, { type: "physical_count_changed", value: "2" });
  assert.equal(state.request_id, null);
  state = fail(state, "inventory-request-5");
  state = createInventoryFlow(state, { type: "note_changed", value: "Conteo de depósito" });
  assert.equal(state.request_id, null);
  state = fail(state, "inventory-request-6");
  state = createInventoryFlow(state, { type: "reason_changed", value: "Ajuste de inventario" });
  assert.equal(state.request_id, null);
});
