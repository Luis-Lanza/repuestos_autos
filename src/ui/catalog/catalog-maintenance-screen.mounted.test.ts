import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { CatalogMaintenanceScreen } from "./catalog-maintenance-screen.ts";

const active = { entity_id: 1, target: "product", label: "Filtro Premium · FIL-PRE-014", activity: "active", revision: 7 };
const archived = { entity_id: 2, target: "category", label: "Encendido", activity: "archived", revision: 3 };
const browseProduct = { product_id: 1, category_id: 4, sku: "FIL-PRE-014", name: "Filtro Premium", category_name: "Filtros", available_quantity: 0, catalog_unit_price_centavos: 12550, list_price_centavos: 12550, minimum_sale_price_centavos: 10000, revision: 7 };
const browse = (products = [browseProduct]) => ({ kind: "success", products, categories: [{ category_id: 4, name: "Filtros" }], page: 1, page_size: 20, total: products.length, total_pages: products.length ? 1 : 0 });
const detail = {
  target: "product", entity_id: 1, category_id: 4, sku: "FIL-PRE-014", name: "Filtro Premium",
  list_price_centavos: 12550, minimum_sale_price_centavos: 10000, activity: "active", revision: 7,
  attribute_definitions: [
    { definition_id: 10, label: "Marca", field_type: "text", required: true, options: [] },
    { definition_id: 11, label: "Altura (mm)", field_type: "number", required: false, options: [] },
    { definition_id: 12, label: "Material", field_type: "option", required: true, options: ["Papel", "Sintético"] },
  ],
  attribute_values: [{ definition_id: 10, value: "Bosch" }, { definition_id: 11, value: "85" }, { definition_id: 12, value: "Papel" }],
};

test("renders loading, empty, and a selected Spanish master-detail hierarchy", async () => {
  let resolveList!: (value: unknown) => void;
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") return new Promise((resolve) => { resolveList = resolve; });
    if (command === "catalog_metadata_detail_command") return detail;
    throw new Error(`Unexpected command: ${command}`);
  });
  const view = render(createElement(CatalogMaintenanceScreen));
  screen.getByRole("heading", { name: "Catálogo", level: 1 });
  assert.ok(screen.getByText("Cargando registros del catálogo…"));
  resolveList({ kind: "success", records: [] });
  assert.ok(await screen.findByText("Todavía no hay registros del catálogo."));
  view.unmount();

  mockIPC((command) => command === "list_catalog_categories_command"
    ? { kind: "success", records: [active, archived] }
    : command === "catalog_metadata_detail_command" ? detail : Promise.reject(new Error(command)));
  render(createElement(CatalogMaintenanceScreen));
  const master = await screen.findByRole("region", { name: "Registros del catálogo" });
  assert.match(master.textContent ?? "", /Filtro Premium.*Activo.*Encendido.*Archivado/);
  assert.ok(within(master).getByRole("button", { name: "Reactivar" }));
  assert.equal(within(master).queryByRole("region", { name: "Productos" }), null);
  const workspace = screen.getByRole("region", { name: "Productos" }).parentElement;
  assert.equal(workspace?.getAttribute("data-ui-catalog-workspace"), "true");
  await userEvent.click(within(master).getByRole("button", { name: /Ver detalles de Filtro Premium/ }));
  const editor = await screen.findByRole("region", { name: "Detalle y edición" });
  assert.match(editor.textContent ?? "", /Filtro Premium.*Activo.*Producto.*Categoría: 4/);
  assert.equal((screen.getByRole("textbox", { name: "Precio de lista (Bs)" }) as HTMLInputElement).value, "125,50");
      assert.equal((screen.getByRole("textbox", { name: "Precio mínimo de venta (Bs)" }) as HTMLInputElement).value, "100,00");
  assert.ok(screen.getByText("Referencia para nuevas ventas. Se guarda en centavos."));
      assert.ok(screen.getByText("No puede superar el precio de lista."));
  assert.ok(screen.getByRole("textbox", { name: "Marca (obligatorio)" }));
  assert.ok(screen.getByRole("combobox", { name: "Material (obligatorio)" }));
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /data-ui-catalog-layout[^}]*336px[^}]*588px/);
  assert.match(css, /data-ui-catalog-workspace[\s\S]*data-ui-product-browser[^}]*repeat\(2, minmax\(0, 1fr\)\)/);
  assert.match(css, /@media \(max-width: 1199px\) and \(min-width: 961px\)[\s\S]*data-ui-catalog-layout[^}]*minmax\(0, 1fr\)/);
  assert.match(css, /max-width: 960px[\s\S]*data-ui-catalog-layout[^}]*minmax\(0, 1fr\)/);
});

