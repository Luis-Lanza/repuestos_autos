import assert from "node:assert/strict";
import test from "node:test";

import { createConfirmSaleCommand } from "./confirm-sale.ts";

test("sends only the reduced confirmation payload", async () => {
  const calls: unknown[] = [];
  const confirmSale = createConfirmSaleCommand(async (command, payload) => {
    calls.push({ command, payload });
    return {
      kind: "error",
      code: "invalid_payment",
      message: "Payment values are invalid.",
    };
  });

  const result = await confirmSale({
    request_id: "550e8400-e29b-41d4-a716-446655440050",
    lines: [{ product_id: 1, quantity: 2, captured_unit_price_centavos: 2_500, captured_revision: 0 }],
    payment: {
      amount_tendered_centavos: 3_000,
      qr_applied_centavos: 2_000,
    },
  });

  assert.deepEqual(result, {
    kind: "error",
    code: "invalid_payment",
    message: "Payment values are invalid.",
  });
  assert.deepEqual(calls, [
    {
      command: "confirm_sale_command",
      payload: {
        request: {
          request_id: "550e8400-e29b-41d4-a716-446655440050",
          lines: [{ product_id: 1, quantity: 2, captured_unit_price_centavos: 2_500, captured_revision: 0 }],
          payment: {
            amount_tendered_centavos: 3_000,
            qr_applied_centavos: 2_000,
          },
        },
      },
    },
  ]);
});

test("forwards captured facts and an exact stale-price acknowledgement", async () => {
  const calls: unknown[] = [];
  const confirmSale = createConfirmSaleCommand(async (command, payload) => {
    calls.push({ command, payload });
    return { kind: "stale_catalog_record", product_id: 1, current_unit_price_centavos: 2700, current_revision: 2 };
  });
  const result = await confirmSale({
    request_id: "550e8400-e29b-41d4-a716-446655440057",
    lines: [{ product_id: 1, quantity: 1, captured_unit_price_centavos: 2500, captured_revision: 0, acknowledged_price_centavos: 2700, acknowledged_revision: 2 }],
    payment: { amount_tendered_centavos: null, qr_applied_centavos: 2700 },
  });
  assert.deepEqual(result, { kind: "stale_catalog_record", product_id: 1, current_unit_price_centavos: 2700, current_revision: 2 });
  assert.deepEqual(calls, [{ command: "confirm_sale_command", payload: { request: { request_id: "550e8400-e29b-41d4-a716-446655440057", lines: [{ product_id: 1, quantity: 1, captured_unit_price_centavos: 2500, captured_revision: 0, acknowledged_price_centavos: 2700, acknowledged_revision: 2 }], payment: { amount_tendered_centavos: null, qr_applied_centavos: 2700 } } } }]);
});

test("preserves nullable payment inputs for cash, QR, and mixed confirmation", async () => {
  const captured: unknown[] = [];
  const confirmSale = createConfirmSaleCommand(async (_command, payload) => {
    captured.push(payload);
    return {
      kind: "error",
      code: "invalid_payment",
      message: "Payment values are invalid.",
    };
  });

  await confirmSale({
    request_id: "550e8400-e29b-41d4-a716-446655440051",
    lines: [{ product_id: 1, quantity: 1, captured_unit_price_centavos: 2_500, captured_revision: 0 }],
    payment: { amount_tendered_centavos: 2_500, qr_applied_centavos: null },
  });
  await confirmSale({
    request_id: "550e8400-e29b-41d4-a716-446655440052",
    lines: [{ product_id: 2, quantity: 1, captured_unit_price_centavos: 2_500, captured_revision: 0 }],
    payment: { amount_tendered_centavos: null, qr_applied_centavos: 2_500 },
  });
  await confirmSale({
    request_id: "550e8400-e29b-41d4-a716-446655440053",
    lines: [{ product_id: 3, quantity: 1, captured_unit_price_centavos: 2_500, captured_revision: 0 }],
    payment: { amount_tendered_centavos: 1_500, qr_applied_centavos: 1_000 },
  });

  assert.deepEqual(captured, [
    {
      request: {
        request_id: "550e8400-e29b-41d4-a716-446655440051",
        lines: [{ product_id: 1, quantity: 1, captured_unit_price_centavos: 2_500, captured_revision: 0 }],
        payment: { amount_tendered_centavos: 2_500, qr_applied_centavos: null },
      },
    },
    {
      request: {
        request_id: "550e8400-e29b-41d4-a716-446655440052",
        lines: [{ product_id: 2, quantity: 1, captured_unit_price_centavos: 2_500, captured_revision: 0 }],
        payment: { amount_tendered_centavos: null, qr_applied_centavos: 2_500 },
      },
    },
    {
      request: {
        request_id: "550e8400-e29b-41d4-a716-446655440053",
        lines: [{ product_id: 3, quantity: 1, captured_unit_price_centavos: 2_500, captured_revision: 0 }],
        payment: {
          amount_tendered_centavos: 1_500,
          qr_applied_centavos: 1_000,
        },
      },
    },
  ]);
});

