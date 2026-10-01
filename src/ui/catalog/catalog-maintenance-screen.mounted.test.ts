import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { mockIPC as nativeMockIPC } from "@tauri-apps/api/mocks";

let mockedCatalogAccessStatus: "setup_required" | "locked" | "unlocked" = "unlocked";
const mockIPC: typeof nativeMockIPC = (handler) => nativeMockIPC((command, payload) => command === "catalog_access_status_command" ? { kind: "status", status: mockedCatalogAccessStatus } : handler(command, payload));
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { CatalogMaintenanceScreen } from "./catalog-maintenance-screen.ts";

const activeCategory = { entity_id: 4, target: "category" as const, label: "Filtros", activity: "active" as const, revision: 2, active_product_count: 3 };
const activeProduct = { entity_id: 1, target: "product" as const, label: "Filtro Premium · FIL-PRE-014", activity: "active" as const, revision: 7 };
const browseProduct = { product_id: 1, category_id: 4, sku: "FIL-PRE-014", name: "Filtro Premium", category_name: "Filtros", available_quantity: 0, catalog_unit_price_centavos: 12550, list_price_centavos: 12550, minimum_sale_price_centavos: 10000, attribute_values: [], revision: 7 };
const browse = (products = [browseProduct]) => ({ kind: "success", products, categories: [{ category_id: 4, name: "Filtros" }], page: 1, page_size: 20, total: products.length, total_pages: products.length ? 1 : 0 });
const categoryDetail = { target: "category" as const, entity_id: 4, name: "Filtros", activity: "active" as const, revision: 2, attribute_definitions: [] };
const productDetail = {
  target: "product" as const, entity_id: 1, category_id: 4, category_revision: 1, sku: "FIL-PRE-014", name: "Filtro Premium",
  purchase_price_centavos: 8000, sale_price_centavos: 12550, minimum_sale_price_centavos: 10000, primary_location_id: null, activity: "active" as const, revision: 7,
  attribute_definitions: [
    { definition_id: 10, label: "Marca", field_type: "text" as const, required: true, options: [], active: true },
    { definition_id: 11, label: "Código histórico", field_type: "text" as const, required: true, options: [], active: false },
  ],
  attribute_values: [{ definition_id: 10, value: "Bosch" }, { definition_id: 11, value: "LEGACY-11" }],
};
const brakeProduct = { ...browseProduct, product_id: 2, sku: "BRK-001", name: "Pastilla de freno" };

function baseIPC(command: string) {
  if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory] };
  if (command === "browse_products_command") return browse();
  throw new Error(`Unexpected command: ${command}`);
}
async function openCategoryEditor() {
  await userEvent.click(await screen.findByRole("button", { name: "Gestionar categorías" }));
  return screen.findByRole("button", { name: "Editar Filtros" });
}

test("opens Category Management and returns to the preserved product browse context", async () => {
  mockIPC((command) => baseIPC(command));
  render(createElement(CatalogMaintenanceScreen));
  const search = await screen.findByRole("searchbox", { name: "Buscar en el catálogo" });
  assert.equal(screen.queryByRole("region", { name: "Registros del catálogo" }), null);
  assert.equal(document.querySelector("[data-ui-catalog-master]"), null);
  await userEvent.type(search, "filtro");
  await userEvent.click(screen.getByRole("button", { name: "Buscar" }));
  await userEvent.click(screen.getByRole("button", { name: "Gestionar categorías" }));
  assert.ok(await screen.findByRole("region", { name: "Gestión de categorías" }));
  assert.equal(screen.queryByRole("button", { name: "Volver a productos" }), null);
  const productsNavigation = screen.getByRole("button", { name: "Productos" });
  productsNavigation.focus();
  await userEvent.keyboard("{Enter}");
  assert.equal(screen.getByRole("searchbox", { name: "Buscar en el catálogo" }).getAttribute("value"), "filtro");
  assert.ok(screen.getByRole("region", { name: "Productos" }));
});

test("shows authoritative category count, state, searchable rows, archive confirmation, direct reactivate, and category edit entry", async () => {
  const archived = { ...activeCategory, entity_id: 5, label: "Pastillas", activity: "archived" as const, revision: 7, active_product_count: 0 };
  const requests: Record<string, unknown>[] = [];
  let categories = [activeCategory, archived];
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: categories };
    if (command === "browse_products_command") return browse();
    if (command === "maintain_catalog_command") { requests.push(payload?.request as Record<string, unknown>); const request = requests.at(-1)!; const updated = { ...(request.entity_id === 4 ? activeCategory : archived), activity: request.intent === "archive" ? "archived" as const : "active" as const, revision: Number(request.expected_revision) + 1 }; categories = categories.map((category) => category.entity_id === request.entity_id ? { ...category, ...updated } : category); return { kind: "success", ...updated }; }
    if (command === "catalog_metadata_detail_command") return { ...categoryDetail, entity_id: payload?.request?.entity_id ?? 4, name: payload?.request?.entity_id === 5 ? "Pastillas" : "Filtros" };
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await screen.findByRole("button", { name: "Gestionar categorías" }));
  const management = await screen.findByRole("region", { name: "Gestión de categorías" });
  const filters = within(management).getByRole("listitem", { name: /Filtros/ });
  assert.match(filters.textContent ?? "", /3/);
  assert.match(filters.textContent ?? "", /Activa/);
  assert.ok(within(filters).getByRole("button", { name: "Archivar Filtros" }));
  const search = within(management).getByRole("searchbox", { name: "Buscar categorías" });
  await userEvent.type(search, "Pastillas");
  assert.equal(within(management).queryByText("Filtros"), null);
  const archivedRow = within(management).getByRole("listitem", { name: /Pastillas/ });
  assert.match(archivedRow.textContent ?? "", /Archivada/);
  await userEvent.click(within(archivedRow).getByRole("button", { name: "Editar Pastillas" }));
  assert.ok(await screen.findByRole("dialog", { name: "Editar Pastillas" }));
  await userEvent.keyboard("{Escape}");
  await userEvent.click(within(archivedRow).getByRole("button", { name: "Reactivar Pastillas" }));
  await waitFor(() => assert.equal(requests[0].intent, "reactivate"));
  assert.equal(screen.queryByRole("dialog", { name: /Confirmar/ }), null);
  await userEvent.clear(search);
  const activeRow = within(management).getByRole("listitem", { name: /Filtros/ });
  const archive = within(activeRow).getByRole("button", { name: "Archivar Filtros" });
  await userEvent.click(archive);
  const confirmation = await screen.findByRole("dialog", { name: "Archivar Filtros" });
  assert.equal(requests.length, 1);
  await userEvent.click(within(confirmation).getByRole("button", { name: "Volver" }));
  assert.equal(requests.length, 1);
  assert.equal(document.activeElement, archive);
  await userEvent.click(archive);
  await screen.findByRole("dialog", { name: "Archivar Filtros" });
  await userEvent.keyboard("{Escape}");
  assert.equal(requests.length, 1);
  assert.equal(document.activeElement, archive);
  await userEvent.click(archive);
  const reopenedConfirmation = await screen.findByRole("dialog", { name: "Archivar Filtros" });
  await userEvent.click(within(reopenedConfirmation).getByRole("button", { name: "Confirmar archivo" }));
  await waitFor(() => assert.deepEqual(requests[1], { target: "category", entity_id: 4, intent: "archive", expected_revision: 2 }));
  assert.equal(screen.queryByRole("dialog", { name: "Archivar Filtros" }), null);
  assert.equal(within(management).queryByText(/ID|total|sincroniz/i), null);
  assert.equal(within(management).queryByRole("button", { name: /Crear|Sincronizar/ }), null);
});

test("reports blocked category archive in context and retries unavailable category loading", async () => {
  let listCalls = 0;
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") return ++listCalls === 1 ? { kind: "error", code: "catalog_unavailable" } : { kind: "success", records: [activeCategory] };
    if (command === "browse_products_command") return browse();
    if (command === "maintain_catalog_command") return { kind: "error", code: "lifecycle_blocked" };
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await screen.findByRole("button", { name: "Gestionar categorías" }));
  const management = await screen.findByRole("region", { name: "Gestión de categorías" });
  assert.ok(within(management).getByText(/no están disponibles/i));
  await userEvent.click(within(management).getByRole("button", { name: "Reintentar categorías" }));
  const archive = await within(management).findByRole("button", { name: "Archivar Filtros" });
  await userEvent.click(archive);
  await userEvent.click(within(await screen.findByRole("dialog", { name: "Archivar Filtros" })).getByRole("button", { name: "Confirmar archivo" }));
  assert.ok(await within(management).findByRole("alert"));
  assert.match(management.textContent ?? "", /no se puede archivar.*productos activos/i);
  assert.equal(listCalls, 2);
});

