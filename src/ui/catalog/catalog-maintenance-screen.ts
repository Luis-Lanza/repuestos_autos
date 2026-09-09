import { createElement, type ChangeEvent, type FormEvent, useEffect, useReducer, useRef, useState } from "react";

import { ATTRIBUTE_FIELD_TYPE, CATALOG_INTENT, catalogMaintenanceCommands, type CatalogMaintenanceRecord, type CatalogMetadataDetail } from "../../commands/catalog.ts";
import { Action, Badge, Feedback, Field } from "../visual-system/controls.ts";
import { Panel } from "../visual-system/structure.ts";
import { createCatalogEditRequest, createCatalogMaintenanceFlow, fieldErrorsForCatalogEdit, formForCatalogDetail, initialCatalogMaintenanceState, type CatalogEditFieldErrors, type CatalogEditForm, type CatalogMaintenanceAction } from "./catalog-maintenance-flow.ts";

type Dispatch = (action: CatalogMaintenanceAction) => void;
type SetForm = (form: CatalogEditForm) => void;
type CatalogLoadCommands = Pick<typeof catalogMaintenanceCommands, "detail" | "list">;

export function CatalogMaintenanceRecovery({ required, onReload }: { required: boolean; onReload: () => void }) {
  return required ? createElement(Action, { variant: "secondary", onClick: onReload }, "Recargar registros del catálogo") : null;
}
export function CatalogSuccessNotice({ notice }: { notice: string | null }) {
  return notice ? createElement(Feedback, { kind: "success" } as never, notice) : null;
}
export async function loadCatalogRecords(commands: CatalogLoadCommands, dispatch: Dispatch) {
  dispatch({ type: "load_started" });
  const response = await commands.list();
  dispatch(response.kind === "success" ? { type: "loaded", records: response.records } : { type: "load_failed" });
}
export async function loadCatalogDetail(commands: CatalogLoadCommands, dispatch: Dispatch, setForm: SetForm, record: CatalogMaintenanceRecord) {
  dispatch({ type: "detail_started", record });
  const response = await commands.detail(record);
  if (response.kind === "success") { setForm(formForCatalogDetail(response.detail)); dispatch({ type: "detail_loaded", detail: response.detail }); }
  else dispatch({ type: "detail_failed", code: response.code });
}
export async function reloadCatalogRecords(commands: CatalogLoadCommands, dispatch: Dispatch, setForm: SetForm, selected: CatalogMaintenanceRecord | null) {
  await loadCatalogRecords(commands, dispatch);
  if (selected) await loadCatalogDetail(commands, dispatch, setForm, selected);
}