test("projects complete sale summaries and both payment variants", async () => {
  const confirmSale = createConfirmSaleCommand(async () => ({ kind: "success", sale_id: 7, request_id: "550e8400-e29b-41d4-a716-446655440054", status: "confirmed", confirmed_at: "2026-03-08T12:00:00Z", outcome: "confirmed", lines: [{ product_id: 1, sku: "SKU-1", product_name: "Filter", quantity: 2, unit_price_centavos: 2750, line_total_centavos: 5500, internal: "hidden" }], payments: [{ method: "cash", amount_applied_centavos: 3000, amount_tendered_centavos: 3500, change_given_centavos: 500, internal: "hidden" }, { method: "qr", amount_applied_centavos: 2500, internal: "hidden" }], total_centavos: 5500, internal: "hidden" }));
  const result = await confirmSale({ request_id: "550e8400-e29b-41d4-a716-446655440054", lines: [{ product_id: 1, quantity: 2, captured_unit_price_centavos: 2500, captured_revision: 0 }], payment: { amount_tendered_centavos: 3500, qr_applied_centavos: 2500 } });
  assert.deepEqual(result, { kind: "success", sale_id: 7, request_id: "550e8400-e29b-41d4-a716-446655440054", status: "confirmed", confirmed_at: "2026-03-08T12:00:00Z", outcome: "confirmed", lines: [{ product_id: 1, sku: "SKU-1", product_name: "Filter", quantity: 2, unit_price_centavos: 2750, line_total_centavos: 5500 }], payments: [{ method: "cash", amount_applied_centavos: 3000, amount_tendered_centavos: 3500, change_given_centavos: 500 }, { method: "qr", amount_applied_centavos: 2500 }], total_centavos: 5500 });
});

test("returns persisted authoritative summaries and backend errors unchanged", async () => {
  const confirmSale = createConfirmSaleCommand(async () => ({
    kind: "success",
    sale_id: 7,
    request_id: "550e8400-e29b-41d4-a716-446655440054",
    status: "confirmed",
    confirmed_at: "2026-03-08T12:00:00Z",
    outcome: "confirmed",
    lines: [
      {
        product_id: 1,
        sku: "SKU-1",
        product_name: "Filter",
        quantity: 2,
        unit_price_centavos: 2_750,
        line_total_centavos: 5_500,
      },
    ],
    payments: [{ method: "qr", amount_applied_centavos: 5_500 }],
    total_centavos: 5_500,
  }));

  const result = await confirmSale({
    request_id: "550e8400-e29b-41d4-a716-446655440054",
    lines: [{ product_id: 1, quantity: 2, captured_unit_price_centavos: 2_500, captured_revision: 0 }],
    payment: { amount_tendered_centavos: null, qr_applied_centavos: 5_500 },
  });

  assert.equal(result.kind, "success");
  assert.equal(result.lines[0].unit_price_centavos, 2_750);
  assert.equal(result.payments[0].method, "qr");
});

