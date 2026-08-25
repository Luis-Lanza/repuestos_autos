import assert from "node:assert/strict";
import test from "node:test";

import { createSaleFlow, initialSaleState } from "./sale-flow.ts";
import type { ProductSearchResult } from "../../commands/catalog.ts";

const brakePad: ProductSearchResult = {
  product_id: 1,
  sku: "BP-100",
  name: "Brake Pad",
  category_name: "Brakes",
  available_quantity: 4,
  catalog_unit_price_centavos: 2_500,
};

test("adds active search results with catalog-price guidance and whole-unit defaults", () => {
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
      catalog_unit_price_centavos: 2_500,
    },
  ]);
});

test("keeps cash, QR, and mixed tender inputs as draft strings", () => {
  const cash = createSaleFlow(initialSaleState, {
    type: "tendered_cash_changed",
    value: "3000",
  });
  const qr = createSaleFlow(initialSaleState, {
    type: "qr_applied_changed",
    value: "2500",
  });
  const mixed = createSaleFlow(cash, {
    type: "qr_applied_changed",
    value: "1000",
  });

  assert.deepEqual(cash.payment, {
    amount_tendered_centavos: "3000",
    qr_applied_centavos: "",
  });
  assert.deepEqual(qr.payment, {
    amount_tendered_centavos: "",
    qr_applied_centavos: "2500",
  });
  assert.deepEqual(mixed.payment, {
    amount_tendered_centavos: "3000",
    qr_applied_centavos: "1000",
  });
});

test("rejects invalid quantity without editable price or derived-cash actions", () => {
  const withLine = createSaleFlow(initialSaleState, {
    type: "add_product",
    product: brakePad,
  });
  const invalidQuantity = createSaleFlow(withLine, {
    type: "line_quantity_changed",
    product_id: 1,
    value: "1.5",
  });

  assert.equal(
    invalidQuantity.feedback,
    "Quantity must be a positive whole number.",
  );
  assert.deepEqual(Object.keys(invalidQuantity.lines[0]), [
    "product_id",
    "sku",
    "product_name",
    "quantity",
    "catalog_unit_price_centavos",
  ]);
  assert.equal("payments" in invalidQuantity, false);
});

test("retains one request ID through failed retries and starts a new intent after success or discard", () => {
  const firstRequestId = "550e8400-e29b-41d4-a716-446655440060";
  const secondRequestId = "550e8400-e29b-41d4-a716-446655440061";
  const thirdRequestId = "550e8400-e29b-41d4-a716-446655440062";
  const pending = createSaleFlow(initialSaleState, {
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
  assert.equal(afterSuccess.persisted_summary, null);
  assert.deepEqual(afterSuccess.payment, {
    amount_tendered_centavos: "",
    qr_applied_centavos: "",
  });
  assert.equal(newIntent.request_id, thirdRequestId);
});