interface CatalogMetadataEditorProps {
  detail: CatalogMetadataDetail; form: CatalogEditForm; pending: boolean; feedback: string | null;
  fieldErrors: CatalogEditFieldErrors; onChange: (field: string, value: string) => void; onSubmit: () => void;
}
export function CatalogMetadataEditor({ detail, form, pending, feedback, fieldErrors, onChange, onSubmit }: CatalogMetadataEditorProps) {
  const attribute = (definition: CatalogMetadataDetail["attribute_definitions"][number]) => {
    const field = `attribute-${definition.definition_id}`;
    const label = `${definition.label}${definition.required ? " (obligatorio)" : " (opcional)"}`;
    const common = { id: `catalog-${field}`, disabled: pending, value: form.attribute_values[definition.definition_id] ?? "", onChange: (event: ChangeEvent<HTMLInputElement | HTMLSelectElement>) => onChange(field, event.target.value) };
    const control = definition.field_type === ATTRIBUTE_FIELD_TYPE.OPTION
      ? createElement("select", common, createElement("option", { value: "" }, "Seleccioná"), definition.options.map((option) => createElement("option", { key: option, value: option }, option)))
      : createElement("input", { ...common, type: definition.field_type === ATTRIBUTE_FIELD_TYPE.NUMBER ? "number" : "text", step: definition.field_type === ATTRIBUTE_FIELD_TYPE.NUMBER ? "any" : undefined });
    return createElement(Field, { key: definition.definition_id, kind: definition.field_type === ATTRIBUTE_FIELD_TYPE.OPTION ? "select" : "text", label, error: fieldErrors[field], control } as never);
  };
  return createElement("form", { onSubmit: (event: FormEvent) => { event.preventDefault(); onSubmit(); }, "aria-busy": pending || undefined, "data-ui-catalog-editor": true },
    createElement("div", { "data-ui-catalog-identity": true },
      createElement("strong", null, detail.name),
      createElement(Badge, { kind: detail.activity, text: detail.activity === "active" ? "Activo" : "Archivado" }),
      createElement("span", null, detail.target === "product" ? "Producto" : "Categoría"),
      detail.target === "product" ? createElement("span", null, `Categoría: ${detail.category_id}`) : null),
    detail.activity === "archived" ? createElement(Feedback, { kind: "advisory" } as never, "Registro archivado. Editar metadatos no lo reactiva.") : null,
    createElement(Field, { kind: "text", label: detail.target === "category" ? "Nombre de la categoría" : "Nombre del producto", error: fieldErrors.name, control: createElement("input", { id: "catalog-edit-name", disabled: pending, value: form.name, onChange: (event: ChangeEvent<HTMLInputElement>) => onChange("name", event.target.value) }) } as never),
    detail.target === "product" ? createElement("fieldset", null,
      createElement("legend", null, "Datos generales"),
      createElement(Field, { kind: "sku", label: "SKU", error: fieldErrors.sku, control: createElement("input", { id: "catalog-edit-sku", disabled: pending, value: form.sku ?? "", onChange: (event: ChangeEvent<HTMLInputElement>) => onChange("sku", event.target.value) }) } as never),
      createElement(Field, { kind: "money", label: "Precio actual del catálogo (Bs)", hint: "Afecta solo ventas futuras. Las ventas confirmadas no cambian.", error: fieldErrors.catalog_unit_price_centavos, control: createElement("input", { id: "catalog-edit-price", disabled: pending, value: form.catalog_unit_price_centavos ?? "", onChange: (event: ChangeEvent<HTMLInputElement>) => onChange("catalog_unit_price_centavos", event.target.value) }) } as never),
      detail.attribute_definitions.length ? createElement("fieldset", null, createElement("legend", null, "Atributos dinámicos"), detail.attribute_definitions.map(attribute)) : null) : null,
    feedback ? createElement(Feedback, { kind: "error" } as never, feedback) : null,
    createElement(Action, { variant: "primary", type: "submit", pending, pendingLabel: "Guardando metadatos…" }, "Guardar metadatos"));
}

