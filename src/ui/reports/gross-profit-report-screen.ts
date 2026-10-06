import { createElement, useEffect, useReducer, useRef } from "react";
import { useActionNotifications } from "../visual-system/action-notifications.ts";
import { grossProfitCommands } from "../../commands/gross-profit.ts";
import { exportGrossProfitOperationsCommand, grossProfitOperationsCommands, type GrossProfitOperation } from "../../commands/gross-profit-operations.ts";
import { createGrossProfitReportFlow, initialGrossProfitReportState, type Period } from "./gross-profit-report-flow.ts";
import { Action, Feedback, Field } from "../visual-system/controls.ts";
import { Panel } from "../visual-system/structure.ts";

const date = (value: Date) => `${value.getFullYear()}-${String(value.getMonth() + 1).padStart(2, "0")}-${String(value.getDate()).padStart(2, "0")}`;
const currentMonth = (): Period => { const now = new Date(); return { from: date(new Date(now.getFullYear(), now.getMonth(), 1)), to: date(new Date(now.getFullYear(), now.getMonth() + 1, 0)) }; };
const money = (centavos: number) => `Bs ${centavos < 0 ? "−" : ""}${(Math.abs(centavos) / 100).toFixed(2)}`;
const signed = (value: number) => `${value > 0 ? "+" : value < 0 ? "−" : ""}${Math.abs(value)}`;
const rowName = (row: GrossProfitOperation) => row.operation_kind === "venta" ? `Venta #${row.sale_id}` : `Devolución #${row.return_id} · Venta #${row.sale_id}`;

