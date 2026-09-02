import assert from "node:assert/strict";
import test from "node:test";
import { renderToStaticMarkup } from "react-dom/server";

import type { PersistedSaleSummary } from "../../commands/confirm-sale.ts";
import { PersistedSaleSummaryView, projectPersistedSaleSummary } from "./persisted-summary.ts";

const summary: PersistedSaleSummary = {
  sale_id: 42,
  request_id: "550e8400-e29b-41d4-a716-446655440070",
  status: "confirmed",
  confirmed_at: "2026-03-10T12:30:00Z",
  outcome: "confirmed",
  lines: [
    { product_id: 1, sku: "BP-100", product_name: "Pastilla de freno", quantity: 2, unit_price_centavos: 2_750, line_total_centavos: 5_501 },
    { product_id: 9, sku: "SKU no disponible", product_name: "Producto no disponible", quantity: 1, unit_price_centavos: 1_250, line_total_centavos: 1_250 },
  ],
  payments: [
    { method: "cash", amount_applied_centavos: 5_000, amount_tendered_centavos: 6_000, change_given_centavos: 1_000 },
    { method: "qr", amount_applied_centavos: 1_750 },
  ],
  total_centavos: 6_750,
};

test("projects every persisted primitive into an independent presentation snapshot", () => {
  const source = structuredClone(summary);
  const details = projectPersistedSaleSummary(source);
  const expected = {
    saleIdentity: "Venta #42", confirmedAt: "10/03/2026, 12:30",
    lines: [
      ["1", "Pastilla de freno", "BP-100", "2", "Bs 27,50", "Bs 55,01"],
      ["9", "Producto no disponible", "SKU no disponible", "1", "Bs 12,50", "Bs 12,50"],
    ],
    payments: [["Efectivo aplicado", "Bs 50,00"], ["Efectivo recibido", "Bs 60,00"], ["Cambio", "Bs 10,00"], ["Pago QR", "Bs 17,50"]],
    total: "Bs 67,50",
  };
  assert.deepEqual(details, expected);

  Object.assign(source.lines[0], { product_id: 88, sku: "MUT", product_name: "Mutado", quantity: 7, unit_price_centavos: 1, line_total_centavos: 2 });
  Object.assign(source.payments[0], { amount_applied_centavos: 1, amount_tendered_centavos: 2, change_given_centavos: 3 });
  source.payments[1].amount_applied_centavos = 4;
  source.total_centavos = 5;
  assert.deepEqual(details, expected);
});

test("formats only real SQLite and ISO wall clocks with a fixed contained fallback", () => {
  const cases = [
    ["2026-08-14 10:42:00", "14/08/2026, 10:42"],
    ["2024-02-29T10:42Z", "29/02/2024, 10:42"],
    ["2026-08-14T10:42:59.123-04:00", "14/08/2026, 10:42"],
    ["2026-08-14T10:42+14:00", "14/08/2026, 10:42"],
    ["2026-08-14T10:42-14:00", "14/08/2026, 10:42"],
    ...["2026-02-30 10:42:00", "2025-02-29T10:42Z", "2026-13-14T10:42Z",
      "2026-08-14T24:42Z", "2026-08-14T10:60Z", "2026-08-14T10:42:60Z",
      "2026-08-14T10:42+14:01", "2026-08-14T10:42-14:01", "2026-08-14T10:42+15:00",
      "2026-08-14T10:42-23:59", "2026-08-14T10:42+24:00", "2026-08-14T10:42+04:60",
      "sin fecha", "x".repeat(10_000)]
      .map((value) => [value, "Fecha no disponible"]),
  ] as const;
  for (const [value, expected] of cases) {
    assert.equal(projectPersistedSaleSummary({ ...summary, confirmed_at: value }).confirmedAt, expected, value.slice(0, 80));
  }
});

test("preserves authoritative zeros and an empty persisted payment collection", () => {
  const details = projectPersistedSaleSummary({ ...summary, sale_id: 0, lines: [{ ...summary.lines[0], product_id: 0, quantity: 0, unit_price_centavos: 0, line_total_centavos: 0 }], payments: [], total_centavos: 0 });
  assert.deepEqual(details, { saleIdentity: "Venta #0", confirmedAt: "10/03/2026, 12:30", lines: [["0", "Pastilla de freno", "BP-100", "0", "Bs 0,00", "Bs 0,00"]], payments: [], total: "Bs 0,00" });
});

test("renders a semantic Spanish read-only summary with Nueva venta as its sole action", () => {
  const html = renderToStaticMarkup(PersistedSaleSummaryView({ details: projectPersistedSaleSummary({ ...summary, payments: [{ method: "qr", amount_applied_centavos: 6_750 }], total_centavos: 6_750 }), onNewSale() {} }));
  assert.match(html, /<main[^>]+aria-labelledby="sale-summary-heading"/);
  assert.match(html, />Venta confirmada</);
  assert.match(html, /<table/);
  assert.match(html, />Pago QR<\/td><td[^>]*>Bs 67,50/);
  assert.doesNotMatch(html, /Efectivo aplicado|Efectivo recibido|Cambio/);
  assert.equal((html.match(/<button/g) ?? []).length, 1);
  assert.match(html, />Nueva venta<\/button>/);
  assert.doesNotMatch(html, /<input|<select|<textarea|Editar|Imprimir|Compartir|Reembolsar|Recibo/);
});
