import assert from "node:assert/strict";
import test from "node:test";

import type { ProductSearchResult } from "../../commands/catalog.ts";
import {
  createSaleFlow,
  draftLineSubtotalCentavos,
  draftTotalCentavos,
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

const brakePad: ProductSearchResult = {
  product_id: 1,
  sku: "BP-100",
  name: "Brake Pad",
  category_name: "Brakes",
  available_quantity: 4,
  catalog_unit_price_centavos: 2_500,
  revision: 0,
};

test("derives checked draft prices and totals while preserving captured facts", () => {
  const captured = createSaleFlow(initialSaleState, { type: "add_product", product: brakePad });
  const stale = createSaleFlow(captured, {
    type: "stale_price_detected",
    product_id: 1,
    current_unit_price_centavos: 2_750,
    current_revision: 2,
  });
  const acknowledged = createSaleFlow(stale, {
    type: "acknowledge_stale_price",
    product_id: 1,
    current_unit_price_centavos: 2_750,
    current_revision: 2,
  });
  const quantityTwo = createSaleFlow(acknowledged, {
    type: "line_quantity_changed",
    product_id: 1,
    value: "2",
  });

  assert.equal(effectiveDraftUnitPriceCentavos(quantityTwo.lines[0]), 2_750);
  assert.equal(draftLineSubtotalCentavos(quantityTwo.lines[0]), 5_500);
  assert.equal(draftTotalCentavos(quantityTwo.lines), 5_500);
  assert.equal(quantityTwo.lines[0].captured_unit_price_centavos, 2_500);
  assert.equal(quantityTwo.lines[0].captured_revision, 0);
});

test("rejects unsafe draft multiplication and accumulation", () => {
  const largeLine = {
    ...createSaleFlow(initialSaleState, { type: "add_product", product: brakePad }).lines[0],
    quantity: Number.MAX_SAFE_INTEGER,
    captured_unit_price_centavos: 2,
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
      captured_unit_price_centavos: 2_500,
      captured_revision: 0,
    },
  ]);
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

test("requires acknowledgement for the exact current stale price and revision", () => {
  const drafted = createSaleFlow(initialSaleState, { type: "add_product", product: brakePad });
  const stale = createSaleFlow(drafted, { type: "stale_price_detected", product_id: 1, current_unit_price_centavos: 2700, current_revision: 2 });
  const acknowledged = createSaleFlow(stale, { type: "acknowledge_stale_price", product_id: 1, current_unit_price_centavos: 2700, current_revision: 2 });
  const changedAgain = createSaleFlow(acknowledged, { type: "stale_price_detected", product_id: 1, current_unit_price_centavos: 2800, current_revision: 3 });
  assert.equal(acknowledged.lines[0].acknowledged_revision, 2);
  assert.equal(changedAgain.lines[0].acknowledged_revision, undefined);
  assert.equal(changedAgain.confirmation, "error");
});

test("removing the stale-price line clears its obsolete confirmation block", () => {
  const drafted = createSaleFlow(initialSaleState, { type: "add_product", product: brakePad });
  const stale = createSaleFlow(drafted, {
    type: "stale_price_detected",
    product_id: 1,
    current_unit_price_centavos: 2_700,
    current_revision: 3,
  });
  const removed = createSaleFlow(stale, { type: "remove_product", product_id: 1 });

  assert.deepEqual(removed.lines, []);
  assert.equal(removed.stale_price, null);
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