test("persists Catalog view mode across remounts without resubmitting browse state", async () => {
  const key = "catalog.product-browser.view-mode";
  const previousDescriptor = Object.getOwnPropertyDescriptor(globalThis, "localStorage");
  const values = new Map<string, string>();
  Object.defineProperty(globalThis, "localStorage", { configurable: true, value: { getItem: (item: string) => values.get(item) ?? null, setItem: (item: string, value: string) => { values.set(item, value); }, removeItem: (item: string) => { values.delete(item); } } });
  const previous = values.get(key) ?? null;
  let browseCalls = 0;
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory] };
    if (command === "browse_products_command") { browseCalls += 1; return { ...browse([browseProduct, brakeProduct]), total_pages: 2, page: payload?.request?.page ?? 1 }; }
    throw new Error(`Unexpected command: ${command}`);
  });
  try {
    localStorage.removeItem(key);
    const first = render(createElement(CatalogMaintenanceScreen));
    const table = await screen.findByRole("button", { name: "Vista de tabla" });
    assert.equal(table.getAttribute("aria-pressed"), "true");
    const search = screen.getByRole("searchbox", { name: "Buscar en el catálogo" });
    await userEvent.type(search, "filtro");
    await userEvent.click(screen.getByRole("button", { name: "Buscar" }));
    await waitFor(() => assert.equal(browseCalls, 2));
    await userEvent.click(screen.getByRole("button", { name: "Siguiente" }));
    await waitFor(() => assert.equal(browseCalls, 3));
    const beforeToggle = browseCalls;
    await userEvent.click(screen.getByRole("button", { name: "Vista de galería" }));
    assert.equal(screen.getByRole("button", { name: "Vista de galería" }).getAttribute("aria-pressed"), "true");
    assert.equal((screen.getByRole("button", { name: "Vista de galería" }) as HTMLButtonElement).disabled, false);
    assert.equal((screen.getByRole("button", { name: "Vista de tabla" }) as HTMLButtonElement).disabled, false);
    assert.equal(browseCalls, beforeToggle);
    await userEvent.click(screen.getByRole("button", { name: "Vista de tabla" }));
    assert.equal(screen.getByRole("button", { name: "Vista de tabla" }).getAttribute("aria-pressed"), "true");
    assert.equal(browseCalls, beforeToggle);
    first.unmount();
    render(createElement(CatalogMaintenanceScreen));
    assert.equal((await screen.findByRole("button", { name: "Vista de tabla" })).getAttribute("aria-pressed"), "true");
    assert.equal((await screen.findByRole("button", { name: "Vista de galería" })).getAttribute("aria-pressed"), "false");
    await waitFor(() => assert.equal(browseCalls, beforeToggle + 1));
  } finally {
    if (previous === null) values.delete(key);
    else values.set(key, previous);
    if (previousDescriptor) Object.defineProperty(globalThis, "localStorage", previousDescriptor);
    else Reflect.deleteProperty(globalThis, "localStorage");
  }
});

test("renders the Catalog controls in the approved toolbar order with a compact layout hook", async () => {
  mockIPC((command) => baseIPC(command));
  render(createElement(CatalogMaintenanceScreen));
  const form = await screen.findByRole("searchbox", { name: "Buscar en el catálogo" }).then((search) => search.closest("form")!);
  const controls = Array.from(form.querySelectorAll(":scope > *"));
  assert.deepEqual(controls.map((control) => control.getAttribute("data-ui-catalog-toolbar-item")), ["search", "category", "activity", "views", "submit"]);
  assert.equal(screen.getByRole("combobox", { name: "Categoría" }) !== null, true);
  assert.equal(screen.getByRole("combobox", { name: "Actividad" }) !== null, true);
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /\[data-ui-catalog-workspace\] \[data-ui-product-browser\] > form \{[^}]*grid-template-columns:/);
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*data-ui-catalog-workspace\] \[data-ui-product-browser\] > form \{[^}]*repeat\(2, minmax\(0, 1fr\)\)/);
});

test("contains Catalog results in its desktop workspace while preserving table and gallery presentation", async () => {
  const products = Array.from({ length: 40 }, (_, index) => ({ ...browseProduct, product_id: index + 1, sku: `FIL-${index + 1}` }));
  mockIPC((command) => command === "list_catalog_categories_command" ? { kind: "success", records: [activeCategory] } : command === "browse_products_command" ? { ...browse(products), total_pages: 2 } : (() => { throw new Error(`Unexpected command: ${command}`); })());
  render(createElement(CatalogMaintenanceScreen));
  const panel = await screen.findByRole("region", { name: "Productos" });
  const form = within(panel).getByRole("searchbox", { name: "Buscar en el catálogo" }).closest("form")!;
  const tableViewport = await within(panel).findByRole("region", { name: "Resultados de productos; desplazamiento horizontal disponible" });
  const table = within(tableViewport).getByRole("list", { name: "Resultados del catálogo" });
  assert.equal(form.querySelector('[data-ui-catalog-toolbar-item="views"]')?.children.length, 2);
  assert.equal(table.getAttribute("data-ui-catalog-table"), "true");
  assert.equal(within(table).getAllByRole("listitem").length, 40);
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /@media \(min-width: 961px\)[\s\S]*\[data-ui-catalog-layout\] \{ min-block-size: 0; flex: 1 1 auto; \}[\s\S]*\[data-ui-catalog-layout\] > \[data-ui-catalog-workspace\] \{ display: flex; min-block-size: 0; flex: 1 1 auto; \}[\s\S]*\[data-ui-catalog-workspace\] > \[data-ui-panel\] \{ display: flex; min-block-size: 0; flex: 1 1 auto; flex-direction: column; \}/);
  assert.match(css, /@media \(min-width: 961px\)[\s\S]*\[data-ui-catalog-workspace\] \[data-ui-catalog-results\] \{ display: flex; min-block-size: 0; flex: 1 1 auto; flex-direction: column; \}[\s\S]*\[data-ui-catalog-workspace\] \[data-ui-catalog-table-scroll\] \{ min-block-size: 0; \}[\s\S]*\[data-ui-catalog-workspace\] \[data-ui-product-browser-list\] \{ min-block-size: 0; flex: 1 1 auto; overflow-y: auto; overscroll-behavior: contain; \}/);
  assert.match(css, /\[data-ui-catalog-workspace\] \[data-ui-catalog-table-scroll\] \{[^}]*overflow-x: auto/);
  assert.match(css, /\[data-ui-catalog-workspace\] \[data-ui-product-browser-list\]\[data-ui-catalog-gallery="true"\] \{[^}]*grid-template-columns: repeat\(5, minmax\(0, 1fr\)\)/);
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*\[data-ui-catalog-workspace\] \[data-ui-product-browser-list\]\[data-ui-catalog-gallery="true"\][^}]*grid-template-columns: repeat\(2, minmax\(0, 1fr\)\)/);
});

test("keeps Gallery results separated from the toolbar and Table columns readable at narrow widths", async () => {
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /\[data-ui-catalog-gallery="true"\][^{]*\{[^}]*margin-block-start:\s*var\(--space-3\)/);
  assert.match(css, /data-ui-catalog-table-scroll[^}]*overflow-x:\s*auto/);
  assert.match(css, /data-ui-catalog-table="true"[^}]*min-inline-size:\s*(?:[0-9.]+rem|[0-9]+px)/);
  assert.match(css, /data-ui-catalog-table="true"\]\s*>\s*li\s*\{[^}]*grid-template-columns:[^}]*minmax\([^)]*\)/);
  assert.match(css, /data-ui-catalog-table-identity[^}]*min-inline-size:\s*(?:[0-9.]+rem|[0-9]+px)/);
});

test("applies a submitted multi-character catalog query", async () => {
  const queries: unknown[] = [];
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory] };
    if (command === "browse_products_command") {
      const query = payload?.request?.query;
      queries.push(query);
      return query === "brake" ? browse([brakeProduct]) : browse([]);
    }
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(CatalogMaintenanceScreen));
  const search = await screen.findByRole("searchbox", { name: "Buscar en el catálogo" });
  await userEvent.type(search, "brake");
  await userEvent.click(screen.getByRole("button", { name: "Buscar" }));
  await waitFor(() => assert.ok(screen.getByText("Pastilla de freno")));
  assert.equal(queries.at(-1), "brake");
});

test("does not let delayed initial browse replace a newer submitted search", async () => {
  let resolveCategories!: (value: unknown) => void;
  const queries: unknown[] = [];
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return new Promise((resolve) => { resolveCategories = resolve; });
    if (command === "browse_products_command") {
      const query = payload?.request?.query;
      queries.push(query);
      return query === "brake" ? browse([brakeProduct]) : browse([]);
    }
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(CatalogMaintenanceScreen));
  const search = await screen.findByRole("searchbox", { name: "Buscar en el catálogo" });
  await userEvent.type(search, "brake");
  await userEvent.click(screen.getByRole("button", { name: "Buscar" }));
  await waitFor(() => assert.ok(screen.getByText("Pastilla de freno")));
  await act(async () => { resolveCategories({ kind: "success", records: [activeCategory] }); });
  await waitFor(() => assert.equal(queries.length, 1));
  assert.equal(queries[0], "brake");
  assert.ok(screen.getByText("Pastilla de freno"));
});

