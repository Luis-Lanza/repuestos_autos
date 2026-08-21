import assert from "node:assert/strict";
import test from "node:test";

import { createConfirmSaleCommand } from "./confirm-sale.ts";

test("builds an integer-safe confirmation payload through the replaceable command seam", async () => {
  const calls: unknown[] = [];
  const confirmSale = createConfirmSaleCommand(async (command, payload) => {
    calls.push({ command, payload });
    return { kind: "error", code: "invalid_payment", message: "Payment values are invalid." };
  });

  const result = await confirmSale({
    request_id: "550e8400-e29b-41d4-a716-446655440050",
    lines: [{ product_id: 1, quantity: 1, negotiated_unit_price_centavos: 2_500 }],
    payments: [{ method: "qr", amount_applied_centavos: 2_500 }],
  });

  assert.equal(result.kind, "error");
  assert.deepEqual(calls, [{
    command: "confirm_sale_command",
    payload: {
      request: {
        request_id: "550e8400-e29b-41d4-a716-446655440050",
        lines: [{ product_id: 1, quantity: 1, negotiated_unit_price_centavos: 2_500 }],
        payments: [{ method: "qr", amount_applied_centavos: 2_500 }],
      },
    },
  }]);
});

test("preserves cash and QR fields in a mixed integer payload", async () => {
  let captured: unknown;
  const confirmSale = createConfirmSaleCommand(async (_command, payload) => {
    captured = payload;
    return { kind: "error", code: "invalid_payment", message: "Payment values are invalid." };
  });

  await confirmSale({
    request_id: "550e8400-e29b-41d4-a716-446655440052",
    lines: [{ product_id: 1, quantity: 1, negotiated_unit_price_centavos: 2_500 }],
    payments: [
      { method: "cash", amount_applied_centavos: 1_000, amount_tendered_centavos: 1_500, change_given_centavos: 500 },
      { method: "qr", amount_applied_centavos: 1_500 },
    ],
  });

  assert.deepEqual(captured, {
    request: {
      request_id: "550e8400-e29b-41d4-a716-446655440052",
      lines: [{ product_id: 1, quantity: 1, negotiated_unit_price_centavos: 2_500 }],
      payments: [
        { method: "cash", amount_applied_centavos: 1_000, amount_tendered_centavos: 1_500, change_given_centavos: 500 },
        { method: "qr", amount_applied_centavos: 1_500 },
      ],
    },
  });
});

test("rejects unsafe integer payloads before command invocation", async () => {
  let invoked = false;
  const confirmSale = createConfirmSaleCommand(async () => {
    invoked = true;
    return { kind: "error", code: "invalid_request", message: "unexpected" };
  });

  await assert.rejects(
    confirmSale({
      request_id: "550e8400-e29b-41d4-a716-446655440051",
      lines: [{ product_id: 1, quantity: 1.5, negotiated_unit_price_centavos: 2_500 }],
      payments: [],
    }),
    /safe integer/,
  );
  assert.equal(invoked, false);
});
