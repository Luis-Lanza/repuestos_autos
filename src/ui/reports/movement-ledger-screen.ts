import { createElement, useEffect, useReducer, useRef } from "react";
import { MOVEMENT_TYPES, movementLedgerCommands, type MovementType } from "../../commands/movement-ledger.ts";
import { Action, Badge, Feedback, Field } from "../visual-system/controls.ts";
import { AlignedData, Panel } from "../visual-system/structure.ts";
import { createMovementLedgerFlow, initialLedgerState, type LedgerState } from "./movement-ledger-flow.ts";

const labels: Record<MovementType, string> = { opening_stock: "Stock inicial", stock_entry: "Ingreso", sale: "Venta", return: "Devolución", adjustment: "Ajuste", cancellation: "Cancelación" };
const columns = [
 { label: "Fecha y hora", align: "start", kind: "text" }, { label: "Producto · datos actuales", align: "start", kind: "text" },
 { label: "Tipo", align: "start", kind: "text" }, { label: "Cambio", align: "end", kind: "numeric" },
 { label: "Saldo registrado", align: "end", kind: "numeric" }, { label: "Detalle guardado", align: "start", kind: "text" }, { label: "Referencia", align: "start", kind: "text" },
] as const;
const emptyMessage = (state: LedgerState) => state.status === "empty" ? "No hay movimientos para los filtros aplicados." : "";
export function MovementLedgerScreen() {
 const [state, dispatch] = useReducer(createMovementLedgerFlow, undefined, () => initialLedgerState());
 const mounted = useRef(true); const listAttempt = useRef(0); const productsAttempt = useRef(0); const exportAttempt = useRef(0);
 useEffect(() => { mounted.current = true; return () => { mounted.current = false; listAttempt.current++; productsAttempt.current++; exportAttempt.current++; }; }, []);
 const load = (page: number, filters = state.applied) => {
  const request_id = ++listAttempt.current;
  exportAttempt.current++;
  dispatch({ type: "page_requested", request_id, page });
  void movementLedgerCommands.list(filters, page).then(response => { if (!mounted.current || request_id !== listAttempt.current) return; dispatch(response.kind === "success" ? { type: "list_succeeded", request_id, response } : { type: "list_failed", request_id }); });
 };
 const loadProducts = () => {
  const request_id = ++productsAttempt.current; dispatch({ type: "products_started", request_id });
  void movementLedgerCommands.productOptions().then(response => { if (mounted.current && request_id === productsAttempt.current) dispatch({ type: "products_finished", request_id, response }); });
 };
 useEffect(() => {
  loadProducts();
  const request_id = ++listAttempt.current; dispatch({ type: "filters_applied", request_id });
  void movementLedgerCommands.list(state.applied, 1).then(response => { if (!mounted.current || request_id !== listAttempt.current) return; dispatch(response.kind === "success" ? { type: "list_succeeded", request_id, response } : { type: "list_failed", request_id }); });
 }, []);
 const apply = () => { const request_id = ++listAttempt.current; exportAttempt.current++; dispatch({ type: "filters_applied", request_id }); void movementLedgerCommands.list(state.draft, 1).then(response => { if (!mounted.current || request_id !== listAttempt.current) return; dispatch(response.kind === "success" ? { type: "list_succeeded", request_id, response } : { type: "list_failed", request_id }); }); };
 const retry = () => load(state.page, state.applied);
 const change = (key: keyof typeof state.draft, value: string) => dispatch({ type: "draft_changed", filters: { ...state.draft, [key]: key === "product_id" ? (value ? Number(value) : null) : key === "movement_type" ? (value || null) : value } as typeof state.draft });
 const exportPdf = () => { const request_id = ++exportAttempt.current; dispatch({ type: "export_started", request_id }); void movementLedgerCommands.export(state.applied).then(response => { if (mounted.current && request_id === exportAttempt.current) dispatch({ type: "export_finished", request_id, response }); }); };
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
 const exportMessage = state.export_status === "pending" ? "Preparando el PDF…" : state.export_status === "success" ? "El PDF se generó correctamente." : state.export_status === "cancelled" ? "Se canceló la exportación." : state.export_status === "error" ? "No se pudo exportar el registro." : null;
 return createElement("main", { "aria-labelledby": "movement-ledger-heading", "data-ui-movement-ledger": true, "aria-busy": state.status === "loading" || undefined },
  createElement("header", { "data-ui-report-header": true }, createElement("h1", { id: "movement-ledger-heading" }, "Registro de movimientos"), createElement("p", null, "Consulta de movimientos de inventario guardados. Este informe es de solo lectura.")),
  createElement(Panel, { label: "Filtros" },
   createElement("form", { "data-ui-ledger-filters": true, onSubmit: (event: Event) => { event.preventDefault(); if (filtersValid) apply(); } },
    createElement(Field, { kind: "date", label: "Desde", control: createElement("input", { type: "date", required: true, value: state.draft.from, onChange: (event: Event) => change("from", (event.currentTarget as HTMLInputElement).value) }) }),
    createElement(Field, { kind: "date", label: "Hasta", control: createElement("input", { type: "date", required: true, value: state.draft.to, onChange: (event: Event) => change("to", (event.currentTarget as HTMLInputElement).value) }) }),
    createElement(Field, { kind: "select", label: "Producto", hint: state.products_status === "error" ? "No se pudieron cargar los productos históricos." : undefined, control: createElement("select", { value: state.draft.product_id === null ? "" : String(state.draft.product_id), onChange: (event: Event) => change("product_id", (event.currentTarget as HTMLSelectElement).value), disabled: state.products_status !== "ready" }, createElement("option", { value: "" }, "Todos los productos"), ...state.products.map(product => createElement("option", { key: product.product_id, value: String(product.product_id) }, `${product.product_name} · ${product.product_sku}${product.active ? "" : " · Archivado"}`))) }),
    createElement(Field, { kind: "select", label: "Tipo de movimiento", control: createElement("select", { value: state.draft.movement_type ?? "", onChange: (event: Event) => change("movement_type", (event.currentTarget as HTMLSelectElement).value) }, createElement("option", { value: "" }, "Todos"), ...MOVEMENT_TYPES.map(type => createElement("option", { key: type, value: type }, labels[type]))) }),
    createElement(Action, { variant: "primary", type: "submit", disabled: !filtersValid }, "Aplicar filtros"))),
  createElement(Panel, { label: "Movimientos" }, status,
   state.status === "ready" || state.status === "empty" ? createElement("div", { "data-ui-ledger-pages": true }, createElement("span", { role: "status" }, `Página ${state.page}`), createElement(Action, { variant: "secondary", disabled: state.page <= 1 || state.status === "loading", onClick: () => load(state.page - 1) }, "Anterior"), createElement(Action, { variant: "secondary", disabled: !state.has_more || state.status === "loading", onClick: () => load(state.page + 1) }, "Siguiente"), createElement(Action, { variant: "secondary", disabled: state.status !== "ready", pending: state.export_status === "pending", onClick: exportPdf }, "Exportar PDF")) : null,
   exportMessage ? createElement(Feedback, { kind: state.export_status === "error" ? "error" : state.export_status === "success" ? "success" : "advisory" }, exportMessage) : null));
}
