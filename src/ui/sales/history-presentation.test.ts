import assert from "node:assert/strict";
import test from "node:test";

import type { SalesHistoryDetail } from "../../commands/sales-history.ts";
import {
  formatHistoryDate,
  projectCorrectionHistory,
  projectHistoryDetail,
} from "./history-presentation.ts";

const detail: SalesHistoryDetail = {
  sale_id: 184,
  confirmed_at: "2026-08-14 10:42:00",
  status: "confirmed",
  total_centavos: 35_000,
  lines: [
    {
      sale_line_id: 41,
      product_id: 4,
      sku: null,
      product_name: null,
      quantity: 2,
      unit_price_centavos: 8_550,
      line_total_centavos: 17_100,
      returned_quantity: 0,
      cancellation_restored_quantity: 0,
      remaining_returnable_quantity: 2,
    },
  ],
  payments: [
    {
      method: "cash",
      amount_applied_centavos: 20_000,
      amount_tendered_centavos: 25_000,
      change_given_centavos: 5_000,
    },
    { method: "qr", amount_applied_centavos: 15_000 },
  ],
  returns: [],
  cancellation: null,
};

test("projects only persisted original detail facts with Spanish unavailable labels", () => {
  assert.deepEqual(projectHistoryDetail(detail), {
    identity: "Venta #184",
    date: "14/08/2026, 10:42",
    status: "Confirmada",
    lines: [
      [
        "Producto no disponible",
        "SKU no disponible",
        "2",
        "Bs 85,50",
        "Bs 171,00",
      ],
    ],
    payments: [
      ["Efectivo aplicado", "Bs 200,00"],
      ["Efectivo recibido", "Bs 250,00"],
      ["Cambio", "Bs 50,00"],
      ["Pago QR", "Bs 150,00"],
    ],
    total: "Bs 350,00",
  });
});

test("validates persisted wall-clock dates and contains malformed values", () => {
  assert.equal(
    formatHistoryDate("2024-02-29T23:59:59.123-14:00"),
    "29/02/2024, 23:59",
  );
  for (const value of [
    "2026-02-30 10:42:00",
    "2026-08-14T24:00:00Z",
    "2026-08-14T10:42:00+14:01",
    "x".repeat(10_000),
  ])
    assert.equal(formatHistoryDate(value), "Fecha no disponible");
});

test("projects persisted correction records without dropping identifiers or zero quantities", () => {
  const corrected: SalesHistoryDetail = { ...detail,
    returns: [{ return_id: 81, request_id: "return-request-exact", occurred_at: "2026-08-15 09:03:02", lines: [{ sale_line_id: 41, product_id: 4, quantity: 0 }] }],
    cancellation: { cancellation_id: 91, request_id: "cancel-request-exact", occurred_at: "2026-08-16 11:04:03", reason: "Venta duplicada", lines: [{ sale_line_id: 41, product_id: 4, restored_quantity: 0 }] } };
  assert.deepEqual(projectCorrectionHistory(corrected), {
    returns: [{ identity: "Devolución #81", requestId: "return-request-exact", occurredAt: "2026-08-15 09:03:02", status: "Confirmada", lines: [["41", "4", "0"]] }],
    cancellation: { identity: "Cancelación #91", requestId: "cancel-request-exact", occurredAt: "2026-08-16 11:04:03", status: "Cancelada", reason: "Venta duplicada", lines: [["41", "4", "0"]] },
  });
});
