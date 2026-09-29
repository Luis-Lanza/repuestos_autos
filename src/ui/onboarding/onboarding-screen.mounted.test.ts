import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { OnboardingScreen } from "./onboarding-screen.ts";

const category = { category_id: 1, name: "Filtros", fields: [{ definition_id: 10, label: "Marca", field_type: "text", required: true, options: [] }] };
const decimalCategory = { category_id: 1, name: "Filtros", fields: [{ definition_id: 11, label: "Longitud", field_type: "number", required: true, options: [] }] };
const success = () => ({ kind: "success", categories: [category] });
async function enterValidProduct(user: ReturnType<typeof userEvent.setup>) {
  await user.type(screen.getByRole("textbox", { name: "SKU" }), "FIL-1");
  await user.type(screen.getByRole("textbox", { name: "Nombre del producto" }), "Filtro");
  await user.type(screen.getByRole("textbox", { name: "Precio de compra (Bs)" }), "80,00");
  await user.type(screen.getByRole("textbox", { name: "Precio de venta (Bs)" }), "125,50");
  await user.type(screen.getByRole("textbox", { name: "Precio mínimo de venta (Bs)" }), "100,00");
  await user.type(screen.getByRole("spinbutton", { name: "Stock inicial (unidades enteras)" }), "3");
  await user.type(screen.getByRole("textbox", { name: "Marca" }), "ACDelco");
}

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
  await user.type(screen.getByRole("textbox", { name: "SKU" }), "FIL-1"); await user.type(screen.getByRole("textbox", { name: "Nombre del producto" }), "Filtro"); await user.type(screen.getByRole("textbox", { name: "Precio de compra (Bs)" }), "80,00"); await user.type(screen.getByRole("textbox", { name: "Precio de venta (Bs)" }), "125,50"); await user.type(screen.getByRole("textbox", { name: "Precio mínimo de venta (Bs)" }), "100,00"); await user.type(screen.getByRole("spinbutton", { name: "Stock inicial (unidades enteras)" }), "3"); await user.type(screen.getByRole("textbox", { name: "Marca" }), "ACDelco"); await user.click(screen.getByRole("button", { name: "Crear producto" }));
  assert.deepEqual(request, { request: { sku: "FIL-1", name: "Filtro", category_id: 1, purchase_price_centavos: 8000, sale_price_centavos: 12550, minimum_sale_price_centavos: 10000, low_stock_threshold: 1, opening_quantity: 3, attribute_values: [{ definition_id: 10, value: "ACDelco" }] } }); assert.equal((await screen.findByText("Producto creado: FIL-1. Stock inicial: 3 unidades.", { exact: true })).getAttribute("role"), "status");
});
test("submits a chosen low-stock threshold and rejects zero or fractional values with accessible field feedback", async () => {
  let request: unknown;
  let createCalls = 0;
  mockIPC((command, payload) => command === "list_categories_command" ? success() : command === "create_product_command" ? (createCalls++, request = payload, { kind: "success", product_id: 2, sku: "FIL-1", name: "Filtro", category_id: 1, category_name: "Filtros", purchase_price_centavos: 8000, sale_price_centavos: 12550, minimum_sale_price_centavos: 10000, low_stock_threshold: 4, available_quantity: 3, active: true }) : undefined);
  const user = userEvent.setup({ document });
  render(createElement(OnboardingScreen, { onBack: () => undefined }));
  await screen.findByLabelText("Marca");
  await enterValidProduct(user);
  const threshold = screen.getByRole("spinbutton", { name: "Umbral de stock bajo (opcional)" });
  await user.type(threshold, "4");
  await user.click(screen.getByRole("button", { name: "Crear producto" }));
  await screen.findByText("Producto creado: FIL-1. Stock inicial: 3 unidades.", { exact: true });
  assert.equal((request as { request: Record<string, unknown> }).request.low_stock_threshold, 4);

  await enterValidProduct(user);
  await user.type(screen.getByRole("spinbutton", { name: "Umbral de stock bajo (opcional)" }), "0");
  await user.click(screen.getByRole("button", { name: "Crear producto" }));
  const invalid = screen.getByRole("spinbutton", { name: "Umbral de stock bajo (opcional)" });
  assert.equal(invalid.getAttribute("aria-invalid"), "true");
  assert.equal(screen.getByText("Ingresá un umbral entero mayor o igual a 1.").id, "low-stock-threshold-error");
  assert.equal(document.activeElement, invalid);
  assert.equal(createCalls, 1);
  await user.clear(invalid);
  await user.type(invalid, "1.5");
  await user.click(screen.getByRole("button", { name: "Crear producto" }));
  assert.equal(screen.getByRole("spinbutton", { name: "Umbral de stock bajo (opcional)" }).getAttribute("aria-invalid"), "true");
  assert.equal(createCalls, 1);
});

