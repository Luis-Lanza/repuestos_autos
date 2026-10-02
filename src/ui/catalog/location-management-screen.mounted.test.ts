import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { mockIPC as nativeMockIPC } from "@tauri-apps/api/mocks";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { CatalogMaintenanceScreen } from "./catalog-maintenance-screen.ts";

const mockIPC: typeof nativeMockIPC = (handler) => nativeMockIPC((command, payload) => command === "catalog_access_status_command" ? { kind: "status", status: "unlocked" } : handler(command, payload));

const schema = { revision: 0, segments: [] as { id: number; label: string; position: number }[] };
const row = { location_id: 8, code: "PB-12", values: ["PB", "12"], active: true, revision: 2 };
const inactiveRow = { location_id: 10, code: "PA-2", values: ["PA", "2"], active: false, revision: 1 };
const emptyBrowse = { kind: "success", products: [], categories: [], page: 1, page_size: 20, total: 0, total_pages: 0 };

function controlForLabel(root: Element, label: string) {
  const field = Array.from(root.querySelectorAll("[data-ui-field]")).find((item) => item.querySelector("label")?.textContent === label);
  assert.ok(field, `Missing field ${label}`);
  return field.querySelector("input, select") as HTMLInputElement | HTMLSelectElement;
}

async function openLocations() {
  render(createElement(CatalogMaintenanceScreen));
  await userEvent.click(await screen.findByRole("button", { name: "Gestionar ubicaciones" }));
  await screen.findByRole("heading", { name: "Gestionar ubicaciones" });
  return screen.getByText("Definí cómo se identifican las ubicaciones físicas y administrá sus códigos generados.").closest("section")!;
}

test("builds ordered schemas from templates, previews generated codes, and creates generated-code locations", async () => {
  let currentSchema = schema;
  let locations: typeof row[] = [];
  const requests: Record<string, unknown>[] = [];
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [] };
    if (command === "browse_products_command") return emptyBrowse;
    if (command === "location_schema_command") return { kind: "schema_success", schema: currentSchema };
    if (command === "list_product_locations_command") return { kind: "locations_success", locations };
    if (command === "save_location_schema_command") { requests.push(payload?.request as Record<string, unknown>); currentSchema = { revision: currentSchema.revision + 1, segments: (payload?.request?.segments as string[]).map((label, position) => ({ id: position + 1, label, position })) }; return { kind: "schema_success", schema: currentSchema }; }
    if (command === "create_product_location_command") { requests.push(payload?.request as Record<string, unknown>); const values = payload?.request?.values as string[]; const created = { location_id: 9, code: values.map((value) => value.replace(/[^a-z0-9]/gi, "").toUpperCase()).join("-"), values, active: true, revision: 0 }; locations = [...locations, created]; return { kind: "location_success", location: created }; }
    throw new Error(`Unexpected command: ${command}`);
  });
  const management = await openLocations();
  const templateSelect = controlForLabel(management, "Plantilla");
  await userEvent.selectOptions(templateSelect, "Sector · Gaveta");
  assert.equal(management.querySelectorAll('[data-ui-location-segments] input').length, 2);
  await userEvent.clear(controlForLabel(management, "Segmento 1"));
  await userEvent.type(controlForLabel(management, "Segmento 1"), "Sector");
  await userEvent.click(within(management).getByRole("button", { name: "Guardar esquema" }));
  await waitFor(() => assert.deepEqual(requests[0], { expected_revision: 0, segments: ["Sector", "Gaveta"] }));
  await userEvent.type(controlForLabel(management, "Sector"), "b-2");
  await userEvent.type(controlForLabel(management, "Gaveta"), "12");
  assert.match(within(management).getByText(/Código generado:/).textContent ?? "", /B2-12/);
  await userEvent.click(within(management).getByRole("button", { name: "Crear ubicación" }));
  await waitFor(() => assert.deepEqual(requests[1], { values: ["b-2", "12"] }));
  assert.ok(await within(management).findByText("Ubicación B2-12 creada."));
  assert.equal(management.querySelector('[data-ui-location-row] code')?.textContent, "B2-12");
  assert.equal((templateSelect as HTMLSelectElement).disabled, true);
  assert.equal((within(management).getByRole("button", { name: "Agregar segmento" }) as HTMLButtonElement).disabled, true);
});