export function CatalogMaintenanceScreen() {
  const [state, dispatch] = useReducer(createCatalogMaintenanceFlow, initialCatalogMaintenanceState);
  const [form, setForm] = useState<CatalogEditForm | null>(null);
  const mounted = useRef(true);
  const attempt = useRef(0);
  const mutationLocked = useRef(false);
  useEffect(() => () => { mounted.current = false; attempt.current += 1; mutationLocked.current = true; }, []);

  const load = async () => {
    const current = ++attempt.current;
    dispatch({ type: "load_started" });
    const response = await catalogMaintenanceCommands.list();
    if (mounted.current && current === attempt.current) dispatch(response.kind === "success" ? { type: "loaded", records: response.records } : { type: "load_failed" });
  };
  const loadDetail = async (record: CatalogMaintenanceRecord) => {
    const current = ++attempt.current;
    dispatch({ type: "detail_started", record });
    const response = await catalogMaintenanceCommands.detail(record);
    if (!mounted.current || current !== attempt.current) return;
    if (response.kind === "success") { setForm(formForCatalogDetail(response.detail)); dispatch({ type: "detail_loaded", detail: response.detail }); }
    else dispatch({ type: "detail_failed", code: response.code });
  };
  useEffect(() => { void load(); }, []);
  useEffect(() => {
    const first = Object.keys(state.field_errors)[0];
    if (first) document.getElementById(first.startsWith("attribute-") ? `catalog-${first}` : `catalog-edit-${first === "catalog_unit_price_centavos" ? "price" : first}`)?.focus();
  }, [state.field_errors]);

  const reload = async () => {
    const selected = state.selected;
    const selectionAttempt = attempt.current;
    await load();
    if (mounted.current && selected && attempt.current === selectionAttempt + 1) await loadDetail(selected);
  };
  const maintain = async (record: CatalogMaintenanceRecord) => {
    if (mutationLocked.current) return;
    mutationLocked.current = true;
    dispatch({ type: "mutation_started" });
    const response = await catalogMaintenanceCommands.maintain({ target: record.target, entity_id: record.entity_id, intent: record.activity === "active" ? CATALOG_INTENT.ARCHIVE : CATALOG_INTENT.REACTIVATE, expected_revision: record.revision });
    if (!mounted.current) return;
    mutationLocked.current = false;
    dispatch(response.kind === "success" ? { type: "mutation_succeeded", record: response } : { type: "mutation_failed", code: response.code });
  };
  const edit = async () => {
    if (!state.detail || !form || mutationLocked.current) return;
    const request = createCatalogEditRequest(state.detail, form);
    if (!request) { dispatch({ type: "edit_validation_failed", field_errors: fieldErrorsForCatalogEdit(state.detail, form) }); return; }
    mutationLocked.current = true;
    dispatch({ type: "edit_started" });
    const response = await catalogMaintenanceCommands.edit(request);
    if (!mounted.current) return;
    mutationLocked.current = false;
    if (response.kind === "error") { dispatch({ type: "edit_failed", code: response.code }); return; }
    dispatch({ type: "edit_succeeded" });
    await load();
    await loadDetail({ target: response.target, entity_id: response.entity_id, label: response.label, activity: response.activity, revision: response.revision });
  };
  const change = (field: string, value: string) => setForm((current) => !current ? current : field.startsWith("attribute-") ? { ...current, attribute_values: { ...current.attribute_values, [Number(field.slice(10))]: value } } : { ...current, [field]: value });
  const pending = state.status === "pending";

  return createElement("main", { "aria-labelledby": "catalog-maintenance-heading" },
    createElement("h1", { id: "catalog-maintenance-heading" }, "Catálogo"),
    createElement("p", null, "Editá metadatos o cambiá el estado de categorías y productos."),
    createElement(CatalogSuccessNotice, { notice: state.success_notice }),
    state.status === "loading" && !state.selected ? createElement(Feedback, { kind: "loading" } as never, "Cargando registros del catálogo…") : null,
    state.status === "unavailable" && !state.selected ? createElement(Feedback, { kind: "unavailable" } as never, createElement("span", null, "El catálogo no está disponible. ", createElement(Action, { variant: "tertiary", onClick: reload }, "Reintentar catálogo"))) : null,
    createElement("div", { "data-ui-catalog-layout": true },
      createElement(Panel, { label: "Registros del catálogo" } as never,
        state.status === "ready" && state.records.length === 0 ? createElement(Feedback, { kind: "empty" } as never, "Todavía no hay registros del catálogo.") : null,
        createElement("ul", { "data-ui-catalog-master": true }, state.records.map((record) => createElement("li", { key: `${record.target}-${record.entity_id}`, "data-ui-selected": state.selected?.target === record.target && state.selected.entity_id === record.entity_id || undefined },
          createElement(Action, { variant: "tertiary", disabled: pending, onClick: () => void loadDetail(record), "aria-label": `Ver detalles de ${record.label}` }, createElement("span", null, createElement("strong", null, record.label), createElement("small", null, record.target === "product" ? "Producto" : "Categoría"), createElement(Badge, { kind: record.activity, text: record.activity === "active" ? "Activo" : "Archivado" }))),
          createElement(Action, { variant: record.activity === "active" ? "destructive" : "secondary", disabled: pending, onClick: () => void maintain(record) }, record.activity === "active" ? "Archivar" : "Reactivar"))))),
      createElement(Panel, { label: "Detalle y edición" } as never,
        state.selected && !state.detail ? createElement("div", { "data-ui-catalog-identity": true }, createElement("strong", null, `Selección: ${state.selected.label}`), createElement(Badge, { kind: state.selected.activity, text: state.selected.activity === "active" ? "Activo" : "Archivado" })) : null,
        state.status === "loading" && state.selected ? createElement(Feedback, { kind: "loading" } as never, "Cargando el registro seleccionado…") : null,
        state.feedback && !state.detail ? createElement(Feedback, { kind: state.recovery_required ? "stale" : "unavailable" } as never, state.feedback) : null,
        createElement(CatalogMaintenanceRecovery, { required: state.recovery_required || state.status === "unavailable" && !!state.selected, onReload: reload }),
        !state.selected ? createElement(Feedback, { kind: "initial" } as never, "Seleccioná un registro para ver sus datos y editarlo.") : null,
        state.detail && form ? createElement(CatalogMetadataEditor, { detail: state.detail, form, pending, feedback: state.feedback, fieldErrors: state.field_errors, onChange: change, onSubmit: edit }) : null)));
}