export function GrossProfitReportScreen({ period }: { period?: Period } = {}) {
 const [state, dispatch] = useReducer(createGrossProfitReportFlow, undefined, () => initialGrossProfitReportState(currentMonth()));
 const notifications = useActionNotifications();
 const mounted = useRef(true); const rangeRequest = useRef(0); const operationsRequest = useRef(0); const pageRequestInFlight = useRef<number | null>(null); const exportInFlight = useRef(false);
 useEffect(() => { mounted.current = true; return () => { mounted.current = false; rangeRequest.current++; operationsRequest.current++; pageRequestInFlight.current = null; }; }, []);
 const runRange = (period: Period) => {
  const rangeId = ++rangeRequest.current; const operationsId = ++operationsRequest.current; pageRequestInFlight.current = null;
  dispatch({ type: "range_applied", period, range_id: rangeId, operations_request_id: operationsId });
  void grossProfitCommands.load(period.from, period.to).then(response => { if (!mounted.current || rangeId !== rangeRequest.current) return; dispatch(response.kind === "success" ? { type: "summary_succeeded", range_id: rangeId, report: response.report } : { type: "summary_failed", range_id: rangeId }); });
  void grossProfitOperationsCommands.load(period.from, period.to, 1).then(response => { if (!mounted.current || rangeId !== rangeRequest.current || operationsId !== operationsRequest.current) return; dispatch(response.kind === "success" ? { type: "operations_succeeded", range_id: rangeId, operations_request_id: operationsId, report: response.report } : { type: "operations_failed", range_id: rangeId, operations_request_id: operationsId }); });
 };
 useEffect(() => { runRange(period ?? state.applied); }, [period?.from, period?.to]);
 const valid = Boolean(state.draft.from && state.draft.to && state.draft.from <= state.draft.to);
 const exportReady = state.summary_status === "ready" && state.operations_status === "ready";
 const requestPage = (page: number) => {
  if (pageRequestInFlight.current !== null || state.operations_status !== "ready") return;
  const requestId = ++operationsRequest.current; const rangeId = state.range_id; pageRequestInFlight.current = requestId;
  dispatch({ type: "page_requested", page, operations_request_id: requestId });
  void grossProfitOperationsCommands.load(state.applied.from, state.applied.to, page).then(response => { if (!mounted.current || rangeId !== rangeRequest.current || requestId !== operationsRequest.current) return; dispatch(response.kind === "success" ? { type: "operations_succeeded", range_id: rangeId, operations_request_id: requestId, report: response.report } : { type: "operations_failed", range_id: rangeId, operations_request_id: requestId }); }).finally(() => { if (pageRequestInFlight.current === requestId) pageRequestInFlight.current = null; });
 };
 const exportPdf = () => {
  if (!exportReady || exportInFlight.current) return;
  exportInFlight.current = true;
  dispatch({ type: "export_started" });
  void exportGrossProfitOperationsCommand(state.applied.from, state.applied.to).then(response => {
   if (!mounted.current) return;
   const outcome = response.kind === "success" ? "success" : response.kind === "cancelled" ? "cancelled" : response.code === "resource_limit" ? "resource_limit" : "error";
   notifications?.publish({ severity: outcome === "success" ? "success" : outcome === "cancelled" ? "info" : "error", message: outcome === "success" ? "PDF guardado correctamente." : outcome === "cancelled" ? "Exportación cancelada." : outcome === "resource_limit" ? "El informe supera los límites de recursos y no se guardó. Ajustá el período e intentá de nuevo." : "No se pudo guardar el PDF." });
   dispatch({ type: "export_completed", outcome });
  }).finally(() => { exportInFlight.current = false; });
 };
 const exportFeedback = state.export_status === "pending" ? "Guardando PDF…" : state.export_status === "cancelled" ? "Exportación cancelada." : state.export_status === "success" ? "PDF guardado correctamente." : state.export_status === "resource_limit" ? "El informe supera los límites de recursos y no se guardó. Ajustá el período e intentá de nuevo." : state.export_status === "error" ? "No se pudo guardar el PDF." : null;
 const status = state.summary_status === "loading" ? createElement(Feedback, { kind: "loading" }, "Cargando ganancia bruta…")
  : state.summary_status === "error" ? createElement(Feedback, { kind: "error" }, createElement("span", null, "No se pudo cargar el informe. ", createElement(Action, { variant: "tertiary", onClick: () => runRange(state.applied) }, "Reintentar")))
  : state.summary_status === "empty" ? createElement(Feedback, { kind: "empty" }, "No hay ventas ni devoluciones en el período aplicado.") : null;
 const operationsStatus = state.operations_status === "loading" ? createElement(Feedback, { kind: "loading" }, "Cargando operaciones…")
  : state.operations_status === "error" ? createElement(Feedback, { kind: "error" }, createElement("span", null, "No se pudieron cargar las operaciones. ", createElement(Action, { variant: "tertiary", onClick: () => runRange(state.applied) }, "Reintentar")))
  : state.operations_status === "empty" ? createElement(Feedback, { kind: "empty" }, "No hay operaciones en el período aplicado.") : null;
 const rows = state.operations?.rows.map(row => createElement("tr", { key: `${row.occurred_at}-${row.operation_kind}-${row.return_id ?? row.sale_id}-${row.sku}` },
  createElement("td", null, row.occurred_at), createElement("td", null, rowName(row)), createElement("td", null, `${row.product_name} (SKU ${row.sku})`),
  createElement("td", null, signed(row.signed_quantity)), createElement("td", null, money(row.negotiated_unit_price_centavos)),
  createElement("td", null, row.cost_state === "unknown" ? "No disponible (no calculable)" : money(row.unit_cost_snapshot_centavos!)),
  createElement("td", null, row.signed_gross_profit_centavos === null ? "No calculable" : money(row.signed_gross_profit_centavos))));
 return createElement("section", { "aria-label": "Resultados de ganancia bruta", "data-ui-gross-profit-report": true, "aria-busy": state.summary_status === "loading" || state.operations_status === "loading" || undefined },
  period ? null : createElement("header", { "data-ui-report-header": true }, createElement("h1", { id: "gross-profit-heading" }, "Ganancia bruta"), createElement("p", null, "Total del período según ventas confirmadas y devoluciones registradas.")),
  (period ? null : createElement(Panel, { label: "Período" }, createElement("form", { "data-ui-gross-profit-filters": true, onSubmit: (event: Event) => { event.preventDefault(); if (valid) runRange(state.draft); } },
   createElement(Field, { kind: "date", label: "Desde", control: createElement("input", { type: "date", required: true, value: state.draft.from, onChange: (event: Event) => dispatch({ type: "draft_changed", period: { ...state.draft, from: (event.currentTarget as HTMLInputElement).value } }) }) }),
   createElement(Field, { kind: "date", label: "Hasta", control: createElement("input", { type: "date", required: true, value: state.draft.to, onChange: (event: Event) => dispatch({ type: "draft_changed", period: { ...state.draft, to: (event.currentTarget as HTMLInputElement).value } }) }) }),
   createElement(Action, { variant: "primary", type: "submit", disabled: !valid }, "Aplicar período")))),
  createElement(Panel, { label: period ? "Resultados" : "Ganancia bruta" },
   createElement("div", { "data-ui-report-actions": true }, createElement(Action, { variant: "secondary", onClick: exportPdf, disabled: !exportReady, pending: state.export_status === "pending", pendingLabel: "Exportar PDF" }, "Exportar PDF")),
   status,
   exportFeedback && (!notifications || state.export_status === "pending") ? createElement(Feedback, { kind: state.export_status === "error" || state.export_status === "resource_limit" ? "error" : state.export_status === "success" ? "success" : "advisory" }, exportFeedback) : null,
   state.report ? createElement("section", { "aria-label": "Total de ganancia bruta", "data-ui-gross-profit-total": true }, createElement("p", null, `Período: ${state.applied.from} al ${state.applied.to}`), createElement("h2", null, money(state.report.amount_centavos)), state.report.missing_cost_line_count > 0 ? createElement(Feedback, { kind: "advisory" }, `Cálculo parcial: ${state.report.missing_cost_line_count} línea(s) no tienen costo histórico conocido. Esas líneas se excluyen del total; no se estima su costo con el catálogo actual.`) : null) : null,
   createElement("section", { "aria-label": "Operaciones de ganancia bruta" }, operationsStatus,
    state.operations && (state.operations_status === "ready" || state.operations_status === "loading") ? createElement("div", null,
     createElement("table", null, createElement("caption", null, `Operaciones del período aplicado: ${state.applied.from} al ${state.applied.to}`), createElement("thead", null, createElement("tr", null, ...["Fecha y hora", "Operación", "Producto", "Cantidad", "Precio final", "Costo histórico", "Ganancia bruta"].map(label => createElement("th", { scope: "col", key: label }, label)))), createElement("tbody", null, rows)),
     createElement("nav", { "aria-label": "Paginación de operaciones", "aria-busy": state.operations_status === "loading" || undefined }, createElement(Action, { variant: "tertiary", disabled: state.operations.page <= 1 || state.operations_status === "loading", onClick: () => requestPage(state.operations!.page - 1) }, "Anterior"), createElement("span", { "aria-live": "polite" }, `Página ${state.operations.page} de ${state.operations.total_pages}`), createElement(Action, { variant: "tertiary", disabled: state.operations.page >= state.operations.total_pages || state.operations_status === "loading", onClick: () => requestPage(state.operations!.page + 1) }, "Siguiente"))
    ) : null)));
}
