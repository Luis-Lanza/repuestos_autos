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
  minimum_unit_price_centavos: 2_500,
};

test("adds active search results with minimum-price and whole-unit defaults", () => {
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
      negotiated_unit_price_centavos: 2_500,
      minimum_unit_price_centavos: 2_500,
    },
  ]);
});

test("draft edits give local integer feedback and discard has no command effect", () => {
  const withLine = createSaleFlow(initialSaleState, {
    type: "add_product",
    product: brakePad,
  });
  const invalidQuantity = createSaleFlow(withLine, {
    type: "line_quantity_changed",
    product_id: 1,
    value: "1.5",
  });
  const validPrice = createSaleFlow(invalidQuantity, {
    type: "line_price_changed",
    product_id: 1,
    value: "2750",
  });
  const discarded = createSaleFlow(validPrice, { type: "discard" });

  assert.equal(
    invalidQuantity.feedback,
    "Quantity must be a positive whole number.",
  );
  assert.equal(validPrice.lines[0].negotiated_unit_price_centavos, 2_750);
  assert.deepEqual(discarded.lines, []);
  assert.deepEqual(discarded.payments, []);
});

test("retains one request ID through pending and error retry, then replaces it after discard", () => {
  const pending = createSaleFlow(initialSaleState, {
    type: "confirmation_started",
    request_id: "550e8400-e29b-41d4-a716-446655440060",
  });
  const retry = createSaleFlow(
    createSaleFlow(pending, {
      type: "confirmation_failed",
      message: "Retry the sale.",
    }),
    {
      type: "confirmation_started",
      request_id: "550e8400-e29b-41d4-a716-446655440061",
    },
  );
  const newIntent = createSaleFlow(createSaleFlow(retry, { type: "discard" }), {
    type: "confirmation_started",
    request_id: "550e8400-e29b-41d4-a716-446655440061",
  });

  assert.equal(retry.request_id, "550e8400-e29b-41d4-a716-446655440060");
  assert.equal(newIntent.request_id, "550e8400-e29b-41d4-a716-446655440061");
});

test("keeps payment draft state for cash and QR", () => {
  const cash = createSaleFlow(initialSaleState, {
    type: "cash_payment_changed",
    amount_applied_centavos: "2500",
    amount_tendered_centavos: "3000",
    change_given_centavos: "500",
  });
  const mixed = createSaleFlow(cash, {
    type: "qr_payment_changed",
    amount_applied_centavos: "1000",
  });

  assert.deepEqual(mixed.payments, [
    {
      method: "cash",
      amount_applied_centavos: "2500",
      amount_tendered_centavos: "3000",
      change_given_centavos: "500",
    },
    { method: "qr", amount_applied_centavos: "1000" },
  ]);
});
