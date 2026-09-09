import type { SalesHistorySummary } from "../../commands/sales-history.ts";
import { formatBs } from "./sale-flow.ts";

const UNAVAILABLE_DATE = "Fecha no disponible";

export function formatHistoryDate(value: string): string {
  const match = value.match(/^(\d{4})-(\d{2})-(\d{2})[ T](\d{2}):(\d{2})(?::\d{2}(?:\.\d+)?)?(?:Z|[+-]\d{2}:\d{2})?$/);
  if (!match) return UNAVAILABLE_DATE;
  const [, yearText, monthText, dayText, hourText, minuteText] = match;
  const year = Number(yearText);
  const month = Number(monthText);
  const day = Number(dayText);
  const hour = Number(hourText);
  const minute = Number(minuteText);
  const calendar = new Date(Date.UTC(year, month - 1, day));
  if (
    year < 1 || month < 1 || month > 12 || day < 1 ||
    calendar.getUTCFullYear() !== year || calendar.getUTCMonth() !== month - 1 ||
    calendar.getUTCDate() !== day || hour > 23 || minute > 59
  ) return UNAVAILABLE_DATE;
  return `${dayText}/${monthText}/${yearText}, ${hourText}:${minuteText}`;
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
