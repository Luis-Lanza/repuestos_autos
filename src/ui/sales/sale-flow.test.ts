import assert from "node:assert/strict";
import test from "node:test";

import type { SalesBrowseProduct } from "../../commands/catalog.ts";
import {
  createSaleFlow,
  draftLineSubtotalCentavos,
  draftTotalCentavos,
  draftTotalUnits,
  effectiveDraftUnitPriceCentavos,
  formatBs,
  initialSaleState,
  parseOptionalBs,
} from "./sale-flow.ts";

test("formats integer centavos as deterministic Spanish Bs presentation", () => {
  assert.equal(formatBs(12_550), "Bs 125,50");
  assert.equal(formatBs(Number.MAX_SAFE_INTEGER), "Bs 90071992547409,91");
});

test("parses optional Spanish Bs input without floating-point authority", () => {
  assert.equal(parseOptionalBs(""), null);
  assert.equal(parseOptionalBs("0"), 0);
  assert.equal(parseOptionalBs("125,5"), 12_550);
  assert.equal(parseOptionalBs("0002,09"), 209);
  assert.equal(parseOptionalBs("90071992547409,91"), Number.MAX_SAFE_INTEGER);
});

test("rejects malformed and unsafe Bs input with the public correction", () => {
  const correction = "Ingresá un monto válido en Bs, con hasta dos decimales.";
  for (const value of ["-1", "+1", "1e2", "1.25", "1,234", "12,", " 1", "9x", "90071992547409,92"]) {
    assert.throws(() => parseOptionalBs(value), { name: "RangeError", message: correction });
  }
});

const brakePad: SalesBrowseProduct = {
  product_id: 1,
  category_id: 2,
  attribute_values: [{ definition_id: 1, label: "Material", value: "Cerámica" }],
  sku: "BP-100",
  name: "Brake Pad",
  category_name: "Brakes",
  available_quantity: 4,
  purchase_price_centavos: 1_800,
  primary_location_code: null,
  catalog_unit_price_centavos: 2_500,
  sale_price_centavos: 2_500,
  list_price_centavos: 2_500,
  minimum_sale_price_centavos: 2_500,
};

test("same-name products remain separate cart intents and confirmation identities", () => {
  const other = { ...brakePad, product_id: 2, sku: "BP-200", sale_price_centavos: 3000 };
  let state = createSaleFlow(initialSaleState, { type: "add_product", product: brakePad });
  state = createSaleFlow(state, { type: "add_product", product: other });
  state = createSaleFlow(state, { type: "line_quantity_changed", product_id: 2, value: "2" });
  assert.deepEqual(state.lines.map(({ product_id, sku, product_name, quantity, captured_unit_price_centavos }) => ({ product_id, sku, product_name, quantity, captured_unit_price_centavos })), [
    { product_id: 1, sku: "BP-100", product_name: "Brake Pad", quantity: 1, captured_unit_price_centavos: 2500 },
    { product_id: 2, sku: "BP-200", product_name: "Brake Pad", quantity: 2, captured_unit_price_centavos: 3000 },
  ]);
  assert.equal(draftTotalCentavos(state.lines), 8500);
  state = createSaleFlow(state, { type: "confirmation_started", request_id: "same-names" });
  state = createSaleFlow(state, { type: "confirmation_failed", message: "Retry" });
  const retry = createSaleFlow(state, { type: "confirmation_started", request_id: "ignored-retry" });
  assert.equal(retry.request_id, "same-names");
  assert.deepEqual(retry.lines, state.lines);
  const removed = createSaleFlow(state, { type: "remove_product", product_id: 1 });
  assert.equal(removed.request_id, null);
  assert.deepEqual(removed.lines.map((line) => [line.product_id, line.sku, line.quantity]), [[2, "BP-200", 2]]);
});

test("derives checked draft prices and totals while preserving captured facts", () => {
  const captured = createSaleFlow(initialSaleState, { type: "add_product", product: brakePad });
  const quantityTwo = createSaleFlow(captured, {
    type: "line_quantity_changed",
    product_id: 1,
    value: "2",
  });

  assert.equal(effectiveDraftUnitPriceCentavos(quantityTwo.lines[0]), 2_500);
  assert.equal(draftLineSubtotalCentavos(quantityTwo.lines[0]), 5_000);
  assert.equal(draftTotalCentavos(quantityTwo.lines), 5_000);
  assert.equal(draftTotalUnits(quantityTwo.lines), 2);
  assert.equal(draftTotalUnits([quantityTwo.lines[0], { ...quantityTwo.lines[0], product_id: 2, quantity: 3 }]), 5);
  assert.equal(quantityTwo.lines[0].captured_unit_price_centavos, 2_500);
  assert.equal(quantityTwo.lines[0].product_snapshot.available_quantity, 4);
  assert.deepEqual(quantityTwo.lines[0].product_snapshot, brakePad);
});