test("keeps browsing free of the inline editor and opens a named category modal with authoritative detail", async () => {
  mockIPC((command) => command === "catalog_metadata_detail_command" ? categoryDetail : baseIPC(command));
  render(createElement(CatalogMaintenanceScreen));
  const opener = await openCategoryEditor();
  await userEvent.click(opener);
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  assert.equal(dialog.getAttribute("data-ui-catalog-edit-dialog"), "true");
  assert.ok(within(dialog).getByRole("form", { name: "Formulario para editar categoría" }));
  assert.equal(within(dialog).getByRole("textbox", { name: "Nombre de la categoría" }).getAttribute("value"), "Filtros");
  assert.equal(screen.queryByRole("region", { name: "Detalle y edición" }), null);
  await userEvent.keyboard("{Escape}");
  await waitFor(() => assert.equal(document.activeElement, opener));
});

test("adds category fields and retires existing fields only after confirmation", async () => {
  const detail = { ...categoryDetail, attribute_definitions: [
    { definition_id: 11, label: "Material", field_type: "text" as const, required: true, options: [], active: true },
    { definition_id: 12, label: "Legacy code", field_type: "number" as const, required: false, options: [], active: false },
  ] };
  let schemaRequest: Record<string, unknown> | undefined;
  mockIPC((command, payload) => {
    if (command === "catalog_metadata_detail_command") return detail;
    if (command === "edit_category_schema_command") { schemaRequest = payload?.request as Record<string, unknown>; return { kind: "success", entity_id: 4, target: "category", label: "", activity: "active", revision: 3 }; }
    return baseIPC(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await openCategoryEditor());
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  assert.ok(within(dialog).getByRole("textbox", { name: "Nombre de la categoría" }));
  const existingField = within(dialog).getByRole("group", { name: "Campo Material" });
  assert.match(existingField.textContent ?? "", /Texto.*Obligatorio/);
  assert.equal(within(existingField).queryByRole("textbox"), null);
  assert.ok(within(dialog).getByText(/Legacy code — Retirado.*no editable ni reactivable/));
  assert.equal(within(dialog).queryByRole("button", { name: /Reactivar.*Legacy code/ }), null);
  await userEvent.click(within(dialog).getByRole("button", { name: "Retirar Material" }));
  const confirmation = await screen.findByRole("dialog", { name: "Retirar Material" });
  assert.equal(schemaRequest, undefined);
  await userEvent.click(within(confirmation).getByRole("button", { name: "Volver" }));
  assert.ok(within(dialog).getByRole("button", { name: "Retirar Material" }));
  await userEvent.click(within(dialog).getByRole("button", { name: "Retirar Material" }));
  await userEvent.click(within(await screen.findByRole("dialog", { name: "Retirar Material" })).getByRole("button", { name: "Confirmar retiro" }));
  await userEvent.click(within(dialog).getByRole("button", { name: "Agregar campo" }));
  const draft = within(dialog).getByRole("group", { name: "Campo nuevo 1" });
  const fieldNames = within(draft).getAllByRole("textbox", { name: "Nombre del campo" });
  assert.equal(fieldNames.length, 1);
  await userEvent.type(fieldNames[0], "Length");
  assert.equal(within(dialog).getAllByRole("button", { name: /Guardar/ }).length, 1);
  await userEvent.click(within(dialog).getByRole("button", { name: "Guardar cambios" }));
  await waitFor(() => assert.deepEqual(schemaRequest, { category_id: 4, expected_revision: 2, fields: [{ definition_id: null, label: "Length", field_type: "text", required: false, options: [] }] }));
});

test("keeps category identity out of the sticky scroll layer and supports discarding only an unsaved draft", async () => {
  mockIPC((command) => command === "catalog_metadata_detail_command" ? categoryDetail : baseIPC(command));
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await openCategoryEditor());
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  assert.ok(dialog.querySelector("[data-ui-catalog-identity]"));
  await userEvent.click(within(dialog).getByRole("button", { name: "Agregar campo" }));
  await userEvent.click(within(dialog).getByRole("button", { name: "Agregar campo" }));
  const firstDraft = within(dialog).getByRole("group", { name: "Campo nuevo 1" });
  const secondDraft = within(dialog).getByRole("group", { name: "Campo nuevo 2" });
  const required = within(firstDraft).getByRole("checkbox", { name: "Obligatorio" });
  assert.ok(required.id);
  assert.ok(firstDraft.querySelector("[data-ui-category-schema-draft-name]")?.contains(within(firstDraft).getByRole("textbox", { name: "Nombre del campo" })));
  const discard = within(firstDraft).getByRole("button", { name: "Descartar campo nuevo 1" });
  assert.equal(discard.textContent, "×");
  assert.equal(discard.getAttribute("data-ui-category-schema-discard"), "true");
  assert.equal(firstDraft.querySelector("fieldset"), null);
  assert.equal(firstDraft.querySelector("[data-ui-category-schema-draft-heading]"), null);
  assert.equal(firstDraft.querySelectorAll("[data-ui-category-schema-draft-controls] > *").length, 4);
  assert.equal(within(firstDraft).queryByRole("textbox", { name: "Opciones (separadas por coma)" }), null);
  await userEvent.selectOptions(within(firstDraft).getByRole("combobox", { name: "Tipo" }), "option");
  assert.ok(within(firstDraft).getByRole("textbox", { name: "Opciones (separadas por coma)" }));
  await userEvent.selectOptions(within(firstDraft).getByRole("combobox", { name: "Tipo" }), "text");
  assert.equal(within(firstDraft).queryByRole("textbox", { name: "Opciones (separadas por coma)" }), null);
  await userEvent.type(within(secondDraft).getByRole("textbox", { name: "Nombre del campo" }), "Keep this draft");
  assert.ok(discard);
  await userEvent.click(within(firstDraft).getByRole("button", { name: "Descartar campo nuevo 1" }));
  assert.equal(dialog.querySelectorAll("[data-ui-category-schema-draft]").length, 1);
  assert.equal((within(dialog).getByRole("textbox", { name: "Nombre del campo" }) as HTMLInputElement).value, "Keep this draft");
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /\[data-ui-catalog-identity\] \{(?![^}]*position:\s*sticky)[^}]*display:\s*flex/);
  assert.match(css, /\[data-ui-category-schema-draft-name\] \{ flex: 1 1 0; min-inline-size: 0; \}/);
  assert.match(css, /\[data-ui-category-schema-draft-controls\] \{ display: flex; flex-wrap: nowrap/);
  assert.match(css, /\[data-ui-category-schema-draft-controls\] > \[data-ui-field="select"\] \{ flex: 0 0 9rem; min-inline-size: 9rem; \}/);
  assert.match(css, /\[data-ui-category-schema-draft-controls\] \[data-ui-kind="checkbox"\] > label \{ min-inline-size: 0; min-height: 0; white-space: nowrap; \}/);
  assert.match(css, /@media \(max-width: 699px\) \{\s*\[data-ui-category-schema-draft-controls\] \{ flex-wrap: wrap; \}\s*\[data-ui-category-schema-draft-name\] \{ flex-basis: 100%; \}/);
  assert.match(css, /\[data-ui-category-schema-draft-controls\] \[data-ui-kind="checkbox"\] input\[type="checkbox"\] \{ inline-size: 1rem; block-size: 1rem; flex: 0 0 1rem; appearance: auto/);
  assert.match(css, /\[data-ui-category-schema-discard\] \{ flex: 0 0 auto; inline-size: 2\.5rem; min-block-size: 2\.5rem/);
  assert.match(css, /\[data-ui-category-schema-existing\] \{ min-block-size: 3rem/);
  assert.match(css, /\[data-ui-retired-category-field\] \{ display: flex; min-block-size: 3rem/);
  assert.match(css, /@media \(min-width: 700px\) \{\s*\[data-ui-catalog-edit-dialog\] \{ inline-size: min\(680px, 100%\)/);
  assert.match(css, /\[data-ui-category-schema-options\] \{ inline-size: 100%; min-inline-size: 0/);
});

test("keeps one compact scroll region and one category save action", async () => {
  mockIPC((command) => command === "catalog_metadata_detail_command" ? { ...categoryDetail, attribute_definitions: [{ definition_id: 11, label: "Material", field_type: "text" as const, required: false, options: [], active: true }] } : baseIPC(command));
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await openCategoryEditor());
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  assert.equal(within(dialog).getAllByRole("button", { name: /Guardar/ }).length, 1);
  assert.equal(dialog.querySelectorAll("[data-ui-catalog-edit-content]").length, 1);
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /\[data-ui-catalog-edit-dialog\][^{]*\{[^}]*overflow: hidden/);
  assert.match(css, /\[data-ui-catalog-edit-dialog\] \[data-ui-catalog-edit-content\][^{]*\{[^}]*overflow-y: auto/);
});

test("reloads authoritative category fields after a stale schema result without repeating the mutation", async () => {
  let details = 0;
  let schemaCalls = 0;
  mockIPC((command) => {
    if (command === "catalog_metadata_detail_command") { details += 1; return { ...categoryDetail, revision: details === 1 ? 2 : 4, attribute_definitions: [{ definition_id: 21, label: details === 1 ? "Old" : "Current", field_type: "text" as const, required: false, options: [], active: true }] }; }
    if (command === "edit_category_schema_command") { schemaCalls += 1; return { kind: "error", code: "stale_category_schema", message: "Stale" }; }
    return baseIPC(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await openCategoryEditor());
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  await userEvent.click(within(dialog).getByRole("button", { name: "Agregar campo" }));
  await userEvent.type(within(dialog).getByRole("textbox", { name: "Nombre del campo" }), "Nuevo");
  await userEvent.click(within(dialog).getByRole("button", { name: "Guardar cambios" }));
  const retry = await within(dialog).findByRole("button", { name: "Reintentar actualización" });
  assert.equal(schemaCalls, 1);
  assert.ok(within(dialog).getByRole("alert").textContent?.includes("Recargá antes de guardar"));
  await userEvent.click(retry);
  await waitFor(() => assert.ok(within(dialog).getByRole("group", { name: "Campo Current" })));
  assert.equal(schemaCalls, 1);
  assert.equal(details, 2);
});

test("loads product detail before editing and confirms archive with adjacent feedback and refreshed browse", async () => {
  let listCalls = 0;
  let browseCalls = 0;
  let maintainRequest: Record<string, unknown> | undefined;
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") { listCalls += 1; return { kind: "success", records: [activeCategory] }; }
    if (command === "browse_products_command") { browseCalls += 1; return browse(); }
    if (command === "catalog_metadata_detail_command") return productDetail;
    if (command === "maintain_catalog_command") { maintainRequest = payload?.request as Record<string, unknown>; return { kind: "success", ...activeProduct, activity: "archived", revision: 8 }; }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  const products = await screen.findByRole("region", { name: "Productos" });
  const editButton = await within(products).findByRole("button", { name: "Editar" });
  await userEvent.click(editButton);
  const dialog = await screen.findByRole("dialog", { name: /Editar Filtro Premium/ });
  assert.equal((within(dialog).getByRole("textbox", { name: "SKU" }) as HTMLInputElement).value, "FIL-PRE-014");
  const archive = within(dialog).getByRole("button", { name: "Archivar" });
  await userEvent.click(archive);
  const confirmation = await screen.findByRole("dialog", { name: "Archivar FIL-PRE-014 — Filtro Premium" });
  assert.equal(maintainRequest, undefined);
  await userEvent.keyboard("{Escape}");
  assert.equal(maintainRequest, undefined);
  assert.equal(document.activeElement, archive);
  await userEvent.click(archive);
  const reopenedConfirmation = await screen.findByRole("dialog", { name: "Archivar FIL-PRE-014 — Filtro Premium" });
  await userEvent.click(within(reopenedConfirmation).getByRole("button", { name: "Confirmar archivo" }));
  await waitFor(() => assert.deepEqual(maintainRequest, { target: "product", entity_id: 1, intent: "archive", expected_revision: 7 }));
  const lifecycle = within(dialog).getByRole("region", { name: "Acciones de ciclo de vida" });
  assert.ok(within(lifecycle).getByRole("status"));
  assert.ok(within(lifecycle).getByText("Catálogo actualizado."));
  await waitFor(() => assert.equal(listCalls, 2));
  assert.equal(browseCalls, 2);
  assert.ok(within(dialog).getByRole("button", { name: "Reactivar" }));
});

test("keeps the newest selected detail when requests resolve in reverse order", async () => {
  const secondCategory = { ...activeCategory, entity_id: 5, label: "Pastillas", revision: 3 };
  const firstDetail = { ...categoryDetail, name: "Filtros A" };
  const secondDetail = { ...categoryDetail, entity_id: 5, name: "Pastillas B", revision: 3 };
  let resolveFirst!: (value: unknown) => void;
  let resolveSecond!: (value: unknown) => void;
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory, secondCategory] };
    if (command === "browse_products_command") return browse();
    if (command === "catalog_metadata_detail_command") {
      return payload?.request?.entity_id === activeCategory.entity_id
        ? new Promise((resolve) => { resolveFirst = resolve; })
        : new Promise((resolve) => { resolveSecond = resolve; });
    }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await openCategoryEditor());
  await waitFor(() => assert.ok(resolveFirst));
  await userEvent.click(screen.getByRole("button", { name: "Editar Pastillas" }));
  await waitFor(() => assert.ok(resolveSecond));
  await act(async () => { resolveSecond(secondDetail); });
  await act(async () => { resolveFirst(firstDetail); });
  const dialog = await screen.findByRole("dialog", { name: "Editar Pastillas B" });
  assert.equal((within(dialog).getByRole("textbox", { name: "Nombre de la categoría" }) as HTMLInputElement).value, "Pastillas B");
});

test("keeps the modal actionable without repeating a successful mutation when list refresh fails", async () => {
  let listCalls = 0;
  let maintainCalls = 0;
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") {
      listCalls += 1;
      if (listCalls === 2) return { kind: "error", code: "catalog_unavailable" };
      return { kind: "success", records: listCalls === 1 ? [activeCategory] : [{ ...activeCategory, activity: "archived", revision: 3 }] };
    }
    if (command === "browse_products_command") return browse();
    if (command === "catalog_metadata_detail_command") return categoryDetail;
    if (command === "maintain_catalog_command") { maintainCalls += 1; return { kind: "success", ...activeCategory, activity: "archived", revision: 3 }; }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await openCategoryEditor());
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  await userEvent.click(within(dialog).getByRole("button", { name: "Archivar" }));
  await userEvent.click(within(await screen.findByRole("dialog", { name: "Archivar Filtros" })).getByRole("button", { name: "Confirmar archivo" }));
  const retry = await within(dialog).findByRole("button", { name: "Reintentar actualización" });
  assert.equal(maintainCalls, 1);
  assert.equal((within(dialog).getByRole("textbox", { name: "Nombre de la categoría" }) as HTMLInputElement).disabled, true);
  assert.equal((within(dialog).getByRole("button", { name: "Reactivar" }) as HTMLButtonElement).disabled, true);
  const lifecycle = within(dialog).getByRole("region", { name: "Acciones de ciclo de vida" });
  assert.match(within(lifecycle).getByRole("alert").textContent ?? "", /No se pudieron actualizar/);
  await userEvent.click(retry);
  await waitFor(() => assert.equal(listCalls, 3));
  assert.equal(maintainCalls, 1);
  await waitFor(() => assert.equal(within(dialog).queryByRole("button", { name: "Reintentar actualización" }), null));
  assert.ok(within(dialog).getByRole("button", { name: "Reactivar" }));
});

test("keeps recovery locked while product browse refresh is delayed", async () => {
  let listCalls = 0;
  let browseCalls = 0;
  let resolveRefreshBrowse!: (value: unknown) => void;
  let maintainCalls = 0;
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") {
      listCalls += 1;
      return { kind: "success", records: listCalls === 1 ? [activeCategory] : [{ ...activeCategory, activity: "archived", revision: 3 }] };
    }
    if (command === "browse_products_command") {
      browseCalls += 1;
      return browseCalls === 2 ? new Promise((resolve) => { resolveRefreshBrowse = resolve; }) : browse();
    }
    if (command === "catalog_metadata_detail_command") return categoryDetail;
    if (command === "maintain_catalog_command") {
      maintainCalls += 1;
      return { kind: "success", ...activeCategory, activity: "archived", revision: 3 };
    }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await openCategoryEditor());
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  await userEvent.click(within(dialog).getByRole("button", { name: "Archivar" }));
  await userEvent.click(within(await screen.findByRole("dialog", { name: "Archivar Filtros" })).getByRole("button", { name: "Confirmar archivo" }));
  await waitFor(() => assert.equal(listCalls, 2));
  assert.equal(browseCalls, 2);
  assert.equal(maintainCalls, 1);
  const retry = within(dialog).getByRole("button", { name: "Reintentar actualización" });
  assert.equal((retry as HTMLButtonElement).disabled, true);
  assert.equal((within(dialog).getByRole("textbox", { name: "Nombre de la categoría" }) as HTMLInputElement).disabled, true);
  assert.equal((within(dialog).getByRole("button", { name: "Reactivar" }) as HTMLButtonElement).disabled, true);
  await act(async () => { resolveRefreshBrowse(browse()); });
  await waitFor(() => assert.equal(within(dialog).queryByRole("button", { name: "Reintentar actualización" }), null));
  assert.equal((within(dialog).getByRole("button", { name: "Reactivar" }) as HTMLButtonElement).disabled, false);
});

test("keeps recovery locked after browse refresh failure and fully recovers without repeating mutation", async () => {
  let listCalls = 0;
  let browseCalls = 0;
  let maintainCalls = 0;
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") {
      listCalls += 1;
      return { kind: "success", records: listCalls === 1 ? [activeCategory] : [{ ...activeCategory, activity: "archived", revision: 3 }] };
    }
    if (command === "browse_products_command") {
      browseCalls += 1;
      return browseCalls === 2 ? { kind: "error", code: "catalog_unavailable" } : browse();
    }
    if (command === "catalog_metadata_detail_command") return categoryDetail;
    if (command === "maintain_catalog_command") {
      maintainCalls += 1;
      return { kind: "success", ...activeCategory, activity: "archived", revision: 3 };
    }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await openCategoryEditor());
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  await userEvent.click(within(dialog).getByRole("button", { name: "Archivar" }));
  await userEvent.click(within(await screen.findByRole("dialog", { name: "Archivar Filtros" })).getByRole("button", { name: "Confirmar archivo" }));
  const retry = await within(dialog).findByRole("button", { name: "Reintentar actualización" });
  assert.equal(browseCalls, 2);
  assert.equal(maintainCalls, 1);
  assert.equal((within(dialog).getByRole("textbox", { name: "Nombre de la categoría" }) as HTMLInputElement).disabled, true);
  assert.equal((within(dialog).getByRole("button", { name: "Reactivar" }) as HTMLButtonElement).disabled, true);
  await userEvent.click(retry);
  await waitFor(() => assert.equal(listCalls, 3));
  await waitFor(() => assert.equal(within(dialog).queryByRole("button", { name: "Reintentar actualización" }), null));
  assert.equal(browseCalls, 3);
  assert.equal(maintainCalls, 1);
  assert.equal((within(dialog).getByRole("button", { name: "Reactivar" }) as HTMLButtonElement).disabled, false);
});

test("recovers an edited detail after list refresh failure without repeating the save", async () => {
  let listCalls = 0;
  let detailCalls = 0;
  let editCalls = 0;
  const updatedDetail = { ...categoryDetail, name: "Filtros nuevos", revision: 3 };
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") {
      listCalls += 1;
      if (listCalls === 2) return { kind: "error", code: "catalog_unavailable" };
      return { kind: "success", records: [{ ...activeCategory, label: listCalls > 2 ? "Filtros nuevos" : activeCategory.label, revision: listCalls > 2 ? 3 : activeCategory.revision }] };
    }
    if (command === "browse_products_command") return browse();
    if (command === "catalog_metadata_detail_command") return detailCalls++ === 0 ? categoryDetail : updatedDetail;
    if (command === "edit_catalog_command") { editCalls += 1; return { kind: "success", ...activeCategory, label: "Filtros nuevos", revision: 3 }; }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await openCategoryEditor());
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  const name = within(dialog).getByRole("textbox", { name: "Nombre de la categoría" });
  await userEvent.clear(name);
  await userEvent.type(name, "Filtros nuevos");
  await userEvent.click(within(dialog).getByRole("button", { name: "Guardar cambios" }));
  const retry = await within(dialog).findByRole("button", { name: "Reintentar actualización" });
  assert.equal(editCalls, 1);
  assert.equal((name as HTMLInputElement).disabled, true);
  await userEvent.click(retry);
  await waitFor(() => assert.equal(detailCalls, 2));
  assert.equal(editCalls, 1);
  await waitFor(() => assert.equal((within(dialog).getByRole("textbox", { name: "Nombre de la categoría" }) as HTMLInputElement).value, "Filtros nuevos"));
});