test("contains catalog product rows separately from search, pagination, and the editor", async () => {
  const manyProducts = Array.from({ length: 100 }, (_, index) => ({ ...browseProduct, product_id: index + 1, sku: `FIL-${index + 1}` }));
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [] };
    if (command === "browse_products_command") return { ...browse(manyProducts), total_pages: 5 };
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(CatalogMaintenanceScreen));
  const productsPanel = await screen.findByRole("region", { name: "Productos" });
  const list = within(productsPanel).getByRole("list", { name: "Resultados del catálogo" });
  assert.equal(list.previousElementSibling?.tagName, "FORM");
  assert.equal(list.nextElementSibling?.getAttribute("data-ui-product-browser-pages"), "true");
  assert.equal(within(productsPanel).getAllByRole("listitem").length, 100);
  assert.ok(screen.getByRole("region", { name: "Detalle y edición" }));
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /data-ui-catalog-master[^}]*flex:\s*1 1 auto[^}]*overflow-y:\s*auto/s);
  assert.doesNotMatch(css, /data-ui-catalog-master[^}]*max-block-size:\s*(?:520|208)px/s);
  assert.match(css, /data-ui-catalog-layout[^}]*grid-template-rows:\s*minmax\(0,\s*1fr\)/s);
});

test("validates and focuses dynamic fields, then preserves edit and lifecycle contracts", async () => {
  let resolveEdit!: (value: unknown) => void;
  let editRequest: Record<string, unknown> | undefined;
  let maintainRequest: Record<string, unknown> | undefined;
  let edits = 0;
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [active] };
    if (command === "catalog_metadata_detail_command") return detail;
    if (command === "edit_catalog_command") { edits += 1; editRequest = payload?.request as Record<string, unknown>; return new Promise((resolve) => { resolveEdit = resolve; }); }
    if (command === "maintain_catalog_command") { maintainRequest = payload?.request as Record<string, unknown>; return { kind: "success", ...active, activity: "archived", revision: 8 }; }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  const user = userEvent.setup({ document });
  await user.click(await screen.findByRole("button", { name: /Ver detalles/ }));
  const brand = await screen.findByRole("textbox", { name: "Marca (obligatorio)" });
  await user.clear(brand);
  await user.selectOptions(screen.getByRole("combobox", { name: "Material (obligatorio)" }), "");
  await user.click(screen.getByRole("button", { name: "Guardar metadatos" }));
  assert.equal((await screen.findAllByText("Ingresá un valor.")).length, 2);
  await waitFor(() => assert.equal(document.activeElement, brand));
  await user.type(brand, "Mann");
  await user.selectOptions(screen.getByRole("combobox", { name: "Material (obligatorio)" }), "Sintético");
  await user.clear(screen.getByRole("textbox", { name: "Precio de lista (Bs)" }));
  await user.type(screen.getByRole("textbox", { name: "Precio de lista (Bs)" }), "130,25");
  await user.click(screen.getByRole("button", { name: "Guardar metadatos" }));
  assert.equal(screen.getByRole("button", { name: "Guardando metadatos…" }).hasAttribute("disabled"), true);
  await user.click(screen.getByRole("button", { name: "Guardando metadatos…" }));
  assert.equal(edits, 1);
  assert.equal(editRequest?.expected_revision, 7);
  assert.equal(editRequest?.list_price_centavos, 13025);
      assert.equal(editRequest?.minimum_sale_price_centavos, 10000);
  resolveEdit({ kind: "success", ...active, revision: 8 });
  assert.ok(await screen.findByText("Catálogo actualizado."));
  await user.click(screen.getByRole("button", { name: "Archivar" }));
  await waitFor(() => assert.deepEqual(maintainRequest, { target: "product", entity_id: 1, intent: "archive", expected_revision: 7 }));
});