test("reloads changed category fields and blocks submission with obsolete definition IDs", async () => {
  let categoryLoads = 0, productCalls = 0;
  mockIPC((command) => {
    if (command === "list_categories_command") {
      categoryLoads++;
      return { kind: "success", categories: [categoryLoads === 1 ? category : { category_id: 1, name: "Filtros", fields: [{ definition_id: 12, label: "Modelo", field_type: "text", required: true, options: [] }] }] };
    }
    if (command === "create_product_command") productCalls++;
    return undefined;
  });
  const user = userEvent.setup({ document });
  render(createElement(OnboardingScreen, { onBack: () => undefined }));
  await screen.findByLabelText("Marca");
  await enterValidProduct(user);
  await user.click(screen.getByRole("button", { name: "Crear producto" }));
  assert.ok(await screen.findByLabelText("Modelo"));
  assert.ok(await screen.findByRole("alert").then((alert) => alert.textContent?.includes("Los campos de esta categoría cambiaron")));
  assert.equal(productCalls, 0);
  assert.equal(screen.queryByLabelText("Marca"), null);
});

test("submits finite decimal values for number category attributes", async () => {
  let request: unknown;
  mockIPC((command, payload) => command === "list_categories_command" ? { kind: "success", categories: [decimalCategory] } : command === "create_product_command" ? (request = payload, { kind: "success", product_id: 2, sku: "FIL-1", name: "Filtro", category_id: 1, category_name: "Filtros", purchase_price_centavos: 8000, sale_price_centavos: 12550, minimum_sale_price_centavos: 10000, available_quantity: 3, active: true }) : undefined);
  const user = userEvent.setup({ document });
  render(createElement(OnboardingScreen, { onBack: () => undefined }));
  const attribute = await screen.findByRole("spinbutton", { name: "Longitud" });
  assert.equal(attribute.getAttribute("step"), "any");
  await user.type(screen.getByRole("textbox", { name: "SKU" }), "FIL-1");
  await user.type(screen.getByRole("textbox", { name: "Nombre del producto" }), "Filtro");
  await user.type(screen.getByRole("textbox", { name: "Precio de compra (Bs)" }), "80,00");
  await user.type(screen.getByRole("textbox", { name: "Precio de venta (Bs)" }), "125,50");
  await user.type(screen.getByRole("textbox", { name: "Precio mínimo de venta (Bs)" }), "100,00");
  await user.type(screen.getByRole("spinbutton", { name: "Stock inicial (unidades enteras)" }), "3");
  await user.type(attribute, "1.25");
  await user.click(screen.getByRole("button", { name: "Crear producto" }));
  assert.deepEqual(request, { request: { sku: "FIL-1", name: "Filtro", category_id: 1, purchase_price_centavos: 8000, sale_price_centavos: 12550, minimum_sale_price_centavos: 10000, low_stock_threshold: 1, opening_quantity: 3, attribute_values: [{ definition_id: 11, value: "1.25" }] } });
  assert.ok(await screen.findByText("Producto creado: FIL-1. Stock inicial: 3 unidades.", { exact: true }));
});

