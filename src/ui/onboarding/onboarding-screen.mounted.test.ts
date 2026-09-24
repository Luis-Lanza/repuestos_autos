import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { OnboardingScreen } from "./onboarding-screen.ts";

const category = { category_id: 1, name: "Filtros", fields: [{ definition_id: 10, label: "Marca", field_type: "text", required: true, options: [] }] };
const success = () => ({ kind: "success", categories: [category] });

test("shows loading then exact empty-category guidance", async () => {
  let resolve!: (value: unknown) => void;
  mockIPC((command) => command === "list_categories_command" ? new Promise((done) => { resolve = done; }) : undefined);
  render(createElement(OnboardingScreen, { onBack: () => undefined }));
  assert.equal(screen.getByRole("status").textContent, "Cargando categorías…");
  resolve({ kind: "success", categories: [] });
  assert.ok(await screen.findByText("Creá una categoría para habilitar el alta de productos.", { exact: true }));
  assert.equal(screen.queryByLabelText("Categoría"), null);
});
test("renders shared Spanish panels and submits required purchase and sale prices", async () => {
  let request: unknown;
  mockIPC((command, payload) => command === "list_categories_command" ? success() : command === "create_product_command" ? (request = payload, { kind: "success", product_id: 2, sku: "FIL-1", name: "Filtro", category_id: 1, category_name: "Filtros", purchase_price_centavos: 8000, sale_price_centavos: 12550, minimum_sale_price_centavos: 10000, available_quantity: 3, active: true }) : undefined);
  const user = userEvent.setup({ document }); render(createElement(OnboardingScreen, { onBack: () => undefined }));
  assert.ok(screen.getByRole("heading", { name: "Alta de productos", level: 1 })); assert.ok(screen.getByRole("heading", { name: "Crear categoría", level: 2 }));
  await screen.findByLabelText("Marca"); assert.ok(screen.getByRole("heading", { name: "Crear producto activo", level: 2 }));
  await user.type(screen.getByRole("textbox", { name: "SKU" }), "FIL-1"); await user.type(screen.getByRole("textbox", { name: "Nombre del producto" }), "Filtro"); await user.type(screen.getByRole("textbox", { name: "Precio de compra (Bs)" }), "80,00"); await user.type(screen.getByRole("textbox", { name: "Precio de lista (Bs)" }), "125,50"); await user.type(screen.getByRole("textbox", { name: "Precio mínimo de venta (Bs)" }), "100,00"); await user.type(screen.getByRole("spinbutton", { name: "Stock inicial (unidades enteras)" }), "3"); await user.type(screen.getByRole("textbox", { name: "Marca" }), "ACDelco"); await user.click(screen.getByRole("button", { name: "Crear producto" }));
  assert.deepEqual(request, { request: { sku: "FIL-1", name: "Filtro", category_id: 1, purchase_price_centavos: 8000, sale_price_centavos: 12550, minimum_sale_price_centavos: 10000, opening_quantity: 3, attribute_values: [{ definition_id: 10, value: "ACDelco" }] } }); assert.equal((await screen.findByRole("status")).textContent, "Producto creado: FIL-1. Stock inicial: 3 unidades.");
});
test("prevents duplicate category submission and localizes failure", async () => {
  let resolve!: (value: unknown) => void, calls = 0;
  mockIPC((command) => command === "list_categories_command" ? { kind: "success", categories: [] } : command === "create_category_command" ? (calls++, new Promise((done) => { resolve = done; })) : undefined);
  const user = userEvent.setup({ document }); render(createElement(OnboardingScreen, { onBack: () => undefined })); await user.type(screen.getByRole("textbox", { name: "Nombre de la categoría" }), "Filtros");
  const submit = screen.getByRole("button", { name: "Crear categoría" }); await user.click(submit); await user.click(submit); assert.equal(calls, 1); assert.ok(screen.getByRole("button", { name: "Creando categoría…" })); resolve({ kind: "error", code: "category_failed", message: "native detail" }); await waitFor(() => assert.equal(screen.getByRole("alert").textContent, "No se pudo crear la categoría."));
});
test("focuses the first invalid product field", async () => {
  mockIPC((command) => command === "list_categories_command" ? success() : undefined); const user = userEvent.setup({ document }); render(createElement(OnboardingScreen, { onBack: () => undefined })); await screen.findByLabelText("Marca");
  await user.type(screen.getByRole("textbox", { name: "SKU" }), "FIL-1"); await user.type(screen.getByRole("textbox", { name: "Nombre del producto" }), "Filtro"); await user.type(screen.getByRole("textbox", { name: "Precio de compra (Bs)" }), "0"); await user.click(screen.getByRole("button", { name: "Crear producto" }));
  const purchasePrice = screen.getByRole("textbox", { name: "Precio de compra (Bs)" });
  assert.equal(document.activeElement, purchasePrice);
  assert.equal(purchasePrice.getAttribute("aria-invalid"), "true");
  assert.equal(screen.getByText("Ingresá un precio de compra positivo y válido en Bs.").id, "purchase-price-error");
});
test("ignores product completion after unmount", async () => {
  let resolve!: (value: unknown) => void;
  mockIPC((command) => command === "list_categories_command" ? success() : command === "create_product_command" ? new Promise((done) => { resolve = done; }) : undefined);
  const user = userEvent.setup({ document }); const view = render(createElement(OnboardingScreen, { onBack: () => undefined })); await screen.findByLabelText("Marca");
  await user.type(screen.getByRole("textbox", { name: "SKU" }), "FIL-1"); await user.type(screen.getByRole("textbox", { name: "Nombre del producto" }), "Filtro"); await user.type(screen.getByRole("textbox", { name: "Precio de compra (Bs)" }), "1"); await user.type(screen.getByRole("textbox", { name: "Precio de lista (Bs)" }), "1"); await user.type(screen.getByRole("textbox", { name: "Precio mínimo de venta (Bs)" }), "1"); await user.type(screen.getByRole("spinbutton", { name: "Stock inicial (unidades enteras)" }), "1"); await user.type(screen.getByRole("textbox", { name: "Marca" }), "ACDelco"); await user.click(screen.getByRole("button", { name: "Crear producto" })); view.unmount(); resolve({ kind: "success", product_id: 2, sku: "FIL-1", name: "Filtro", category_id: 1, category_name: "Filtros", purchase_price_centavos: 100, sale_price_centavos: 100, minimum_sale_price_centavos: 100, available_quantity: 1, active: true }); await Promise.resolve(); assert.equal(document.body.textContent?.includes("Producto creado"), false);
});
