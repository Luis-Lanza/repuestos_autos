import assert from "node:assert/strict";
import test from "node:test";

import type { PersistedSaleSummary } from "../../commands/confirm-sale.ts";
import { formatBs, persistedSummaryDetails } from "./persisted-summary";

const summary: PersistedSaleSummary = {
  sale_id: 42,
  request_id: "550e8400-e29b-41d4-a716-446655440070",
  status: "confirmed",
  confirmed_at: "2026-03-10T12:30:00Z",
  outcome: "confirmed",
  lines: [
    {
      product_id: 1,
      sku: "BP-100",
      product_name: "Brake Pad",
      quantity: 2,
      unit_price_centavos: 2_750,
      line_total_centavos: 5_500,
    },
  ],
  payments: [
    {
      method: "cash",
      amount_applied_centavos: 5_500,
      amount_tendered_centavos: 6_000,
      change_given_centavos: 500,
    },
  ],
  total_centavos: 5_500,
};

test("formats stored unit prices and derived cash facts from the persisted summary", () => {
  const details = persistedSummaryDetails(summary);

  assert.equal(formatBs(5_500), "Bs 55.00");
  assert.deepEqual(details, {
    saleId: "42",
    requestId: "550e8400-e29b-41d4-a716-446655440070",
    status: "confirmed",
    confirmedAt: "2026-03-10T12:30:00Z",
    outcome: "confirmed",
    lines: ["BP-100 — Brake Pad · 2 × Bs 27.50"],
    payments: ["Cash: Bs 55.00 · Tendered: Bs 60.00 · Change: Bs 5.00"],
    total: "Bs 55.00",
  });
});

test("formats QR-only and mixed persisted payment facts without cash fields on QR", () => {
  const qrOnly = persistedSummaryDetails({
    ...summary,
    payments: [{ method: "qr", amount_applied_centavos: 5_500 }],
  });
  const mixed = persistedSummaryDetails({
    ...summary,
    payments: [
      { method: "qr", amount_applied_centavos: 2_000 },
      {
        method: "cash",
        amount_applied_centavos: 3_500,
        amount_tendered_centavos: 4_000,
        change_given_centavos: 500,
      },
    ],
  });

  assert.deepEqual(qrOnly.payments, ["QR: Bs 55.00"]);
  assert.deepEqual(mixed.payments, [
    "QR: Bs 20.00",
    "Cash: Bs 35.00 · Tendered: Bs 40.00 · Change: Bs 5.00",
  ]);
});
