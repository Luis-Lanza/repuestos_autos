import { createElement, useEffect, useReducer, useRef, useState } from "react";

import { productLocationCommands, type ProductLocationRecord } from "../../commands/catalog.ts";
import { Action, Feedback, Field } from "../visual-system/controls.ts";
import { ConfirmationDialog } from "../visual-system/confirmation-dialog.ts";
import { createLocationManagementFlow, generateLocationCode, initialLocationManagementState, LOCATION_TEMPLATES, validateLocationSchema, validateLocationValues } from "./location-management-flow.ts";

const locationError = (code: string) => code === "validation_error" ? "Revisá los nombres de segmentos y valores; no pueden estar vacíos ni repetirse." : code === "location_schema_in_use" ? "Ya existen ubicaciones físicas. El esquema y su orden no se pueden cambiar." : code === "duplicate_location_code" ? "Ya existe una ubicación con ese código. Elegí otros valores." : code === "location_in_use" ? "Esta ubicación está asignada a un producto. Para desactivarla o eliminarla, quitá la asignación desde la edición del producto y después volvé a intentarlo." : code === "stale_location" ? "La ubicación cambió. Recargá los datos antes de volver a intentar." : code === "location_unavailable" ? "La ubicación ya no está disponible. Recargá los datos." : "No se pudo completar el cambio de ubicación.";

