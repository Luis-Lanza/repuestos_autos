import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { CatalogMaintenanceScreen } from "./catalog-maintenance-screen.ts";

const activeCategory = { entity_id: 4, target: "category" as const, label: "Filtros", activity: "active" as const, revision: 2, active_product_count: 3 };
const activeProduct = { entity_id: 1, target: "product" as const, label: "Filtro Premium · FIL-PRE-014", activity: "active" as const, revision: 7 };
const browseProduct = { product_id: 1, category_id: 4, sku: "FIL-PRE-014", name: "Filtro Premium", category_name: "Filtros", available_quantity: 0, catalog_unit_price_centavos: 12550, list_price_centavos: 12550, minimum_sale_price_centavos: 10000, revision: 7 };
const browse = (products = [browseProduct]) => ({ kind: "success", products, categories: [{ category_id: 4, name: "Filtros" }], page: 1, page_size: 20, total: products.length, total_pages: products.length ? 1 : 0 });
const categoryDetail = { target: "category" as const, entity_id: 4, name: "Filtros", activity: "active" as const, revision: 2, attribute_definitions: [] };
const productDetail = {
  target: "product" as const, entity_id: 1, category_id: 4, sku: "FIL-PRE-014", name: "Filtro Premium",
  list_price_centavos: 12550, minimum_sale_price_centavos: 10000, activity: "active" as const, revision: 7,
  attribute_definitions: [{ definition_id: 10, label: "Marca", field_type: "text" as const, required: true, options: [] }],
  attribute_values: [{ definition_id: 10, value: "Bosch" }],
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
  const returnButton = screen.getByRole("button", { name: "Volver a productos" });
  assert.equal((returnButton as HTMLButtonElement).tagName, "BUTTON");
  returnButton.focus();
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
  await userEvent.click(within(products).getByRole("button", { name: "Editar" }));
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
  await userEvent.click(within(dialog).getByRole("button", { name: "Guardar metadatos" }));
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
  await userEvent.click(within(dialog).getByRole("button", { name: "Guardar metadatos" }));
  const retry = await within(dialog).findByRole("button", { name: "Reintentar actualización" });
  assert.equal(editCalls, 1);
  assert.equal((within(dialog).getByRole("button", { name: "Guardar metadatos" }) as HTMLButtonElement).disabled, true);
  await userEvent.click(retry);
  await waitFor(() => assert.equal(detailCalls, 2));
  await waitFor(() => assert.equal((within(dialog).getByRole("textbox", { name: "Nombre de la categoría" }) as HTMLInputElement).value, "Filtros autoritativos"));
  assert.equal(editCalls, 1);
  await userEvent.click(within(dialog).getByRole("button", { name: "Guardar metadatos" }));
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
