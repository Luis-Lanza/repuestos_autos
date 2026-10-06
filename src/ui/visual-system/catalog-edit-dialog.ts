import { createElement, useEffect, useRef, useState, type ChangeEvent, type FormEvent, type RefObject } from "react";

import { ATTRIBUTE_FIELD_TYPE, productLocationCommands, type CatalogMaintenanceRecord, type CatalogMetadataDetail, type ProductLocationRecord, type ProductLocationSegment } from "../../commands/catalog.ts";
import { Action, Badge, Feedback, Field } from "./controls.ts";
import { LocationPicker } from "./location-picker.ts";
import { ConfirmationDialog } from "./confirmation-dialog.ts";
import type { CatalogEditFieldErrors, CatalogEditForm } from "../catalog/catalog-maintenance-flow.ts";

export interface CatalogEditDialogProps {
  record: CatalogMaintenanceRecord;
  detail: CatalogMetadataDetail | null;
  form: CatalogEditForm | null;
  loading: boolean;
  pending: boolean;
  feedback: string | null;
  lifecycleFeedback: string | null;
  recoveryRequired: boolean;
  fieldErrors: CatalogEditFieldErrors;
  locations?: ProductLocationRecord[];
  locationsStatus?: "loading" | "ready" | "error";
  onChange: (field: string, value: string) => void;
  onSubmit: () => void;
  onLifecycle: () => void;
  onAddCategoryField?: () => void;
  onChangeCategoryField?: (index: number, field: string, value: string | boolean) => void;
  onRetireCategoryField?: (index: number) => void;
  onRemoveCategoryFieldDraft?: (index: number) => void;
  onSaveCategorySchema?: () => void;
  imageThumbnail?: string | null;
  imagePending?: boolean;
  imageFeedback?: string | null;
  imageFeedbackKind?: "success" | "error" | "advisory";
  onChooseImage?: () => void;
  onRemoveImage?: () => void;
  onReload: () => void;
  onCancel: () => void;
}

interface CatalogMetadataEditorProps {
  detail: CatalogMetadataDetail;
  form: CatalogEditForm;
  nameRef?: RefObject<HTMLInputElement>;
  pending: boolean;
  feedback: string | null;
  fieldErrors: CatalogEditFieldErrors;
  locations?: ProductLocationRecord[];
  locationsStatus?: "loading" | "ready" | "error";
  onChange: (field: string, value: string) => void;
  onSubmit: () => void;
}

