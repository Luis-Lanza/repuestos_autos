import { createElement, type FormEvent, useEffect, useReducer, useRef, useState } from "react";

import { searchProducts } from "../../commands/catalog.ts";
import { confirmSale, type ConfirmSaleRequest } from "../../commands/confirm-sale.ts";
import { Action, Badge, Feedback, Field } from "../visual-system/controls.ts";
import { Panel } from "../visual-system/structure.ts";
import { catalogResultDetails } from "./catalog-result.ts";
import { createSaleFlow, draftLineSubtotalCentavos, draftTotalCentavos, effectiveDraftUnitPriceCentavos, formatBs, initialSaleState, parseOptionalBs } from "./sale-flow.ts";
import { persistedSummaryDetails } from "./persisted-summary";

const INVALID_BS = "Ingresá un monto válido en Bs, con hasta dos decimales.";
const failureByCode: Record<string, string> = {
  invalid_request: "Revisá los datos de la venta e intentá nuevamente.",
  invalid_quantity: "Revisá que las cantidades sean números enteros mayores que cero.",
  invalid_payment: "Revisá los montos de pago e intentá nuevamente.",
  inactive_product: "Uno de los productos ya no está activo.",
  missing_product: "Uno de los productos ya no está disponible.",
  insufficient_stock: "No hay stock suficiente para completar la venta.",
  persistence_failure: "No se pudo confirmar la venta. Intentá nuevamente.",
};
const requestId = () => crypto.randomUUID();