test("shows validation, duplicate-code recovery, inactive filtering, and assigned-location lifecycle protection", async () => {
  const list = [row, inactiveRow];
  let createCalls = 0;
  let deactivateCalls = 0;
  let deleteCalls = 0;
  mockIPC((command, payload) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [] };
    if (command === "browse_products_command") return emptyBrowse;
    if (command === "location_schema_command") return { kind: "schema_success", schema: { revision: 4, segments: [{ id: 1, label: "Piso", position: 0 }, { id: 2, label: "Estante", position: 1 }] } };
    if (command === "list_product_locations_command") return { kind: "locations_success", locations: list };
    if (command === "create_product_location_command") { createCalls += 1; return { kind: "error", code: "duplicate_location_code", message: "duplicate" }; }
    if (command === "deactivate_product_location_command") { deactivateCalls += 1; return { kind: "error", code: "location_in_use", message: "assigned" }; }
    if (command === "delete_product_location_command") { deleteCalls += 1; return { kind: "error", code: "location_in_use", message: "assigned" }; }
    throw new Error(`Unexpected command: ${command} ${JSON.stringify(payload)}`);
  });
  const management = await openLocations();
  await userEvent.click(within(management).getByRole("button", { name: "Crear ubicación" }));
  assert.equal(within(management).getAllByText("Completá este valor.").length, 2);
  assert.equal(createCalls, 0);
  await userEvent.type(controlForLabel(management, "Piso"), "PB");
  await userEvent.type(controlForLabel(management, "Estante"), "12");
  await userEvent.click(within(management).getByRole("button", { name: "Crear ubicación" }));
  assert.ok(await within(management).findByRole("alert"));
  assert.match(management.textContent ?? "", /Ya existe una ubicación con ese código/);
  assert.equal(createCalls, 1);
  await userEvent.click(within(management).getByRole("button", { name: "Desactivar PB-12" }));
  const confirmation = await screen.findByRole("dialog", { name: "Desactivar PB-12" });
  assert.equal(deactivateCalls, 0);
  await userEvent.click(within(confirmation).getByRole("button", { name: "Confirmar desactivación" }));
  await waitFor(() => assert.equal(deactivateCalls, 1));
  assert.match(management.textContent ?? "", /quitá la asignación desde la edición del producto y después volvé a intentarlo/);
  await userEvent.click(within(management).getByRole("button", { name: "Eliminar PB-12" }));
  const deleteConfirmation = await screen.findByRole("dialog", { name: "Eliminar PB-12" });
  assert.equal(deleteCalls, 0);
  await userEvent.click(within(deleteConfirmation).getByRole("button", { name: "Confirmar eliminación" }));
  await waitFor(() => assert.equal(deleteCalls, 1));
  assert.match(management.textContent ?? "", /quitá la asignación desde la edición del producto y después volvé a intentarlo/);
  assert.ok(within(management).getByRole("button", { name: "Eliminar PB-12" }));
  await userEvent.click(management.querySelector('[data-ui-location-list] input[type="checkbox"]')!);
  assert.equal(management.querySelectorAll('[data-ui-location-row]').length, 1);
  assert.equal(management.querySelector('[data-ui-location-row] code')?.textContent, "PB-12");
});

test("recovers from unavailable location loading and exposes retry feedback", async () => {
  let schemaCalls = 0;
  mockIPC((command) => {
    if (command === "list_catalog_categories_command") return { kind: "success", records: [] };
    if (command === "browse_products_command") return emptyBrowse;
    if (command === "location_schema_command") return ++schemaCalls === 1 ? { kind: "error", code: "persistence_failure", message: "failed" } : { kind: "schema_success", schema: { revision: 0, segments: [] } };
    if (command === "list_product_locations_command") return { kind: "locations_success", locations: [] };
    throw new Error(command);
  });
  const management = await openLocations();
  assert.ok(await within(management).findByText(/No se pudieron cargar el esquema y las ubicaciones/));
  await userEvent.click(within(management).getByRole("button", { name: "Reintentar carga" }));
  assert.ok(await within(management).findByRole("heading", { name: "Esquema de ubicaciones" }));
  assert.equal(schemaCalls, 2);
});
