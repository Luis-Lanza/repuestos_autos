import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

import { createCatalogMaintenanceFlow, initialCatalogMaintenanceState } from "./catalog-maintenance-flow.ts";
import { CatalogMaintenanceRecovery } from "./catalog-maintenance-screen.ts";

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
