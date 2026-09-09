import { ATTRIBUTE_FIELD_TYPE, CATALOG_TARGET, type CatalogEditInput, type CatalogMaintenanceError, type CatalogMaintenanceRecord, type CatalogMetadataDetail } from "../../commands/catalog.ts";

const STATUS = { LOADING: "loading", READY: "ready", PENDING: "pending", UNAVAILABLE: "unavailable" } as const;
export interface CatalogEditForm { name: string; sku?: string; catalog_unit_price_centavos?: string; attribute_values: Record<number, string>; }
export type CatalogEditFieldErrors = Record<string, string>;
export interface CatalogMaintenanceState { status: (typeof STATUS)[keyof typeof STATUS]; records: CatalogMaintenanceRecord[]; selected: CatalogMaintenanceRecord | null; detail: CatalogMetadataDetail | null; feedback: string | null; field_errors: CatalogEditFieldErrors; success_notice: string | null; recovery_required: boolean; }
export const initialCatalogMaintenanceState: CatalogMaintenanceState = { status: STATUS.LOADING, records: [], selected: null, detail: null, feedback: null, field_errors: {}, success_notice: null, recovery_required: false };
export type CatalogMaintenanceAction = { type: "load_started" } | { type: "loaded"; records: CatalogMaintenanceRecord[] } | { type: "load_failed" } | { type: "detail_started"; record: CatalogMaintenanceRecord } | { type: "detail_loaded"; detail: CatalogMetadataDetail } | { type: "detail_failed"; code: CatalogMaintenanceError["code"] } | { type: "mutation_started" | "edit_started" } | { type: "mutation_succeeded"; record: CatalogMaintenanceRecord } | { type: "edit_succeeded" } | { type: "edit_validation_failed"; field_errors: CatalogEditFieldErrors } | { type: "mutation_failed" | "edit_failed"; code: CatalogMaintenanceError["code"] };
const message = (code: CatalogMaintenanceError["code"]) => code === "validation_error" ? "Revisá los valores del catálogo e intentá nuevamente." : code === "stale_catalog_record" ? "Registro desactualizado. Recargá los registros del catálogo." : code === "lifecycle_blocked" ? "Este cambio de estado no está permitido." : code === "catalog_unavailable" ? "Este registro no está disponible. Recargá el catálogo." : "No se pudo completar el cambio del catálogo.";
export function formatCatalogBs(centavos: number) { return `${Math.trunc(centavos / 100)},${String(Math.abs(centavos % 100)).padStart(2, "0")}`; }
export function parseCatalogBs(value: string) { const match = value.trim().match(/^(\d+)(?:[,.](\d{1,2}))?$/); if (!match) return null; const centavos = Number(match[1]) * 100 + Number((match[2] ?? "").padEnd(2, "0")); return Number.isSafeInteger(centavos) && centavos > 0 ? centavos : null; }
export function formForCatalogDetail(detail: CatalogMetadataDetail): CatalogEditForm { return detail.target === CATALOG_TARGET.CATEGORY ? { name: detail.name, attribute_values: {} } : { sku: detail.sku, name: detail.name, catalog_unit_price_centavos: formatCatalogBs(detail.catalog_unit_price_centavos), attribute_values: Object.fromEntries(detail.attribute_values.map((value) => [value.definition_id, value.value])) }; }
export function fieldErrorsForCatalogEdit(detail: CatalogMetadataDetail, form: CatalogEditForm): CatalogEditFieldErrors {
  const errors: CatalogEditFieldErrors = {};
  if (form.name.trim() === "") errors.name = "Ingresá un nombre.";
  if (detail.target === CATALOG_TARGET.CATEGORY) return errors;
  if (form.sku?.trim() === "") errors.sku = "Ingresá un SKU.";
  if (parseCatalogBs(form.catalog_unit_price_centavos ?? "") === null) errors.catalog_unit_price_centavos = "Ingresá un precio válido en Bs.";
  detail.attribute_definitions.forEach((field) => { const value = form.attribute_values[field.definition_id] ?? ""; if (field.required && value.trim() === "") errors[`attribute-${field.definition_id}`] = "Ingresá un valor."; else if (field.field_type === ATTRIBUTE_FIELD_TYPE.OPTION && value !== "" && !field.options.includes(value)) errors[`attribute-${field.definition_id}`] = "Elegí una opción de la lista."; else if (field.field_type === ATTRIBUTE_FIELD_TYPE.NUMBER && value !== "" && !Number.isFinite(Number(value))) errors[`attribute-${field.definition_id}`] = "Ingresá un número."; });
  return errors;
}
export function createCatalogEditRequest(detail: CatalogMetadataDetail, form: CatalogEditForm): CatalogEditInput | null {
  if (Object.keys(fieldErrorsForCatalogEdit(detail, form)).length > 0) return null;
  if (detail.target === CATALOG_TARGET.CATEGORY) return { target: detail.target, entity_id: detail.entity_id, expected_revision: detail.revision, name: form.name };
  const values = detail.attribute_definitions.map((field) => ({ definition_id: field.definition_id, value: form.attribute_values[field.definition_id] ?? "" }));
  return { target: detail.target, entity_id: detail.entity_id, expected_revision: detail.revision, sku: form.sku ?? "", name: form.name, catalog_unit_price_centavos: parseCatalogBs(form.catalog_unit_price_centavos ?? "") as number, attribute_values: values.filter((value) => detail.attribute_definitions.find((field) => field.definition_id === value.definition_id)?.required || value.value !== "") };
}
export function createCatalogMaintenanceFlow(state: CatalogMaintenanceState, action: CatalogMaintenanceAction): CatalogMaintenanceState {
  switch (action.type) {
    case "load_started": return { ...state, status: STATUS.LOADING, feedback: null, field_errors: {}, recovery_required: false };
    case "loaded": return { ...state, status: STATUS.READY, records: action.records, feedback: null, field_errors: {}, recovery_required: false };
    case "load_failed": return { ...state, status: STATUS.UNAVAILABLE, feedback: "El catálogo no está disponible. Reintentá.", field_errors: {}, success_notice: null, recovery_required: false };
    case "detail_started": return { ...state, status: STATUS.LOADING, selected: action.record, detail: null, feedback: null, field_errors: {}, recovery_required: false };
    case "detail_loaded": return { ...state, status: STATUS.READY, detail: action.detail, feedback: null, field_errors: {}, recovery_required: false };
    case "detail_failed": return { ...state, status: STATUS.UNAVAILABLE, detail: null, feedback: message(action.code), field_errors: {}, success_notice: null, recovery_required: action.code === "stale_catalog_record" };
    case "mutation_started": case "edit_started": return { ...state, status: STATUS.PENDING, feedback: null, field_errors: {}, success_notice: null, recovery_required: false };
    case "mutation_succeeded": return { ...state, status: STATUS.READY, feedback: null, success_notice: "Catálogo actualizado.", field_errors: {}, recovery_required: false, records: state.records.map((record) => record.entity_id === action.record.entity_id && record.target === action.record.target ? { ...record, activity: action.record.activity, revision: action.record.revision } : record), selected: state.selected?.entity_id === action.record.entity_id && state.selected.target === action.record.target ? { ...state.selected, activity: action.record.activity, revision: action.record.revision } : state.selected, detail: state.detail?.entity_id === action.record.entity_id && state.detail.target === action.record.target ? { ...state.detail, activity: action.record.activity, revision: action.record.revision } : state.detail };
    case "edit_succeeded": return { ...state, status: STATUS.READY, feedback: null, field_errors: {}, success_notice: "Catálogo actualizado.", recovery_required: false };
    case "edit_validation_failed": return { ...state, status: STATUS.READY, feedback: "Corregí los campos indicados e intentá nuevamente.", field_errors: action.field_errors, success_notice: null, recovery_required: false };
    case "mutation_failed": case "edit_failed": return { ...state, status: STATUS.READY, feedback: message(action.code), field_errors: {}, success_notice: null, recovery_required: action.code === "stale_catalog_record" };
  }
}
