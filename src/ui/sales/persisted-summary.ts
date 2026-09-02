import { createElement } from "react";

import type { PersistedSaleSummary } from "../../commands/confirm-sale.ts";
import { Action } from "../visual-system/controls.ts";
import { AlignedData } from "../visual-system/structure.ts";
import { formatBs } from "./sale-flow.ts";

export type PersistedSummaryDetails = {
  saleIdentity: string;
  confirmedAt: string;
  lines: readonly (readonly string[])[];
  payments: readonly (readonly string[])[];
  total: string;
};

const UNAVAILABLE_DATE = "Fecha no disponible";

function formatPersistedWallClock(value: string): string {
  const match = value.match(/^(\d{4})-(\d{2})-(\d{2})([ T])(\d{2}):(\d{2})(?::(\d{2})(\.\d+)?)?(Z|[+-](\d{2}):(\d{2}))?$/);
  if (!match) return UNAVAILABLE_DATE;
  const [, yearText, monthText, dayText, separator, hourText, minuteText, secondText, fraction, zone, offsetHourText, offsetMinuteText] = match;
  if (separator === " " && (secondText === undefined || fraction !== undefined || zone !== undefined)) return UNAVAILABLE_DATE;
  const [year, month, day, hour, minute, second] = [yearText, monthText, dayText, hourText, minuteText, secondText ?? "0"].map(Number);
  const leap = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  const daysInMonth = [31, leap ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
  if (month < 1 || month > 12 || day < 1 || day > daysInMonth[month - 1]
    || hour > 23 || minute > 59 || second > 59
    || (offsetHourText !== undefined && (Number(offsetHourText) > 14 || Number(offsetMinuteText) > 59
      || (Number(offsetHourText) === 14 && Number(offsetMinuteText) !== 0)))) return UNAVAILABLE_DATE;
  return `${dayText}/${monthText}/${yearText}, ${hourText}:${minuteText}`;
}

export function projectPersistedSaleSummary(summary: PersistedSaleSummary): PersistedSummaryDetails {
  return {
    saleIdentity: `Venta #${summary.sale_id}`,
    confirmedAt: formatPersistedWallClock(String(summary.confirmed_at)),
    lines: summary.lines.map((line) => [
      String(line.product_id), String(line.product_name), String(line.sku), String(line.quantity),
      formatBs(line.unit_price_centavos), formatBs(line.line_total_centavos),
    ]),
    payments: summary.payments.flatMap((payment) => payment.method === "cash" ? [
      ["Efectivo aplicado", formatBs(payment.amount_applied_centavos)],
      ["Efectivo recibido", formatBs(payment.amount_tendered_centavos)],
      ["Cambio", formatBs(payment.change_given_centavos)],
    ] : [["Pago QR", formatBs(payment.amount_applied_centavos)]]),
    total: formatBs(summary.total_centavos),
  };
}

const itemColumns = [
  { label: "ID artículo", align: "start", kind: "numeric" },
  { label: "Producto", align: "start", kind: "text" },
  { label: "SKU", align: "start", kind: "sku" },
  { label: "Cantidad", align: "end", kind: "numeric" },
  { label: "Precio unitario", align: "end", kind: "money" },
  { label: "Subtotal", align: "end", kind: "money" },
] as const;
const paymentColumns = [
  { label: "Dato de pago", align: "start", kind: "text" },
  { label: "Importe", align: "end", kind: "money" },
] as const;

export function PersistedSaleSummaryView({ details, onNewSale }: { details: PersistedSummaryDetails; onNewSale: () => void }) {
  return createElement("main", { "aria-labelledby": "sale-summary-heading", "data-ui-persisted-summary": true },
    createElement("header", { "data-ui-summary-header": true },
      createElement("h1", { id: "sale-summary-heading" }, "Venta confirmada"),
      createElement("p", null, "La venta quedó guardada y estos datos son de solo lectura.")),
    createElement("section", { "aria-labelledby": "sale-identity-heading" },
      createElement("h2", { id: "sale-identity-heading" }, "Identificación de la venta"),
      createElement("dl", null,
        createElement("dt", null, "Venta"), createElement("dd", null, details.saleIdentity),
        createElement("dt", null, "Fecha y hora"), createElement("dd", { "data-ui-type": "numeric" }, details.confirmedAt))),
    createElement(AlignedData, { caption: "Artículos confirmados", columns: itemColumns, rows: details.lines }),
    createElement(AlignedData, { caption: "Pagos confirmados", columns: paymentColumns, rows: details.payments }),
    createElement("p", { "data-ui-summary-total": true }, createElement("span", null, "Total persistido"), createElement("strong", null, details.total)),
    createElement(Action, { variant: "primary", onClick: onNewSale }, "Nueva venta"));
}