test("decodes request conflicts without coercing them to persistence failures", async () => {
  const confirmSale = createConfirmSaleCommand(async () => ({
    kind: "error",
    code: "request_conflict",
    message: "SQLite digest details",
  }));

  assert.deepEqual(
    await confirmSale({
      request_id: "550e8400-e29b-41d4-a716-446655440055",
      lines: [{ product_id: 1, quantity: 1, captured_unit_price_centavos: 2500, captured_revision: 0 }],
      payment: { amount_tendered_centavos: null, qr_applied_centavos: 2500 },
    }),
    {
      kind: "error",
      code: "request_conflict",
      message: "The request ID was already used with different sale data.",
    },
  );
});

test("rejects malformed sale responses atomically and bounds native errors", async () => {
  const request = { request_id: "550e8400-e29b-41d4-a716-446655440055", lines: [{ product_id: 1, quantity: 1, captured_unit_price_centavos: 2500, captured_revision: 0 }], payment: { amount_tendered_centavos: null, qr_applied_centavos: 2500 } };
  const valid = { kind: "success", sale_id: 7, request_id: request.request_id, status: "confirmed", confirmed_at: "now", outcome: "confirmed", lines: [{ product_id: 1, sku: "SKU", product_name: "Filter", quantity: 1, unit_price_centavos: 2500, line_total_centavos: 2500 }], payments: [{ method: "qr", amount_applied_centavos: 2500 }], total_centavos: 2500 };
  for (const response of [{ ...valid, lines: [{ ...valid.lines[0], quantity: 1.5 }] }, { ...valid, payments: [{ method: "bitcoin", amount_applied_centavos: 2500 }] }, { ...valid, payments: [{ ...valid.payments[0], amount_applied_centavos: Number.MAX_SAFE_INTEGER + 1 }] }, { kind: "unknown" }, { kind: "stale_catalog_record", product_id: 1, current_unit_price_centavos: 2.5, current_revision: 1 }]) {
    const confirmSale = createConfirmSaleCommand(async () => response);
    assert.deepEqual(await confirmSale(request), { kind: "error", code: "persistence_failure", message: "The sale could not be persisted." });
  }
  for (const response of [{ kind: "error", code: "invalid_payment", message: "SQL /panic native text" }, { kind: "error", code: "unknown", message: "native" }, { kind: "error", code: "invalid_payment" }]) {
    const confirmSale = createConfirmSaleCommand(async () => response);
    const result = await confirmSale(request);
    assert.equal(result.kind, "error");
    assert.equal(result.code, response.code === "invalid_payment" && typeof response.message === "string" ? "invalid_payment" : "persistence_failure");
    assert.ok(!result.message.includes("SQL") && !result.message.includes("native"));
  }
  const rejected = createConfirmSaleCommand(async () => { throw new Error("SQL /panic native text"); });
  assert.deepEqual(await rejected(request), { kind: "error", code: "persistence_failure", message: "The sale could not be persisted." });
});

test("rejects invalid request identities before invocation", async () => {
  let invoked = false;
  const confirmSale = createConfirmSaleCommand(async () => {
    invoked = true;
    return { kind: "error", code: "invalid_request", message: "unexpected" };
  });

  await assert.rejects(
    confirmSale({
      request_id: "550E8400-E29B-41D4-A716-446655440055",
      lines: [{ product_id: 1, quantity: 1 }],
      payment: { amount_tendered_centavos: null, qr_applied_centavos: null },
    }),
    /canonical UUID v4/,
  );

  assert.equal(invoked, false);
});

