import { createElement, type FormEvent, useCallback, useEffect, useId, useReducer, useRef, useState } from "react";

import { browseProducts, type ProductBrowseResult, type ProductSearchResult, type ProductStockState } from "../../commands/catalog.ts";
import { inventoryCommands, type InventoryResponse } from "../../commands/inventory.ts";
import { Action, Badge, Feedback, Field } from "../visual-system/controls.ts";
import { Panel } from "../visual-system/structure.ts";
import { createInventoryFlow, initialInventoryState, projectedBalance, type InventoryState } from "./inventory-flow.ts";
import { createProductBrowserFlow, initialProductBrowserState, ProductBrowser } from "../catalog/product-browser.ts";

export const inventoryScreenDescription = "Operaciones de entrada de stock y conteo físico.";
type Product = ProductBrowseResult;
type LoadState = "initial" | "loading" | "ready" | "unavailable";

export function inventoryProductLabel(product: Product) { return `${product.sku} — ${product.name} (${product.available_quantity})`; }

export function InventoryProductResults({ products, onSelect }: { products: Product[]; onSelect: (product: Product) => void }) {
  return createElement("ul", { "aria-label": "Resultados del catálogo", "data-ui-inventory-results": true }, products.map((product) =>
    createElement("li", { key: product.product_id }, createElement("span", null, inventoryProductLabel(product)), createElement(Action, { variant: "secondary", onClick: () => onSelect(product) }, "Seleccionar"))));
}

export function createInventoryCatalogInteraction(searchActiveProducts: (query: string) => Promise<ProductSearchResult[]> = async () => []) {
  return { search: searchActiveProducts, select: (product: Product | ProductSearchResult) => ({ type: "product_selected" as const, product }) };
}

export function InventoryOperationChoices({ operation, onChange, disabled = false }: { operation: InventoryState["operation"]; onChange: (operation: InventoryState["operation"]) => void; disabled?: boolean }) {
  const name = useId();
  return createElement("fieldset", { "data-ui-inventory-choices": true, disabled },
    createElement("legend", null, "Operación"),
    createElement("div", { "data-ui-inventory-choice-list": true },
      ([
        ["stock_entry", "Entrada de stock", "Sumar unidades al inventario"],
        ["physical_count", "Conteo físico", "Ajuste directo de existencias"],
      ] as const).map(([value, label, description]) => createElement("label", { key: value, "data-ui-inventory-choice": true },
        createElement("input", { type: "radio", name, value, checked: operation === value, disabled, onChange: () => onChange(value) }),
        createElement("span", null, createElement("strong", null, label), createElement("small", null, description))))));
}