test("rejects unsafe draft multiplication and accumulation", () => {
  const largeLine = {
    ...createSaleFlow(initialSaleState, { type: "add_product", product: brakePad }).lines[0],
    quantity: Number.MAX_SAFE_INTEGER,
    captured_unit_price_centavos: 2,
    final_price_input: "",
  };
  const halfMaxLine = {
    ...largeLine,
    quantity: 1,
    captured_unit_price_centavos: Math.floor(Number.MAX_SAFE_INTEGER / 2) + 1,
  };

  assert.throws(() => draftLineSubtotalCentavos(largeLine), RangeError);
  assert.throws(() => draftTotalCentavos([halfMaxLine, halfMaxLine]), RangeError);
});

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
      product_snapshot: brakePad,
      captured_unit_price_centavos: 2_500,
      sale_price_centavos: 2_500,
      minimum_price_centavos: 2_500,
      final_price_input: "25,00",
    },
  ]);
});

test("uses canonical sale price for a draft and falls back to legacy prices", () => {
  const canonical = { ...brakePad, sale_price_centavos: 3_000, list_price_centavos: 2_700, catalog_unit_price_centavos: 2_500 };
  const canonicalDraft = createSaleFlow(initialSaleState, { type: "add_product", product: canonical });
  assert.equal(canonicalDraft.lines[0].captured_unit_price_centavos, 3_000);
  assert.equal(canonicalDraft.lines[0].sale_price_centavos, 3_000);
  assert.equal(canonicalDraft.lines[0].final_price_input, "30,00");

  const legacy = { ...brakePad, sale_price_centavos: undefined, list_price_centavos: 2_700 } as unknown as ProductSearchResult;
  const legacyDraft = createSaleFlow(initialSaleState, { type: "add_product", product: legacy });
  assert.equal(legacyDraft.lines[0].sale_price_centavos, 2_700);
});

test("keeps the newest catalog query when search completions arrive in reverse order", () => {
  const first = createSaleFlow(initialSaleState, {
    type: "catalog_search_started",
    query: "pastillas",
    request_id: 1,
  });
  const second = createSaleFlow(first, {
    type: "catalog_search_started",
    query: "filtros",
    request_id: 2,
  });
  const newerResult = { ...brakePad, product_id: 2, sku: "OF-200", name: "Oil Filter" };
  const completedSecond = createSaleFlow(second, {
    type: "catalog_search_succeeded",
    request_id: 2,
    results: [newerResult],
  });
  const staleFirst = createSaleFlow(completedSecond, {
    type: "catalog_search_succeeded",
    request_id: 1,
    results: [brakePad],
  });

  assert.deepEqual(staleFirst.catalog_discovery, {
    status: "results",
    query: "filtros",
    request_id: 2,
    results: [newerResult],
    error: null,
  });
  assert.equal(
    createSaleFlow(staleFirst, {
      type: "catalog_search_started",
      query: "obsolete",
      request_id: 2,
    }),
    staleFirst,
  );
});

test("distinguishes catalog loading, empty, and error while retaining the query", () => {
  const loading = createSaleFlow(initialSaleState, {
    type: "catalog_search_started",
    query: "correa",
    request_id: 7,
  });
  const empty = createSaleFlow(loading, {
    type: "catalog_search_succeeded",
    request_id: 7,
    results: [],
  });
  const retrying = createSaleFlow(empty, {
    type: "catalog_search_started",
    query: "correa",
    request_id: 8,
  });
  const failed = createSaleFlow(retrying, {
    type: "catalog_search_failed",
    request_id: 8,
    message: "No se pudo buscar en el catálogo local.",
  });

  assert.equal(initialSaleState.catalog_discovery.status, "initial");
  assert.equal(loading.catalog_discovery.status, "loading");
  assert.equal(empty.catalog_discovery.status, "empty");
  assert.deepEqual(empty.catalog_discovery.results, []);
  assert.equal(failed.catalog_discovery.status, "error");
  assert.equal(failed.catalog_discovery.query, "correa");
  assert.equal(failed.catalog_discovery.error, "No se pudo buscar en el catálogo local.");
});

test("maps backend non-positive final price to the focused field error", () => {
  const drafted = createSaleFlow(initialSaleState, { type: "add_product", product: brakePad });
  const invalid = createSaleFlow({ ...drafted, lines: [{ ...drafted.lines[0], final_price_input: "0" }] }, {
    type: "final_price_validation_failed",
    message: "The final price must be positive.",
  });

  assert.equal(invalid.price_errors[1], "El precio de venta debe ser mayor que cero.");
  assert.equal(invalid.focus_price_product_id, 1);
  assert.equal(invalid.feedback, "El precio de venta debe ser mayor que cero.");
});

