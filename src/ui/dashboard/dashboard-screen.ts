import { createElement, useEffect, useReducer, useRef, type ReactNode } from "react";

import { dashboardCommands, type DashboardMetrics } from "../../commands/dashboard.ts";
import { Action, Badge, Feedback } from "../visual-system/controls.ts";
import { AlignedData, Panel } from "../visual-system/structure.ts";
import { createDashboardFlow, initialDashboardState, type DashboardState } from "./dashboard-flow.ts";

const formatBs = (centavos: number) => `Bs ${Math.floor(centavos / 100)},${String(centavos % 100).padStart(2, "0")}`;
const metricFacts = (metrics: DashboardMetrics) => [
  ["Ventas efectivas", String(metrics.effective_sale_count)],
  ["Total efectivo", formatBs(metrics.effective_total_centavos)],
  ["Unidades netas", String(metrics.net_units_out)],
  ["Ventas canceladas", String(metrics.cancelled_sale_count)],
] as const;
const topColumns = [{ label: "ID", align: "start", kind: "numeric" }, { label: "Producto", align: "start", kind: "text" }, { label: "SKU", align: "start", kind: "sku" }, { label: "Unidades netas", align: "end", kind: "numeric" }] as const;
const stockColumns = [{ label: "ID", align: "start", kind: "numeric" }, { label: "Producto", align: "start", kind: "text" }, { label: "SKU", align: "start", kind: "sku" }, { label: "Cantidad actual", align: "end", kind: "numeric" }, { label: "Estado", align: "start", kind: "text" }] as const;
const paymentColumns = [{ label: "Medio", align: "start", kind: "text" }, { label: "Importe aplicado", align: "end", kind: "money" }] as const;
const recentColumns = [{ label: "Venta", align: "end", kind: "numeric" }, { label: "Fecha", align: "start", kind: "text" }, { label: "Estado", align: "start", kind: "text" }, { label: "Total", align: "end", kind: "money" }] as const;

function MetricsPanel({ label, metrics }: { label: string; metrics: DashboardMetrics }) {
  return createElement(Panel, { label },
    createElement("dl", { "data-ui-dashboard-metrics": true },
      ...metricFacts(metrics).map(([factLabel, value]) => createElement("div", { key: factLabel, "data-ui-dashboard-metric": true },
        createElement("dt", null, factLabel),
        createElement("dd", null, value)))));
}

function QuietSectionMessage({ kind, children }: { kind: "loading" | "error" | "empty"; children: ReactNode }) {
  return createElement("div", {
    "aria-busy": kind === "loading" || undefined,
    "data-ui-feedback": kind,
    "data-ui-dashboard-state": kind,
  }, children);
}

function SectionMessage({ status, hasItems, loadingMessage, errorMessage, emptyMessage, children }: {
  status: DashboardState["status"];
  hasItems: boolean;
  loadingMessage: string;
  errorMessage: string;
  emptyMessage: string;
  children: ReactNode;
}) {
  if (status === "loading") return createElement(QuietSectionMessage, { kind: "loading" }, loadingMessage);
  if (status === "error") return createElement(QuietSectionMessage, { kind: "error" }, errorMessage);
  return hasItems ? children : createElement(QuietSectionMessage, { kind: "empty" }, emptyMessage);
}