test("reconstructs the IPC payload from allowlisted request fields", async () => {
  const calls: unknown[] = [];
  const confirmSale = createConfirmSaleCommand(async (_command, payload) => {
    calls.push(payload);
    return { kind: "error", code: "invalid_request", message: "unexpected" };
  });
  const request = {
    request_id: "550e8400-e29b-41d4-a716-446655440056",
    lines: [
      {
        product_id: 1,
        quantity: 1,
        captured_unit_price_centavos: 2_500,
        captured_revision: 0,
        negotiated_unit_price_centavos: 2_500,
      },
    ],
    payment: {
      amount_tendered_centavos: 2_500,
      qr_applied_centavos: null,
      amount_applied_centavos: 2_500,
    },
    payments: [{ method: "cash", amount_applied_centavos: 2_500 }],
  } as ConfirmSaleRequest;

  await confirmSale(request);

  assert.deepEqual(calls, [
    {
      request: {
        request_id: "550e8400-e29b-41d4-a716-446655440056",
        lines: [{ product_id: 1, quantity: 1, captured_unit_price_centavos: 2_500, captured_revision: 0 }],
        payment: { amount_tendered_centavos: 2_500, qr_applied_centavos: null },
      },
    },
  ]);
});

test("rejects unsafe, non-integer, negative, and non-positive values before invocation", async () => {
  let invoked = false;
  const confirmSale = createConfirmSaleCommand(async () => {
    invoked = true;
    return { kind: "error", code: "invalid_request", message: "unexpected" };
  });
  const request = {
    request_id: "550e8400-e29b-41d4-a716-446655440055",
    lines: [{ product_id: 1, quantity: 1 }],
    payment: { amount_tendered_centavos: null, qr_applied_centavos: null },
  };

  for (const invalidRequest of [
    { ...request, lines: [{ product_id: 0, quantity: 1 }] },
    { ...request, lines: [{ product_id: 1.5, quantity: 1 }] },
    { ...request, lines: [{ product_id: 1, quantity: 0 }] },
    {
      ...request,
      lines: [{ product_id: 1, quantity: Number.MAX_SAFE_INTEGER + 1 }],
    },
    {
      ...request,
      payment: { amount_tendered_centavos: -1, qr_applied_centavos: null },
    },
    {
      ...request,
      payment: { amount_tendered_centavos: 1.5, qr_applied_centavos: null },
    },
    {
      ...request,
      payment: {
        amount_tendered_centavos: null,
        qr_applied_centavos: Number.MAX_SAFE_INTEGER + 1,
      },
    },
  ]) {
    await assert.rejects(
      confirmSale(invalidRequest),
      /safe integer|positive|non-negative/,
    );
  }

  assert.equal(invoked, false);
});


test("decodes non-positive final-price validation as a bounded field error", async () => {
  const confirmSale = createConfirmSaleCommand(async () => ({
    kind: "error",
    code: "invalid_final_price",
    message: "The final price must be positive.",
  }));

  assert.deepEqual(await confirmSale({
    request_id: "550e8400-e29b-41d4-a716-446655440058",
    lines: [{ product_id: 1, quantity: 1, captured_unit_price_centavos: 2500, captured_revision: 0, final_unit_price_centavos: 2500 }],
    payment: { amount_tendered_centavos: null, qr_applied_centavos: 2500 },
  }), {
    kind: "error",
    code: "invalid_final_price",
    message: "The final price must be positive.",
  });
});

test("decodes the bounded minimum-price violation and nullable snapshots", async () => {
  const confirmSale = createConfirmSaleCommand(async () => ({
    kind: "minimum_price_violation",
    product_id: 1,
    current_minimum_unit_price_centavos: 2_700,
  }));
  assert.deepEqual(await confirmSale({
    request_id: "550e8400-e29b-41d4-a716-446655440058",
    lines: [{ product_id: 1, quantity: 1, captured_unit_price_centavos: 2_500, captured_revision: 0 }],
    payment: { amount_tendered_centavos: null, qr_applied_centavos: 2_500 },
  }), {
    kind: "minimum_price_violation",
    product_id: 1,
    current_minimum_unit_price_centavos: 2_700,
  });

  const withSnapshots = createConfirmSaleCommand(async () => ({
    kind: "success",
    sale_id: 8,
    request_id: "550e8400-e29b-41d4-a716-446655440059",
    status: "confirmed",
    confirmed_at: "2026-03-08T12:00:00Z",
    outcome: "confirmed",
    lines: [{ product_id: 1, sku: "SKU-1", product_name: "Filter", quantity: 1, unit_price_centavos: 2_750, minimum_unit_price_snapshot_centavos: 2_500, list_price_snapshot_centavos: null, line_total_centavos: 2_750 }],
    payments: [{ method: "qr", amount_applied_centavos: 2_750 }],
    total_centavos: 2_750,
  }));
  const result = await withSnapshots({
    request_id: "550e8400-e29b-41d4-a716-446655440059",
    lines: [{ product_id: 1, quantity: 1, captured_unit_price_centavos: 2_500, captured_revision: 0 }],
    payment: { amount_tendered_centavos: null, qr_applied_centavos: 2_750 },
  });
  assert.equal(result.kind, "success");
  if (result.kind === "success") {
    assert.equal(result.lines[0].minimum_unit_price_snapshot_centavos, 2_500);
    assert.equal(result.lines[0].list_price_snapshot_centavos, null);
   }
 });