test("rehydrates authoritative detail before unlocking stale lifecycle recovery", async () => {
  let detailCalls = 0;
  let maintainCalls = 0;
  let resolveRecoveryDetail!: (value: unknown) => void;
  const recoveryDetail = { ...categoryDetail, activity: "archived" as const, revision: 4 };
  const maintainRequests: Record<string, unknown>[] = [];
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [maintainCalls > 1 ? { ...activeCategory, activity: "active", revision: 5 } : activeCategory] };
    if (command === "browse_products_command") return browse();
    if (command === "catalog_metadata_detail_command") {
      detailCalls += 1;
      return detailCalls === 1 ? categoryDetail : new Promise((resolve) => { resolveRecoveryDetail = resolve; });
    }
    if (command === "maintain_catalog_command") {
      maintainCalls += 1;
      maintainRequests.push(payload?.request as Record<string, unknown>);
      return maintainCalls === 1 ? { kind: "error", code: "stale_catalog_record" } : { kind: "success", ...activeCategory, activity: "active", revision: 5 };
    }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await openCategoryEditor());
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  await userEvent.click(within(dialog).getByRole("button", { name: "Archivar" }));
  await userEvent.click(within(await screen.findByRole("dialog", { name: "Archivar Filtros" })).getByRole("button", { name: "Confirmar archivo" }));
  const retry = await within(dialog).findByRole("button", { name: "Reintentar actualización" });
  assert.equal(maintainCalls, 1);
  assert.equal((within(dialog).getByRole("textbox", { name: "Nombre de la categoría" }) as HTMLInputElement).disabled, true);
  await userEvent.click(retry);
  await waitFor(() => assert.equal(detailCalls, 2));
  assert.ok(within(dialog).getByText("Cargando el detalle autorizado…"));
  assert.equal(within(dialog).queryByRole("button", { name: "Archivar" }), null);
  assert.equal(within(dialog).queryByRole("button", { name: "Reactivar" }), null);
  await act(async () => { resolveRecoveryDetail(recoveryDetail); });
  await waitFor(() => assert.equal(within(dialog).getByRole("button", { name: "Reactivar" }).hasAttribute("disabled"), false));
  assert.equal(maintainCalls, 1);
  await userEvent.click(within(dialog).getByRole("button", { name: "Reactivar" }));
  await waitFor(() => assert.equal(maintainCalls, 2));
  assert.equal(maintainRequests[1].expected_revision, 4);
});

