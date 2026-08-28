import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

import { createCatalogEditRequest, createCatalogMaintenanceFlow, fieldErrorsForCatalogEdit, formForCatalogDetail, initialCatalogMaintenanceState } from "./catalog-maintenance-flow.ts";
import { CatalogMaintenanceRecovery, CatalogMetadataEditor, CatalogSuccessNotice, loadCatalogDetail, reloadCatalogRecords } from "./catalog-maintenance-screen.ts";
import { createCatalogMaintenanceCommands } from "../../commands/catalog.ts";

const archived = { entity_id: 1, target: "product" as const, label: "Filter", activity: "archived" as const, revision: 2 };

test("surfaces loading, unavailable, validation, conflict, failure, recovery, and archived records", () => {
  const loading = createCatalogMaintenanceFlow(initialCatalogMaintenanceState, { type: "load_started" });
  const ready = createCatalogMaintenanceFlow(loading, { type: "loaded", records: [archived] });
  const validation = createCatalogMaintenanceFlow(ready, { type: "mutation_failed", code: "validation_error" });
  const conflict = createCatalogMaintenanceFlow(validation, { type: "mutation_failed", code: "stale_catalog_record" });
  const failure = createCatalogMaintenanceFlow(conflict, { type: "load_failed" });
  const restored = createCatalogMaintenanceFlow(ready, { type: "mutation_succeeded", record: { ...archived, activity: "active", revision: 3, label: "" } });
  assert.equal(loading.status, "loading");
  assert.equal(ready.records[0].activity, "archived");
  assert.equal(validation.feedback, "Review the catalog values and try again.");
  assert.equal(conflict.feedback, "This catalog record changed. Reload and try again.");
  assert.equal(conflict.recovery_required, true);
  assert.match(renderToStaticMarkup(createElement(CatalogMaintenanceRecovery, { required: conflict.recovery_required, onReload: () => undefined })), /Reload catalog records/);
  assert.equal(failure.status, "unavailable");
  assert.equal(restored.records[0].label, "Filter");
});

test("loads editable metadata, validates typed values, and reloads stable conflicts", () => {
  const detail = { target: "product" as const, entity_id: 1, category_id: 2, sku: "FLT", name: "Filter", catalog_unit_price_centavos: 2500, activity: "archived" as const, revision: 2, attribute_definitions: [{ definition_id: 4, label: "Material", field_type: "option" as const, required: true, options: ["Paper"] }], attribute_values: [{ definition_id: 4, value: "Paper" }] };
  const loading = createCatalogMaintenanceFlow(initialCatalogMaintenanceState, { type: "detail_started" });
  const ready = createCatalogMaintenanceFlow(loading, { type: "detail_loaded", detail });
  const pending = createCatalogMaintenanceFlow(ready, { type: "edit_started" });
  const conflict = createCatalogMaintenanceFlow(pending, { type: "edit_failed", code: "stale_catalog_record" });
  const unavailable = createCatalogMaintenanceFlow(ready, { type: "detail_failed", code: "catalog_unavailable" });
  assert.deepEqual(formForCatalogDetail(detail), { sku: "FLT", name: "Filter", catalog_unit_price_centavos: "2500", attribute_values: { 4: "Paper" } });
  assert.equal(createCatalogEditRequest(detail, { sku: "FLT", name: "Filter", catalog_unit_price_centavos: "2.5", attribute_values: { 4: "Paper" } }), null);
  assert.equal(pending.status, "pending");
  assert.equal(conflict.recovery_required, true);
  assert.equal(unavailable.status, "unavailable");
  const screen = renderToStaticMarkup(createElement(CatalogMetadataEditor, { detail, form: formForCatalogDetail(detail), pending: true, feedback: "Price must be whole centavos.", fieldErrors: { catalog_unit_price_centavos: "Price must be whole centavos." }, onChange: () => undefined, onSubmit: () => undefined }));
  assert.match(screen, /Current catalog price.*centavos.*future sales/i);
  assert.match(screen, /Material/);
  assert.match(screen, /disabled/);
  assert.match(screen, /aria-invalid="true"/);
  assert.match(screen, /Archived record/);
});

test("keeps selected detail identity through failure and retries the same request", async () => {
  const calls: string[] = [];
  const detail = { target: "category" as const, entity_id: 2, name: "Filters", activity: "active" as const, revision: 1, attribute_definitions: [] };
  const commands = createCatalogMaintenanceCommands(async (command) => { calls.push(command); return command === "list_catalog_maintenance_command" ? { kind: "success", records: [] } : calls.filter((item) => item === "catalog_metadata_detail_command").length === 1 ? { kind: "error", code: "catalog_unavailable" } : detail; });
  let state = initialCatalogMaintenanceState;
  const dispatch = (action: never) => { state = createCatalogMaintenanceFlow(state, action); };
  let form = null;
  const selected = { entity_id: 2, target: "category" as const, label: "Filters", activity: "active" as const, revision: 1 };
  await loadCatalogDetail(commands, dispatch, (next) => { form = next; }, selected);
  assert.deepEqual(state.selected, selected);
  await reloadCatalogRecords(commands, dispatch, (next) => { form = next; }, state.selected);
  assert.deepEqual(calls, ["catalog_metadata_detail_command", "list_catalog_maintenance_command", "catalog_metadata_detail_command"]);
  assert.equal(state.detail?.target, "category");
  assert.deepEqual(form, { name: "Filters", attribute_values: {} });
});

test("keeps success announced during refresh and scopes validation to invalid fields", () => {
  const detail = { target: "product" as const, entity_id: 1, category_id: 2, sku: "FLT", name: "Filter", catalog_unit_price_centavos: 2500, activity: "active" as const, revision: 2, attribute_definitions: [{ definition_id: 4, label: "Material", field_type: "text" as const, required: true, options: [] }, { definition_id: 5, label: "Length", field_type: "number" as const, required: false, options: [] }, { definition_id: 6, label: "Grade", field_type: "option" as const, required: false, options: ["A"] }], attribute_values: [] };
  const invalid = fieldErrorsForCatalogEdit(detail, { sku: "", name: "", catalog_unit_price_centavos: "2.5", attribute_values: { 4: "", 5: "not-a-number", 6: "B" } });
  const saved = createCatalogMaintenanceFlow({ ...initialCatalogMaintenanceState, detail }, { type: "edit_succeeded" });
  const loading = createCatalogMaintenanceFlow(saved, { type: "load_started" });
  const listed = createCatalogMaintenanceFlow(loading, { type: "loaded", records: [] });
  const refreshed = createCatalogMaintenanceFlow(listed, { type: "detail_loaded", detail });
  const conflict = renderToStaticMarkup(createElement(CatalogMetadataEditor, { detail, form: formForCatalogDetail(detail), pending: false, feedback: "This catalog record changed. Reload and try again.", fieldErrors: {}, onChange: () => undefined, onSubmit: () => undefined }));
  assert.deepEqual(Object.keys(invalid).sort(), ["attribute-4", "attribute-5", "attribute-6", "catalog_unit_price_centavos", "name", "sku"]);
  assert.equal(loading.success_notice, "Catalog updated.");
  assert.match(renderToStaticMarkup(createElement(CatalogSuccessNotice, { notice: listed.success_notice })), /role="status".*Catalog updated/);
  assert.equal(refreshed.success_notice, null);
  assert.doesNotMatch(conflict, /aria-invalid/);
  assert.match(conflict, /role="alert"/);
});