test("sale draft carries no client-owned catalog revision", () => {
  const drafted = createSaleFlow(initialSaleState, { type: "add_product", product: brakePad });
  assert.equal(drafted.lines[0].captured_unit_price_centavos, 2_500);
  assert.equal("captured_revision" in drafted.lines[0], false);
  assert.equal("revision" in drafted.lines[0].product_snapshot, false);
});

test("removing a drafted line clears its intent and confirmation state", () => {
  const drafted = createSaleFlow(initialSaleState, { type: "add_product", product: brakePad });
  const failed = createSaleFlow(drafted, { type: "confirmation_failed", message: "The product price changed." });
  const removed = createSaleFlow(failed, { type: "remove_product", product_id: 1 });

  assert.deepEqual(removed.lines, []);
  assert.equal(removed.confirmation, "idle");
  assert.equal(removed.feedback, null);
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
    "Ingresá una cantidad entera mayor que cero.",
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

test("keeps request identity for same-value sale actions", () => {
  const withLine = createSaleFlow(initialSaleState, { type: "add_product", product: brakePad });
  let state = createSaleFlow(withLine, { type: "confirmation_started", request_id: "sale-request-exact" });
  state = createSaleFlow(state, { type: "confirmation_failed", message: "Retry the sale." });

  for (const action of [
    { type: "add_product", product: brakePad },
    { type: "remove_product", product_id: 99 },
    { type: "line_quantity_changed", product_id: 1, value: "1" },
    { type: "line_final_price_changed", product_id: 1, value: "25,00" },
    { type: "payment_changed", field: "amount_tendered_centavos" as const, value: "" },
  ] as const) {
    const repeated = createSaleFlow(state, action);
    assert.equal(repeated.request_id, "sale-request-exact", action.type);
    state = repeated;
  }
});

test("invalidates request identity when the final sale price changes", () => {
  const drafted = createSaleFlow(initialSaleState, { type: "add_product", product: brakePad });
  const failed = createSaleFlow(
    createSaleFlow(drafted, { type: "confirmation_started", request_id: "sale-request-price" }),
    { type: "confirmation_failed", message: "Retry the sale." },
  );
  const changed = createSaleFlow(failed, { type: "line_final_price_changed", product_id: 1, value: "26,00" });
  assert.equal(changed.request_id, null);
  assert.equal(changed.lines[0].final_price_input, "26,00");
});

test("replaces request identity for every changed sale payload", () => {
  const firstRequestId = "550e8400-e29b-41d4-a716-446655440070";
  const secondRequestId = "550e8400-e29b-41d4-a716-446655440071";
  const thirdRequestId = "550e8400-e29b-41d4-a716-446655440072";
  const otherProduct = { ...brakePad, product_id: 2, sku: "OF-200", name: "Oil Filter" };
  const withLine = createSaleFlow(initialSaleState, { type: "add_product", product: brakePad });
  const failed = createSaleFlow(
    createSaleFlow(withLine, { type: "confirmation_started", request_id: firstRequestId }),
    { type: "confirmation_failed", message: "Retry the sale." },
  );

  const changedQuantity = createSaleFlow(failed, { type: "line_quantity_changed", product_id: 1, value: "2" });
  assert.equal(changedQuantity.request_id, null);
  const failedQuantity = createSaleFlow(
    createSaleFlow(changedQuantity, { type: "confirmation_started", request_id: secondRequestId }),
    { type: "confirmation_failed", message: "Retry the sale." },
  );
  const changedPayment = createSaleFlow(failedQuantity, { type: "payment_changed", field: "amount_tendered_centavos", value: "2500" });
  assert.equal(changedPayment.request_id, null);
  const failedPayment = createSaleFlow(
    createSaleFlow(changedPayment, { type: "confirmation_started", request_id: thirdRequestId }),
    { type: "confirmation_failed", message: "Retry the sale." },
  );
  const addedLine = createSaleFlow(failedPayment, { type: "add_product", product: otherProduct });
  assert.equal(addedLine.request_id, null);
  const failedAdd = createSaleFlow(
    createSaleFlow(addedLine, { type: "confirmation_started", request_id: firstRequestId }),
    { type: "confirmation_failed", message: "Retry the sale." },
  );
  const removedLine = createSaleFlow(failedAdd, { type: "remove_product", product_id: 2 });
  assert.equal(removedLine.request_id, null);

});
