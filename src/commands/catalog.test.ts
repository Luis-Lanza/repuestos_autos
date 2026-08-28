import assert from "node:assert/strict";
import test from "node:test";

import { CATALOG_INTENT, CATALOG_TARGET, createCatalogMaintenanceCommands } from "./catalog.ts";

test("allowlists maintenance payloads and preserves only stable opaque outcomes", async () => {
  const calls: unknown[] = [];
  const commands = createCatalogMaintenanceCommands(async (command, payload) => {
    calls.push({ command, payload });
    return { kind: "error", code: "stale_catalog_record", message: "SQLite busy: internal details" };
  });
  const result = await commands.maintain({ target: CATALOG_TARGET.PRODUCT, entity_id: 1, intent: CATALOG_INTENT.ARCHIVE, expected_revision: 0, ignored: true } as never);
  assert.deepEqual(calls, [{ command: "maintain_catalog_command", payload: { request: { target: "product", entity_id: 1, intent: "archive", expected_revision: 0 } } }]);
  assert.deepEqual(result, { kind: "error", code: "stale_catalog_record", message: "This catalog record changed. Reload and try again." });
});

test("maps malformed maintenance results and invoke failures to an opaque failure", async () => {
  const malformed = createCatalogMaintenanceCommands(async () => ({ kind: "success", sql: "never expose" }));
  const unavailable = createCatalogMaintenanceCommands(async () => { throw new Error("SQLite details"); });
  assert.deepEqual(await malformed.list(), { kind: "error", code: "persistence_failure", message: "The catalog could not be loaded." });
  assert.deepEqual(await unavailable.list(), { kind: "error", code: "persistence_failure", message: "The catalog could not be loaded." });
});

test("projects successful maintenance records without backend-only fields", async () => {
  const commands = createCatalogMaintenanceCommands(async (command) => command === "list_catalog_maintenance_command" ? { kind: "success", records: [{ entity_id: 1, target: "product", label: "Filter", activity: "active", revision: 0, sql: "hidden" }] } : { kind: "success", entity_id: 1, target: "product", label: "Filter", activity: "archived", revision: 1, sql: "hidden" });
  assert.deepEqual(await commands.list(), { kind: "success", records: [{ entity_id: 1, target: "product", label: "Filter", activity: "active", revision: 0 }] });
  assert.deepEqual(await commands.maintain({ target: CATALOG_TARGET.PRODUCT, entity_id: 1, intent: CATALOG_INTENT.ARCHIVE, expected_revision: 0 }), { kind: "success", entity_id: 1, target: "product", label: "Filter", activity: "archived", revision: 1 });
});

test("allowlists metadata detail and edit payloads with typed values", async () => {
  const calls: unknown[] = [];
  const commands = createCatalogMaintenanceCommands(async (command, payload) => {
    calls.push({ command, payload });
    return command === "catalog_metadata_detail_command"
      ? { target: "product", entity_id: 1, category_id: 2, sku: "FLT", name: "Filter", catalog_unit_price_centavos: 2500, activity: "archived", revision: 3, attribute_definitions: [{ definition_id: 4, label: "Material", field_type: "option", required: true, options: ["Paper"] }], attribute_values: [{ definition_id: 4, value: "Paper" }], sql: "hidden" }
      : { kind: "success", entity_id: 1, target: "product", label: "", activity: "archived", revision: 4, sql: "hidden" };
  });
  const detail = await commands.detail({ target: CATALOG_TARGET.PRODUCT, entity_id: 1, ignored: true } as never);
  const edited = await commands.edit({ target: CATALOG_TARGET.PRODUCT, entity_id: 1, expected_revision: 3, sku: "FLT", name: "Filter", catalog_unit_price_centavos: 2500, attribute_values: [{ definition_id: 4, value: "Paper", ignored: true }], ignored: true } as never);
  assert.deepEqual(calls, [
    { command: "catalog_metadata_detail_command", payload: { request: { target: "product", entity_id: 1 } } },
    { command: "edit_catalog_command", payload: { request: { target: "product", entity_id: 1, expected_revision: 3, sku: "FLT", name: "Filter", catalog_unit_price_centavos: 2500, attribute_values: [{ definition_id: 4, value: "Paper" }] } } },
  ]);
  assert.deepEqual(detail, { kind: "success", detail: { target: "product", entity_id: 1, category_id: 2, sku: "FLT", name: "Filter", catalog_unit_price_centavos: 2500, activity: "archived", revision: 3, attribute_definitions: [{ definition_id: 4, label: "Material", field_type: "option", required: true, options: ["Paper"] }], attribute_values: [{ definition_id: 4, value: "Paper" }] } });
  assert.deepEqual(edited, { kind: "success", entity_id: 1, target: "product", label: "", activity: "archived", revision: 4 });
});

test("projects category detail and rejects malformed detail payloads", async () => {
  const category = createCatalogMaintenanceCommands(async () => ({ target: "category", entity_id: 2, name: "Filters", activity: "active", revision: 1, attribute_definitions: [], sql: "hidden" }));
  const malformed = createCatalogMaintenanceCommands(async () => ({ target: "product", entity_id: 1, name: "Filter" }));
  assert.deepEqual(await category.detail({ target: CATALOG_TARGET.CATEGORY, entity_id: 2 }), { kind: "success", detail: { target: "category", entity_id: 2, name: "Filters", activity: "active", revision: 1, attribute_definitions: [] } });
  assert.deepEqual(await malformed.detail({ target: CATALOG_TARGET.PRODUCT, entity_id: 1 }), { kind: "error", code: "persistence_failure", message: "The catalog could not be loaded." });
});
