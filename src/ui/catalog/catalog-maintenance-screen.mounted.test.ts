import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { CatalogMaintenanceScreen } from "./catalog-maintenance-screen.ts";

const active = { entity_id: 1, target: "product", label: "Filtro Premium · FIL-PRE-014", activity: "active", revision: 7 };
const archived = { entity_id: 2, target: "category", label: "Encendido", activity: "archived", revision: 3 };
const detail = {
  target: "product", entity_id: 1, category_id: 4, sku: "FIL-PRE-014", name: "Filtro Premium",
  catalog_unit_price_centavos: 12550, activity: "active", revision: 7,
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
    if (command === "list_catalog_maintenance_command") return new Promise((resolve) => { resolveList = resolve; });
    if (command === "catalog_metadata_detail_command") return detail;
    throw new Error(`Unexpected command: ${command}`);
  });
  const view = render(createElement(CatalogMaintenanceScreen));
  screen.getByRole("heading", { name: "Catálogo", level: 1 });
  assert.ok(screen.getByText("Cargando registros del catálogo…"));
  resolveList({ kind: "success", records: [] });
  assert.ok(await screen.findByText("Todavía no hay registros del catálogo."));
  view.unmount();

  mockIPC((command) => command === "list_catalog_maintenance_command"
    ? { kind: "success", records: [active, archived] }
    : command === "catalog_metadata_detail_command" ? detail : Promise.reject(new Error(command)));
  render(createElement(CatalogMaintenanceScreen));
  const master = await screen.findByRole("region", { name: "Registros del catálogo" });
  assert.match(master.textContent ?? "", /Filtro Premium.*Activo.*Encendido.*Archivado/);
  assert.ok(within(master).getByRole("button", { name: "Reactivar" }));
  await userEvent.click(within(master).getByRole("button", { name: /Ver detalles de Filtro Premium/ }));
  const editor = await screen.findByRole("region", { name: "Detalle y edición" });
  assert.match(editor.textContent ?? "", /Filtro Premium.*Activo.*Producto.*Categoría: 4/);
  assert.equal((screen.getByRole("textbox", { name: "Precio actual del catálogo (Bs)" }) as HTMLInputElement).value, "125,50");
  assert.ok(screen.getByText("Afecta solo ventas futuras. Las ventas confirmadas no cambian."));
  assert.ok(screen.getByRole("textbox", { name: "Marca (obligatorio)" }));
  assert.ok(screen.getByRole("combobox", { name: "Material (obligatorio)" }));
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /data-ui-catalog-layout[^}]*336px[^}]*588px/);
  assert.match(css, /max-width: 960px[\s\S]*data-ui-catalog-layout[^}]*minmax\(0, 1fr\)/);
});

test("validates and focuses dynamic fields, then preserves edit and lifecycle contracts", async () => {
  let resolveEdit!: (value: unknown) => void;
  let editRequest: Record<string, unknown> | undefined;
  let maintainRequest: Record<string, unknown> | undefined;
  let edits = 0;
  mockIPC((command, payload) => {
    if (command === "list_catalog_maintenance_command") return { kind: "success", records: [active] };
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
  await user.clear(screen.getByRole("textbox", { name: "Precio actual del catálogo (Bs)" }));
  await user.type(screen.getByRole("textbox", { name: "Precio actual del catálogo (Bs)" }), "130,25");
  await user.click(screen.getByRole("button", { name: "Guardar metadatos" }));
  assert.equal(screen.getByRole("button", { name: "Guardando metadatos…" }).hasAttribute("disabled"), true);
  await user.click(screen.getByRole("button", { name: "Guardando metadatos…" }));
  assert.equal(edits, 1);
  assert.equal(editRequest?.expected_revision, 7);
  assert.equal(editRequest?.catalog_unit_price_centavos, 13025);
  resolveEdit({ kind: "success", ...active, revision: 8 });
  assert.ok(await screen.findByText("Catálogo actualizado."));
  await user.click(screen.getByRole("button", { name: "Archivar" }));
  await waitFor(() => assert.deepEqual(maintainRequest, { target: "product", entity_id: 1, intent: "archive", expected_revision: 7 }));
});

test("distinguishes initial unavailable from empty and recovers selected unavailable detail", async () => {
  let lists = 0;
  let details = 0;
  mockIPC((command) => {
    if (command === "list_catalog_maintenance_command") return ++lists === 1 ? { kind: "error", code: "catalog_unavailable" } : { kind: "success", records: [active] };
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
    if (command === "list_catalog_maintenance_command") return ++lists === 1 ? { kind: "success", records: [active, archived] } : new Promise((resolve) => { resolveReload = resolve; });
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
    if (command === "list_catalog_maintenance_command") return { kind: "success", records: [active] };
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