test("rehydrates authoritative detail before unlocking stale edit recovery", async () => {
  let detailCalls = 0;
  let editCalls = 0;
  const editRequests: Record<string, unknown>[] = [];
  const recoveryDetail = { ...categoryDetail, name: "Filtros autoritativos", revision: 4 };
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory] };
    if (command === "browse_products_command") return browse();
    if (command === "catalog_metadata_detail_command") return detailCalls++ === 0 ? categoryDetail : recoveryDetail;
    if (command === "edit_catalog_command") {
      editCalls += 1;
      editRequests.push(payload?.request as Record<string, unknown>);
      return editCalls === 1 ? { kind: "error", code: "stale_catalog_record" } : { kind: "success", ...activeCategory, label: "Filtros autoritativos", revision: 5 };
    }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await openCategoryEditor());
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  const name = within(dialog).getByRole("textbox", { name: "Nombre de la categoría" });
  await userEvent.clear(name);
  await userEvent.type(name, "Cambio en conflicto");
  await userEvent.click(within(dialog).getByRole("button", { name: "Guardar cambios" }));
  const retry = await within(dialog).findByRole("button", { name: "Reintentar actualización" });
  assert.equal(editCalls, 1);
  assert.equal((within(dialog).getByRole("button", { name: "Guardar cambios" }) as HTMLButtonElement).disabled, true);
  await userEvent.click(retry);
  await waitFor(() => assert.equal(detailCalls, 2));
  await waitFor(() => assert.equal((within(dialog).getByRole("textbox", { name: "Nombre de la categoría" }) as HTMLInputElement).value, "Filtros autoritativos"));
  assert.equal(editCalls, 1);
  await userEvent.click(within(dialog).getByRole("button", { name: "Guardar cambios" }));
  await waitFor(() => assert.equal(editCalls, 2));
  assert.equal(editRequests[1].expected_revision, 4);
});

test("reactivates an archived record without a confirmation step", async () => {
  let maintainRequest: Record<string, unknown> | undefined;
  const archived = { ...activeCategory, activity: "archived" as const, revision: 9 };
  const detail = { ...categoryDetail, activity: "archived" as const, revision: 9 };
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [archived] };
    if (command === "browse_products_command") return browse();
    if (command === "catalog_metadata_detail_command") return detail;
    if (command === "maintain_catalog_command") { maintainRequest = payload?.request as Record<string, unknown>; return { kind: "success", ...archived, activity: "active", revision: 10 }; }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await openCategoryEditor());
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  await userEvent.click(within(dialog).getByRole("button", { name: "Reactivar" }));
  await waitFor(() => assert.deepEqual(maintainRequest, { target: "category", entity_id: 4, intent: "reactivate", expected_revision: 9 }));
  assert.equal(screen.queryByRole("dialog", { name: /Confirmar/ }), null);
});