test("focuses each validation field, including SKU and both prices", async () => {
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [active] };
    if (command === "catalog_metadata_detail_command") return detail;
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  const user = userEvent.setup({ document });
  await user.click(await screen.findByRole("button", { name: /Ver detalles/ }));
  const sku = await screen.findByRole("textbox", { name: "SKU" });
  const name = screen.getByRole("textbox", { name: "Nombre del producto" });
  const listPrice = screen.getByRole("textbox", { name: "Precio de lista (Bs)" });
  const minimumPrice = screen.getByRole("textbox", { name: "Precio mínimo de venta (Bs)" });
  const save = () => user.click(screen.getByRole("button", { name: "Guardar metadatos" }));

  await user.clear(sku);
  await save();
  await waitFor(() => assert.equal(document.activeElement, sku));
  await user.type(sku, "FIL-PRE-014");

  await user.clear(name);
  await save();
  await waitFor(() => assert.equal(document.activeElement, name));
  await user.type(name, "Filtro Premium");

  await user.clear(listPrice);
  await save();
  await waitFor(() => assert.equal(document.activeElement, listPrice));
  await user.type(listPrice, "125,50");

  await user.clear(minimumPrice);
  await save();
  await waitFor(() => assert.equal(document.activeElement, minimumPrice));
});

test("distinguishes initial unavailable from empty and recovers selected unavailable detail", async () => {
  let lists = 0;
  let details = 0;
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") return ++lists === 1 ? { kind: "error", code: "catalog_unavailable" } : { kind: "success", records: [active] };
    if (command === "catalog_metadata_detail_command") return ++details === 1 ? { kind: "error", code: "catalog_unavailable" } : detail;
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  const user = userEvent.setup({ document });
  assert.ok(await screen.findByText("El catálogo no está disponible."));
  assert.equal(screen.queryByText("Todavía no hay registros del catálogo."), null);
  await user.click(screen.getByRole("button", { name: "Reintentar catálogo" }));
  await user.click(await screen.findByRole("button", { name: /Ver detalles/ }));
  assert.ok(await screen.findByText("Este registro no está disponible. Recargá el catálogo."));
  await user.click(screen.getByRole("button", { name: "Recargar registros del catálogo" }));
  assert.ok(await screen.findByRole("textbox", { name: "Nombre del producto" }));
});

test("does not let a stale reload chain supersede a newer selection", async () => {
  let lists = 0, resolveReload!: (value: unknown) => void;
  const details: number[] = [];
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return ++lists === 1 ? { kind: "success", records: [active, archived] } : new Promise((resolve) => { resolveReload = resolve; });
    if (command === "catalog_metadata_detail_command") { const id = (payload?.request as { entity_id: number }).entity_id; details.push(id); return id === 1 ? { kind: "error", code: "stale_catalog_record" } : { target: "category", entity_id: 2, name: "Encendido", activity: "archived", revision: 3, attribute_definitions: [] }; }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  const user = userEvent.setup({ document });
  await user.click(await screen.findByRole("button", { name: /Ver detalles de Filtro/ }));
  await user.click(await screen.findByRole("button", { name: "Recargar registros del catálogo" }));
  await user.click(screen.getByRole("button", { name: /Ver detalles de Encendido/ }));
  resolveReload({ kind: "success", records: [active, archived] });
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.deepEqual(details, [1, 2]);
});

test("keeps selected identity during unavailable and stale reload recovery", async () => {
  let details = 0;
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [active] };
    if (command === "catalog_metadata_detail_command") return ++details === 1 ? { kind: "error", code: "stale_catalog_record", message: "Native" } : detail;
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  const user = userEvent.setup({ document });
  await user.click(await screen.findByRole("button", { name: /Ver detalles/ }));
  assert.ok(await screen.findByText("Registro desactualizado. Recargá los registros del catálogo."));
  assert.ok(screen.getByText(/Selección: Filtro Premium/));
  await user.click(screen.getByRole("button", { name: "Recargar registros del catálogo" }));
  assert.ok(await screen.findByRole("region", { name: "Detalle y edición" }));
  assert.equal(details, 2);
});

