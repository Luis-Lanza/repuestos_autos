import { createElement, useEffect, useReducer, useRef, useState } from "react";
import { useActionNotifications } from "../visual-system/action-notifications.ts";
import { MOVEMENT_TYPES, movementLedgerCommands, type MovementType } from "../../commands/movement-ledger.ts";
import { Action, Badge, Feedback, Field } from "../visual-system/controls.ts";
import { AlignedData, Panel } from "../visual-system/structure.ts";
import { createMovementLedgerFlow, initialLedgerState, type LedgerState } from "./movement-ledger-flow.ts";
import { GrossProfitReportScreen } from "./gross-profit-report-screen.ts";

type ReportMode = MovementType | "" | "gross_profit";
const labels: Record<MovementType, string> = { opening_stock: "Stock inicial", stock_entry: "Ingreso", sale: "Venta", return: "Devolución", adjustment: "Ajuste", cancellation: "Cancelación" };
const columns = [
 { label: "Fecha y hora", align: "start", kind: "text" }, { label: "Producto · datos actuales", align: "start", kind: "text" },
 { label: "Tipo", align: "start", kind: "text" }, { label: "Cambio", align: "end", kind: "numeric" },
 { label: "Saldo registrado", align: "end", kind: "numeric" }, { label: "Detalle guardado", align: "start", kind: "text" }, { label: "Referencia", align: "start", kind: "text" },
] as const;
const emptyMessage = (state: LedgerState) => state.status === "empty" ? "No hay movimientos para los filtros aplicados." : "";
export function MovementLedgerScreen() {
 const [state, dispatch] = useReducer(createMovementLedgerFlow, undefined, () => initialLedgerState());
 const [reportMode, setReportMode] = useState<ReportMode>("");
 const [appliedMode, setAppliedMode] = useState<ReportMode | null>(null);
 const [profitPeriod, setProfitPeriod] = useState({ from: state.applied.from, to: state.applied.to });
 const notifications = useActionNotifications();
 const exportInFlight = useRef(false);
 const [exportResourceLimit, setExportResourceLimit] = useState(false);
 const mounted = useRef(true); const listAttempt = useRef(0); const productsAttempt = useRef(0); const exportAttempt = useRef(0);
 useEffect(() => { mounted.current = true; return () => { mounted.current = false; listAttempt.current++; productsAttempt.current++; exportAttempt.current++; }; }, []);
 const load = (page: number, filters = state.applied) => {
  const request_id = ++listAttempt.current;
  exportAttempt.current++;
  dispatch({ type: "page_requested", request_id, page });
  void movementLedgerCommands.list(filters, page).then(response => { if (!mounted.current || request_id !== listAttempt.current) return; dispatch(response.kind === "success" ? { type: "list_succeeded", request_id, response } : { type: "list_failed", request_id }); });
 };
 const searchProducts = (query = state.product_query, page = 1) => {
  const request_id = ++productsAttempt.current;
  dispatch({ type: "product_search_started", request_id, query, page });
  void movementLedgerCommands.productOptions(query, page).then(response => {
   if (!mounted.current || request_id !== productsAttempt.current) return;
   dispatch(response.kind === "success" ? { type: "product_search_succeeded", request_id, response } : { type: "product_search_failed", request_id });
  });
 };
 useEffect(() => {
  const request_id = ++listAttempt.current; dispatch({ type: "filters_applied", request_id });
  void movementLedgerCommands.list(state.applied, 1).then(response => { if (!mounted.current || request_id !== listAttempt.current) return; dispatch(response.kind === "success" ? { type: "list_succeeded", request_id, response } : { type: "list_failed", request_id }); });
 }, []);
 const apply = () => {
  const filters = { ...state.draft, movement_type: reportMode === "gross_profit" ? null : reportMode || null };
  dispatch({ type: "draft_changed", filters });
  setAppliedMode(reportMode || null);
  const request_id = ++listAttempt.current; exportAttempt.current++;
  if (reportMode === "gross_profit") { setProfitPeriod({ from: filters.from, to: filters.to }); return; }
  dispatch({ type: "filters_applied", request_id });
  void movementLedgerCommands.list(filters, 1).then(response => { if (!mounted.current || request_id !== listAttempt.current) return; dispatch(response.kind === "success" ? { type: "list_succeeded", request_id, response } : { type: "list_failed", request_id }); });
 };
 const retry = () => load(state.page, state.applied);
 const change = (key: keyof typeof state.draft, value: string) => dispatch({ type: "draft_changed", filters: { ...state.draft, [key]: key === "product_id" ? (value ? Number(value) : null) : key === "movement_type" ? (value || null) : value } as typeof state.draft });
 const changeProductQuery = (query: string) => dispatch({ type: "product_query_changed", query, request_id: ++productsAttempt.current });
 const selectProduct = (product: typeof state.products[number]) => dispatch({ type: "product_selected", product, request_id: ++productsAttempt.current });
 const clearProduct = () => dispatch({ type: "product_cleared", request_id: ++productsAttempt.current });
 const exportPdf = () => {
  if (exportInFlight.current || state.status !== "ready") return;
  exportInFlight.current = true; setExportResourceLimit(false);
  const request_id = ++exportAttempt.current;
  dispatch({ type: "export_started", request_id });
  void movementLedgerCommands.export(state.applied).then(response => {
   if (!mounted.current || request_id !== exportAttempt.current) return;
   const resourceLimit = response.kind === "error" && response.code === "resource_limit";
   setExportResourceLimit(resourceLimit);
   notifications?.publish({ severity: response.kind === "success" ? "success" : response.kind === "cancelled" ? "info" : "error", message: response.kind === "success" ? "El PDF se generó correctamente." : response.kind === "cancelled" ? "Se canceló la exportación." : resourceLimit ? "El registro supera los límites de recursos y no se guardó. Ajustá los filtros e intentá de nuevo." : "No se pudo exportar el registro." });
   dispatch({ type: "export_finished", request_id, response });
  }).finally(() => { exportInFlight.current = false; });
 };
 const filtersValid = Boolean(state.draft.from && state.draft.to && state.draft.from <= state.draft.to);
 const rows = state.rows.map(row => [
  row.occurred_at.replace("T", " "), createElement("span", null, createElement("strong", null, row.product_name), " ", createElement("code", null, row.product_sku), createElement("small", { "data-ui-ledger-current": true }, "datos actuales")),
  createElement(Badge, { kind: row.movement_type === "cancellation" ? "cancelled" : row.movement_type === "adjustment" ? "stale" : "confirmed", text: labels[row.movement_type] }),
  `${row.quantity_delta > 0 ? "+" : ""}${row.quantity_delta}`,
  row.resulting_quantity === null ? "No registrado" : String(row.resulting_quantity),
  [row.reason, row.note].filter((detail): detail is string => detail !== null && detail.length > 0).join(" · ") || "No disponible", row.sale_id === null ? "—" : `Venta #${row.sale_id}${row.sale_line_id === null ? "" : ` · línea #${row.sale_line_id}`}`,
 ]);
 const status = state.status === "initial" ? createElement(Feedback, { kind: "initial" }, "Preparando el registro…")
  : state.status === "loading" ? createElement(Feedback, { kind: "loading" }, "Cargando movimientos…")
  : state.status === "error" ? createElement(Feedback, { kind: "error" }, createElement("span", null, "No se pudieron cargar los movimientos. ", createElement(Action, { variant: "tertiary", onClick: retry }, "Reintentar")))
  : state.status === "empty" ? createElement(Feedback, { kind: "empty" }, emptyMessage(state))
  : createElement(AlignedData, { caption: "Movimientos persistidos; los datos de producto reflejan el catálogo actual.", columns, rows });
 const productStatus = state.products_status === "loading" ? createElement(Feedback, { kind: "loading" }, "Buscando productos…")
  : state.products_status === "empty" ? createElement(Feedback, { kind: "empty" }, `No encontramos productos para “${state.product_submitted_query}”.`)
  : state.products_status === "error" ? createElement(Feedback, { kind: "error" }, createElement("span", null, "No se pudo buscar en el catálogo local. Reintentá. ", createElement(Action, { variant: "tertiary", onClick: () => searchProducts(state.product_submitted_query, state.products_page) }, "Reintentar")))
  : state.products_status === "initial" ? createElement(Feedback, { kind: "initial" }, createElement("span", null, "Todos los productos. Buscá un producto por nombre o SKU para filtrar el registro.")) : null;
 const exportMessage = state.export_status === "pending" ? "Preparando el PDF…" : state.export_status === "success" ? "El PDF se generó correctamente." : state.export_status === "cancelled" ? "Se canceló la exportación." : state.export_status === "error" ? exportResourceLimit ? "El registro supera los límites de recursos y no se guardó. Ajustá los filtros e intentá de nuevo." : "No se pudo exportar el registro." : null;
 return createElement("main", { "aria-labelledby": "movement-ledger-heading", "data-ui-movement-ledger": true, "data-ui-report-mode": reportMode === "gross_profit" ? "gross-profit" : "movement",
  "data-ui-report-applied-mode": appliedMode === "gross_profit" ? "gross-profit" : "movement", "aria-busy": (appliedMode === "gross_profit" ? false : state.status === "loading") || undefined },
  createElement("header", { "data-ui-report-header": true }, createElement("h1", { id: "movement-ledger-heading" }, "Reportes"), createElement("p", null, "Consultá los movimientos de inventario o la ganancia bruta del período.")),
  createElement(Panel, { label: "Filtros" },
   createElement("div", { "data-ui-ledger-filters": true },
    createElement("div", { "data-ui-ledger-date-row": true },
     createElement(Field, { kind: "date", label: "Desde", control: createElement("input", { type: "date", required: true, value: state.draft.from, onChange: (event: Event) => change("from", (event.currentTarget as HTMLInputElement).value) }) }),
     createElement(Field, { kind: "date", label: "Hasta", control: createElement("input", { type: "date", required: true, value: state.draft.to, onChange: (event: Event) => change("to", (event.currentTarget as HTMLInputElement).value) }) })),
    createElement("div", { "data-ui-ledger-filter-row": true },
     reportMode !== "gross_profit" ? createElement("div", { "data-ui-ledger-product-filter": true },
      createElement("form", { "data-ui-ledger-product-search": true, onSubmit: (event: Event) => { event.preventDefault(); searchProducts(state.product_query, 1); } },
       createElement(Field, { kind: "search", label: "Buscar producto por nombre o SKU", control: createElement("input", { maxLength: 100, value: state.product_query, onChange: (event: Event) => changeProductQuery((event.currentTarget as HTMLInputElement).value), onKeyDown: (event: KeyboardEvent) => { if (event.key === "Enter") { event.preventDefault(); searchProducts(state.product_query, 1); } } }) }),
       createElement(Action, { variant: "secondary", type: "submit", disabled: !state.product_query.trim() }, "Buscar"))) : null,
     createElement(Field, { kind: "select", label: "Tipo de movimiento", control: createElement("select", { value: reportMode, onChange: (event: Event) => { const value = (event.currentTarget as HTMLSelectElement).value; setReportMode(value === "gross_profit" ? "gross_profit" : value as MovementType | ""); } }, createElement("option", { value: "" }, "Todos"), ...MOVEMENT_TYPES.map(type => createElement("option", { key: type, value: type }, labels[type])), createElement("option", { value: "gross_profit" }, "Ganancia bruta")) }),
     createElement("form", { "data-ui-ledger-apply-form": true, onSubmit: (event: Event) => { event.preventDefault(); apply(); } },
      createElement("span", { "aria-hidden": true, "data-ui-ledger-apply-label-space": true }, "Tipo de movimiento"),
      createElement(Action, { variant: "primary", type: "submit", disabled: !filtersValid }, "Aplicar filtros"))),
    reportMode !== "gross_profit" ? createElement("div", { "data-ui-ledger-product-feedback-row": true },
     state.selected_product ? createElement("div", { "data-ui-ledger-selected-product": true }, createElement("span", null, `${state.selected_product.product_name} · ${state.selected_product.product_sku}${state.selected_product.active ? "" : " · Archivado"}`), createElement(Action, { variant: "tertiary", onClick: clearProduct }, "Quitar producto")) : null,
     productStatus,
     state.products_status === "ready" ? createElement("ul", { "aria-label": "Resultados de productos", "data-ui-ledger-product-results": true }, state.products.map(product => createElement("li", { key: product.product_id }, createElement("span", null, `${product.product_name} · ${product.product_sku}${product.active ? "" : " · Archivado"}`), createElement(Action, { variant: "secondary", onClick: () => selectProduct(product), "aria-label": `Seleccionar ${product.product_name} (SKU: ${product.product_sku})` }, "Seleccionar")))) : null,
     state.products_status === "ready" && state.products_has_more ? createElement(Action, { variant: "tertiary", onClick: () => searchProducts(state.product_submitted_query, state.products_page + 1) }, "Más productos") : null) : null)),
  appliedMode === "gross_profit" ? createElement(GrossProfitReportScreen, { period: profitPeriod }) : createElement(Panel, { label: "Resultados" },
   createElement("div", { "data-ui-report-actions": true }, createElement(Action, { variant: "secondary", disabled: state.status !== "ready", pending: state.export_status === "pending", pendingLabel: "Exportar PDF", onClick: exportPdf }, "Exportar PDF")),
   status,
   state.status === "ready" || state.status === "empty" ? createElement("div", { "data-ui-ledger-pages": true }, createElement("span", { role: "status" }, `Página ${state.page}`), createElement(Action, { variant: "secondary", disabled: state.page <= 1 || state.status === "loading", onClick: () => load(state.page - 1) }, "Anterior"), createElement(Action, { variant: "secondary", disabled: !state.has_more || state.status === "loading", onClick: () => load(state.page + 1) }, "Siguiente")) : null,
   exportMessage && (!notifications || state.export_status === "pending") ? createElement(Feedback, { kind: state.export_status === "error" ? "error" : state.export_status === "success" ? "success" : "advisory" }, exportMessage) : null));
}
