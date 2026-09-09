import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { InventoryScreen } from "./inventory-screen.ts";

const product = { product_id: 1, sku: "FLT", name: "Filter", category_name: "Filters", available_quantity: 8, catalog_unit_price_centavos: 2500, revision: 0 };
const success = (request_id: string) => ({ kind: "success", request_id, product_id: 1, previous_quantity: 10, quantity_delta: 3, resulting_quantity: 11, occurred_at: "2025-01-01T00:00:00Z", note: null });

async function searchAndSelect() {
  const user = userEvent.setup({ document });
  await user.type(screen.getByRole("searchbox", { name: "Buscar producto" }), "filter{Enter}");
  const select = await screen.findByRole("button", { name: "Seleccionar" });
  select.focus();
  await user.keyboard("{Enter}");
  return user;
}

test("renders Spanish selection, whole-unit projection, pending lock, and success", async () => {
  let resolve!: (value: ReturnType<typeof success>) => void;
  let requestId = "";
  let confirmations = 0;
  mockIPC((command, payload) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "search_products_command") return [product];
    if (command === "confirm_stock_entry_command") {
      confirmations += 1;
      requestId = String((payload?.request as { request_id?: unknown }).request_id);
      return new Promise((done) => { resolve = done; });
    }
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen, {}));
  screen.getByRole("heading", { name: "Inventario", level: 1 });
  const user = await searchAndSelect();
  assert.ok(screen.getByText("Filter"));
  assert.ok(screen.getByText("SKU: FLT"));
  assert.ok(screen.getByText("Stock actual: 8"));
  await user.type(screen.getByRole("spinbutton", { name: "Cantidad (unidades enteras)" }), "3");
  assert.ok(screen.getByText("Saldo proyectado: 11"));
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  const saving = screen.getByRole("button", { name: "Guardando…" });
  assert.equal((saving as HTMLButtonElement).disabled, true);
  await user.click(saving);
  assert.equal(confirmations, 1);
  resolve(success(requestId));
  assert.ok(await screen.findByText("Operación guardada. Stock actual: 11."));
  assert.ok(screen.getByText("Saldo proyectado desactualizado. Revisá el stock actual."));
});

test("shows loading/no-results, physical-count validation, and exact prioritized stock cues", async () => {
  let searches = 0;
  let resolveSearch!: (value: unknown) => void;
  let physicalCount: unknown;
  mockIPC((command, payload) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [
      { product_id: 2, product_name: "Bujía", quantity: 1, classification: "low_stock" },
      { product_id: 3, product_name: "Correa", quantity: 0, classification: "out_of_stock" },
    ] };
    if (command === "search_products_command") return ++searches === 1 ? new Promise((resolve) => { resolveSearch = resolve; }) : [product];
    if (command === "confirm_physical_count_command") { physicalCount = (payload?.request as { count?: unknown }).count; return success("count-request"); }
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen, {}));
  assert.ok(screen.getByText("Seleccioná un producto para comenzar."));
  const user = userEvent.setup({ document });
  const search = screen.getByRole("searchbox", { name: "Buscar producto" });
  await user.type(search, "nada{Enter}");
  assert.ok(screen.getByText("Buscando productos…"));
  resolveSearch([]);
  assert.ok(await screen.findByText("No encontramos productos para “nada”."));
  await user.clear(search);
  await user.type(search, "filter{Enter}");
  await user.click(await screen.findByRole("button", { name: "Seleccionar" }));
  await user.selectOptions(screen.getByRole("combobox", { name: "Operación" }), "physical_count");
  await user.type(screen.getByRole("textbox", { name: "Motivo" }), "Producto dañado");
  assert.equal(screen.queryByText("Saldo proyectado: 0"), null);
  assert.equal((screen.getByRole("button", { name: "Confirmar operación" }) as HTMLButtonElement).disabled, true);
  await user.type(screen.getByRole("spinbutton", { name: "Conteo físico (unidades enteras)" }), "0");
  assert.ok(screen.getByText("Saldo proyectado: 0"));
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  await waitFor(() => assert.equal(physicalCount, 0));
  const items = within(screen.getByRole("region", { name: "Alertas de stock" })).getAllByRole("listitem");
  assert.match(items[0].textContent ?? "", /Sin stock: 0.*Correa/);
  assert.match(items[1].textContent ?? "", /Stock bajo: 1.*Bujía/);
});

test("localizes failure and preserves retry", async () => {
  mockIPC((command) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "search_products_command") return [product];
    if (command === "confirm_stock_entry_command") return { kind: "error", code: "persistence_failure", message: "Native" };
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen, {}));
  const user = await searchAndSelect();
  await user.type(screen.getByRole("spinbutton", { name: "Cantidad (unidades enteras)" }), "3");
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  assert.match((await screen.findByRole("alert")).textContent ?? "", /^No se pudo guardar la operación de inventario\. Reintentá\./);
  assert.ok(screen.getByRole("button", { name: "Reintentar" }));
});

test("clears the public stock cue while alerts load or are unavailable", async () => {
  let resolveAlerts!: (value: unknown) => void;
  let calls = 0;
  mockIPC((command) => {
    if (command === "list_inventory_alerts_command") {
      calls += 1;
      if (calls === 1) return new Promise((resolve) => { resolveAlerts = resolve; });
      return { kind: "alerts", alerts: [] };
    }
    throw new Error(`Unexpected command: ${command}`);
  });
  const cues: Array<string | null> = [];
  render(createElement(InventoryScreen, { onAlertCueChange: (cue) => cues.push(cue) }));
  assert.ok(screen.getByText("Cargando alertas de stock…"));
  await waitFor(() => assert.equal(typeof resolveAlerts, "function"));
  resolveAlerts({ kind: "error", code: "persistence_failure", message: "Native" });
  assert.ok(await screen.findByText("Las alertas de stock no están disponibles."));
  assert.equal(cues.at(-1), null);
  await userEvent.click(screen.getByRole("button", { name: "Reintentar" }));
  await waitFor(() => assert.equal(calls, 2));
  assert.ok(await screen.findByText("No hay alertas de stock."));
});