export function CatalogMetadataEditor({ detail, form, nameRef, pending, feedback, fieldErrors, onChange, onSubmit, onAddCategoryField, onChangeCategoryField, onRetireCategoryField, onRemoveCategoryFieldDraft, locations = [], locationsStatus = "loading" }: CatalogMetadataEditorProps & Pick<CatalogEditDialogProps, "onAddCategoryField" | "onChangeCategoryField" | "onRetireCategoryField" | "onRemoveCategoryFieldDraft">) {
  const [locationSegments, setLocationSegments] = useState<ProductLocationSegment[]>([]);
  useEffect(() => {
    if (detail.target !== "product") { setLocationSegments([]); return; }
    let current = true;
    void productLocationCommands.schema().then((response) => { if (current) setLocationSegments(response.kind === "schema_success" ? response.schema.segments : []); });
    return () => { current = false; };
  }, [detail.target, detail.entity_id]);
  const attribute = (definition: CatalogMetadataDetail["attribute_definitions"][number]) => {
    const field = `attribute-${definition.definition_id}`;
    const label = `${definition.label}${definition.required ? " (obligatorio)" : " (opcional)"}`;
    const common = { id: `catalog-${field}`, disabled: pending, value: form.attribute_values[definition.definition_id] ?? "", onChange: (event: ChangeEvent<HTMLInputElement | HTMLSelectElement>) => onChange(field, event.target.value) };
    const control = definition.field_type === ATTRIBUTE_FIELD_TYPE.OPTION
      ? createElement("select", common, createElement("option", { value: "" }, "Seleccioná"), definition.options.map((option) => createElement("option", { key: option, value: option }, option)))
      : createElement("input", { ...common, type: definition.field_type === ATTRIBUTE_FIELD_TYPE.NUMBER ? "number" : "text", step: definition.field_type === ATTRIBUTE_FIELD_TYPE.NUMBER ? "any" : undefined });
    return createElement(Field, { key: definition.definition_id, kind: definition.field_type === ATTRIBUTE_FIELD_TYPE.OPTION ? "select" : "text", label, error: fieldErrors[field], control } as never);
  };
  return createElement("form", {
    id: "catalog-edit-form",
    "aria-label": `Formulario para editar ${detail.target === "product" ? "producto" : "categoría"}`,
    onSubmit: (event: FormEvent) => { event.preventDefault(); onSubmit(); },
    "aria-busy": pending || undefined,
    "data-ui-catalog-editor": true,
  },
    createElement("div", { "data-ui-catalog-identity": true },
      createElement("strong", null, detail.name),
      createElement(Badge, { kind: detail.activity, text: detail.activity === "active" ? "Activo" : "Archivado" }),
      createElement("span", null, detail.target === "product" ? "Producto" : "Categoría"),
      detail.target === "product" ? createElement("span", null, `Categoría: ${detail.category_id}`) : null),
    detail.activity === "archived" ? createElement(Feedback, { kind: "advisory" } as never, "Registro archivado. Editar metadatos no lo reactiva.") : null,
    createElement(Field, { kind: "text", label: detail.target === "category" ? "Nombre de la categoría" : "Nombre del producto", error: fieldErrors.name, control: createElement("input", { id: "catalog-edit-name", ref: nameRef, disabled: pending, value: form.name, onChange: (event: ChangeEvent<HTMLInputElement>) => onChange("name", event.target.value) }) } as never),
    detail.target === "category" ? createElement("fieldset", { "data-ui-category-schema": true },
      createElement("legend", null, "Campos de la categoría"),
      createElement("p", null, "Los campos retirados quedan como historial y no se pueden editar ni reactivar."),
      (form.category_fields ?? []).map((field, index) => field.active
        ? field.definition_id !== null
          ? createElement("div", { key: field.definition_id, role: "group", "aria-label": `Campo ${field.label}`, "data-ui-category-schema-field": true, "data-ui-category-schema-existing": true },
            createElement("strong", null, field.label),
            createElement("span", null, `Tipo: ${field.field_type === "text" ? "Texto" : field.field_type === "number" ? "Número" : "Opciones"}`),
            createElement("span", null, field.required ? "Obligatorio" : "Opcional"),
            createElement(Action, { variant: "destructive", disabled: pending, onClick: () => onRetireCategoryField?.(index), "aria-label": `Retirar ${field.label}` }, "Retirar"))
          : createElement("div", { key: `new-${index}`, role: "group", "aria-label": `Campo nuevo ${(form.category_fields ?? []).slice(0, index).filter((item) => item.definition_id === null).length + 1}`, "data-ui-category-schema-field": true, "data-ui-category-schema-draft": true },
            createElement("div", { "data-ui-category-schema-draft-controls": true },
              createElement("div", { "data-ui-category-schema-draft-name": true }, createElement(Field, { kind: "text", label: "Nombre del campo", control: createElement("input", { disabled: pending, value: field.label, onChange: (event: ChangeEvent<HTMLInputElement>) => onChangeCategoryField?.(index, "label", event.currentTarget.value) }) } as never)),
              createElement(Field, { kind: "select", label: "Tipo", control: createElement("select", { disabled: pending, value: field.field_type, onChange: (event: ChangeEvent<HTMLSelectElement>) => onChangeCategoryField?.(index, "field_type", event.currentTarget.value) }, createElement("option", { value: "text" }, "Texto"), createElement("option", { value: "number" }, "Número"), createElement("option", { value: "option" }, "Opciones")) } as never),
              createElement(Field, { kind: "checkbox", label: "Obligatorio", control: createElement("input", { type: "checkbox", disabled: pending, checked: field.required, onChange: (event: ChangeEvent<HTMLInputElement>) => onChangeCategoryField?.(index, "required", event.currentTarget.checked) }) } as never),
              createElement(Action, { variant: "tertiary", disabled: pending, onClick: () => onRemoveCategoryFieldDraft?.(index), "aria-label": `Descartar campo nuevo ${(form.category_fields ?? []).slice(0, index).filter((item) => item.definition_id === null).length + 1}`, "data-ui-category-schema-discard": true }, "×")),
            field.field_type === "option" ? createElement("div", { "data-ui-category-schema-options": true }, createElement(Field, { kind: "text", label: "Opciones (separadas por coma)", control: createElement("input", { disabled: pending, value: field.options, onChange: (event: ChangeEvent<HTMLInputElement>) => onChangeCategoryField?.(index, "options", event.currentTarget.value) }) } as never)) : null)
        : createElement("p", { key: field.definition_id ?? `new-${index}`, "data-ui-retired-category-field": true }, `${field.label} — Retirado (histórico; no editable ni reactivable)`)),
      createElement(Action, { variant: "secondary", disabled: pending, onClick: onAddCategoryField }, "Agregar campo")) : null,
    detail.target === "product" ? createElement("fieldset", null,
      createElement("legend", null, "Datos generales"),
      createElement(Field, { kind: "sku", label: "SKU", error: fieldErrors.sku, control: createElement("input", { id: "catalog-edit-sku", disabled: pending, value: form.sku ?? "", onChange: (event: ChangeEvent<HTMLInputElement>) => onChange("sku", event.target.value) }) } as never),
      createElement(Field, { kind: "money", label: "Precio actual de compra (Bs)", hint: "Costo actual de compra. Se guarda en centavos.", error: fieldErrors.purchase_price_centavos, control: createElement("input", { id: "catalog-edit-purchase-price", disabled: pending, value: form.purchase_price_centavos ?? "", "aria-required": true, onChange: (event: ChangeEvent<HTMLInputElement>) => onChange("purchase_price_centavos", event.target.value) }) } as never),
      createElement(Field, { kind: "money", label: "Precio de venta (Bs)", hint: "Referencia para nuevas ventas. Se guarda en centavos.", error: fieldErrors.sale_price_centavos, control: createElement("input", { id: "catalog-edit-sale-price", disabled: pending, value: form.sale_price_centavos ?? "", onChange: (event: ChangeEvent<HTMLInputElement>) => onChange("sale_price_centavos", event.target.value) }) } as never),
      createElement(Field, { kind: "money", label: "Precio mínimo de venta (Bs)", hint: "No puede superar el precio de venta.", error: fieldErrors.minimum_sale_price_centavos, control: createElement("input", { id: "catalog-edit-minimum-price", disabled: pending, value: form.minimum_sale_price_centavos ?? "", onChange: (event: ChangeEvent<HTMLInputElement>) => onChange("minimum_sale_price_centavos", event.target.value) }) } as never),
      createElement(Field, { kind: "quantity", label: "Umbral de stock bajo (opcional)", hint: "Dejalo vacío para usar 1 unidad.", error: fieldErrors.low_stock_threshold, control: createElement("input", { id: "catalog-edit-low-stock-threshold", min: 1, disabled: pending, value: form.low_stock_threshold ?? "1", onChange: (event: ChangeEvent<HTMLInputElement>) => onChange("low_stock_threshold", event.target.value) }) } as never),
      createElement(LocationPicker, { id: "catalog-edit-primary-location", label: "Ubicación principal (opcional)", locations, segments: locationSegments, selectedId: form.primary_location_id == null ? "" : String(form.primary_location_id), disabled: pending, status: locationsStatus, onChange: (value: string) => onChange("primary_location_id", value) }),
      locationsStatus === "error" ? createElement("p", null, "No se pudieron cargar ubicaciones activas.") : null,
      detail.attribute_definitions.some((field) => field.active !== false) ? createElement("fieldset", null, createElement("legend", null, "Atributos dinámicos"), detail.attribute_definitions.filter((field) => field.active !== false).map(attribute)) : null) : null,
    createElement("button", { type: "submit", tabIndex: -1, "aria-hidden": "true", hidden: true }, "Guardar"),
    feedback ? createElement(Feedback, { kind: "error" } as never, feedback) : null);
}

