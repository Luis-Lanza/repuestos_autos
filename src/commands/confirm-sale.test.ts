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
    lines: [{ product_id: 1, quantity: 2 }],
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
          lines: [{ product_id: 1, quantity: 2 }],
          payment: {
            amount_tendered_centavos: 3_000,
            qr_applied_centavos: 2_000,
          },
        },
      },
    },
  ]);
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
    lines: [{ product_id: 1, quantity: 1 }],
    payment: { amount_tendered_centavos: 2_500, qr_applied_centavos: null },
  });
  await confirmSale({
    request_id: "550e8400-e29b-41d4-a716-446655440052",
    lines: [{ product_id: 2, quantity: 1 }],
    payment: { amount_tendered_centavos: null, qr_applied_centavos: 2_500 },
  });
  await confirmSale({
    request_id: "550e8400-e29b-41d4-a716-446655440053",
    lines: [{ product_id: 3, quantity: 1 }],
    payment: { amount_tendered_centavos: 1_500, qr_applied_centavos: 1_000 },
  });

  assert.deepEqual(captured, [
    {
      request: {
        request_id: "550e8400-e29b-41d4-a716-446655440051",
        lines: [{ product_id: 1, quantity: 1 }],
        payment: { amount_tendered_centavos: 2_500, qr_applied_centavos: null },
      },
    },
    {
      request: {
        request_id: "550e8400-e29b-41d4-a716-446655440052",
        lines: [{ product_id: 2, quantity: 1 }],
        payment: { amount_tendered_centavos: null, qr_applied_centavos: 2_500 },
      },
    },
    {
      request: {
        request_id: "550e8400-e29b-41d4-a716-446655440053",
        lines: [{ product_id: 3, quantity: 1 }],
        payment: {
          amount_tendered_centavos: 1_500,
          qr_applied_centavos: 1_000,
        },
      },
    },
  ]);
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
    lines: [{ product_id: 1, quantity: 2 }],
    payment: { amount_tendered_centavos: null, qr_applied_centavos: 5_500 },
  });

  assert.equal(result.kind, "success");
  assert.equal(result.lines[0].unit_price_centavos, 2_750);
  assert.equal(result.payments[0].method, "qr");
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
