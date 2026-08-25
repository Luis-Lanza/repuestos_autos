import assert from "node:assert/strict";
import test from "node:test";

import type { ProductSearchResult } from "../../commands/catalog.ts";
import { createSaleFlow, initialSaleState } from "./sale-flow.ts";

const brakePad: ProductSearchResult = {
  product_id: 1,
  sku: "BP-100",
  name: "Brake Pad",
  category_name: "Brakes",
  available_quantity: 4,
  catalog_unit_price_centavos: 2_500,
};

test("adds active search results as quantity-only sale intent", () => {
  const state = createSaleFlow(initialSaleState, {
    type: "search_succeeded",
    results: [brakePad],
  });
  const withLine = createSaleFlow(state, {
    type: "add_product",
    product: brakePad,
  });

  assert.deepEqual(withLine.lines, [
    {
      product_id: 1,
      sku: "BP-100",
      product_name: "Brake Pad",
      quantity: 1,
    },
  ]);
});

test("draft edits give local quantity feedback and discard clears reduced payment intent", () => {
  const withLine = createSaleFlow(initialSaleState, {
    type: "add_product",
    product: brakePad,
  });
  const invalidQuantity = createSaleFlow(withLine, {
    type: "line_quantity_changed",
    product_id: 1,
    value: "1.5",
  });
  const payment = createSaleFlow(invalidQuantity, {
    type: "payment_changed",
    field: "amount_tendered_centavos",
    value: "2750",
  });
  const discarded = createSaleFlow(payment, { type: "discard" });

  assert.equal(
    invalidQuantity.feedback,
    "Quantity must be a positive whole number.",
  );
  assert.equal(payment.payment.amount_tendered_centavos, "2750");
  assert.deepEqual(discarded.lines, []);
  assert.deepEqual(discarded.payment, {
    amount_tendered_centavos: "",
    qr_applied_centavos: "",
  });
});

test("retains request and draft intent through failed retries", () => {
  const firstRequestId = "550e8400-e29b-41d4-a716-446655440060";
  const secondRequestId = "550e8400-e29b-41d4-a716-446655440061";
  const thirdRequestId = "550e8400-e29b-41d4-a716-446655440062";
  const withLine = createSaleFlow(initialSaleState, {
    type: "add_product",
    product: brakePad,
  });
  const withPayment = createSaleFlow(withLine, {
    type: "payment_changed",
    field: "qr_applied_centavos",
    value: "2500",
  });
  const pending = createSaleFlow(withPayment, {
    type: "confirmation_started",
    request_id: firstRequestId,
  });
  const retry = createSaleFlow(
    createSaleFlow(pending, {
      type: "confirmation_failed",
      message: "Retry the sale.",
    }),
    { type: "confirmation_started", request_id: secondRequestId },
  );
  const succeeded = createSaleFlow(retry, {
    type: "confirmation_succeeded",
    summary: { request_id: firstRequestId } as never,
  });
  const afterSuccess = createSaleFlow(succeeded, { type: "discard" });
  const newIntent = createSaleFlow(afterSuccess, {
    type: "confirmation_started",
    request_id: thirdRequestId,
  });

  assert.equal(retry.request_id, firstRequestId);
  assert.deepEqual(retry.lines, withLine.lines);
  assert.deepEqual(retry.payment, withPayment.payment);
  assert.equal(afterSuccess.persisted_summary, null);
  assert.equal(newIntent.request_id, thirdRequestId);
});
