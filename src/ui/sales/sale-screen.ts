import { createElement, type FormEvent, useEffect, useReducer, useRef, useState } from "react";
import { browseProducts, type ProductBrowseResult } from "../../commands/catalog.ts";
import { confirmSale, type ConfirmSaleRequest } from "../../commands/confirm-sale.ts";
import { Action, Feedback, Field } from "../visual-system/controls.ts";
import { Panel } from "../visual-system/structure.ts";
import { CheckoutDialog } from "../visual-system/checkout-dialog.ts";
import { createSaleFlow, draftLineSubtotalCentavos, draftTotalCentavos, draftTotalUnits, finalPriceCentavos, formatBs, initialSaleState, parseOptionalBs, type DraftLine } from "./sale-flow.ts";
import { createProductBrowserFlow, initialProductBrowserState, ProductBrowser } from "../catalog/product-browser.ts";
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

export function SaleScreen(props: { onInventoryAlertsRefresh?: () => void } = {}) {
  const [state, dispatch] = useReducer(createSaleFlow, initialSaleState);
  const [browser, browserDispatch] = useReducer(createProductBrowserFlow, initialProductBrowserState);
  const [paymentErrors, setPaymentErrors] = useState<Partial<Record<"amount_tendered_centavos" | "qr_applied_centavos", string>>>({});
  const [persistedDetails, setPersistedDetails] = useState<PersistedSummaryDetails | null>(null);
  const [checkoutOpen, setCheckoutOpen] = useState(false);
  const cashRef = useRef<HTMLInputElement>(null), qrRef = useRef<HTMLInputElement>(null), draftRef = useRef<HTMLElement>(null), checkoutInitialFocusRef = useRef<HTMLInputElement>(null), checkoutTriggerRef = useRef<HTMLButtonElement>(null);
  const searchSequence = useRef(0), confirmationSequence = useRef(0), confirming = useRef(false), mounted = useRef(true);
  useEffect(() => () => { mounted.current = false; searchSequence.current += 1; confirmationSequence.current += 1; }, []);
  useEffect(() => {
    if (state.focus_price_product_id === null || !mounted.current) return;
    draftRef.current?.querySelector<HTMLInputElement>(`#sale-final-price-${state.focus_price_product_id}`)?.focus();
  }, [state.focus_price_product_id]);
  const draftDispatch = (action: Parameters<typeof dispatch>[0]) => { if (!confirming.current) dispatch(action); };

  async function browsePage(query: string, category_id: number | null, page: number) {
    const attempt = Math.max(++searchSequence.current, browser.request_id + 1);
    searchSequence.current = attempt;
    browserDispatch({ type: "browse_started", query, category_id, stock_state: "all", activity: "active", page, request_id: attempt });
    try {
      const result = await browseProducts({ query, category_id, stock_state: "all", activity: "active", page, page_size: 20 });
      if (mounted.current && attempt === searchSequence.current) browserDispatch({ type: "browse_succeeded", request_id: attempt, result });
    } catch { if (mounted.current && attempt === searchSequence.current) browserDispatch({ type: "browse_failed", request_id: attempt, message: "No se pudo buscar en el catálogo local." }); }
  }
  const initialBrowseStarted = useRef(false);
  useEffect(() => {
    if (initialBrowseStarted.current) return;
    initialBrowseStarted.current = true;
    void browsePage("", null, 1);
  }, []);
  async function search(event: FormEvent<HTMLFormElement>) {
    event.preventDefault(); if (confirming.current) return;
    await browsePage(browser.query, browser.category_id, 1);
  }
  async function changePage(page: number) {
    if (page < 1 || confirming.current) return;
    await browsePage(browser.query, browser.category_id, page);
  }

  async function confirm() {
    if (confirming.current || state.lines.length === 0) return;
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
      if (response.kind === "success") { setPersistedDetails(projectPersistedSaleSummary(response)); dispatch({ type: "confirmation_succeeded", summary: response }); props.onInventoryAlertsRefresh?.(); }
      else if (response.kind === "minimum_price_violation") dispatch({ type: "minimum_price_violation", ...response });
      else if (response.kind === "stale_catalog_record") dispatch({ type: "confirmation_failed", message: "El catálogo cambió. Revisá el precio de venta e intentá nuevamente." });
      else if (response.code === "invalid_final_price") dispatch({ type: "final_price_validation_failed", message: response.message });
      else dispatch({ type: "confirmation_failed", message: failureByCode[response.code] ?? "No se pudo confirmar la venta. Intentá nuevamente." });
    } catch { if (mounted.current && attempt === confirmationSequence.current) dispatch({ type: "confirmation_failed", message: "No se pudo confirmar la venta. Intentá nuevamente." }); }
    finally { if (attempt === confirmationSequence.current) confirming.current = false; }
  }

  const discardDraft = () => {
    if (confirming.current) return;
    confirmationSequence.current += 1;
    draftDispatch({ type: "discard" });
    setCheckoutOpen(false);
    browserDispatch({ type: "browse_started", query: "", category_id: null, stock_state: "all", activity: "active", page: 1, request_id: ++searchSequence.current });
    setPaymentErrors({});
  };
  if (state.persisted_summary && persistedDetails) return createElement(PersistedSaleSummaryView, { details: persistedDetails, onNewSale: () => { setCheckoutOpen(false); setPersistedDetails(null); void browsePage("", null, 1); setPaymentErrors({}); dispatch({ type: "discard" }); } });
  const total = formatBs(draftTotalCentavos(state.lines));
  const totalUnits = draftTotalUnits(state.lines);
  const pending = state.confirmation === "pending";
  const addProduct = (product: ProductBrowseResult) => draftDispatch({ type: "add_product", product });
  const cartItems = state.lines.map((line, index) => createElement("li", { key: line.product_id, "data-ui-sale-cart-row": true },
    createElement("div", { "data-ui-sale-cart-primary": true },
      createElement("div", { "data-ui-sale-product": true },
        createElement("strong", null, line.product_name), createElement("span", { "data-ui-sku": true }, line.sku)),
      createElement("p", { "data-ui-sale-price-facts": true }, `Lista ${formatBs(line.list_price_centavos)} · Mín. ${formatBs(line.minimum_price_centavos)}`),
      createElement(Action, { variant: "tertiary", "aria-label": `Quitar ${line.product_name}`, disabled: pending, onClick: () => draftDispatch({ type: "remove_product", product_id: line.product_id }) }, "Quitar")),
    createElement("div", { "data-ui-sale-cart-controls": true },
      createElement(Field, { kind: "quantity", label: `Cantidad de ${line.product_name}`, error: state.feedback === "Ingresá una cantidad entera mayor que cero." ? state.feedback : undefined, control: createElement("input", { min: 1, value: line.quantity, disabled: pending, onChange: (event) => draftDispatch({ type: "line_quantity_changed", product_id: line.product_id, value: event.target.value }) }) } as never),
      createElement(Field, { kind: "money", label: "Precio de venta (Bs)", error: state.price_errors[line.product_id], control: createElement("input", { id: `sale-final-price-${line.product_id}`, ref: index === 0 ? checkoutInitialFocusRef : undefined, value: line.final_price_input, disabled: pending, onChange: (event) => draftDispatch({ type: "line_final_price_changed", product_id: line.product_id, value: event.target.value }) }) } as never),
      createElement("p", { "data-ui-sale-subtotal": true }, createElement("span", { "data-ui-money": true }, `Subtotal: ${formatBs(draftLineSubtotalCentavos(line))}`)))));
  const checkoutContent = createElement("div", { "data-ui-checkout-content": true },
    createElement("section", { "aria-labelledby": "checkout-cart-heading" },
      createElement("h3", { id: "checkout-cart-heading" }, "Carrito"),
      state.lines.length === 0 ? createElement(Feedback, { kind: "empty" } as never, "El carrito está vacío.") : null,
      createElement("ul", { "aria-label": "Carrito", "data-ui-sale-cart": true }, cartItems)),

    createElement("div", { "data-ui-checkout-rail": true },
      createElement("section", { "aria-labelledby": "checkout-payment-heading" },
        createElement("h3", { id: "checkout-payment-heading" }, "Pago"),
        createElement(Field, { kind: "money", label: "Efectivo recibido", error: paymentErrors.amount_tendered_centavos, control: createElement("input", { ref: cashRef, value: state.payment.amount_tendered_centavos, disabled: pending, onChange: (event) => { if (confirming.current) return; setPaymentErrors((old) => ({ ...old, amount_tendered_centavos: undefined })); dispatch({ type: "payment_changed", field: "amount_tendered_centavos", value: event.target.value }); } }) } as never),
        createElement(Field, { kind: "money", label: "Pago QR", error: paymentErrors.qr_applied_centavos, control: createElement("input", { ref: qrRef, value: state.payment.qr_applied_centavos, disabled: pending, onChange: (event) => { if (confirming.current) return; setPaymentErrors((old) => ({ ...old, qr_applied_centavos: undefined })); dispatch({ type: "payment_changed", field: "qr_applied_centavos", value: event.target.value }); } }) } as never)),
      createElement("p", { "data-ui-checkout-total": true, "data-ui-type": "total" }, `Total actual: ${total}`),
      state.feedback && state.feedback !== "Ingresá una cantidad entera mayor que cero." ? createElement(Feedback, { kind: state.confirmation === "error" ? "error" : "success" } as never, state.feedback) : null,
      createElement("div", { "data-ui-sale-actions": true },
        createElement(Action, { variant: "tertiary", disabled: pending, onClick: discardDraft }, "Descartar borrador"))));
  return createElement("main", { ref: draftRef, "aria-labelledby": "sale-heading", "aria-busy": pending || undefined, "data-ui-sale": true },
    createElement("h1", { id: "sale-heading" }, "Ventas"),
    createElement("div", { "data-ui-sale-layout": true },
      createElement(Panel, { label: "Catálogo" } as never,
        createElement(ProductBrowser, { state: browser, loadingMessage: "Buscando productos…", onQueryChange: (query) => browserDispatch({ type: "query_changed", value: query }), onCategoryChange: (category_id) => browserDispatch({ type: "category_changed", value: category_id }), onSubmit: search, onPageChange: changePage, onSelect: addProduct, actionLabel: "Agregar", disabledProductIds: new Set(state.lines.map((line) => line.product_id)), disabled: pending }) as never),
      createElement(Panel, { label: "Resumen de venta" } as never,
        createElement("div", { "data-ui-sale-summary": true },
          createElement("p", { "data-ui-quantity": true }, `Unidades: ${totalUnits}`),
          createElement("p", { "data-ui-type": "total" }, `Total: ${total}`),
          createElement("button", { ref: checkoutTriggerRef, type: "button", "data-ui-action": "primary", "aria-controls": "checkout-dialog", "aria-expanded": checkoutOpen, disabled: pending || state.lines.length === 0, onClick: () => setCheckoutOpen(true) } , "Revisar y cobrar")))),
    createElement(CheckoutDialog, { open: checkoutOpen, title: "Revisar y cobrar", description: "Revisá los productos, los precios y los medios de pago antes de confirmar la venta.", pending, confirmDisabled: state.lines.length === 0, initialFocusRef: checkoutInitialFocusRef, confirmLabel: "Confirmar venta", pendingLabel: "Confirmando…", onCancel: () => { if (!pending && !confirming.current) setCheckoutOpen(false); }, onConfirm: confirm, children: checkoutContent }));
}