test("does not resume the post-edit detail load after a newer selection", async () => {
  let lists = 0;
  let resolveEdit!: (value: unknown) => void;
  let resolveReload!: (value: unknown) => void;
  const details: number[] = [];
  const newer = { entity_id: 2, target: "category", label: "Encendido", activity: "archived", revision: 3 };
  const newerDetail = { target: "category", entity_id: 2, name: "Encendido", activity: "archived", revision: 3, attribute_definitions: [] };
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return ++lists === 1
      ? { kind: "success", records: [active, newer] }
      : new Promise((resolve) => { resolveReload = resolve; });
    if (command === "catalog_metadata_detail_command") {
      const entityId = (payload?.request as { entity_id: number }).entity_id;
      details.push(entityId);
      return entityId === 1 ? detail : newerDetail;
    }
    if (command === "edit_catalog_command") return new Promise((resolve) => { resolveEdit = resolve; });
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  const user = userEvent.setup({ document });
  const master = await screen.findByRole("region", { name: "Registros del catálogo" });
  await user.click(within(master).getByRole("button", { name: /Ver detalles de Filtro Premium/ }));
  const name = await screen.findByRole("textbox", { name: "Nombre del producto" });
  await user.clear(name);
  await user.type(name, "Filtro actualizado");
  await user.click(screen.getByRole("button", { name: "Guardar metadatos" }));
  resolveEdit({ kind: "success", ...active, revision: 8 });
  await waitFor(() => assert.ok(resolveReload));

  await user.click(within(master).getByRole("button", { name: /Ver detalles de Encendido/ }));
  assert.ok(await screen.findByRole("textbox", { name: "Nombre de la categoría" }));
  resolveReload({ kind: "success", records: [active, newer] });
  await new Promise((resolve) => setTimeout(resolve, 0));

  assert.deepEqual(details, [1, 2]);
  assert.ok(screen.getByRole("textbox", { name: "Nombre de la categoría" }));
});

test("allows editing an out-of-stock product from the paged catalog browser", async () => {
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [{ entity_id: 4, target: "category", label: "Filtros", activity: "active", revision: 1 }] };
    if (command === "browse_products_command") return browse();
    if (command === "catalog_metadata_detail_command") return detail;
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  const product = await screen.findByText("Filtro Premium");
  assert.ok(product);
  const edit = screen.getByRole("button", { name: "Editar" });
  assert.equal((edit as HTMLButtonElement).disabled, false);
  await userEvent.click(edit);
  assert.ok(await screen.findByRole("textbox", { name: "Nombre del producto" }));
});

test("does not replace a newer selected detail when an older response resolves later", async () => {
  const newer = { entity_id: 2, target: "category", label: "Encendido", activity: "archived", revision: 3 };
  const newerDetail = { target: "category", entity_id: 2, name: "Encendido", activity: "archived", revision: 3, attribute_definitions: [] };
  const detailResolvers = new Map<number, (value: unknown) => void>();
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [active, newer] };
    if (command === "catalog_metadata_detail_command") {
      const entityId = (payload?.request as { entity_id: number }).entity_id;
      return new Promise((resolve) => detailResolvers.set(entityId, resolve));
    }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  const user = userEvent.setup({ document });
  const master = await screen.findByRole("region", { name: "Registros del catálogo" });
  await user.click(within(master).getByRole("button", { name: /Ver detalles de Filtro Premium/ }));
  await user.click(within(master).getByRole("button", { name: /Ver detalles de Encendido/ }));
  assert.ok(screen.getByText("Cargando el registro seleccionado…"));
  assert.ok(screen.getByText(/Selección: Encendido/));

  await act(async () => {
    detailResolvers.get(2)!(newerDetail);
  });
  const editor = await screen.findByRole("region", { name: "Detalle y edición" });
  assert.match(editor.textContent ?? "", /Encendido/);

  await act(async () => {
    detailResolvers.get(1)!(detail);
  });
  assert.match(editor.textContent ?? "", /Encendido/);
  assert.doesNotMatch(editor.textContent ?? "", /Filtro Premium/);
});
