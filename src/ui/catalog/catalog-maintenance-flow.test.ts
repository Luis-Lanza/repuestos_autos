import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

import { createCatalogEditRequest, createCatalogMaintenanceFlow, fieldErrorsForCatalogEdit, filterCatalogCategories, formForCatalogDetail, initialCatalogMaintenanceState } from "./catalog-maintenance-flow.ts";
import { CatalogMaintenanceRecovery, CatalogMetadataEditor, CatalogSuccessNotice, loadCatalogDetail, reloadCatalogRecords } from "./catalog-maintenance-screen.ts";
import { createCatalogMaintenanceCommands } from "../../commands/catalog.ts";

const archived = { entity_id: 1, target: "product" as const, label: "Filter", activity: "archived" as const, revision: 2 };

test("filters category management rows by category name without changing authoritative metadata", () => {
  const categories = [
    { entity_id: 1, target: "category" as const, label: "Filtros", activity: "active" as const, revision: 2, active_product_count: 8 },
    { entity_id: 2, target: "category" as const, label: "Pastillas", activity: "archived" as const, revision: 3, active_product_count: 0 },
  ];
  assert.deepEqual(filterCatalogCategories(categories, "  past "), [categories[1]]);
  assert.deepEqual(filterCatalogCategories(categories, " "), categories);
});

test("keeps Category Management category records available while Products browses separately", () => {
  const category = { entity_id: 4, target: "category" as const, label: "Filtros", activity: "active" as const, revision: 2, active_product_count: 3 };
  const ready = createCatalogMaintenanceFlow(initialCatalogMaintenanceState, { type: "loaded", records: [category] });
  const selected = createCatalogMaintenanceFlow(ready, { type: "detail_started", record: category });
  const returnedToManagement = createCatalogMaintenanceFlow(selected, { type: "selection_cleared" });
  assert.deepEqual(returnedToManagement.records, [category]);
  assert.equal(returnedToManagement.status, "ready");
  assert.equal(returnedToManagement.selected, null);
});

test("surfaces loading, unavailable, validation, conflict, failure, recovery, and archived records", () => {
  const loading = createCatalogMaintenanceFlow(initialCatalogMaintenanceState, { type: "load_started" });
  const ready = createCatalogMaintenanceFlow(loading, { type: "loaded", records: [archived] });
  const validation = createCatalogMaintenanceFlow(ready, { type: "mutation_failed", code: "validation_error" });
  const conflict = createCatalogMaintenanceFlow(validation, { type: "mutation_failed", code: "stale_catalog_record" });
  const failure = createCatalogMaintenanceFlow(conflict, { type: "load_failed" });
  const restored = createCatalogMaintenanceFlow(ready, { type: "mutation_succeeded", record: { ...archived, activity: "active", revision: 3, label: "" } });
  assert.equal(loading.status, "loading");
  assert.equal(ready.records[0].activity, "archived");
  assert.equal(validation.feedback, "Revisá los valores del catálogo e intentá nuevamente.");
  assert.equal(conflict.feedback, "Registro desactualizado. Recargá los registros del catálogo.");
  assert.equal(conflict.recovery_required, true);
  assert.equal(conflict.lifecycle_feedback, "Registro desactualizado. Recargá los registros del catálogo.");
  assert.match(renderToStaticMarkup(createElement(CatalogMaintenanceRecovery, { required: conflict.recovery_required, onReload: () => undefined })), /Recargar registros del catálogo/);
  assert.equal(failure.status, "unavailable");
  assert.equal(restored.records[0].label, "Filter");
});