export function DashboardScreen({ onOpenInventoryAlerts }: { onOpenInventoryAlerts: () => void }) {
  const [state, dispatch] = useReducer(createDashboardFlow, initialDashboardState);
  const mounted = useRef(true);
  const attempt = useRef(0);
  useEffect(() => () => { mounted.current = false; attempt.current += 1; }, []);
  const load = () => {
    const request_id = ++attempt.current;
    dispatch({ type: "load_started", request_id });
    void dashboardCommands.load().then((response) => {
      if (!mounted.current || request_id !== attempt.current) return;
      dispatch(response.kind === "success" ? { type: "load_succeeded", request_id, report: response.report } : { type: "load_failed", request_id });
    });
  };
  useEffect(() => { load(); }, []);

  const report = state.report;
  const periodPanels = report
    ? [createElement(MetricsPanel, { key: "today", label: "Hoy", metrics: report.today.metrics }), createElement(MetricsPanel, { key: "month", label: "Este mes", metrics: report.month.metrics })]
    : [
        createElement(Panel, { key: "today", label: "Hoy" }, createElement(QuietSectionMessage, { kind: state.status === "loading" ? "loading" : "error" }, state.status === "loading" ? "Cargando indicadores…" : "Los indicadores de hoy no están disponibles.")),
        createElement(Panel, { key: "month", label: "Este mes" }, createElement(QuietSectionMessage, { kind: state.status === "loading" ? "loading" : "error" }, state.status === "loading" ? "Cargando indicadores…" : "Los indicadores del mes no están disponibles.")),
      ];
  const top = report?.top_products.map((item) => [item.product_id, item.product_name, item.sku, item.net_units_out] as const) ?? [];
  const payments = report?.payment_distribution.map((item) => [item.method === "cash" ? "Efectivo" : "QR", formatBs(item.amount_applied_centavos)] as const) ?? [];
  const recent = report?.recent_sales.map((item) => [item.sale_id, item.confirmed_at, createElement(Badge, { kind: item.status === "cancelled" ? "cancelled" : "confirmed", text: item.status === "cancelled" ? "Cancelada" : "Confirmada" }), formatBs(item.total_centavos)] as const) ?? [];
  const alerts = report?.stock_alerts.map((item) => [
    item.product_id,
    item.product_name,
    item.sku,
    item.quantity,
    createElement(Badge, { kind: item.classification === "out_of_stock" ? "out-of-stock" : "low-stock", text: item.classification === "out_of_stock" ? "Sin stock" : "Stock bajo" }),
  ] as const) ?? [];

  return createElement("main", { "aria-labelledby": "dashboard-heading", "data-ui-dashboard": true, "aria-busy": state.status === "loading" || undefined },
    createElement("header", { "data-ui-dashboard-header": true },
      createElement("h1", { id: "dashboard-heading" }, "Dashboard"),
      createElement("p", null, "Resumen operativo de ventas e inventario.")),
    state.status === "loading" ? createElement(Feedback, { kind: "loading" }, "Cargando dashboard…") : null,
    state.status === "error" ? createElement(Feedback, { kind: "error" }, createElement("span", null, "No se pudo cargar el dashboard. ", createElement(Action, { variant: "tertiary", onClick: load }, "Reintentar"))) : null,
    createElement("div", { "data-ui-dashboard-periods": true }, ...periodPanels),
    createElement("div", { "data-ui-dashboard-stock-alerts": true },
      createElement(Panel, { label: "Alertas de stock" },
        createElement(SectionMessage, { status: state.status, hasItems: alerts.length > 0, loadingMessage: "Cargando alertas de stock…", errorMessage: "No se pudieron cargar las alertas de stock.", emptyMessage: "No hay alertas de stock." },
          createElement(AlignedData, { caption: "Productos que requieren atención", columns: stockColumns, rows: alerts })),
        state.status === "ready" ? createElement(Action, { variant: "secondary", onClick: onOpenInventoryAlerts }, "Ver en Inventario") : null)),
    createElement("div", { "data-ui-dashboard-supporting": true },
      createElement("div", { "data-ui-dashboard-top-products": true },
        createElement(Panel, { label: "Productos más vendidos · Este mes" },
          createElement(SectionMessage, { status: state.status, hasItems: top.length > 0, loadingMessage: "Cargando productos más vendidos…", errorMessage: "No se pudieron cargar los productos más vendidos.", emptyMessage: "No hay productos vendidos en este período." },
            createElement(AlignedData, { caption: "Productos por unidades netas", columns: topColumns, rows: top })))),
      createElement("div", { "data-ui-dashboard-payments": true },
        createElement(Panel, { label: "Distribución de pagos · Este mes" },
          createElement(SectionMessage, { status: state.status, hasItems: payments.length > 0, loadingMessage: "Cargando distribución de pagos…", errorMessage: "No se pudo cargar la distribución de pagos.", emptyMessage: "No hay pagos registrados en este período." },
            createElement(AlignedData, { caption: "Importes aplicados en Bs", columns: paymentColumns, rows: payments })))),
      createElement("div", { "data-ui-dashboard-recent-sales": true },
        createElement(Panel, { label: "Ventas recientes · Todas las fechas" },
          createElement(SectionMessage, { status: state.status, hasItems: recent.length > 0, loadingMessage: "Cargando ventas recientes…", errorMessage: "No se pudieron cargar las ventas recientes.", emptyMessage: "No hay ventas recientes." },
            createElement(AlignedData, { caption: "Últimas ventas", columns: recentColumns, rows: recent }))))));
}