test("keeps blocked category lifecycle feedback adjacent to the action", async () => {
  let lifecycleCalls = 0;
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory] };
    if (command === "browse_products_command") return browse();
    if (command === "catalog_metadata_detail_command") return categoryDetail;
    if (command === "maintain_catalog_command") { lifecycleCalls += 1; return { kind: "error", code: "lifecycle_blocked" }; }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await openCategoryEditor());
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  const lifecycle = within(dialog).getByRole("region", { name: "Acciones de ciclo de vida" });
  await userEvent.click(within(lifecycle).getByRole("button", { name: "Archivar" }));
  const confirmation = await screen.findByRole("dialog", { name: "Archivar Filtros" });
  assert.equal(lifecycleCalls, 0);
  await userEvent.click(within(confirmation).getByRole("button", { name: "Confirmar archivo" }));
  assert.equal(lifecycleCalls, 1);
  assert.ok(within(lifecycle).getByRole("alert"));
  assert.match(lifecycle.textContent ?? "", /no está permitido/i);
});

test("locks modal controls and Escape while lifecycle is pending", async () => {
  let resolveMaintain!: (value: unknown) => void;
  let calls = 0;
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory] };
    if (command === "browse_products_command") return browse();
    if (command === "catalog_metadata_detail_command") return categoryDetail;
    if (command === "maintain_catalog_command") { calls += 1; return new Promise((resolve) => { resolveMaintain = resolve; }); }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  const opener = await openCategoryEditor();
  await userEvent.click(opener);
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  const archive = within(dialog).getByRole("button", { name: "Archivar" });
  await userEvent.click(archive);
  const confirmation = await screen.findByRole("dialog", { name: "Archivar Filtros" });
  await userEvent.click(within(confirmation).getByRole("button", { name: "Confirmar archivo" }));
  await waitFor(() => assert.equal(calls, 1));
  await userEvent.keyboard("{Escape}");
  assert.equal(calls, 1);
  assert.ok(screen.getByRole("dialog", { name: "Archivar Filtros" }));
  assert.equal((within(confirmation).getByRole("button", { name: "Archivando…" }) as HTMLButtonElement).disabled, true);
  resolveMaintain({ kind: "success", ...activeCategory, activity: "archived", revision: 3 });
});

test("selects and revision-checks an optional generated primary location in product editing", async () => {
  const requests: Array<{ command: string; payload: unknown }> = [];
  let assigned = false;
  mockIPC((command, payload) => {
    requests.push({ command, payload });
    if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory] };
    if (command === "browse_products_command") return browse([{ ...browseProduct, primary_location_code: assigned ? "A1-SHELF2" : null, revision: assigned ? 9 : 7 }]);
    if (command === "catalog_metadata_detail_command") return { ...productDetail, primary_location_id: assigned ? 9 : null, revision: assigned ? 9 : 7 };
    if (command === "list_product_locations_command") return { kind: "locations_success", locations: [{ location_id: 9, code: "A1-SHELF2", values: ["A-1", "Shelf 2"], active: true, revision: 0 }] };
    if (command === "location_schema_command") return { kind: "schema_success", schema: { revision: 1, segments: [{ id: 1, label: "Sector", position: 0 }, { id: 2, label: "Gaveta", position: 1 }] } };
    if (command === "edit_catalog_command") return { kind: "success", entity_id: 1, target: "product", label: "Filtro Premium · FIL-PRE-014", activity: "active", revision: 8 };
    if (command === "assign_product_primary_location_command") { assigned = true; return { kind: "assignment_success", product_id: 1, location_id: 9, revision: 9 }; }
    if (command === "catalog_product_image_thumbnail_command") return { kind: "error", code: "image_unavailable", message: "Unavailable" };
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await screen.findByRole("button", { name: "Editar" }));
  const dialog = await screen.findByRole("dialog", { name: /Editar Filtro Premium/ });
  assert.ok(await within(dialog).findByRole("group", { name: "Ubicación principal (opcional)" }));
  assert.equal(within(dialog).queryByRole("searchbox", { name: "Buscar ubicaciones" }), null);
  assert.ok(within(dialog).getByRole("combobox", { name: "Sector" }));
  assert.ok(within(dialog).getByRole("combobox", { name: "Gaveta" }));
  await userEvent.selectOptions(within(dialog).getByRole("combobox", { name: "Sector" }), "A-1");
  await userEvent.selectOptions(within(dialog).getByRole("combobox", { name: "Gaveta" }), "Shelf 2");
  assert.equal((within(dialog).getByRole("combobox", { name: "Gaveta" }) as HTMLSelectElement).value, "Shelf 2");
  await userEvent.click(within(dialog).getByRole("button", { name: "Guardar metadatos" }));
  await waitFor(() => assert.ok(requests.some((request) => request.command === "assign_product_primary_location_command")));
  assert.deepEqual(requests.find((request) => request.command === "assign_product_primary_location_command")?.payload, { request: { product_id: 1, expected_revision: 8, location_id: 9 } });
  await waitFor(() => assert.equal((within(dialog).getByRole("combobox", { name: "Gaveta" }) as HTMLSelectElement).value, "Shelf 2"));
});

test("recovers after product metadata saves but primary-location assignment fails without repeating the save", async () => {
  let detailCalls = 0;
  let editCalls = 0;
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory] };
    if (command === "browse_products_command") return browse();
    if (command === "catalog_metadata_detail_command") return { ...productDetail, revision: detailCalls++ === 0 ? 7 : 8, primary_location_id: null };
    if (command === "location_schema_command") return { kind: "schema_success", schema: { revision: 1, segments: [{ id: 1, label: "Sector", position: 0 }, { id: 2, label: "Gaveta", position: 1 }] } };
    if (command === "list_product_locations_command") return { kind: "locations_success", locations: [{ location_id: 9, code: "A1-SHELF2", values: ["A-1", "Shelf 2"], active: true, revision: 0 }] };
    if (command === "edit_catalog_command") { editCalls += 1; return { kind: "success", entity_id: 1, target: "product", label: "Filtro Premium · FIL-PRE-014", activity: "active", revision: 8 }; }
    if (command === "assign_product_primary_location_command") return { kind: "error", code: "stale_location", message: "Stale assignment" };
    if (command === "catalog_product_image_thumbnail_command") return { kind: "error", code: "image_unavailable", message: "Unavailable" };
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await screen.findByRole("button", { name: "Editar" }));
  const dialog = await screen.findByRole("dialog", { name: /Editar Filtro Premium/ });
  await userEvent.selectOptions(await within(dialog).findByRole("combobox", { name: "Sector" }), "A-1");
  await userEvent.selectOptions(within(dialog).getByRole("combobox", { name: "Gaveta" }), "Shelf 2");
  await userEvent.click(within(dialog).getByRole("button", { name: "Guardar metadatos" }));
  const retry = await within(dialog).findByRole("button", { name: "Reintentar actualización" });
  assert.equal(editCalls, 1);
  assert.match(within(dialog).getByRole("alert").textContent ?? "", /no se pudo asignar la ubicación principal/i);
  await userEvent.click(retry);
  await waitFor(() => assert.equal(detailCalls, 2));
  assert.equal(editCalls, 1);
  assert.equal((within(dialog).getByRole("combobox", { name: "Sector" }) as HTMLSelectElement).value, "");
});

test("loads the saved low-stock threshold, validates it accessibly, and saves a changed integer", async () => {
  let threshold = 5;
  let editRequest: Record<string, unknown> | undefined;
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory] };
    if (command === "browse_products_command") return browse();
    if (command === "catalog_metadata_detail_command") return { ...productDetail, low_stock_threshold: threshold };
    if (command === "catalog_product_image_thumbnail_command") return { kind: "error", code: "image_unavailable", message: "Unavailable" };
    if (command === "edit_catalog_command") { editRequest = payload?.request as Record<string, unknown>; threshold = Number(editRequest.low_stock_threshold); return { kind: "success", ...activeProduct, revision: 8 }; }
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await screen.findByRole("button", { name: "Editar" }));
  const dialog = await screen.findByRole("dialog", { name: /Editar Filtro Premium/ });
  const thresholdInput = within(dialog).getByRole("spinbutton", { name: "Umbral de stock bajo (opcional)" }) as HTMLInputElement;
  assert.equal(thresholdInput.value, "5");
  await userEvent.clear(thresholdInput);
  await userEvent.click(within(dialog).getByRole("button", { name: "Guardar metadatos" }));
  await waitFor(() => assert.equal(editRequest?.low_stock_threshold, 1));
  await waitFor(() => assert.equal((within(dialog).getByRole("spinbutton", { name: "Umbral de stock bajo (opcional)" }) as HTMLInputElement).value, "1"));
  await userEvent.clear(thresholdInput);
  await userEvent.type(thresholdInput, "0");
  await userEvent.click(within(dialog).getByRole("button", { name: "Guardar metadatos" }));
  assert.equal(thresholdInput.getAttribute("aria-invalid"), "true");
  assert.equal(within(dialog).getByText("Ingresá un umbral entero mayor o igual a 1.").id, "catalog-edit-low-stock-threshold-error");
  assert.equal(editRequest?.low_stock_threshold, 1);
  await userEvent.clear(thresholdInput);
  await userEvent.type(thresholdInput, "8");
  await userEvent.click(within(dialog).getByRole("button", { name: "Guardar metadatos" }));
  await waitFor(() => assert.equal(editRequest?.low_stock_threshold, 8));
  await waitFor(() => assert.equal((within(dialog).getByRole("spinbutton", { name: "Umbral de stock bajo (opcional)" }) as HTMLInputElement).value, "8"));
});

