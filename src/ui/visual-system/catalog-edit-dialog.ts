import { createElement, useEffect, useRef, type ChangeEvent, type FormEvent, type RefObject } from "react";

import { ATTRIBUTE_FIELD_TYPE, type CatalogMaintenanceRecord, type CatalogMetadataDetail } from "../../commands/catalog.ts";
import { Action, Badge, Feedback, Field } from "./controls.ts";
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
  onChange: (field: string, value: string) => void;
  onSubmit: () => void;
  onLifecycle: () => void;
  onAddCategoryField?: () => void;
  onChangeCategoryField?: (index: number, field: string, value: string | boolean) => void;
  onRetireCategoryField?: (index: number) => void;
  onSaveCategorySchema?: () => void;
  imageThumbnail?: string | null;
  imagePending?: boolean;
  imageFeedback?: string | null;
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
  onChange: (field: string, value: string) => void;
  onSubmit: () => void;
}

export function CatalogMetadataEditor({ detail, form, nameRef, pending, feedback, fieldErrors, onChange, onSubmit, onAddCategoryField, onChangeCategoryField, onRetireCategoryField, onSaveCategorySchema }: CatalogMetadataEditorProps & Pick<CatalogEditDialogProps, "onAddCategoryField" | "onChangeCategoryField" | "onRetireCategoryField" | "onSaveCategorySchema">) {
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
        ? createElement("fieldset", { key: field.definition_id ?? `new-${index}`, "data-ui-category-schema-field": true },
          createElement("legend", null, field.definition_id === null ? `Campo nuevo ${index + 1}` : `Campo ${field.label}`),
          createElement(Field, { kind: "text", label: "Nombre del campo", control: createElement("input", { disabled: pending || field.definition_id !== null, value: field.label, onChange: (event: ChangeEvent<HTMLInputElement>) => onChangeCategoryField?.(index, "label", event.currentTarget.value) }) } as never),
          createElement(Field, { kind: "select", label: "Tipo de campo", control: createElement("select", { disabled: pending || field.definition_id !== null, value: field.field_type, onChange: (event: ChangeEvent<HTMLSelectElement>) => onChangeCategoryField?.(index, "field_type", event.currentTarget.value) }, createElement("option", { value: "text" }, "Texto"), createElement("option", { value: "number" }, "Número"), createElement("option", { value: "option" }, "Opciones")) } as never),
          createElement(Field, { kind: "checkbox", label: "Campo obligatorio", control: createElement("input", { type: "checkbox", disabled: pending || field.definition_id !== null, checked: field.required, onChange: (event: ChangeEvent<HTMLInputElement>) => onChangeCategoryField?.(index, "required", event.currentTarget.checked) }) } as never),
          field.field_type === "option" ? createElement(Field, { kind: "text", label: "Opciones (separadas por coma)", control: createElement("input", { disabled: pending || field.definition_id !== null, value: field.options, onChange: (event: ChangeEvent<HTMLInputElement>) => onChangeCategoryField?.(index, "options", event.currentTarget.value) }) } as never) : null,
          field.definition_id !== null ? createElement(Action, { variant: "destructive", disabled: pending, onClick: () => onRetireCategoryField?.(index), "aria-label": `Retirar ${field.label}` }, "Retirar campo") : null)
        : createElement("p", { key: field.definition_id ?? `new-${index}`, "data-ui-retired-category-field": true }, `${field.label} — Retirado (histórico; no editable ni reactivable)`)),
      createElement(Action, { variant: "secondary", disabled: pending, onClick: onAddCategoryField }, "Agregar campo"),
      createElement(Action, { variant: "primary", disabled: pending, onClick: onSaveCategorySchema }, "Guardar campos de categoría")) : null,
    detail.target === "product" ? createElement("fieldset", null,
      createElement("legend", null, "Datos generales"),
      createElement(Field, { kind: "sku", label: "SKU", error: fieldErrors.sku, control: createElement("input", { id: "catalog-edit-sku", disabled: pending, value: form.sku ?? "", onChange: (event: ChangeEvent<HTMLInputElement>) => onChange("sku", event.target.value) }) } as never),
      createElement(Field, { kind: "money", label: "Precio actual de compra (Bs)", hint: "Costo actual de compra. Se guarda en centavos.", error: fieldErrors.purchase_price_centavos, control: createElement("input", { id: "catalog-edit-purchase-price", disabled: pending, value: form.purchase_price_centavos ?? "", "aria-required": true, onChange: (event: ChangeEvent<HTMLInputElement>) => onChange("purchase_price_centavos", event.target.value) }) } as never),
      createElement(Field, { kind: "money", label: "Precio de venta (Bs)", hint: "Referencia para nuevas ventas. Se guarda en centavos.", error: fieldErrors.sale_price_centavos, control: createElement("input", { id: "catalog-edit-sale-price", disabled: pending, value: form.sale_price_centavos ?? "", onChange: (event: ChangeEvent<HTMLInputElement>) => onChange("sale_price_centavos", event.target.value) }) } as never),
      createElement(Field, { kind: "money", label: "Precio mínimo de venta (Bs)", hint: "No puede superar el precio de venta.", error: fieldErrors.minimum_sale_price_centavos, control: createElement("input", { id: "catalog-edit-minimum-price", disabled: pending, value: form.minimum_sale_price_centavos ?? "", onChange: (event: ChangeEvent<HTMLInputElement>) => onChange("minimum_sale_price_centavos", event.target.value) }) } as never),
      detail.attribute_definitions.some((field) => field.active !== false) ? createElement("fieldset", null, createElement("legend", null, "Atributos dinámicos"), detail.attribute_definitions.filter((field) => field.active !== false).map(attribute)) : null) : null,
    createElement("button", { type: "submit", tabIndex: -1, "aria-hidden": "true", hidden: true }, "Guardar"),
    feedback ? createElement(Feedback, { kind: "error" } as never, feedback) : null);
}