test("rejects unsafe and nonpositive persisted sale facts", async () => {
  const request = { request_id: "550e8400-e29b-41d4-a716-446655440060", lines: [{ product_id: 1, quantity: 1, captured_unit_price_centavos: 2500, captured_revision: 0 }], payment: { amount_tendered_centavos: null, qr_applied_centavos: 2500 } };
  const valid = { kind: "success", sale_id: 7, request_id: request.request_id, status: "confirmed", confirmed_at: "now", outcome: "confirmed", lines: [{ product_id: 1, sku: "SKU", product_name: "Filter", quantity: 1, unit_price_centavos: 2500, line_total_centavos: 2500 }], payments: [{ method: "qr", amount_applied_centavos: 2500 }], total_centavos: 2500 };
  const invalidResponses = [
    { ...valid, sale_id: 0 },
    { ...valid, sale_id: Number.MAX_SAFE_INTEGER + 1 },
    { ...valid, lines: [{ ...valid.lines[0], product_id: 0 }] },
    { ...valid, lines: [{ ...valid.lines[0], quantity: 0 }] },
    { ...valid, lines: [{ ...valid.lines[0], unit_price_centavos: -1 }] },
    { ...valid, lines: [{ ...valid.lines[0], line_total_centavos: Number.MAX_SAFE_INTEGER + 1 }] },
    { ...valid, payments: [{ method: "qr", amount_applied_centavos: -1 }] },
    { ...valid, payments: [{ method: "cash", amount_applied_centavos: 1, amount_tendered_centavos: 0, change_given_centavos: 0 }] },
    { ...valid, payments: [{ method: "cash", amount_applied_centavos: 1, amount_tendered_centavos: 1, change_given_centavos: -1 }] },
    { ...valid, total_centavos: -1 },
  ];
  for (const response of invalidResponses) {
    const confirmSale = createConfirmSaleCommand(async () => response);
    assert.deepEqual(await confirmSale(request), { kind: "error", code: "persistence_failure", message: "The sale could not be persisted." });
  }
});

test("rejects invalid stale identifiers, prices, and revisions", async () => {
  const request = { request_id: "550e8400-e29b-41d4-a716-446655440061", lines: [{ product_id: 1, quantity: 1, captured_unit_price_centavos: 2500, captured_revision: 0 }], payment: { amount_tendered_centavos: null, qr_applied_centavos: 2500 } };
  for (const response of [
    { kind: "stale_catalog_record", product_id: 0, current_unit_price_centavos: 2500, current_revision: 1 },
    { kind: "stale_catalog_record", product_id: 1, current_unit_price_centavos: 0, current_revision: 1 },
    { kind: "stale_catalog_record", product_id: 1, current_unit_price_centavos: 2500, current_revision: -1 },
    { kind: "stale_catalog_record", product_id: Number.MAX_SAFE_INTEGER + 1, current_unit_price_centavos: 2500, current_revision: 1 },
  ]) {
    const confirmSale = createConfirmSaleCommand(async () => response);
    assert.deepEqual(await confirmSale(request), { kind: "error", code: "persistence_failure", message: "The sale could not be persisted." });
  }
});