test("keeps historical retired values out of product edit controls while preserving authoritative detail", async () => {
  let editRequest: Record<string, unknown> | undefined;
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory] };
    if (command === "browse_products_command") return browse();
    if (command === "catalog_metadata_detail_command") return productDetail;
    if (command === "catalog_product_image_thumbnail_command") return { kind: "error", code: "image_unavailable", message: "Unavailable" };
    if (command === "edit_catalog_command") { editRequest = payload?.request as Record<string, unknown>; return { kind: "success", ...activeProduct, revision: 8 }; }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await screen.findByRole("button", { name: "Editar" }));
  const dialog = await screen.findByRole("dialog", { name: /Editar Filtro Premium/ });
  assert.ok(within(dialog).getByRole("textbox", { name: "Marca (obligatorio)" }));
  assert.equal(within(dialog).queryByRole("textbox", { name: "Código histórico (obligatorio)" }), null);
  assert.equal(productDetail.attribute_values.find((value) => value.definition_id === 11)?.value, "LEGACY-11");
  await userEvent.click(within(dialog).getByRole("button", { name: "Guardar metadatos" }));
  await waitFor(() => assert.ok(editRequest));
  assert.deepEqual(editRequest?.attribute_values, [{ definition_id: 10, value: "Bosch" }]);
});

test("recovers stale category-schema product edits by reloading authoritative selected detail", async () => {
  let detailCalls = 0;
  let editCalls = 0;
  let listCalls = 0;
  const currentDetail = { ...productDetail, name: "Filtro actualizado desde otra sesión", category_revision: 2, revision: 8 };
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") { listCalls += 1; return { kind: "success", records: [activeCategory] }; }
    if (command === "browse_products_command") return browse();
    if (command === "catalog_metadata_detail_command") return detailCalls++ === 0 ? productDetail : currentDetail;
    if (command === "catalog_product_image_thumbnail_command") return { kind: "error", code: "image_unavailable", message: "Unavailable" };
    if (command === "edit_catalog_command") { editCalls += 1; return { kind: "error", code: "stale_category_schema", message: "Stale schema" }; }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await screen.findByRole("button", { name: "Editar" }));
  const dialog = await screen.findByRole("dialog", { name: /Editar Filtro Premium/ });
  await userEvent.click(within(dialog).getByRole("button", { name: "Guardar metadatos" }));
  const retry = await within(dialog).findByRole("button", { name: "Reintentar actualización" });
  assert.equal(editCalls, 1);
  assert.equal(detailCalls, 1);
  assert.equal(listCalls, 1);
  assert.match(within(dialog).getByRole("alert").textContent ?? "", /campos de la categoría cambiaron/i);
  await userEvent.click(retry);
  await waitFor(() => assert.equal(detailCalls, 2));
  await waitFor(() => assert.equal((within(dialog).getByRole("textbox", { name: "Nombre del producto" }) as HTMLInputElement).value, currentDetail.name));
  assert.equal(listCalls, 2);
  assert.equal(editCalls, 1);
});

test("preserves stale product-record recovery behavior", async () => {
  let detailCalls = 0;
  let editCalls = 0;
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory] };
    if (command === "browse_products_command") return browse();
    if (command === "catalog_metadata_detail_command") return { ...productDetail, name: detailCalls++ === 0 ? productDetail.name : "Authoritative product", revision: 9 };
    if (command === "catalog_product_image_thumbnail_command") return { kind: "error", code: "image_unavailable", message: "Unavailable" };
    if (command === "edit_catalog_command") { editCalls += 1; return { kind: "error", code: "stale_catalog_record", message: "Stale record" }; }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await screen.findByRole("button", { name: "Editar" }));
  const dialog = await screen.findByRole("dialog", { name: /Editar Filtro Premium/ });
  await userEvent.clear(within(dialog).getByRole("textbox", { name: "Nombre del producto" }));
  await userEvent.type(within(dialog).getByRole("textbox", { name: "Nombre del producto" }), "My stale edit");
  await userEvent.click(within(dialog).getByRole("button", { name: "Guardar metadatos" }));
  await userEvent.click(await within(dialog).findByRole("button", { name: "Reintentar actualización" }));
  await waitFor(() => assert.equal((within(dialog).getByRole("textbox", { name: "Nombre del producto" }) as HTMLInputElement).value, "Authoritative product"));
  assert.equal(editCalls, 1);
});

test("replaces a product image through the native picker and reloads its authoritative thumbnail", async () => {
  let imageMutations = 0;
  let detailCalls = 0;
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory] };
    if (command === "browse_products_command") return browse([{ ...browseProduct, revision: imageMutations ? 8 : 7 }]);
    if (command === "catalog_metadata_detail_command") { detailCalls += 1; return { ...productDetail, revision: imageMutations ? 8 : 7 }; }
    if (command === "catalog_product_image_thumbnail_command") return imageMutations ? { kind: "success", product_id: 1, revision: 8, mime_type: "image/jpeg", encoding: "base64", bytes: "/9j/2Q==" } : { kind: "error", code: "image_unavailable", message: "This product image is unavailable." };
    if (command === "choose_product_image_command") { imageMutations += 1; return { kind: "success", product_id: 1, revision: 8 }; }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await screen.findByRole("button", { name: "Editar" }));
  const dialog = await screen.findByRole("dialog", { name: /Editar Filtro Premium/ });
  assert.ok(within(dialog).getByText("Sin imagen"));
  assert.equal(within(dialog).queryByText("No se pudo cargar la vista previa."), null);
  await userEvent.click(within(dialog).getByRole("button", { name: "Elegir imagen" }));
  const image = await within(dialog).findByRole("img", { name: "Imagen de Filtro Premium" });
  assert.equal(image.getAttribute("src"), "data:image/jpeg;base64,/9j/2Q==");
  assert.equal(detailCalls, 2);
});

test("keeps unavailable-thumbnail failures distinct from normal image absence", async () => {
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory] };
    if (command === "browse_products_command") return browse();
    if (command === "catalog_metadata_detail_command") return productDetail;
    if (command === "catalog_product_image_thumbnail_command") return { kind: "error", code: "catalog_unavailable", message: "This catalog record is unavailable." };
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await screen.findByRole("button", { name: "Editar" }));
  const dialog = await screen.findByRole("dialog", { name: /Editar Filtro Premium/ });
  assert.ok(within(dialog).getByText("Sin imagen"));
  assert.ok(within(dialog).getByText("No se pudo cargar la vista previa."));
});

test("treats picker cancellation and malformed path-bearing replies as safe nonmutations", async () => {
  let pickerCalls = 0;
  let detailCalls = 0;
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory] };
    if (command === "browse_products_command") return browse();
    if (command === "catalog_metadata_detail_command") { detailCalls += 1; return productDetail; }
    if (command === "catalog_product_image_thumbnail_command") return { kind: "error", code: "image_unavailable", message: "This product image is unavailable." };
    if (command === "choose_product_image_command") return ++pickerCalls === 1 ? { kind: "cancelled" } : { kind: "success", product_id: 1, revision: 8, path: "/private/secret.jpg" };
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await screen.findByRole("button", { name: "Editar" }));
  const dialog = await screen.findByRole("dialog", { name: /Editar Filtro Premium/ });
  await userEvent.click(within(dialog).getByRole("button", { name: "Elegir imagen" }));
  assert.ok(await within(dialog).findByText("No se modificó la imagen."));
  await userEvent.click(within(dialog).getByRole("button", { name: "Elegir imagen" }));
  assert.ok(await within(dialog).findByText("No se pudo actualizar la imagen del producto."));
  assert.doesNotMatch(dialog.textContent ?? "", /private|secret\.jpg/);
  assert.equal(detailCalls, 1);
});