export function CatalogEditDialog({ record, detail, form, loading, pending, feedback, lifecycleFeedback, recoveryRequired, fieldErrors, imageThumbnail = null, imagePending = false, imageFeedback = null, onChooseImage, onRemoveImage, onChange, onSubmit, onLifecycle, onAddCategoryField, onChangeCategoryField, onRetireCategoryField, onSaveCategorySchema, onReload, onCancel }: CatalogEditDialogProps) {
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
    confirmLabel: "Guardar metadatos",
    pending: dialogPending || imagePending,
    pendingLabel: imagePending ? "Procesando imagen…" : loading ? "Cargando detalle…" : "Guardando metadatos…",
    confirmDisabled: !detail || !form || recoveryRequired || imagePending,
    dialogId: "catalog-edit-dialog",
    dialogDataAttribute: "catalog-edit-dialog",
    onCancel,
    onConfirm: onSubmit,
    children: detail && form ? createElement("div", { "data-ui-catalog-edit-content": true },
      createElement(CatalogMetadataEditor, { detail, form, nameRef, pending: dialogPending || imagePending || recoveryRequired, feedback, fieldErrors, onChange, onSubmit, onAddCategoryField, onChangeCategoryField, onRetireCategoryField, onSaveCategorySchema }),
      detail.target === "product" ? createElement("section", { "aria-label": "Imagen del producto", "data-ui-catalog-product-image": true, "aria-busy": imagePending || undefined },
        createElement("h3", null, "Imagen del producto"),
        imageThumbnail ? createElement("img", { src: imageThumbnail, alt: `Imagen de ${detail.name}`, "data-ui-catalog-product-image-preview": true }) : createElement("p", null, "Sin imagen"),
        createElement(Action, { variant: "secondary", disabled: dialogPending || imagePending || recoveryRequired, pending: imagePending, pendingLabel: "Procesando imagen…", onClick: onChooseImage }, "Elegir imagen"),
        imageThumbnail ? createElement(Action, { variant: "tertiary", disabled: dialogPending || imagePending || recoveryRequired, onClick: onRemoveImage }, "Quitar imagen") : null,
        imageFeedback ? createElement(Feedback, { kind: imageFeedback === "No se modificó la imagen." ? "advisory" : imageFeedback.includes("actualizada") ? "success" : "error" } as never, imageFeedback) : null) : null,
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
