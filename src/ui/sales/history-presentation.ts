import type { SalesHistoryDetail, SalesHistorySummary } from "../../commands/sales-history.ts";
import { formatBs } from "./sale-flow.ts";

const UNAVAILABLE_DATE = "Fecha no disponible";

export function formatHistoryDate(value: string): string {
  const match = value.match(/^(\d{4})-(\d{2})-(\d{2})([ T])(\d{2}):(\d{2})(?::(\d{2})(\.\d+)?)?(Z|[+-](\d{2}):(\d{2}))?$/);
  if (!match) return UNAVAILABLE_DATE;
  const [, yearText, monthText, dayText, separator, hourText, minuteText, secondText, fraction, zone, offsetHourText, offsetMinuteText] = match;
  if (separator === " " && (secondText === undefined || fraction !== undefined || zone !== undefined)) return UNAVAILABLE_DATE;
  const [year, month, day, hour, minute, second] = [yearText, monthText, dayText, hourText, minuteText, secondText ?? "0"].map(Number);
  const leap = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  const daysInMonth = [31, leap ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
  if (year < 1 || month < 1 || month > 12 || day < 1 || day > daysInMonth[month - 1]
    || hour > 23 || minute > 59 || second > 59
    || (offsetHourText !== undefined && (Number(offsetHourText) > 14 || Number(offsetMinuteText) > 59
      || (Number(offsetHourText) === 14 && Number(offsetMinuteText) !== 0)))) return UNAVAILABLE_DATE;
  return `${dayText}/${monthText}/${yearText}, ${hourText}:${minuteText}`;
}

export function projectHistoryDetail(detail: SalesHistoryDetail) {
  return {
    identity: `Venta #${detail.sale_id}`,
    date: formatHistoryDate(detail.confirmed_at),
    status: detail.status === "confirmed" ? "Confirmada" : "Cancelada",
    lines: detail.lines.map((line) => [line.product_name ?? "Producto no disponible", line.sku ?? "SKU no disponible", String(line.quantity), formatBs(line.unit_price_centavos), formatBs(line.line_total_centavos)]),
    payments: detail.payments.flatMap((payment) => payment.method === "cash"
      ? [["Efectivo aplicado", formatBs(payment.amount_applied_centavos)], ["Efectivo recibido", formatBs(payment.amount_tendered_centavos)], ["Cambio", formatBs(payment.change_given_centavos)]]
      : [["Pago QR", formatBs(payment.amount_applied_centavos)]]),
    total: formatBs(detail.total_centavos),
  };
}

export function projectCorrectionHistory(detail: SalesHistoryDetail) {
  const correctionLine = (line: { sale_line_id: number; product_id: number }, quantity: number) =>
    [String(line.sale_line_id), String(line.product_id), String(quantity)];
  return {
    returns: detail.returns.map((record) => ({ identity: `Devolución #${record.return_id}`,
      requestId: record.request_id, occurredAt: record.occurred_at, status: "Confirmada",
      lines: record.lines.map((line) => correctionLine(line, line.quantity)) })),
    cancellation: detail.cancellation ? { identity: `Cancelación #${detail.cancellation.cancellation_id}`,
      requestId: detail.cancellation.request_id, occurredAt: detail.cancellation.occurred_at,
      status: "Cancelada", reason: detail.cancellation.reason,
      lines: detail.cancellation.lines.map((line) => correctionLine(line, line.restored_quantity)) } : null,
  };
}

export function historySummaryCells(sale: SalesHistorySummary) {
  const methods = sale.payment_methods.map((method) => method === "cash" ? "Efectivo" : "QR").join(" y ");
  return {
    identity: `Venta #${sale.sale_id}`,
    date: formatHistoryDate(sale.confirmed_at),
    status: sale.status === "confirmed" ? "Confirmada" : "Cancelada",
    items: `${sale.line_count} ${sale.line_count === 1 ? "artículo" : "artículos"}`,
    payments: `${sale.payment_count} ${sale.payment_count === 1 ? "pago" : "pagos"} · ${methods || "Sin métodos"}`,
    total: formatBs(sale.total_centavos),
  };
}

export function historyListError(code: "invalid_range" | "sale_not_found" | "persistence_failure") {
  return code === "invalid_range"
    ? "Revisá que el rango de fechas sea válido."
    : "No se pudo cargar el historial de ventas.";
}