export function CatalogEditDialog({ record, detail, form, loading, pending, feedback, lifecycleFeedback, recoveryRequired, fieldErrors, locations = [], locationsStatus = "loading", imageThumbnail = null, imagePending = false, imageFeedback = null, imageFeedbackKind = "error", onChooseImage, onRemoveImage, onChange, onSubmit, onLifecycle, onAddCategoryField, onChangeCategoryField, onRetireCategoryField, onRemoveCategoryFieldDraft, onSaveCategorySchema, onReload, onCancel }: CatalogEditDialogProps) {
  const nameRef = useRef<HTMLInputElement>(null);
  const hasFocusedDetail = useRef(false);
  const dialogPending = loading || pending;
  const title = `Editar ${detail?.name ?? record.label}`;

  useEffect(() => {
    if (!detail || loading || hasFocusedDetail.current) return;
    hasFocusedDetail.current = true;
    nameRef.current?.focus();
  }, [detail, loading]);

  return createElement(ConfirmationDialog, {
    open: true,
    purpose: "routine",
    title,
    description: detail ? "Editá los metadatos del registro seleccionado y guardá los cambios." : loading ? "Cargando el detalle autorizado del registro seleccionado…" : "No se pudo cargar el detalle del registro seleccionado.",
    confirmLabel: detail?.target === "category" ? "Guardar cambios" : "Guardar metadatos",
    pending: dialogPending || imagePending,
    pendingLabel: imagePending ? "Procesando imagen…" : loading ? "Cargando detalle…" : "Guardando metadatos…",
    confirmDisabled: !detail || !form || recoveryRequired || imagePending,
    dialogId: "catalog-edit-dialog",
    dialogDataAttribute: "catalog-edit-dialog",
    onCancel,
    onConfirm: detail?.target === "category" ? onSaveCategorySchema ?? onSubmit : onSubmit,
    children: detail && form ? createElement("div", { "data-ui-catalog-edit-content": true },
      createElement(CatalogMetadataEditor, { detail, form, nameRef, pending: dialogPending || imagePending || recoveryRequired, feedback, fieldErrors, locations, locationsStatus, onChange, onSubmit, onAddCategoryField, onChangeCategoryField, onRetireCategoryField, onRemoveCategoryFieldDraft }),
      detail.target === "product" ? createElement("section", { "aria-label": "Imagen del producto", "data-ui-catalog-product-image": true, "aria-busy": imagePending || undefined },
        createElement("h3", null, "Imagen del producto"),
        imageThumbnail ? createElement("img", { src: imageThumbnail, alt: `Imagen de ${detail.name}`, "data-ui-catalog-product-image-preview": true }) : createElement("p", null, "Sin imagen"),
        createElement(Action, { variant: "secondary", disabled: dialogPending || imagePending || recoveryRequired, pending: imagePending, pendingLabel: "Procesando imagen…", onClick: onChooseImage }, "Elegir imagen"),
        imageThumbnail ? createElement(Action, { variant: "tertiary", disabled: dialogPending || imagePending || recoveryRequired, onClick: onRemoveImage }, "Quitar imagen") : null,
        imageFeedback ? createElement(Feedback, { kind: imageFeedbackKind } as never, imageFeedback) : null) : null,
      createElement("section", { "aria-label": "Acciones de ciclo de vida", "data-ui-catalog-lifecycle": true },
        createElement(Action, { variant: detail.activity === "active" ? "destructive" : "secondary", disabled: dialogPending || imagePending || recoveryRequired, onClick: onLifecycle, "data-ui-catalog-lifecycle-action": detail.activity }, detail.activity === "active" ? "Archivar" : "Reactivar"),
        lifecycleFeedback ? createElement(Feedback, { kind: lifecycleFeedback === "Catálogo actualizado." ? "success" : "error" } as never, lifecycleFeedback) : null,
        recoveryRequired ? createElement(Action, { variant: "secondary", disabled: dialogPending || imagePending, onClick: onReload }, "Reintentar actualización") : null))
      : loading ? createElement(Feedback, { kind: "loading" } as never, "Cargando el detalle autorizado…")
        : createElement("div", { "data-ui-catalog-edit-error": true },
          createElement(Feedback, { kind: "error" } as never, feedback ?? "No se pudo cargar el registro."),
          createElement(Action, { variant: "secondary", onClick: onReload }, "Recargar registro")),
  });
}