export function InventoryScreen(props: { onAlertCueChange?: (cue: string | null) => void; onInventoryAlertsRefresh?: () => void; initialStockState?: ProductStockState }) {
  const onAlertCueChange = props?.onAlertCueChange;
  const [state, dispatch] = useReducer(createInventoryFlow, initialInventoryState);
  const [browser, browserDispatch] = useReducer(createProductBrowserFlow, { ...initialProductBrowserState, stock_state: props.initialStockState ?? "all" });
  const [alertState, setAlertState] = useState<LoadState>("loading");
  const mounted = useRef(true);
  const searchAttempt = useRef(0);
  const alertAttempt = useRef(0);
  const confirmLocked = useRef(false);
  const catalog = createInventoryCatalogInteraction();

  useEffect(() => () => { mounted.current = false; searchAttempt.current += 1; alertAttempt.current += 1; confirmLocked.current = true; }, []);
  const refreshAlerts = useCallback(async () => {
    setAlertState("loading");
    const attempt = ++alertAttempt.current;
    const response = await inventoryCommands.listAlerts();
    if (!mounted.current || attempt !== alertAttempt.current) return;
    if (response.kind === "success") { dispatch({ type: "alerts_refreshed", alerts: response.alerts }); setAlertState("ready"); }
    else setAlertState("unavailable");
  }, []);
  useEffect(() => { void refreshAlerts(); }, [refreshAlerts]);
  useEffect(() => {
    const cue = alertState === "ready" && state.alerts.length ? `${state.alerts.length} ${state.alerts.length === 1 ? "alerta" : "alertas"} de stock` : null;
    onAlertCueChange?.(cue);
  }, [alertState, state.alerts.length, onAlertCueChange]);

  async function browsePage(query: string, category_id: number | null, stock_state: ProductStockState, page: number) {
    const attempt = Math.max(++searchAttempt.current, browser.request_id + 1);
    searchAttempt.current = attempt;
    browserDispatch({ type: "browse_started", query, category_id, stock_state, activity: "active", page, request_id: attempt });
    try {
      const result = await browseProducts({ query, category_id, stock_state, activity: "active", page, page_size: 20 });
      if (mounted.current && attempt === searchAttempt.current) browserDispatch({ type: "browse_succeeded", request_id: attempt, result });
    } catch { if (mounted.current && attempt === searchAttempt.current) browserDispatch({ type: "browse_failed", request_id: attempt, message: "No se pudo buscar en el catálogo local. Reintentá." }); }
  }
  const initialBrowseStarted = useRef(false);
  useEffect(() => {
    if (initialBrowseStarted.current) return;
    initialBrowseStarted.current = true;
    void browsePage("", null, props.initialStockState ?? "all", 1);
  }, []);
  const search = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    await browsePage(browser.query, browser.category_id, browser.stock_state, 1);
  };
  const changePage = async (page: number) => {
    if (page < 1) return;
    await browsePage(browser.query, browser.category_id, browser.stock_state, page);
  };
  const confirm = async () => {
    if (!state.product || confirmLocked.current) return;
    confirmLocked.current = true;
    const request_id = state.request_id ?? crypto.randomUUID();
    dispatch({ type: "confirmation_started", request_id });
    const response: InventoryResponse = state.operation === "stock_entry"
      ? await inventoryCommands.confirmStockEntry({ request_id, product_id: state.product.product_id, quantity: Number(state.entry_quantity), note: state.note || null })
      : await inventoryCommands.confirmPhysicalCount({ request_id, product_id: state.product.product_id, count: Number(state.physical_count), reason: state.reason });
    if (!mounted.current) return;
    confirmLocked.current = false;
    if (response.kind === "success") {
      dispatch({ type: "confirmation_succeeded", result: response });
      props.onInventoryAlertsRefresh?.();
      await refreshAlerts();
    } else dispatch({ type: "confirmation_failed", message: response.code === "request_conflict" ? "El ID de solicitud ya fue usado con datos de inventario diferentes. Reintentá con los datos correctos." : "No se pudo guardar la operación de inventario. Reintentá." });
  };

  const projection = projectedBalance(state);
  const pending = state.confirmation === "pending";
  const value = state.operation === "stock_entry" ? state.entry_quantity : state.physical_count;
  const whole = value.trim() !== "" && Number.isSafeInteger(Number(value)) && (state.operation === "stock_entry" ? Number(value) > 0 : Number(value) >= 0);
  const valid = whole && (state.operation !== "physical_count" || Boolean(state.reason.trim()));
  const sortedAlerts = [...state.alerts].sort((a, b) => a.classification === b.classification ? 0 : a.classification === "out_of_stock" ? -1 : 1);

  const sparseBrowser = browser.status === "results" && (browser.result?.products.length ?? 0) <= 3;
  return createElement("main", {
    "aria-labelledby": "inventory-heading",
    "data-ui-inventory": true,
    "data-ui-density": sparseBrowser ? "sparse" : undefined,
  },
    createElement("header", { "data-ui-inventory-header": true },
      createElement("h1", { id: "inventory-heading" }, "Inventario"),
      createElement("p", null, inventoryScreenDescription)),
    createElement("div", { "data-ui-inventory-layout": true },
      createElement(Panel, { label: "Operación de inventario" } as never,
        !state.product && browser.status === "results" ? createElement("p", { "data-ui-inventory-intro": true }, "Seleccioná un producto para comenzar.") : null,
        !state.product ? createElement(ProductBrowser, { state: browser, loadingMessage: "Buscando productos…", searchLabel: "Buscar producto", initialMessage: "Seleccioná un producto para comenzar.", onQueryChange: (value) => browserDispatch({ type: "query_changed", value }), onCategoryChange: (value) => browserDispatch({ type: "category_changed", value }), onStockStateChange: (value) => browserDispatch({ type: "stock_state_changed", value }), onSubmit: search, onPageChange: changePage, onSelect: (product) => dispatch(catalog.select(product)), actionLabel: "Seleccionar", allowUnavailableSelection: true, disabled: pending }) as never : null,
        state.product ? createElement("div", { "data-ui-inventory-operation": true, "aria-busy": pending || undefined },
          createElement("div", { "data-ui-inventory-selection": true },
            createElement("div", { "data-ui-inventory-identity": true }, createElement("strong", null, state.product.name), createElement("span", { "data-ui-kind": "sku" }, `SKU: ${(state.product as Product).sku}`)),
            createElement("span", { "data-ui-inventory-stock": true }, `Stock actual: ${state.product.available_quantity}`)),
          createElement(InventoryOperationChoices, { operation: state.operation, disabled: pending, onChange: (operation) => dispatch({ type: "operation_changed", operation }) }),
          createElement("div", { "data-ui-inventory-fields": true }, state.operation === "stock_entry"
            ? createElement(Field, { kind: "quantity", label: "Cantidad (unidades enteras)", hint: "Solo unidades enteras positivas.", error: value && !whole ? "Ingresá una cantidad entera mayor que cero." : undefined, control: createElement("input", { min: 1, value, disabled: pending, onChange: (event) => dispatch({ type: "entry_quantity_changed", value: event.target.value }) }) } as never)
            : createElement(Field, { kind: "quantity", label: "Conteo físico (unidades enteras)", error: value && !whole ? "Ingresá un conteo entero igual o mayor que cero." : undefined, control: createElement("input", { min: 0, value, disabled: pending, onChange: (event) => dispatch({ type: "physical_count_changed", value: event.target.value }) }) } as never),
            createElement(Field, { kind: "text", label: state.operation === "stock_entry" ? "Nota (opcional)" : "Motivo", error: state.operation === "physical_count" && !state.reason.trim() ? "Ingresá el motivo del conteo físico." : undefined, control: createElement("input", { required: state.operation === "physical_count", value: state.operation === "stock_entry" ? state.note : state.reason, disabled: pending, onChange: (event) => dispatch({ type: state.operation === "stock_entry" ? "note_changed" : "reason_changed", value: event.target.value } as Parameters<typeof dispatch>[0]) }) } as never)),
          projection !== null ? createElement("p", { "data-ui-inventory-projection": true }, `Saldo proyectado: ${projection}`) : null,
          state.advisory_notice ? createElement(Feedback, { kind: "stale" } as never, "Saldo proyectado desactualizado. Revisá el stock actual.") : null,
          createElement("div", { "data-ui-inventory-actions": true }, createElement(Action, { variant: "tertiary", disabled: pending, onClick: () => dispatch({ type: "discard" }) }, "Nueva operación"), createElement(Action, { variant: "primary", pending, pendingLabel: "Guardando…", disabled: !valid, onClick: confirm }, "Confirmar operación")),
          state.result ? createElement(Feedback, { kind: "success" } as never, `Operación guardada. Stock actual: ${state.result.resulting_quantity}.`) : null,
          state.feedback ? createElement(Feedback, { kind: "error" } as never, createElement("span", null, state.feedback, " ", createElement(Action, { variant: "tertiary", onClick: confirm }, "Reintentar"))) : null) : null),
      createElement(Panel, { label: "Alertas de stock" } as never,
        createElement("p", { "data-ui-inventory-alert-description": true }, "Productos en catálogo activo con stock crítico o agotado. Solo lectura."),
        alertState === "loading" ? createElement(Feedback, { kind: "loading" } as never, "Cargando alertas de stock…")
          : alertState === "unavailable" ? createElement(Feedback, { kind: "unavailable" } as never, createElement("span", null, "Las alertas de stock no están disponibles. ", createElement(Action, { variant: "tertiary", onClick: refreshAlerts }, "Reintentar")))
          : sortedAlerts.length === 0 ? createElement(Feedback, { kind: "empty" } as never, "No hay alertas de stock.")
          : createElement("ul", { "data-ui-inventory-alerts": true }, sortedAlerts.map((alert) => createElement("li", { key: alert.product_id }, createElement(Badge, { kind: alert.classification === "out_of_stock" ? "out-of-stock" : "low-stock", text: alert.classification === "out_of_stock" ? `Sin stock: ${alert.quantity}` : `Stock bajo: ${alert.quantity}` }), createElement("span", null, alert.product_name)))))));
}
