import { createElement, type FormEvent, useEffect, useReducer, useRef, useState } from "react";
import { searchProducts } from "../../commands/catalog.ts";
import { confirmSale, type ConfirmSaleRequest } from "../../commands/confirm-sale.ts";
import { Action, Badge, Feedback, Field } from "../visual-system/controls.ts";
import { Panel } from "../visual-system/structure.ts";
import { catalogResultDetails } from "./catalog-result.ts";
import { createSaleFlow, draftLineSubtotalCentavos, draftTotalCentavos, effectiveDraftUnitPriceCentavos, finalPriceCentavos, formatBs, initialSaleState, parseOptionalBs, type DraftLine } from "./sale-flow.ts";
import { PersistedSaleSummaryView, projectPersistedSaleSummary, type PersistedSummaryDetails } from "./persisted-summary.ts";

const INVALID_BS = "Ingresá un monto válido en Bs, con hasta dos decimales.";
const INVALID_FINAL_PRICE = "Ingresá un precio de venta válido en Bs, con hasta dos decimales.";
const POSITIVE_FINAL_PRICE = "El precio de venta debe ser mayor que cero.";
const failureByCode: Record<string, string> = {
  invalid_request: "Revisá los datos de la venta e intentá nuevamente.", invalid_quantity: "Revisá que las cantidades sean números enteros mayores que cero.", invalid_payment: "Revisá los montos de pago e intentá nuevamente.", inactive_product: "Uno de los productos ya no está activo.", missing_product: "Uno de los productos ya no está disponible.", insufficient_stock: "No hay stock suficiente para completar la venta.", request_conflict: "El ID de solicitud ya fue usado con datos de venta diferentes. Revisá la venta antes de intentar nuevamente.", minimum_price_violation: "El precio de venta está por debajo del mínimo actual.", invalid_final_price: POSITIVE_FINAL_PRICE, persistence_failure: "No se pudo confirmar la venta. Intentá nuevamente.",
};
const requestId = () => crypto.randomUUID();
function finalPriceError(line: DraftLine): string | undefined {
  let price: number | null;
  try { price = parseOptionalBs(line.final_price_input); } catch { return INVALID_FINAL_PRICE; }
  if (price === null) return INVALID_FINAL_PRICE;
  if (price <= 0) return POSITIVE_FINAL_PRICE;
  return price < line.minimum_price_centavos ? `El precio de venta no puede ser menor que el precio mínimo de ${formatBs(line.minimum_price_centavos)}.` : undefined;
}