test("optionally assigns a generated active location after product creation", async () => {
  const calls: Array<{ command: string; payload: unknown }> = [];
  mockIPC((command, payload) => {
    calls.push({ command, payload });
    if (command === "list_categories_command") return success();
    if (command === "location_schema_command") return { kind: "schema_success", schema: { revision: 1, segments: [{ id: 1, label: "Sector", position: 0 }, { id: 2, label: "Gaveta", position: 1 }] } };
    if (command === "list_product_locations_command") return { kind: "locations_success", locations: [{ location_id: 8, code: "A1-SHELF2", values: ["A-1", "Shelf 2"], active: true, revision: 0 }] };
    if (command === "create_product_command") return { kind: "success", product_id: 2, sku: "FIL-1", name: "Filtro", category_id: 1, category_name: "Filtros", purchase_price_centavos: 8000, sale_price_centavos: 12550, minimum_sale_price_centavos: 10000, available_quantity: 3, active: true };
    if (command === "catalog_metadata_detail_command") return { target: "product", entity_id: 2, category_id: 1, category_revision: 1, sku: "FIL-1", name: "Filtro", purchase_price_centavos: 8000, sale_price_centavos: 12550, minimum_sale_price_centavos: 10000, primary_location_id: null, activity: "active", revision: 0, attribute_definitions: [], attribute_values: [] };
    if (command === "assign_product_primary_location_command") return { kind: "assignment_success", product_id: 2, location_id: 8, revision: 1 };
    throw new Error(`Unexpected command: ${command}`);
  });
  const user = userEvent.setup({ document });
  render(createElement(OnboardingScreen, { onBack: () => undefined }));
  await screen.findByLabelText("Marca");
  await user.type(screen.getByRole("textbox", { name: "SKU" }), "FIL-1");
  await user.type(screen.getByRole("textbox", { name: "Nombre del producto" }), "Filtro");
  await user.type(screen.getByRole("textbox", { name: "Precio de compra (Bs)" }), "80,00");
  await user.type(screen.getByRole("textbox", { name: "Precio de venta (Bs)" }), "125,50");
  await user.type(screen.getByRole("textbox", { name: "Precio mínimo de venta (Bs)" }), "100,00");
  await user.type(screen.getByRole("spinbutton", { name: "Stock inicial (unidades enteras)" }), "3");
  await user.type(screen.getByRole("textbox", { name: "Marca" }), "ACDelco");
  await screen.findByRole("option", { name: "A-1" });
  await user.selectOptions(screen.getByRole("combobox", { name: "Sector" }), "A-1");
  await user.selectOptions(screen.getByRole("combobox", { name: "Gaveta" }), "Shelf 2");
  await user.click(screen.getByRole("button", { name: "Crear producto" }));
  assert.ok(await screen.findByText(/Producto creado: FIL-1.*Ubicación principal: A1-SHELF2/));
  assert.deepEqual(calls.filter((call) => call.command === "assign_product_primary_location_command")[0], {
    command: "assign_product_primary_location_command",
    payload: { request: { product_id: 2, expected_revision: 0, location_id: 8 } },
  });
  assert.deepEqual(calls.find((call) => call.command === "create_product_command")?.payload, { request: { sku: "FIL-1", name: "Filtro", category_id: 1, purchase_price_centavos: 8000, sale_price_centavos: 12550, minimum_sale_price_centavos: 10000, low_stock_threshold: 1, opening_quantity: 3, attribute_values: [{ definition_id: 10, value: "ACDelco" }] } });
});