export function SaleScreen() {
  const [state, dispatch] = useReducer(createSaleFlow, initialSaleState);
  const [query, setQuery] = useState("");
  const [paymentErrors, setPaymentErrors] = useState<Partial<Record<"amount_tendered_centavos" | "qr_applied_centavos", string>>>({});
  const cashRef = useRef<HTMLInputElement>(null), qrRef = useRef<HTMLInputElement>(null), draftRef = useRef<HTMLElement>(null);
  const searchSequence = useRef(0), confirmationSequence = useRef(0), confirming = useRef(false), mounted = useRef(true);
  useEffect(() => () => { mounted.current = false; searchSequence.current += 1; confirmationSequence.current += 1; }, []);
  useEffect(() => { const action = draftRef.current?.querySelector<HTMLButtonElement>("#stale-price-accept"); if (state.stale_price && mounted.current && action?.isConnected) action.focus(); }, [state.stale_price]);
  const draftDispatch = (action: Parameters<typeof dispatch>[0]) => { if (!confirming.current) dispatch(action); };

  async function search(event: FormEvent<HTMLFormElement>) {
    event.preventDefault(); if (confirming.current) return;
    const attempt = ++searchSequence.current;
    dispatch({ type: "catalog_search_started", query, request_id: attempt });
    try {
      const results = await searchProducts(query);
      if (mounted.current && attempt === searchSequence.current) dispatch({ type: "catalog_search_succeeded", request_id: attempt, results });
    } catch {
      if (mounted.current && attempt === searchSequence.current) dispatch({ type: "catalog_search_failed", request_id: attempt, message: "No se pudo buscar en el catálogo local." });
    }
  }

  async function confirm() {
    if (confirming.current) return;
    const errors: typeof paymentErrors = {};
    let cash: number | null = null, qr: number | null = null;
    try { cash = parseOptionalBs(state.payment.amount_tendered_centavos); } catch { errors.amount_tendered_centavos = INVALID_BS; }
    try { qr = parseOptionalBs(state.payment.qr_applied_centavos); } catch { errors.qr_applied_centavos = INVALID_BS; }
    setPaymentErrors(errors);
    if (Object.keys(errors).length) { (errors.amount_tendered_centavos ? cashRef : qrRef).current?.focus(); return; }

    confirming.current = true;
    const attempt = ++confirmationSequence.current;
    const currentRequestId = state.request_id ?? requestId();
    dispatch({ type: "confirmation_started", request_id: currentRequestId });
    const request: ConfirmSaleRequest = {
      request_id: currentRequestId,
      lines: state.lines.map(({ product_id, quantity, captured_unit_price_centavos, captured_revision, acknowledged_price_centavos, acknowledged_revision }) => ({ product_id, quantity, captured_unit_price_centavos, captured_revision, acknowledged_price_centavos, acknowledged_revision })),
      payment: { amount_tendered_centavos: cash, qr_applied_centavos: qr },
    };
    try {
      const response = await confirmSale(request);
      if (!mounted.current || attempt !== confirmationSequence.current) return;
      if (response.kind === "success") dispatch({ type: "confirmation_succeeded", summary: response });
      else if (response.kind === "stale_catalog_record") dispatch({ type: "stale_price_detected", ...response });
      else dispatch({ type: "confirmation_failed", message: failureByCode[response.code] ?? "No se pudo confirmar la venta. Intentá nuevamente." });
    } catch {
      if (mounted.current && attempt === confirmationSequence.current) dispatch({ type: "confirmation_failed", message: "No se pudo confirmar la venta. Intentá nuevamente." });
    } finally {
      if (attempt === confirmationSequence.current) confirming.current = false;
    }
  }

  if (state.persisted_summary) {
    const details = persistedSummaryDetails(state.persisted_summary);
    return createElement(
      "main",
      { "aria-labelledby": "sale-summary-heading" },
      createElement("h1", { id: "sale-summary-heading" }, "Sale confirmed"),
      createElement("p", null, `Sale ${details.saleId} · ${details.status} · ${details.outcome}`),
      createElement("dl", null,
        createElement("dt", null, "Request ID"), createElement("dd", null, details.requestId),
        createElement("dt", null, "Confirmed at"), createElement("dd", null, details.confirmedAt),
        createElement("dt", null, "Total"), createElement("dd", null, details.total)),
      createElement("h2", null, "Products"),
      createElement("ul", null, details.lines.map((line) => createElement("li", { key: line }, line))),
      createElement("h2", null, "Payments"),
      createElement("ul", null, details.payments.map((payment) => createElement("li", { key: payment }, payment))),
      createElement("button", { type: "button", onClick: () => dispatch({ type: "discard" }) }, "New sale"),
    );
  }

  const discovery = state.catalog_discovery;
  const discoveryFeedback = discovery.status === "initial" ? createElement(Feedback, { kind: "initial" } as never, "Buscá un producto para comenzar.")
    : discovery.status === "loading" ? createElement(Feedback, { kind: "loading", "aria-label": "Buscando productos…" } as never, "Buscando productos…")
    : discovery.status === "empty" ? createElement(Feedback, { kind: "empty" } as never, `No encontramos productos para “${discovery.query}”.`)
    : discovery.status === "error" ? createElement(Feedback, { kind: "error" } as never, discovery.error)
    : null;
  const total = formatBs(draftTotalCentavos(state.lines));
  const staleLine = state.stale_price && state.lines.find((line) => line.product_id === state.stale_price!.product_id);
  const pending = state.confirmation === "pending";

  return createElement("main", { ref: draftRef, "aria-labelledby": "sale-heading", "aria-busy": pending || undefined, "data-ui-sale": true },
    createElement("h1", { id: "sale-heading" }, "Ventas"),
    createElement("div", { "data-ui-sale-layout": true },
      createElement(Panel, { label: "Catálogo" } as never,
        createElement("form", { onSubmit: search, "data-ui-sale-search": true },
          createElement(Field, { kind: "search", label: "Buscar en el catálogo", control: createElement("input", { value: query, disabled: pending, onChange: (event) => { if (!confirming.current) setQuery(event.target.value); } }) } as never),
          createElement(Action, { variant: "secondary", type: "submit", disabled: pending }, "Buscar")),
        discoveryFeedback,
        createElement("ul", { "aria-label": "Resultados del catálogo", "data-ui-sale-list": true }, discovery.results.map((product) => {
          const details = catalogResultDetails(product, { inCart: state.lines.some((line) => line.product_id === product.product_id) });
          return createElement("li", { key: details.product.id },
            createElement("div", null, createElement("strong", null, details.product.name), createElement("span", { "data-ui-sku": true }, details.product.sku), createElement("span", null, details.product.category)),
            createElement("span", { "data-ui-money": true }, details.price.text),
            createElement(Badge, { kind: details.stock.kind === "low" ? "low-stock" : details.stock.kind === "out" ? "out-of-stock" : "available", text: details.stock.text }),
            createElement(Action, { variant: "secondary", disabled: pending || details.availability !== "available", onClick: () => draftDispatch({ type: "add_product", product }) }, "Agregar"));
        }))),
      createElement(Panel, { label: "Carrito" } as never,
        state.lines.length === 0 ? createElement(Feedback, { kind: "empty" } as never, "El carrito está vacío.") : null,
        createElement("ul", { "aria-label": "Carrito", "data-ui-sale-list": true }, state.lines.map((line) => createElement("li", { key: line.product_id },
          createElement("div", null, createElement("strong", null, line.product_name), createElement("span", { "data-ui-sku": true }, line.sku), createElement("span", { "data-ui-money": true }, `Precio unitario: ${formatBs(effectiveDraftUnitPriceCentavos(line))}`)),
          createElement(Field, { kind: "quantity", label: `Cantidad de ${line.product_name}`, error: state.feedback === "Ingresá una cantidad entera mayor que cero." ? state.feedback : undefined, control: createElement("input", { min: 1, value: line.quantity, disabled: pending, onChange: (event) => draftDispatch({ type: "line_quantity_changed", product_id: line.product_id, value: event.target.value }) }) } as never),
          createElement("span", { "data-ui-money": true }, `Subtotal: ${formatBs(draftLineSubtotalCentavos(line))}`),
          createElement(Action, { variant: "tertiary", disabled: pending, onClick: () => draftDispatch({ type: "remove_product", product_id: line.product_id }) }, "Quitar"))))),
      createElement(Panel, { label: "Pago" } as never,
        createElement(Field, { kind: "money", label: "Efectivo recibido", error: paymentErrors.amount_tendered_centavos, control: createElement("input", { ref: cashRef, value: state.payment.amount_tendered_centavos, disabled: pending, onChange: (event) => { if (confirming.current) return; setPaymentErrors((old) => ({ ...old, amount_tendered_centavos: undefined })); dispatch({ type: "payment_changed", field: "amount_tendered_centavos", value: event.target.value }); } }) } as never),
        createElement(Field, { kind: "money", label: "Pago QR", error: paymentErrors.qr_applied_centavos, control: createElement("input", { ref: qrRef, value: state.payment.qr_applied_centavos, disabled: pending, onChange: (event) => { if (confirming.current) return; setPaymentErrors((old) => ({ ...old, qr_applied_centavos: undefined })); dispatch({ type: "payment_changed", field: "qr_applied_centavos", value: event.target.value }); } }) } as never)),
      createElement(Panel, { label: "Resumen" } as never,
        createElement("p", { "data-ui-type": "total" }, `Total: ${total}`),
        staleLine && state.stale_price ? createElement(Feedback, { kind: "stale" } as never,
          createElement("p", null, `El precio de ${staleLine.product_name} cambió de ${formatBs(staleLine.captured_unit_price_centavos)} a ${formatBs(state.stale_price.current_unit_price_centavos)}.`),
          createElement(Action, { id: "stale-price-accept", variant: "secondary", onClick: () => draftDispatch({ type: "acknowledge_stale_price", product_id: state.stale_price!.product_id, current_unit_price_centavos: state.stale_price!.current_unit_price_centavos, current_revision: state.stale_price!.current_revision }) }, "Aceptar precio actual")) : null,
        state.feedback && state.feedback !== "Ingresá una cantidad entera mayor que cero." ? createElement(Feedback, { kind: state.confirmation === "error" ? "error" : "success" } as never, state.feedback) : null,
        createElement("div", { "data-ui-sale-actions": true },
          createElement(Action, { variant: "primary", pending, pendingLabel: "Confirmando…", disabled: state.lines.length === 0 || state.stale_price !== null, onClick: confirm }, "Confirmar venta"),
          createElement(Action, { variant: "tertiary", disabled: pending, onClick: () => { if (confirming.current) return; confirmationSequence.current += 1; draftDispatch({ type: "discard" }); setQuery(""); setPaymentErrors({}); } }, "Descartar borrador")))));
}
