import { createElement, type ChangeEvent, type FormEvent, useEffect, useReducer, useState } from "react";

import { searchProducts } from "../../commands/catalog.ts";
import { inventoryCommands, type InventoryResponse } from "../../commands/inventory.ts";
import { createInventoryFlow, initialInventoryState, projectedBalance, type InventoryState } from "./inventory-flow.ts";

export const inventoryScreenDescription = "Inventory stock entry and physical count workflow.";

export function inventoryProductLabel(product: Awaited<ReturnType<typeof searchProducts>>[number]) {
  return `${product.sku} — ${product.name} (${product.available_quantity})`;
}

export function InventoryProductResults({ products, onSelect }: { products: Awaited<ReturnType<typeof searchProducts>>; onSelect: (product: Awaited<ReturnType<typeof searchProducts>>[number]) => void }) {
  return createElement("ul", { "aria-label": "Catalog results" }, products.map((product) => createElement("li", { key: product.product_id }, inventoryProductLabel(product), createElement("button", { type: "button", onClick: () => onSelect(product) }, "Select"))));
}

export function createInventoryCatalogInteraction(searchActiveProducts = searchProducts) {
  return { search: searchActiveProducts, select: (product: Awaited<ReturnType<typeof searchProducts>>[number]) => ({ type: "product_selected" as const, product }) };
}

export function InventoryOperationChoices({ operation, onChange }: { operation: InventoryState["operation"]; onChange: (operation: InventoryState["operation"]) => void }) {
  return createElement("label", null, "Operation", createElement("select", { value: operation, onChange: (event: ChangeEvent<HTMLSelectElement>) => onChange(event.target.value as InventoryState["operation"]) }, createElement("option", { value: "stock_entry" }, "Stock entry"), createElement("option", { value: "physical_count" }, "Physical count")));
}

export function InventoryScreen() {
  const [state, dispatch] = useReducer(createInventoryFlow, initialInventoryState);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Awaited<ReturnType<typeof searchProducts>>>([]);
  const catalog = createInventoryCatalogInteraction();
  const refreshAlerts = async () => {
    const response = await inventoryCommands.listAlerts();
    if (response.kind === "success") dispatch({ type: "alerts_refreshed", alerts: response.alerts });
  };
  useEffect(() => { void refreshAlerts(); }, []);
  const search = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    try { setResults(await catalog.search(query)); } catch { dispatch({ type: "confirmation_failed", message: "Unable to search the local catalog." }); }
  };
  const confirm = async () => {
    if (!state.product) return;
    const request_id = state.request_id ?? crypto.randomUUID();
    dispatch({ type: "confirmation_started", request_id });
    const response: InventoryResponse = state.operation === "stock_entry"
      ? await inventoryCommands.confirmStockEntry({ request_id, product_id: state.product.product_id, quantity: Number(state.entry_quantity), note: state.note || null })
      : await inventoryCommands.confirmPhysicalCount({ request_id, product_id: state.product.product_id, count: Number(state.physical_count), reason: state.reason });
    if (response.kind === "success") { dispatch({ type: "confirmation_succeeded", result: response }); await refreshAlerts(); }
    else dispatch({ type: "confirmation_failed", message: `${response.code}: ${response.message}` });
  };
  const projection = projectedBalance(state);
  return createElement("main", { "aria-labelledby": "inventory-heading" },
    createElement("h1", { id: "inventory-heading" }, "Inventory"),
    createElement("p", null, inventoryScreenDescription),
    createElement("form", { onSubmit: search },
      createElement("label", { htmlFor: "inventory-search" }, "Search catalog"),
      createElement("input", { id: "inventory-search", value: query, onChange: (event) => setQuery(event.target.value) }),
      createElement("button", { type: "submit" }, "Search")),
    createElement(InventoryProductResults, { products: results, onSelect: (product) => dispatch(catalog.select(product)) }),
    state.product ? createElement("section", { "aria-labelledby": "operation-heading" },
      createElement("h2", { id: "operation-heading" }, state.product.name),
      createElement(InventoryOperationChoices, { operation: state.operation, onChange: (operation) => dispatch({ type: "operation_changed", operation }) }),
      state.operation === "stock_entry"
        ? createElement("label", null, "Quantity", createElement("input", { type: "number", min: 1, step: 1, value: state.entry_quantity, onChange: (event) => dispatch({ type: "entry_quantity_changed", value: event.target.value }) }), createElement("input", { "aria-label": "Note", value: state.note, onChange: (event) => dispatch({ type: "note_changed", value: event.target.value }) }))
        : createElement("label", null, "Physical count", createElement("input", { type: "number", min: 0, step: 1, value: state.physical_count, onChange: (event) => dispatch({ type: "physical_count_changed", value: event.target.value }) }), createElement("input", { "aria-label": "Reason", required: true, value: state.reason, onChange: (event) => dispatch({ type: "reason_changed", value: event.target.value }) })),
      projection !== null ? createElement("p", null, `Projected balance: ${projection}`) : null,
      state.advisory_notice ? createElement("p", { role: "status" }, state.advisory_notice) : null,
      createElement("button", { type: "button", disabled: state.confirmation === "pending" || (state.operation === "physical_count" && !state.reason.trim()), onClick: confirm }, state.confirmation === "pending" ? "Saving…" : "Confirm operation"),
      createElement("button", { type: "button", onClick: () => dispatch({ type: "discard" }) }, "New operation")) : null,
    state.result ? createElement("p", { role: "status" }, `Saved ${state.result.quantity_delta}; balance is ${state.result.resulting_quantity}.`) : null,
    state.feedback ? createElement("p", { role: "alert" }, state.feedback) : null,
    createElement("section", { "aria-labelledby": "alerts-heading" }, createElement("h2", { id: "alerts-heading" }, `Alerts (${state.alerts.length})`), createElement("ul", null, state.alerts.map((alert) => createElement("li", { key: alert.product_id }, `${alert.classification === "out_of_stock" ? "Out of stock" : "Low stock"}: ${alert.product_name}`)))));
}