test("keeps product creation successful when the follow-up location assignment fails", async () => {
  mockIPC((command) => {
    if (command === "list_categories_command") return success();
    if (command === "location_schema_command") return { kind: "schema_success", schema: { revision: 1, segments: [{ id: 1, label: "Sector", position: 0 }, { id: 2, label: "Gaveta", position: 1 }] } };
    if (command === "list_product_locations_command") return { kind: "locations_success", locations: [{ location_id: 8, code: "A1-SHELF2", values: ["A-1", "Shelf 2"], active: true, revision: 0 }] };
    if (command === "create_product_command") return { kind: "success", product_id: 2, sku: "FIL-1", name: "Filtro", category_id: 1, category_name: "Filtros", purchase_price_centavos: 8000, sale_price_centavos: 12550, minimum_sale_price_centavos: 10000, available_quantity: 3, active: true };
    if (command === "catalog_metadata_detail_command") return { kind: "error", code: "persistence_failure", message: "detail unavailable" };
    throw new Error(`Unexpected command: ${command}`);
  });
  const user = userEvent.setup({ document });
  render(createElement(OnboardingScreen, { onBack: () => undefined }));
  await screen.findByLabelText("Marca");
  await enterValidProduct(user);
  await screen.findByRole("option", { name: "A-1" });
  await user.selectOptions(screen.getByRole("combobox", { name: "Sector" }), "A-1");
  await user.selectOptions(screen.getByRole("combobox", { name: "Gaveta" }), "Shelf 2");
  await user.click(screen.getByRole("button", { name: "Crear producto" }));
  const feedback = await screen.findByText(/Producto creado: FIL-1.*No se pudo asignar la ubicación principal/);
  assert.equal(feedback.getAttribute("role"), "status");
  assert.equal(screen.queryByRole("alert"), null);
});

test("focuses and explains a missing required category attribute before submission", async () => {
  let calls = 0;
  mockIPC((command) => command === "list_categories_command" ? success() : command === "create_product_command" ? (calls++, undefined) : undefined);
  const user = userEvent.setup({ document });
  render(createElement(OnboardingScreen, { onBack: () => undefined }));
  await screen.findByLabelText("Marca");
  await user.type(screen.getByRole("textbox", { name: "SKU" }), "FIL-1");
  await user.type(screen.getByRole("textbox", { name: "Nombre del producto" }), "Filtro");
  await user.type(screen.getByRole("textbox", { name: "Precio de compra (Bs)" }), "80,00");
  await user.type(screen.getByRole("textbox", { name: "Precio de venta (Bs)" }), "125,50");
  await user.type(screen.getByRole("textbox", { name: "Precio mínimo de venta (Bs)" }), "100,00");
  await user.type(screen.getByRole("spinbutton", { name: "Stock inicial (unidades enteras)" }), "3");
  await user.click(screen.getByRole("button", { name: "Crear producto" }));
  const attribute = screen.getByRole("textbox", { name: "Marca" });
  assert.equal(document.activeElement, attribute);
  assert.equal(attribute.getAttribute("aria-invalid"), "true");
  assert.ok(screen.getByText("Completá este campo."));
  assert.equal(calls, 0);
});

test("focuses and marks the backend-identified category attribute without exposing native details", async () => {
  mockIPC((command) => command === "list_categories_command" ? success() : command === "create_product_command" ? { kind: "error", code: "invalid_attribute_value", message: "raw detail", field_error: { definition_id: 10, reason: "invalid_value", raw: "secret" } } : undefined);
  const user = userEvent.setup({ document });
  render(createElement(OnboardingScreen, { onBack: () => undefined }));
  await screen.findByLabelText("Marca");
  await enterValidProduct(user);
  await user.click(screen.getByRole("button", { name: "Crear producto" }));
  const feedback = await screen.findByRole("alert");
  const attribute = screen.getByRole("textbox", { name: "Marca" });
  assert.equal(feedback.textContent, "Revisá el valor de este campo.");
  assert.equal(document.activeElement, attribute);
  assert.equal(attribute.getAttribute("aria-invalid"), "true");
  assert.ok(screen.getAllByText("Revisá el valor de este campo.").length >= 1);
  assert.equal(screen.queryByText(/raw detail|secret/), null);
  assert.equal(screen.queryByText(/Producto creado:/), null);
});

