import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { CatalogMaintenanceScreen } from "./catalog-maintenance-screen.ts";

const activeCategory = { entity_id: 4, target: "category" as const, label: "Filtros", activity: "active" as const, revision: 2 };
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

function baseIPC(command: string) {
  if (command === "list_catalog_categories_command") return { kind: "success", records: [activeCategory] };
  if (command === "browse_products_command") return browse();
  throw new Error(`Unexpected command: ${command}`);
}

test("keeps browsing free of the inline editor and opens a named category modal with authoritative detail", async () => {
  mockIPC((command) => command === "catalog_metadata_detail_command" ? categoryDetail : baseIPC(command));
  render(createElement(CatalogMaintenanceScreen));
  const opener = await screen.findByRole("button", { name: "Editar Filtros" });
  await userEvent.click(opener);
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  assert.equal(dialog.getAttribute("data-ui-catalog-edit-dialog"), "true");
  assert.ok(within(dialog).getByRole("form", { name: "Formulario para editar categoría" }));
  assert.equal(within(dialog).getByRole("textbox", { name: "Nombre de la categoría" }).getAttribute("value"), "Filtros");
  assert.equal(screen.queryByRole("region", { name: "Detalle y edición" }), null);
  await userEvent.keyboard("{Escape}");
  await waitFor(() => assert.equal(document.activeElement, opener));
});

test("loads product detail before editing and archives immediately with adjacent feedback and refreshed browse", async () => {
  let listCalls = 0;
  let browseCalls = 0;
  let maintainRequest: Record<string, unknown> | undefined;
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: ++listCalls === 1 ? [activeCategory] : [{ ...activeProduct, activity: "archived", revision: 8 }] };
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
  await userEvent.click(within(dialog).getByRole("button", { name: "Archivar" }));
  await waitFor(() => assert.deepEqual(maintainRequest, { target: "product", entity_id: 1, intent: "archive", expected_revision: 7 }));
  const lifecycle = within(dialog).getByRole("region", { name: "Acciones de ciclo de vida" });
  assert.ok(within(lifecycle).getByRole("status"));
  assert.ok(within(lifecycle).getByText("Catálogo actualizado."));
  await waitFor(() => assert.equal(listCalls, 2));
  assert.equal(browseCalls, 2);
  assert.ok(within(dialog).getByRole("button", { name: "Reactivar" }));
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
  await userEvent.click(await screen.findByRole("button", { name: "Editar Filtros" }));
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  await userEvent.click(within(dialog).getByRole("button", { name: "Archivar" }));
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
  await userEvent.click(await screen.findByRole("button", { name: "Editar Filtros" }));
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  await userEvent.click(within(dialog).getByRole("button", { name: "Archivar" }));
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
  await userEvent.click(await screen.findByRole("button", { name: "Editar Filtros" }));
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  await userEvent.click(within(dialog).getByRole("button", { name: "Archivar" }));
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
  await userEvent.click(await screen.findByRole("button", { name: "Editar Filtros" }));
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
  await userEvent.click(await screen.findByRole("button", { name: "Editar Filtros" }));
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  await userEvent.click(within(dialog).getByRole("button", { name: "Archivar" }));
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
  await userEvent.click(await screen.findByRole("button", { name: "Editar Filtros" }));
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
  await userEvent.click(await screen.findByRole("button", { name: "Editar Filtros" }));
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
  await userEvent.click(await screen.findByRole("button", { name: "Editar Filtros" }));
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  const lifecycle = within(dialog).getByRole("region", { name: "Acciones de ciclo de vida" });
  await userEvent.click(within(lifecycle).getByRole("button", { name: "Archivar" }));
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
  const opener = await screen.findByRole("button", { name: "Editar Filtros" });
  await userEvent.click(opener);
  const dialog = await screen.findByRole("dialog", { name: "Editar Filtros" });
  const archive = within(dialog).getByRole("button", { name: "Archivar" });
  await userEvent.click(archive);
  await userEvent.click(archive);
  await userEvent.keyboard("{Escape}");
  assert.equal(calls, 1);
  assert.ok(screen.getByRole("dialog", { name: "Editar Filtros" }));
  assert.equal((within(dialog).getByRole("textbox", { name: "Nombre de la categoría" }) as HTMLInputElement).disabled, true);
  resolveMaintain({ kind: "success", ...activeCategory, activity: "archived", revision: 3 });
});

test("focuses validation errors in the routine form and retains stale feedback", async () => {
  mockIPC((command) => command === "list_catalog_categories_command" ? { kind: "success", records: [activeProduct] } : command === "browse_products_command" ? browse() : command === "catalog_metadata_detail_command" ? productDetail : { kind: "error", code: "stale_catalog_record" });
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await screen.findByRole("button", { name: /Editar Filtro Premium/ }));
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
  assert.match(css, /data-ui-catalog-layout[^}]*grid-template-columns:\s*minmax\(280px,\s*4fr\) minmax\(0,\s*7fr\)/);
  assert.match(css, /\[data-ui-catalog-workspace\] \[data-ui-product-browser\] > form \{[^}]*inline-size:\s*min\(100%,\s*42rem\)/);
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*data-ui-catalog-layout[^}]*grid-template-columns:\s*minmax\(0, 1fr\)/);
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*data-ui-catalog-workspace\] \[data-ui-product-browser\] > form \{[^}]*grid-template-columns:\s*minmax\(0, 1fr\)/);
  assert.doesNotMatch(css, /@media \(max-width: 1199px\) and \(min-width: 961px\)/);
});