test("removes an image once and locks image actions while the revision-checked request is pending", async () => {
  let resolveRemove!: (value: unknown) => void;
  let removeCalls = 0;
  let imageRemoved = false;
  let request: unknown;
  let detailCalls = 0;
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory] };
    if (command === "browse_products_command") return browse([{ ...browseProduct, revision: imageRemoved ? 8 : 7 }]);
    if (command === "catalog_metadata_detail_command") { detailCalls += 1; return { ...productDetail, revision: imageRemoved ? 8 : 7 }; }
    if (command === "catalog_product_image_thumbnail_command") return imageRemoved ? { kind: "error", code: "image_unavailable", message: "This product image is unavailable." } : { kind: "success", product_id: 1, revision: 7, mime_type: "image/jpeg", encoding: "base64", bytes: "/9j/2Q==" };
    if (command === "remove_product_image_command") { removeCalls += 1; request = payload?.request; return new Promise((resolve) => { resolveRemove = resolve; }); }
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await screen.findByRole("button", { name: "Editar" }));
  const dialog = await screen.findByRole("dialog", { name: /Editar Filtro Premium/ });
  const remove = await within(dialog).findByRole("button", { name: "Quitar imagen" });
  await userEvent.click(remove);
  await userEvent.click(remove);
  assert.equal(removeCalls, 1);
  assert.deepEqual(request, { product_id: 1, expected_revision: 7 });
  assert.equal((remove as HTMLButtonElement).disabled, true);
  await act(async () => { imageRemoved = true; resolveRemove({ kind: "success", product_id: 1, revision: 8 }); });
  await waitFor(() => assert.ok(within(dialog).getByText("Sin imagen")));
  assert.equal(detailCalls, 2);
  assert.equal(within(dialog).queryByRole("button", { name: "Quitar imagen" }), null);
});

test("stale image mutation reloads authoritative detail before enabling another attempt", async () => {
  let detailCalls = 0;
  let chooseCalls = 0;
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory] };
    if (command === "browse_products_command") return browse();
    if (command === "catalog_metadata_detail_command") return { ...productDetail, revision: detailCalls++ === 0 ? 7 : 9 };
    if (command === "catalog_product_image_thumbnail_command") return { kind: "error", code: "image_unavailable", message: "This product image is unavailable." };
    if (command === "choose_product_image_command") return ++chooseCalls === 1 ? { kind: "error", code: "stale_catalog_record", message: "Stale" } : { kind: "success", product_id: 1, revision: 10 };
    throw new Error(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await screen.findByRole("button", { name: "Editar" }));
  const dialog = await screen.findByRole("dialog", { name: /Editar Filtro Premium/ });
  await userEvent.click(within(dialog).getByRole("button", { name: "Elegir imagen" }));
  const retry = await within(dialog).findByRole("button", { name: "Reintentar actualización" });
  assert.equal(chooseCalls, 1);
  await userEvent.click(retry);
  await waitFor(() => assert.equal(detailCalls, 2));
  assert.equal(chooseCalls, 1);
  assert.ok(within(dialog).getByRole("button", { name: "Elegir imagen" }));
});

test("focuses validation errors in the routine form and retains stale feedback", async () => {
  mockIPC((command) => command === "list_catalog_categories_command" ? { kind: "success", records: [activeCategory] } : command === "browse_products_command" ? browse() : command === "catalog_metadata_detail_command" ? productDetail : command === "catalog_product_image_thumbnail_command" ? { kind: "error", code: "image_unavailable", message: "This product image is unavailable." } : { kind: "error", code: "stale_catalog_record", message: "Stale" });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await screen.findByRole("button", { name: "Editar" }));
  const dialog = await screen.findByRole("dialog", { name: /Editar Filtro Premium/ });
  const brand = within(dialog).getByRole("textbox", { name: "Marca (obligatorio)" });
  await userEvent.clear(brand);
  await userEvent.click(within(dialog).getByRole("button", { name: "Guardar metadatos" }));
  await waitFor(() => assert.equal(document.activeElement, brand));
  assert.ok(within(dialog).getByRole("alert"));
  assert.match(within(dialog).getByRole("alert").textContent ?? "", /Corregí los campos/);
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /data-ui-catalog-edit-dialog/);
  assert.match(css, /data-ui-catalog-lifecycle-action="active"/);
  assert.match(css, /data-ui-catalog-layout[^}]*grid-template-columns:\s*minmax\(0,\s*1fr\)/);
  assert.match(css, /\[data-ui-catalog-workspace\] \[data-ui-product-browser\] > form \{[^}]*grid-template-columns:\s*minmax\(12rem,\s*2fr\) minmax\(10rem,\s*1fr\) minmax\(9rem,\s*1fr\) auto auto/);
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*data-ui-catalog-layout[^}]*grid-template-rows:\s*minmax\(0, 1fr\)/);
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*data-ui-catalog-workspace\] \[data-ui-product-browser\] > form \{[^}]*grid-template-columns:\s*repeat\(2, minmax\(0, 1fr\)\)/);
  assert.doesNotMatch(css, /@media \(max-width: 1199px\) and \(min-width: 961px\)/);
});

test("serializes password changes, preserves secrets on failure, and clears them after one current success", async () => {
  let resolveChange!: (value: unknown) => void;
  let calls = 0;
  mockIPC((command) => {
    if (command === "catalog_access_change_password_command") { calls += 1; return new Promise((resolve) => { resolveChange = resolve; }); }
    return baseIPC(command);
  });
  render(createElement(CatalogMaintenanceScreen));
  await screen.findByRole("searchbox", { name: "Buscar en el catálogo" });
  await userEvent.click(screen.getByText("Cambiar contraseña del catálogo"));
  const current = screen.getByLabelText("Contraseña actual") as HTMLInputElement;
  const next = screen.getByLabelText("Nueva contraseña") as HTMLInputElement;
  const confirm = screen.getByLabelText("Confirmar nueva contraseña") as HTMLInputElement;
  await userEvent.type(current, "current-secret");
  await userEvent.type(next, "replacement-secret");
  await userEvent.type(confirm, "replacement-secret");
  const submit = screen.getByRole("button", { name: "Guardar contraseña" });
  await userEvent.click(submit);
  await userEvent.click(submit);
  assert.equal(calls, 1);
  assert.equal(current.disabled, true);
  await act(async () => { resolveChange({ kind: "error", code: "invalid_credentials", message: "private detail" }); });
  assert.equal(current.value, "current-secret");
  assert.equal(next.value, "replacement-secret");
  assert.equal(confirm.value, "replacement-secret");
  await userEvent.click(screen.getByRole("button", { name: "Guardar contraseña" }));
  assert.equal(calls, 2);
  await act(async () => { resolveChange({ kind: "success" }); });
  await waitFor(() => assert.equal(current.value, ""));
  assert.equal(next.value, "");
  assert.equal(confirm.value, "");
  assert.equal(screen.getByText("Contraseña actualizada.").getAttribute("role"), "status");
});

test("does not apply a password-change response after its editor unmounts", async () => {
  let resolveChange!: (value: unknown) => void;
  mockIPC((command) => command === "catalog_access_change_password_command" ? new Promise((resolve) => { resolveChange = resolve; }) : baseIPC(command));
  const view = render(createElement(CatalogMaintenanceScreen));
  await screen.findByRole("searchbox", { name: "Buscar en el catálogo" });
  await userEvent.click(screen.getByText("Cambiar contraseña del catálogo"));
  await userEvent.type(screen.getByLabelText("Contraseña actual"), "current-secret");
  await userEvent.type(screen.getByLabelText("Nueva contraseña"), "replacement-secret");
  await userEvent.type(screen.getByLabelText("Confirmar nueva contraseña"), "replacement-secret");
  await userEvent.click(screen.getByRole("button", { name: "Guardar contraseña" }));
  view.unmount();
  await act(async () => { resolveChange({ kind: "success" }); });
  assert.equal(screen.queryByText("Contraseña actualizada."), null);
});

test("requires saving the one-time recovery code before completing first-device Catalog setup", async () => {
  mockedCatalogAccessStatus = "setup_required";
  const calls: string[] = [];
  mockIPC((command) => {
    calls.push(command);
    if (command === "catalog_access_begin_setup_command") return { kind: "recovery_code", recovery_code: "A".repeat(48) };
    if (command === "catalog_access_finish_setup_command") return { kind: "success" };
    if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory] };
    if (command === "browse_products_command") return browse();
    throw new Error(`Unexpected command: ${command}`);
  });
  try {
    render(createElement(CatalogMaintenanceScreen));
    const password = await screen.findByLabelText("Nueva contraseña");
    assert.ok(screen.getByRole("heading", { name: "Acceso al catálogo" }));
    await userEvent.type(password, "device-password");
    await userEvent.type(screen.getByLabelText("Confirmar contraseña"), "device-password");
    await userEvent.click(screen.getByRole("button", { name: "Configurar catálogo" }));
    const code = await screen.findByText("A".repeat(48));
    assert.ok(code);
    assert.equal(calls.includes("list_catalog_categories_command"), false);
    const continueButton = screen.getByRole("button", { name: "Continuar al catálogo" });
    assert.equal((continueButton as HTMLButtonElement).disabled, true);
    await userEvent.click(screen.getByRole("checkbox", { name: "Confirmo que guardé el código de recuperación" }));
    await userEvent.click(continueButton);
    await screen.findByRole("searchbox", { name: "Buscar en el catálogo" });
    assert.equal(calls.filter((command) => command === "catalog_access_finish_setup_command").length, 1);
  } finally {
    mockedCatalogAccessStatus = "unlocked";
  }
});
