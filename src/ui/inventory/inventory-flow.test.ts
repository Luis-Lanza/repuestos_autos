import assert from "node:assert/strict";
import test from "node:test";

import { createInventoryFlow, initialInventoryState } from "./inventory-flow.ts";

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