test("loads editable metadata, validates typed values, and reloads stable conflicts", () => {
  const detail = { target: "product" as const, entity_id: 1, category_id: 2, sku: "FLT", name: "Filter", list_price_centavos: 3000, minimum_sale_price_centavos: 2500, activity: "archived" as const, revision: 2, attribute_definitions: [{ definition_id: 4, label: "Material", field_type: "option" as const, required: true, options: ["Paper"] }], attribute_values: [{ definition_id: 4, value: "Paper" }] };
  const loading = createCatalogMaintenanceFlow(initialCatalogMaintenanceState, { type: "detail_started" });
  const ready = createCatalogMaintenanceFlow(loading, { type: "detail_loaded", detail });
  const pending = createCatalogMaintenanceFlow(ready, { type: "edit_started" });
  const conflict = createCatalogMaintenanceFlow(pending, { type: "edit_failed", code: "stale_catalog_record" });
  const unavailable = createCatalogMaintenanceFlow(ready, { type: "detail_failed", code: "catalog_unavailable" });
  const reactivated = createCatalogMaintenanceFlow({ ...ready, records: [archived], selected: archived }, { type: "mutation_succeeded", record: { ...archived, activity: "active", revision: 3 } });
  assert.deepEqual(formForCatalogDetail(detail), { sku: "FLT", name: "Filter", list_price_centavos: "30,00", minimum_sale_price_centavos: "25,00", attribute_values: { 4: "Paper" } });
  assert.equal(createCatalogEditRequest(detail, { sku: "FLT", name: "Filter", list_price_centavos: "inválido", minimum_sale_price_centavos: "25,00", attribute_values: { 4: "Paper" } }), null);
  assert.equal(createCatalogEditRequest(detail, formForCatalogDetail(detail))?.list_price_centavos, 3000);
  assert.equal(pending.status, "pending");
  assert.equal(conflict.recovery_required, true);
  assert.equal(unavailable.status, "unavailable");
  assert.deepEqual([reactivated.records[0].activity, reactivated.selected?.activity, reactivated.detail?.activity], ["active", "active", "active"]);
  assert.deepEqual([reactivated.records[0].revision, reactivated.selected?.revision, reactivated.detail?.revision], [3, 3, 3]);
  const refreshFailed = createCatalogMaintenanceFlow(reactivated, { type: "refresh_failed" });
  const refreshStarted = createCatalogMaintenanceFlow(refreshFailed, { type: "refresh_started" });
  const listRefreshed = createCatalogMaintenanceFlow(refreshStarted, { type: "refresh_list_succeeded", records: [archived] });
  const refreshRetry = createCatalogMaintenanceFlow(listRefreshed, { type: "refresh_succeeded", records: [archived] });
  const lockedRefresh = createCatalogMaintenanceFlow(listRefreshed, { type: "refresh_succeeded", records: [archived], keep_recovery_locked: true });
  assert.equal(refreshFailed.recovery_required, true);
  assert.equal(refreshFailed.status, "ready");
  assert.deepEqual(refreshFailed.detail, reactivated.detail);
  assert.equal(listRefreshed.recovery_required, true);
  assert.equal(listRefreshed.status, "loading");
  assert.equal(refreshRetry.recovery_required, false);
  assert.equal(lockedRefresh.recovery_required, true);
  assert.deepEqual(refreshRetry.records, [archived]);
  const screen = renderToStaticMarkup(createElement(CatalogMetadataEditor, { detail, form: formForCatalogDetail(detail), pending: true, feedback: "Price must be whole centavos.", fieldErrors: { list_price_centavos: "Price must be whole centavos." }, onChange: () => undefined, onSubmit: () => undefined }));
  assert.match(screen, /Precio de lista \(Bs\).*Referencia para nuevas ventas/i);
  assert.match(screen, /Material/);
  assert.match(screen, /disabled/);
  assert.match(screen, /aria-invalid="true"/);
  assert.match(screen, /Registro archivado/);
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
  assert.deepEqual(calls, ["catalog_metadata_detail_command", "list_catalog_categories_command", "catalog_metadata_detail_command"]);
  assert.equal(state.detail?.target, "category");
  assert.deepEqual(form, { name: "Filters", attribute_values: {} });
});

test("keeps success announced during refresh and scopes validation to invalid fields", () => {
  const detail = { target: "product" as const, entity_id: 1, category_id: 2, sku: "FLT", name: "Filter", list_price_centavos: 3000, minimum_sale_price_centavos: 2500, activity: "active" as const, revision: 2, attribute_definitions: [{ definition_id: 4, label: "Material", field_type: "text" as const, required: true, options: [] }, { definition_id: 5, label: "Length", field_type: "number" as const, required: false, options: [] }, { definition_id: 6, label: "Grade", field_type: "option" as const, required: false, options: ["A"] }], attribute_values: [] };
  const invalid = fieldErrorsForCatalogEdit(detail, { sku: "", name: "", list_price_centavos: "inválido", minimum_sale_price_centavos: "25,00", attribute_values: { 4: "", 5: "not-a-number", 6: "B" } });
  const saved = createCatalogMaintenanceFlow({ ...initialCatalogMaintenanceState, detail }, { type: "edit_succeeded", record: { entity_id: 1, target: "product", label: "Filter", activity: "active", revision: 3 } });
  const loading = createCatalogMaintenanceFlow(saved, { type: "load_started" });
  const listed = createCatalogMaintenanceFlow(loading, { type: "loaded", records: [] });
  const refreshed = createCatalogMaintenanceFlow(listed, { type: "detail_loaded", detail });
  const conflict = renderToStaticMarkup(createElement(CatalogMetadataEditor, { detail, form: formForCatalogDetail(detail), pending: false, feedback: "This catalog record changed. Reload and try again.", fieldErrors: {}, onChange: () => undefined, onSubmit: () => undefined }));
  assert.deepEqual(Object.keys(invalid).sort(), ["attribute-4", "attribute-5", "attribute-6", "list_price_centavos", "name", "sku"]);
  assert.equal(loading.success_notice, "Catálogo actualizado.");
  assert.match(renderToStaticMarkup(createElement(CatalogSuccessNotice, { notice: listed.success_notice })), /role="status".*Catálogo actualizado/);
  assert.equal(refreshed.success_notice, "Catálogo actualizado.");
  assert.doesNotMatch(conflict, /aria-invalid/);
  assert.match(conflict, /role="alert"/);
});