export function SaleScreen() {
  const [state, dispatch] = useReducer(createSaleFlow, initialSaleState);
  const [query, setQuery] = useState("");
  const [paymentErrors, setPaymentErrors] = useState<Partial<Record<"amount_tendered_centavos" | "qr_applied_centavos", string>>>({});
  const [persistedDetails, setPersistedDetails] = useState<PersistedSummaryDetails | null>(null);
  const cashRef = useRef<HTMLInputElement>(null), qrRef = useRef<HTMLInputElement>(null), draftRef = useRef<HTMLElement>(null);
  const searchSequence = useRef(0), confirmationSequence = useRef(0), confirming = useRef(false), mounted = useRef(true);
  useEffect(() => () => { mounted.current = false; searchSequence.current += 1; confirmationSequence.current += 1; }, []);
  useEffect(() => {
    if (state.focus_price_product_id === null || !mounted.current) return;
    draftRef.current?.querySelector<HTMLInputElement>(`#sale-final-price-${state.focus_price_product_id}`)?.focus();
  }, [state.focus_price_product_id]);
  const draftDispatch = (action: Parameters<typeof dispatch>[0]) => { if (!confirming.current) dispatch(action); };

  async function search(event: FormEvent<HTMLFormElement>) {
    event.preventDefault(); if (confirming.current) return;
    const attempt = ++searchSequence.current;
    dispatch({ type: "catalog_search_started", query, request_id: attempt });
    try { const results = await searchProducts(query); if (mounted.current && attempt === searchSequence.current) dispatch({ type: "catalog_search_succeeded", request_id: attempt, results }); }
    catch { if (mounted.current && attempt === searchSequence.current) dispatch({ type: "catalog_search_failed", request_id: attempt, message: "No se pudo buscar en el catálogo local." }); }
  }

  async function confirm() {
    if (confirming.current) return;
    let firstInvalidPrice: number | null = null;
    for (const line of state.lines) { if (finalPriceError(line)) { firstInvalidPrice ??= line.product_id; } }
    if (firstInvalidPrice !== null) {
      dispatch({ type: "line_final_price_changed", product_id: firstInvalidPrice, value: state.lines.find((line) => line.product_id === firstInvalidPrice)!.final_price_input });
      // The reducer keeps the controlled value and validates it; focus runs after this render.
      return;
    }
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
      lines: state.lines.map((line) => ({ product_id: line.product_id, quantity: line.quantity, captured_unit_price_centavos: line.captured_unit_price_centavos, captured_revision: line.captured_revision, final_unit_price_centavos: finalPriceCentavos(line)! })),
      payment: { amount_tendered_centavos: cash, qr_applied_centavos: qr },
    };
    try {
      const response = await confirmSale(request);
      if (!mounted.current || attempt !== confirmationSequence.current) return;
      if (response.kind === "success") { setPersistedDetails(projectPersistedSaleSummary(response)); dispatch({ type: "confirmation_succeeded", summary: response }); }
      else if (response.kind === "minimum_price_violation") dispatch({ type: "minimum_price_violation", ...response });
      else if (response.kind === "stale_catalog_record") dispatch({ type: "confirmation_failed", message: "El catálogo cambió. Revisá el precio de venta e intentá nuevamente." });
      else if (response.code === "invalid_final_price") dispatch({ type: "final_price_validation_failed", message: response.message });
      else dispatch({ type: "confirmation_failed", message: failureByCode[response.code] ?? "No se pudo confirmar la venta. Intentá nuevamente." });
    } catch { if (mounted.current && attempt === confirmationSequence.current) dispatch({ type: "confirmation_failed", message: "No se pudo confirmar la venta. Intentá nuevamente." }); }
    finally { if (attempt === confirmationSequence.current) confirming.current = false; }
  }

  if (state.persisted_summary && persistedDetails) return createElement(PersistedSaleSummaryView, { details: persistedDetails, onNewSale: () => { setPersistedDetails(null); setQuery(""); setPaymentErrors({}); dispatch({ type: "discard" }); } });
  const discovery = state.catalog_discovery;
  const discoveryFeedback = discovery.status === "initial" ? createElement(Feedback, { kind: "initial" } as never, "Buscá un producto para comenzar.") : discovery.status === "loading" ? createElement(Feedback, { kind: "loading", "aria-label": "Buscando productos…" } as never, "Buscando productos…") : discovery.status === "empty" ? createElement(Feedback, { kind: "empty" } as never, `No encontramos productos para “${discovery.query}”.`) : discovery.status === "error" ? createElement(Feedback, { kind: "error" } as never, discovery.error) : null;
  const total = formatBs(draftTotalCentavos(state.lines));
  const pending = state.confirmation === "pending";
  const catalogItems = discovery.results.map((product) => {
    const details = catalogResultDetails(product, { inCart: state.lines.some((line) => line.product_id === product.product_id) });
    return createElement("li", { key: details.product.id },
      createElement("div", null, createElement("strong", null, details.product.name), createElement("span", { "data-ui-sku": true }, details.product.sku), createElement("span", null, details.product.category)),
      createElement("span", { "data-ui-money": true }, details.price.text),
      createElement(Badge, { kind: details.stock.kind === "low" ? "low-stock" : details.stock.kind === "out" ? "out-of-stock" : "available", text: details.stock.text }),
      createElement(Action, { variant: "secondary", disabled: pending || details.availability !== "available", onClick: () => draftDispatch({ type: "add_product", product }) }, "Agregar"));
  });
  const cartItems = state.lines.map((line) => createElement("li", { key: line.product_id },
    createElement("div", null,
      createElement("strong", null, line.product_name), createElement("span", { "data-ui-sku": true }, line.sku),
      createElement("dl", { "data-ui-sale-price-facts": true },
        createElement("dt", null, "Precio de lista"), createElement("dd", { "data-ui-money": true }, formatBs(line.list_price_centavos)),
        createElement("dt", null, "Precio mínimo"), createElement("dd", { "data-ui-money": true }, formatBs(line.minimum_price_centavos)))),
    createElement(Field, { kind: "money", label: "Precio de venta (Bs)", error: state.price_errors[line.product_id], control: createElement("input", { id: `sale-final-price-${line.product_id}`, value: line.final_price_input, disabled: pending, onChange: (event) => draftDispatch({ type: "line_final_price_changed", product_id: line.product_id, value: event.target.value }) }) } as never),
    createElement(Field, { kind: "quantity", label: `Cantidad de ${line.product_name}`, error: state.feedback === "Ingresá una cantidad entera mayor que cero." ? state.feedback : undefined, control: createElement("input", { min: 1, value: line.quantity, disabled: pending, onChange: (event) => draftDispatch({ type: "line_quantity_changed", product_id: line.product_id, value: event.target.value }) }) } as never),
    createElement("span", { "data-ui-money": true }, `Subtotal: ${formatBs(draftLineSubtotalCentavos(line))}`),
    createElement(Action, { variant: "tertiary", disabled: pending, onClick: () => draftDispatch({ type: "remove_product", product_id: line.product_id }) }, "Quitar")));
  return createElement("main", { ref: draftRef, "aria-labelledby": "sale-heading", "aria-busy": pending || undefined, "data-ui-sale": true },
    createElement("h1", { id: "sale-heading" }, "Ventas"),
    createElement("div", { "data-ui-sale-layout": true },
      createElement(Panel, { label: "Catálogo" } as never,
        createElement("form", { onSubmit: search, "data-ui-sale-search": true },
          createElement(Field, { kind: "search", label: "Buscar en el catálogo", control: createElement("input", { value: query, disabled: pending, onChange: (event) => { if (!confirming.current) setQuery(event.target.value); } }) } as never),
          createElement(Action, { variant: "secondary", type: "submit", disabled: pending }, "Buscar")),
        discoveryFeedback, createElement("ul", { "aria-label": "Resultados del catálogo", "data-ui-sale-list": true }, catalogItems)),
      createElement(Panel, { label: "Carrito" } as never,
        state.lines.length === 0 ? createElement(Feedback, { kind: "empty" } as never, "El carrito está vacío.") : null,
        createElement("ul", { "aria-label": "Carrito", "data-ui-sale-list": true, "data-ui-sale-cart": true }, cartItems)),
      createElement(Panel, { label: "Pago" } as never,
        createElement(Field, { kind: "money", label: "Efectivo recibido", error: paymentErrors.amount_tendered_centavos, control: createElement("input", { ref: cashRef, value: state.payment.amount_tendered_centavos, disabled: pending, onChange: (event) => { if (confirming.current) return; setPaymentErrors((old) => ({ ...old, amount_tendered_centavos: undefined })); dispatch({ type: "payment_changed", field: "amount_tendered_centavos", value: event.target.value }); } }) } as never),
        createElement(Field, { kind: "money", label: "Pago QR", error: paymentErrors.qr_applied_centavos, control: createElement("input", { ref: qrRef, value: state.payment.qr_applied_centavos, disabled: pending, onChange: (event) => { if (confirming.current) return; setPaymentErrors((old) => ({ ...old, qr_applied_centavos: undefined })); dispatch({ type: "payment_changed", field: "qr_applied_centavos", value: event.target.value }); } }) } as never)),
      createElement(Panel, { label: "Resumen" } as never,
        createElement("p", { "data-ui-type": "total" }, `Total: ${total}`),
        state.feedback && state.feedback !== "Ingresá una cantidad entera mayor que cero." ? createElement(Feedback, { kind: state.confirmation === "error" ? "error" : "success" } as never, state.feedback) : null,
        createElement("div", { "data-ui-sale-actions": true },
          createElement(Action, { variant: "primary", pending, pendingLabel: "Confirmando…", disabled: state.lines.length === 0, onClick: confirm }, "Confirmar venta"),
          createElement(Action, { variant: "tertiary", disabled: pending, onClick: () => { if (confirming.current) return; confirmationSequence.current += 1; draftDispatch({ type: "discard" }); setQuery(""); setPaymentErrors({}); } }, "Descartar borrador")))));
}
