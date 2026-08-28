import { createElement, useEffect, useReducer } from "react";

import { CATALOG_INTENT, catalogMaintenanceCommands, type CatalogMaintenanceRecord } from "../../commands/catalog.ts";
import { createCatalogMaintenanceFlow, initialCatalogMaintenanceState } from "./catalog-maintenance-flow.ts";

export function CatalogMaintenanceRecovery({ required, onReload }: { required: boolean; onReload: () => void }) { return required ? createElement("button", { type: "button", onClick: onReload }, "Reload catalog records") : null; }
export function CatalogMaintenanceScreen() {
  const [state, dispatch] = useReducer(createCatalogMaintenanceFlow, initialCatalogMaintenanceState);
  const load = async () => { dispatch({ type: "load_started" }); const response = await catalogMaintenanceCommands.list(); dispatch(response.kind === "success" ? { type: "loaded", records: response.records } : { type: "load_failed" }); };
  useEffect(() => { void load(); }, []);
  const maintain = async (record: CatalogMaintenanceRecord) => { dispatch({ type: "mutation_started" }); const response = await catalogMaintenanceCommands.maintain({ target: record.target, entity_id: record.entity_id, intent: record.activity === "active" ? CATALOG_INTENT.ARCHIVE : CATALOG_INTENT.REACTIVATE, expected_revision: record.revision }); dispatch(response.kind === "success" ? { type: "mutation_succeeded", record: response } : { type: "mutation_failed", code: response.code }); };
  return createElement("main", { "aria-labelledby": "catalog-maintenance-heading" }, createElement("h1", { id: "catalog-maintenance-heading" }, "Catalog maintenance"), createElement("p", null, "Archive or reactivate catalog categories and products."), state.status === "loading" ? createElement("p", { role: "status" }, "Loading catalog maintenance records…") : null, state.status === "unavailable" ? createElement("button", { type: "button", onClick: load }, "Retry catalog maintenance") : null, createElement(CatalogMaintenanceRecovery, { required: state.recovery_required, onReload: load }), createElement("ul", { "aria-label": "Catalog maintenance records" }, state.records.map((record) => createElement("li", { key: `${record.target}-${record.entity_id}` }, `${record.label} (${record.target}, ${record.activity})`, createElement("button", { type: "button", disabled: state.status === "pending", onClick: () => void maintain(record) }, record.activity === "active" ? "Archive" : "Reactivate")))), state.feedback ? createElement("p", { role: state.feedback === "Catalog updated." ? "status" : "alert" }, state.feedback) : null);
}
