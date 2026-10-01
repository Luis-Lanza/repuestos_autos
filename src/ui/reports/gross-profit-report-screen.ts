import { createElement, useEffect, useRef, useState } from "react";
import { grossProfitCommands, type GrossProfitReport } from "../../commands/gross-profit.ts";
import { Action, Feedback, Field } from "../visual-system/controls.ts";
import { Panel } from "../visual-system/structure.ts";

interface Period { from: string; to: string; }
type State = { draft: Period; applied: Period; status: "loading" | "ready" | "empty" | "error"; report: GrossProfitReport | null; };
const date = (value: Date) => `${value.getFullYear()}-${String(value.getMonth() + 1).padStart(2, "0")}-${String(value.getDate()).padStart(2, "0")}`;
const currentMonth = (): Period => { const now = new Date(); return { from: date(new Date(now.getFullYear(), now.getMonth(), 1)), to: date(new Date(now.getFullYear(), now.getMonth() + 1, 0)) }; };
const money = (centavos: number) => `Bs ${centavos < 0 ? "−" : ""}${(Math.abs(centavos) / 100).toFixed(2)}`;
export function GrossProfitReportScreen() {
 const [state, setState] = useState<State>(() => { const period = currentMonth(); return { draft: period, applied: period, status: "loading", report: null }; });
 const mounted = useRef(false); const attempt = useRef(0);
 useEffect(() => { mounted.current = true; const request = ++attempt.current; const period = state.applied; void grossProfitCommands.load(period.from, period.to).then(response => { if (!mounted.current || request !== attempt.current) return; setState(current => response.kind === "success" ? { ...current, report: response.report, status: response.report.activity_count === 0 ? "empty" : "ready" } : { ...current, report: null, status: "error" }); }); return () => { mounted.current = false; attempt.current++; }; }, []);
 const load = (period: Period) => { const request = ++attempt.current; setState(current => ({ ...current, applied: period, status: "loading", report: null })); void grossProfitCommands.load(period.from, period.to).then(response => { if (!mounted.current || request !== attempt.current) return; setState(current => response.kind === "success" ? { ...current, report: response.report, status: response.report.activity_count === 0 ? "empty" : "ready" } : { ...current, report: null, status: "error" }); }); };
 const valid = Boolean(state.draft.from && state.draft.to && state.draft.from <= state.draft.to);
 const status = state.status === "loading" ? createElement(Feedback, { kind: "loading" }, "Cargando ganancia bruta…")
  : state.status === "error" ? createElement(Feedback, { kind: "error" }, createElement("span", null, "No se pudo cargar el informe. ", createElement(Action, { variant: "tertiary", onClick: () => load(state.applied) }, "Reintentar")))
  : state.status === "empty" ? createElement(Feedback, { kind: "empty" }, "No hay ventas ni devoluciones en el período aplicado.")
  : null;
 return createElement("section", { "aria-labelledby": "gross-profit-heading", "data-ui-gross-profit-report": true, "aria-busy": state.status === "loading" || undefined },
  createElement("header", { "data-ui-report-header": true }, createElement("h1", { id: "gross-profit-heading" }, "Ganancia bruta"), createElement("p", null, "Total del período según ventas confirmadas y devoluciones registradas.")),
  createElement(Panel, { label: "Período" },
   createElement("form", { "data-ui-gross-profit-filters": true, onSubmit: (event: Event) => { event.preventDefault(); if (valid) load(state.draft); } },
    createElement(Field, { kind: "date", label: "Desde", control: createElement("input", { type: "date", required: true, value: state.draft.from, onChange: (event: Event) => { const value = (event.currentTarget as HTMLInputElement).value; setState(current => ({ ...current, draft: { ...current.draft, from: value } })); } }) }),
    createElement(Field, { kind: "date", label: "Hasta", control: createElement("input", { type: "date", required: true, value: state.draft.to, onChange: (event: Event) => { const value = (event.currentTarget as HTMLInputElement).value; setState(current => ({ ...current, draft: { ...current.draft, to: value } })); } }) }),
    createElement(Action, { variant: "primary", type: "submit", disabled: !valid }, "Aplicar período"))),
  createElement(Panel, { label: "Resultado" },
   status,
   state.report ? createElement("section", { "aria-label": "Total de ganancia bruta", "data-ui-gross-profit-total": true },
    createElement("p", null, `Período: ${state.applied.from} al ${state.applied.to}`),
    createElement("h2", null, money(state.report.amount_centavos)),
    state.report.missing_cost_line_count > 0 ? createElement(Feedback, { kind: "advisory" }, `Cálculo parcial: ${state.report.missing_cost_line_count} línea(s) no tienen costo histórico conocido. Esas líneas se excluyen del total; no se estima su costo con el catálogo actual.`) : null)
    : null));
}