test("shows the safe specific create-product failure reason", async () => {
  mockIPC((command) => command === "list_categories_command" ? success() : command === "create_product_command" ? { kind: "error", code: "duplicate_sku", message: "raw native detail" } : undefined);
  const user = userEvent.setup({ document });
  render(createElement(OnboardingScreen, { onBack: () => undefined }));
  await screen.findByLabelText("Marca");
  await enterValidProduct(user);
  await user.click(screen.getByRole("button", { name: "Crear producto" }));
  assert.equal((await screen.findByRole("alert")).textContent, "SKU already exists.");
});

test("offers guided native controls for active locations with live status and an optional clear choice", async () => {
  mockIPC((command) => {
    if (command === "list_categories_command") return success();
    if (command === "location_schema_command") return { kind: "schema_success", schema: { revision: 1, segments: [{ id: 1, label: "Piso", position: 0 }, { id: 2, label: "Estante", position: 1 }] } };
    if (command === "list_product_locations_command") return { kind: "locations_success", locations: [
      { location_id: 8, code: "PB-12", values: ["PB", "12"], active: true, revision: 0 },
      { location_id: 9, code: "PA-2", values: ["PA", "2"], active: true, revision: 0 },
      { location_id: 10, code: "PB-9", values: ["PB", "9"], active: false, revision: 0 },
    ] };
    return undefined;
  });
  render(createElement(OnboardingScreen, { onBack: () => undefined }));
  const floor = await screen.findByRole("combobox", { name: "Piso" });
  const shelf = screen.getByRole("combobox", { name: "Estante" });
  assert.ok(screen.getByRole("group", { name: "Ubicación principal (opcional)" }));
  assert.equal(screen.queryByRole("combobox", { name: "Ubicación principal (opcional)" }), null);
  assert.equal(screen.getByText("2 ubicaciones activas disponibles.").getAttribute("aria-live"), "polite");
  assert.equal(screen.queryByRole("searchbox", { name: "Buscar ubicaciones" }), null);
  assert.ok(screen.getByRole("option", { name: "PB" }));
  const user = userEvent.setup({ document });
  await user.selectOptions(floor, "PB");
  await user.selectOptions(shelf, "12");
  assert.equal((floor as HTMLSelectElement).value, "PB");
  assert.equal((shelf as HTMLSelectElement).value, "12");
  assert.ok(screen.getByRole("option", { name: "12" }));
  assert.ok(screen.queryByRole("option", { name: "9" }) === null);
  await user.selectOptions(floor, "");
  assert.equal((floor as HTMLSelectElement).value, "");
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
  await user.type(screen.getByRole("textbox", { name: "SKU" }), "FIL-1"); await user.type(screen.getByRole("textbox", { name: "Nombre del producto" }), "Filtro"); await user.type(screen.getByRole("textbox", { name: "Precio de compra (Bs)" }), "1"); await user.type(screen.getByRole("textbox", { name: "Precio de venta (Bs)" }), "1"); await user.type(screen.getByRole("textbox", { name: "Precio mínimo de venta (Bs)" }), "1"); await user.type(screen.getByRole("spinbutton", { name: "Stock inicial (unidades enteras)" }), "1"); await user.type(screen.getByRole("textbox", { name: "Marca" }), "ACDelco"); await user.click(screen.getByRole("button", { name: "Crear producto" })); view.unmount(); resolve({ kind: "success", product_id: 2, sku: "FIL-1", name: "Filtro", category_id: 1, category_name: "Filtros", purchase_price_centavos: 100, sale_price_centavos: 100, minimum_sale_price_centavos: 100, available_quantity: 1, active: true }); await Promise.resolve(); assert.equal(document.body.textContent?.includes("Producto creado"), false);
});
