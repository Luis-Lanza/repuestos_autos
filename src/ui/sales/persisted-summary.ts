import type { PersistedSaleSummary } from "../../commands/confirm-sale.ts";

export type PersistedSummaryDetails = {
  saleId: string;
  requestId: string;
  status: string;
  confirmedAt: string;
  outcome: string;
  lines: string[];
  payments: string[];
  total: string;
};

export function formatBs(centavos: number): string {
  return `Bs ${(centavos / 100).toFixed(2)}`;
}

export function persistedSummaryDetails(
  summary: PersistedSaleSummary,
): PersistedSummaryDetails {
  return {
    saleId: String(summary.sale_id),
    requestId: summary.request_id,
    status: summary.status,
    confirmedAt: summary.confirmed_at,
    outcome: summary.outcome,
    lines: summary.lines.map(
      (line) =>
        `${line.sku} — ${line.product_name} · ${line.quantity} × ${formatBs(line.unit_price_centavos)}`,
    ),
    payments: summary.payments.map((payment) =>
      payment.method === "cash"
        ? `Cash: ${formatBs(payment.amount_applied_centavos)} · Tendered: ${formatBs(payment.amount_tendered_centavos)} · Change: ${formatBs(payment.change_given_centavos)}`
        : `QR: ${formatBs(payment.amount_applied_centavos)}`,
    ),
    total: formatBs(summary.total_centavos),
  };
}