export function LocationManagementScreen() {
  const [state, dispatch] = useReducer(createLocationManagementFlow, initialLocationManagementState);
  const [segments, setSegments] = useState<string[]>([...LOCATION_TEMPLATES[0].segments]);
  const [values, setValues] = useState<string[]>([]);
  const [schemaErrors, setSchemaErrors] = useState<Record<string, string>>({});
  const [valueErrors, setValueErrors] = useState<Record<string, string>>({});
  const [template, setTemplate] = useState(LOCATION_TEMPLATES[0].name);
  const [includeInactive, setIncludeInactive] = useState(true);
  const [confirm, setConfirm] = useState<{ location: ProductLocationRecord; operation: "deactivate" | "delete" } | null>(null);
  const [pending, setPending] = useState(false);
  const mounted = useRef(true);
  const locked = useRef(false);
  const request = useRef(0);
  const locationHeading = useRef<HTMLHeadingElement>(null);
  const schemaFrozen = state.locations.length > 0;
  useEffect(() => { locationHeading.current?.focus(); }, []);
  useEffect(() => () => { mounted.current = false; request.current += 1; locked.current = true; }, []);

  const load = async () => {
    const current = ++request.current;
    dispatch({ type: "started" });
    const [schema, list] = await Promise.all([productLocationCommands.schema(), productLocationCommands.list(true)]);
    if (!mounted.current || current !== request.current) return;
    if (schema.kind !== "schema_success" || list.kind !== "locations_success") { dispatch({ type: "load_failed" }); return; }
    dispatch({ type: "loaded", schema: schema.schema, locations: list.locations });
    const storedSegments = schema.schema.segments.map((segment) => segment.label);
    const initialSegments = storedSegments.length ? storedSegments : [...LOCATION_TEMPLATES[0].segments];
    const matchingTemplate = LOCATION_TEMPLATES.find((item) => item.segments.length === initialSegments.length && item.segments.every((label, index) => label === initialSegments[index]));
    setTemplate(matchingTemplate?.name ?? (storedSegments.length ? "Personalizado" : LOCATION_TEMPLATES[0].name));
    setSegments(initialSegments);
    setValues(initialSegments.map(() => ""));
  };
  useEffect(() => { void load(); }, []);

  const run = async (operation: () => ReturnType<typeof productLocationCommands.saveSchema>) => {
    if (locked.current || state.status === "unavailable") return;
    locked.current = true; setPending(true); dispatch({ type: "started" });
    const response = await operation();
    if (!mounted.current) return;
    locked.current = false; setPending(false);
    if (response.kind === "error") { dispatch({ type: "failed", message: locationError(response.code) }); return; }
    if (response.kind === "schema_success") dispatch({ type: "succeeded", notice: "Esquema actualizado.", schema: response.schema });
    else if (response.kind === "location_success") { dispatch({ type: "succeeded", notice: `Ubicación ${response.location.code} creada.`, location: response.location }); setValues(segments.map(() => "")); setValueErrors({}); }
    else if (response.kind === "deleted") dispatch({ type: "succeeded", notice: "Ubicación eliminada." });
    else if (response.kind === "location_success") return;
    else dispatch({ type: "failed", message: "La respuesta del servicio no coincide con la operación solicitada." });
  };
  const saveSchema = () => {
    if (!state.schema || schemaFrozen || pending) return;
    const errors = validateLocationSchema(segments); setSchemaErrors(errors);
    if (Object.keys(errors).length) return;
    void run(() => productLocationCommands.saveSchema({ expected_revision: state.schema!.revision, segments }));
  };
  const createLocation = () => {
    if (!state.schema || pending) return;
    const errors = validateLocationValues(values, state.schema.segments.length); setValueErrors(errors);
    if (Object.keys(errors).length) return;
    void run(() => productLocationCommands.create(values));
  };
  const updateActivity = async (location: ProductLocationRecord, active: boolean) => {
    if (locked.current) return;
    locked.current = true; setPending(true); dispatch({ type: "started" });
    const response = active ? await productLocationCommands.activate({ location_id: location.location_id, expected_revision: location.revision }) : await productLocationCommands.deactivate({ location_id: location.location_id, expected_revision: location.revision });
    if (!mounted.current) return;
    locked.current = false; setPending(false); setConfirm(null);
    if (response.kind === "error") dispatch({ type: "failed", message: locationError(response.code) });
    else if (response.kind === "location_success") dispatch({ type: "succeeded", notice: `Ubicación ${response.location.code} ${active ? "activada" : "desactivada"}.`, location: response.location });
    else dispatch({ type: "failed", message: "No se pudo confirmar el estado actualizado." });
  };
  const deleteLocation = async () => {
    const location = confirm?.location;
    if (!location || locked.current) return;
    locked.current = true; setPending(true); dispatch({ type: "started" });
    const response = await productLocationCommands.delete({ location_id: location.location_id, expected_revision: location.revision });
    if (!mounted.current) return;
    locked.current = false; setPending(false); setConfirm(null);
    if (response.kind === "error") dispatch({ type: "failed", message: locationError(response.code) });
    else if (response.kind === "deleted") dispatch({ type: "succeeded", notice: `Ubicación ${location.code} eliminada.`, location_id: location.location_id });
    else dispatch({ type: "failed", message: "No se pudo confirmar la eliminación." });
  };

  const selectTemplate = (name: string) => {
    const selected = LOCATION_TEMPLATES.find((item) => item.name === name);
    setTemplate(name);
    if (selected) { setSegments([...selected.segments]); setValues(selected.segments.map(() => "")); setSchemaErrors({}); }
  };
  const addSegment = () => { if (segments.length < 8) { setSegments((current) => [...current, `Segmento ${current.length + 1}`]); setValues((current) => [...current, ""]); } };
  const moveSegment = (index: number, delta: number) => {
    const target = index + delta; if (target < 0 || target >= segments.length) return;
    const next = [...segments]; [next[index], next[target]] = [next[target], next[index]]; setSegments(next);
    const nextValues = [...values]; [nextValues[index], nextValues[target]] = [nextValues[target], nextValues[index]]; setValues(nextValues);
  };
  const shownLocations = state.locations.filter((location) => includeInactive || location.active);
  const physicalLocationList = createElement("section", { "aria-labelledby": "location-list-heading", "data-ui-location-list": true },
    createElement("h3", { id: "location-list-heading" }, "Ubicaciones físicas"),
    createElement("label", null, createElement("input", { type: "checkbox", checked: includeInactive, onChange: (event) => setIncludeInactive(event.currentTarget.checked) }), " Mostrar inactivas"),
    shownLocations.length === 0 ? createElement(Feedback, { kind: "empty" }, state.locations.length ? "No hay ubicaciones que coincidan con este filtro." : "Todavía no hay ubicaciones físicas.") : null,
    createElement("ul", { "aria-label": "Ubicaciones físicas", "data-ui-location-rows": true }, shownLocations.map((location) => createElement("li", { key: location.location_id, "data-ui-location-row": true },
      createElement("code", null, location.code), createElement("span", null, location.values.join(" · ")), createElement("span", { "data-ui-location-state": location.active ? "active" : "inactive" }, location.active ? "Activa" : "Inactiva"),
      createElement("div", null,
        createElement(Action, { variant: location.active ? "destructive" : "secondary", disabled: pending, onClick: () => location.active ? setConfirm({ location, operation: "deactivate" }) : void updateActivity(location, true), "aria-label": `${location.active ? "Desactivar" : "Activar"} ${location.code}` }, location.active ? "Desactivar" : "Activar"),
        createElement(Action, { variant: "tertiary", disabled: pending, onClick: () => setConfirm({ location, operation: "delete" }), "aria-label": `Eliminar ${location.code}` }, "Eliminar"))))));

  return createElement("section", { "aria-labelledby": "catalog-location-heading", "data-ui-location-management": true },
    createElement("h2", { id: "catalog-location-heading", ref: locationHeading, tabIndex: -1 }, "Gestionar ubicaciones"),
    createElement("p", null, "Definí cómo se identifican las ubicaciones físicas y administrá sus códigos generados."),
    state.status === "loading" ? createElement(Feedback, { kind: "loading" }, "Cargando esquema y ubicaciones…") : null,
    state.status === "unavailable" ? createElement(Feedback, { kind: "unavailable" }, createElement("span", null, state.error, " ", createElement(Action, { variant: "secondary", onClick: () => void load() }, "Reintentar carga"))) : null,
    state.error && state.status !== "unavailable" ? createElement(Feedback, { kind: "error" }, state.error) : null,
    state.notice ? createElement(Feedback, { kind: "success" }, state.notice) : null,
    state.status === "ready" && state.schema ? createElement("div", { "data-ui-location-workspace": true },
      createElement("section", { "aria-labelledby": "location-schema-heading", "data-ui-location-schema": true },
        createElement("h3", { id: "location-schema-heading" }, "Esquema de ubicaciones"),
        schemaFrozen ? createElement(Feedback, { kind: "advisory" }, "El esquema queda protegido porque ya existe una ubicación física. Podés seguir creando ubicaciones y administrar su estado.") : null,
        createElement(Field, { kind: "select", label: "Plantilla", control: createElement("select", { value: template, disabled: schemaFrozen || pending, onChange: (event) => selectTemplate(event.currentTarget.value) }, [...(template === "Personalizado" ? [{ name: "Personalizado", segments: [] as string[] }] : []), ...LOCATION_TEMPLATES].map((item) => createElement("option", { key: item.name, value: item.name }, item.name))) }),
        createElement("ol", { "aria-label": "Segmentos ordenados", "data-ui-location-segments": true }, segments.map((segment, index) => createElement("li", { key: index, "data-ui-location-segment": true },
          createElement(Field, { kind: "text", label: `Segmento ${index + 1}`, error: schemaErrors[`segment-${index}`], control: createElement("input", { value: segment, maxLength: 40, disabled: schemaFrozen || pending, onChange: (event) => { const value = event.currentTarget.value; setSegments((current) => current.map((label, position) => position === index ? value : label)); } }) }),
          createElement(Action, { variant: "tertiary", disabled: schemaFrozen || pending || index === 0, "aria-label": `Subir segmento ${index + 1}`, onClick: () => moveSegment(index, -1) }, "Subir"),
          createElement(Action, { variant: "tertiary", disabled: schemaFrozen || pending || index === segments.length - 1, "aria-label": `Bajar segmento ${index + 1}`, onClick: () => moveSegment(index, 1) }, "Bajar"),
          createElement(Action, { variant: "tertiary", disabled: schemaFrozen || pending || segments.length <= 1, "aria-label": `Quitar segmento ${index + 1}`, onClick: () => { setSegments((current) => current.filter((_, position) => position !== index)); setValues((current) => current.filter((_, position) => position !== index)); } }, "Quitar")))),
        schemaErrors.segments ? createElement(Feedback, { kind: "error" }, schemaErrors.segments) : null,
        createElement("div", { "data-ui-location-schema-actions": true }, createElement(Action, { variant: "secondary", disabled: schemaFrozen || pending || segments.length >= 8, onClick: addSegment }, "Agregar segmento"), createElement(Action, { variant: "primary", disabled: schemaFrozen || pending, onClick: saveSchema }, "Guardar esquema")),
        createElement("p", { "data-ui-location-preview": true, "aria-live": "polite" }, "Vista previa del código: ", createElement("code", null, generateLocationCode(segments.map((_, index) => values[index] ?? "")) || "Completá los valores"))),
      createElement("section", { "aria-labelledby": "location-create-heading", "data-ui-location-create": true },
        createElement("h3", { id: "location-create-heading" }, "Nueva ubicación física"),
        createElement("form", { onSubmit: (event) => { event.preventDefault(); createLocation(); } },
          state.schema.segments.map((segment, index) => createElement(Field, { key: segment.id, kind: "text", label: segment.label, error: valueErrors[`value-${index}`], control: createElement("input", { value: values[index] ?? "", maxLength: 40, disabled: pending, onChange: (event) => { const nextValue = event.currentTarget.value; setValues((current) => current.map((value, position) => position === index ? nextValue : value)); } }) })),
          valueErrors.values ? createElement(Feedback, { kind: "error" }, valueErrors.values) : null,
          createElement("p", null, "Código generado: ", createElement("code", null, generateLocationCode(values.slice(0, state.schema.segments.length)) || "Completá los valores")),
          createElement(Action, { variant: "primary", type: "submit", disabled: pending }, "Crear ubicación"))),
      physicalLocationList) : null,
    confirm ? createElement(ConfirmationDialog, { open: true, purpose: "cancellation", title: `${confirm.operation === "delete" ? "Eliminar" : "Desactivar"} ${confirm.location.code}`, description: confirm.operation === "delete" ? `¿Eliminar la ubicación ${confirm.location.code}? Si está asignada a un producto, la operación será rechazada.` : `¿Desactivar ${confirm.location.code}? Si está asignada a un producto, no se podrá desactivar.`, confirmLabel: confirm.operation === "delete" ? "Confirmar eliminación" : "Confirmar desactivación", pending, pendingLabel: "Actualizando…", dialogId: "location-lifecycle-confirmation", onCancel: () => { if (!pending) setConfirm(null); }, onConfirm: () => confirm.operation === "delete" ? void deleteLocation() : void updateActivity(confirm.location, false) }) : null);
}
