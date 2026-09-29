import { createElement as h, useEffect, useReducer, useRef, useState, type ChangeEvent, type FormEvent } from "react";
import { createCategory, createProduct, FIELD_TYPE, listCategories, type Category, type CategoryFieldInput, type FieldType } from "../../commands/onboarding.ts";
import { CATALOG_TARGET, catalogMaintenanceCommands, productLocationCommands, type ProductLocationRecord, type ProductLocationSegment } from "../../commands/catalog.ts";
import { Action, Feedback, Field } from "../visual-system/controls.ts";
import { LocationPicker } from "../visual-system/location-picker.ts";
import { Panel } from "../visual-system/structure.ts";
import { attributeValuesFor, parseBsToCentavos, parsePositiveWhole, validateCategoryAttributes } from "./onboarding-form.ts";
import { canSubmitCategory, canSubmitProduct, createOnboardingFlow, initialOnboardingState } from "./onboarding-flow.ts";

interface Props { onBack: () => void }
type PendingField = CategoryFieldInput & { optionsText: string };
const emptyField: PendingField = { label: "", field_type: FIELD_TYPE.TEXT, required: false, options: [], optionsText: "" };

export function OnboardingScreen({ onBack }: Props) {
  const [state, dispatch] = useReducer(createOnboardingFlow, initialOnboardingState);
  const [categoryName, setCategoryName] = useState("");
  const [pendingField, setPendingField] = useState(emptyField);
  const [categoryFields, setCategoryFields] = useState<CategoryFieldInput[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [sku, setSku] = useState("");
  const [productName, setProductName] = useState("");
  const [purchasePrice, setPurchasePrice] = useState("");
  const [listPrice, setListPrice] = useState("");
  const [minimumSalePrice, setMinimumSalePrice] = useState("");
  const [stock, setStock] = useState("");
  const [lowStockThreshold, setLowStockThreshold] = useState("");
  const [attributes, setAttributes] = useState<Record<number, string>>({});
  const [productLocations, setProductLocations] = useState<ProductLocationRecord[]>([]);
  const [locationSegments, setLocationSegments] = useState<ProductLocationSegment[]>([]);
  const [locationsStatus, setLocationsStatus] = useState<"loading" | "ready" | "error">("loading");
  const [primaryLocationId, setPrimaryLocationId] = useState("");
  const [fieldError, setFieldError] = useState("");
  const [fieldErrorMessage, setFieldErrorMessage] = useState("");
  const mounted = useRef(true), request = useRef(0), mutation = useRef(0), categoryLock = useRef(false), productLock = useRef(false);
  const selected = state.categories.find((category) => category.category_id === Number(selectedId));
  const focus = (id: string) => document.getElementById(id)?.focus();
  useEffect(() => { if (state.productStatus === "error" && fieldError.startsWith("attribute-")) focus(fieldError); }, [state.productStatus, fieldError]);

  const loadCategories = async () => {
    const id = ++request.current;
    dispatch({ type: "categories_started", requestId: id });
    try {
      const response = await listCategories();
      if (!mounted.current || id !== request.current) return;
      dispatch(response.kind === "success" ? { type: "categories_succeeded", requestId: id, categories: response.categories } : { type: "categories_failed", requestId: id });
      if (response.kind === "success" && response.categories[0]) setSelectedId(String(response.categories[0].category_id));
    } catch { if (mounted.current && id === request.current) dispatch({ type: "categories_failed", requestId: id }); }
  };
  const loadProductLocations = async () => {
    const [response, schema] = await Promise.all([productLocationCommands.list(false), productLocationCommands.schema()]);
    if (!mounted.current) return;
    if (response.kind === "locations_success") { setProductLocations(response.locations.filter((location) => location.active)); setLocationsStatus("ready"); }
    else { setProductLocations([]); setLocationsStatus("error"); }
    setLocationSegments(schema.kind === "schema_success" ? schema.schema.segments : []);
  };
  useEffect(() => { void loadCategories(); void loadProductLocations(); return () => { mounted.current = false; request.current++; mutation.current++; }; }, []);

  const addField = () => {
    if (!pendingField.label.trim()) return;
    const options = pendingField.field_type === FIELD_TYPE.OPTION ? pendingField.optionsText.split(",").map((option) => option.trim()).filter(Boolean) : [];
    setCategoryFields((fields) => [...fields, { label: pendingField.label.trim(), field_type: pendingField.field_type, required: pendingField.required, options }]);
    setPendingField(emptyField);
  };
  const submitCategory = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!canSubmitCategory(state) || categoryLock.current) return;
    if (!categoryName.trim()) { setFieldError("category"); focus("category-name"); return; }
    const id = ++mutation.current; categoryLock.current = true; setFieldError(""); dispatch({ type: "category_started", requestId: id });
    try {
      const response = await createCategory({ name: categoryName.trim(), fields: categoryFields });
      if (!mounted.current || id !== mutation.current) return;
      if (response.kind === "success") { dispatch({ type: "category_succeeded", requestId: id, category: response }); setSelectedId(String(response.category_id)); setCategoryName(""); setCategoryFields([]); }
      else dispatch({ type: "category_failed", requestId: id });
    } catch { if (mounted.current && id === mutation.current) dispatch({ type: "category_failed", requestId: id }); }
    finally { categoryLock.current = false; }
  };
  const submitProduct = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!canSubmitProduct(state) || productLock.current) return;
    const productErrors = !selected ? "category" : !sku.trim() ? "sku" : !productName.trim() ? "name" : parseBsToCentavos(purchasePrice) === null ? "purchase-price" : parseBsToCentavos(listPrice) === null ? "list-price" : parseBsToCentavos(minimumSalePrice) === null ? "minimum-price" : parseBsToCentavos(minimumSalePrice)! > parseBsToCentavos(listPrice)! ? "minimum-price" : parsePositiveWhole(stock) === null ? "stock" : lowStockThreshold.trim() !== "" && parsePositiveWhole(lowStockThreshold) === null ? "low-stock-threshold" : "";
    const attributeError = selected && !productErrors ? validateCategoryAttributes(selected, attributes) : null;
    if (productErrors || attributeError) {
      setFieldError(attributeError ? `attribute-${attributeError.definitionId}` : productErrors); setFieldErrorMessage("");
      focus(attributeError ? `attribute-${attributeError.definitionId}` : productErrors === "category" ? "product-category" : productErrors === "sku" ? "product-sku" : productErrors === "name" ? "product-name" : productErrors === "purchase-price" ? "purchase-price" : productErrors === "list-price" ? "list-price" : productErrors === "minimum-price" ? "minimum-sale-price" : productErrors === "low-stock-threshold" ? "low-stock-threshold" : "opening-stock");
      return;
    }
    const id = ++mutation.current; productLock.current = true; setFieldError(""); setFieldErrorMessage(""); dispatch({ type: "product_started", requestId: id });
    try {
      const refreshed = await listCategories();
      if (!mounted.current || id !== mutation.current) return;
      if (refreshed.kind !== "success") {
        dispatch({ type: "product_failed", requestId: id, message: "No se pudo actualizar la categoría. Reintentá antes de crear el producto." });
        return;
      }
      const latest = refreshed.categories.find((category) => category.category_id === selected!.category_id);
      const fieldsChanged = !latest || JSON.stringify(latest.fields) !== JSON.stringify(selected!.fields);
      if (fieldsChanged) {
        dispatch({ type: "category_schema_changed", requestId: id, categories: refreshed.categories });
        setAttributes({});
        if (latest) setSelectedId(String(latest.category_id));
        return;
      }
      const response = await createProduct({ sku: sku.trim(), name: productName.trim(), category_id: selected!.category_id, purchase_price_centavos: parseBsToCentavos(purchasePrice)!, sale_price_centavos: parseBsToCentavos(listPrice)!, minimum_sale_price_centavos: parseBsToCentavos(minimumSalePrice)!, low_stock_threshold: lowStockThreshold.trim() === "" ? 1 : parsePositiveWhole(lowStockThreshold)!, opening_quantity: parsePositiveWhole(stock)!, attribute_values: attributeValuesFor(latest!, attributes) });
      if (!mounted.current || id !== mutation.current) return;
      if (response.kind === "success") {
        let locationMessage = "";
        if (primaryLocationId) {
          locationMessage = " No se pudo asignar la ubicación principal; podés corregirla desde el Catálogo.";
          try {
            const detail = await catalogMaintenanceCommands.detail({ target: CATALOG_TARGET.PRODUCT, entity_id: response.product_id });
            const assignment = detail.kind === "success" && detail.detail.target === CATALOG_TARGET.PRODUCT && detail.detail.entity_id === response.product_id
              ? await productLocationCommands.assignPrimary({ product_id: response.product_id, expected_revision: detail.detail.revision, location_id: Number(primaryLocationId) })
              : null;
            const location = productLocations.find((item) => item.location_id === Number(primaryLocationId));
            if (assignment?.kind === "assignment_success" && assignment.product_id === response.product_id && assignment.location_id === Number(primaryLocationId) && location) {
              locationMessage = ` Ubicación principal: ${location.code}.`;
            }
          } catch {
            // Product creation succeeded; assignment is an independent follow-up operation.
          }
        }
        if (!mounted.current || id !== mutation.current) return;
        dispatch({ type: "product_succeeded", requestId: id, message: `Producto creado: ${response.sku}. Stock inicial: ${response.available_quantity} unidades.${locationMessage}` });
        setSku(""); setProductName(""); setPurchasePrice(""); setListPrice(""); setMinimumSalePrice(""); setStock(""); setLowStockThreshold(""); setAttributes({}); setPrimaryLocationId("");
      }
      else if (response.code === "invalid_attribute_value") {
        const fieldError = response.field_error;
        if (fieldError && selected!.fields.some((field) => field.definition_id === fieldError.definition_id)) {
          const fieldId = `attribute-${fieldError.definition_id}`;
          setFieldError(fieldId);
          focus(fieldId);
          const message = fieldError.reason === "invalid_number" ? "Ingresá un número válido para este campo." : fieldError.reason === "invalid_option" ? "Seleccioná una de las opciones disponibles." : "Revisá el valor de este campo.";
          setFieldErrorMessage(message);
          dispatch({ type: "product_failed", requestId: id, message });
        } else dispatch({ type: "product_failed", requestId: id, message: "No se pudo validar un valor de atributo. Revisá los campos de categoría y corregí cualquier valor que no corresponda a su tipo u opciones." });
      } else dispatch({ type: "product_failed", requestId: id, message: response.message });
    } catch { if (mounted.current && id === mutation.current) dispatch({ type: "product_failed", requestId: id }); }
    finally { productLock.current = false; }
  };
  const pending = state.categoryStatus === "pending" || state.productStatus === "pending";
  const feedback = state.feedback && (state.categoryStatus === "error" || state.productStatus === "error" || state.categoryStatus === "success" || state.productStatus === "success") ? h(Feedback, { kind: state.categoryStatus === "error" || state.productStatus === "error" ? "error" : "success" } as never, state.feedback) : null;
  const fieldControl = (field: Category["fields"][number]) => field.field_type === FIELD_TYPE.OPTION ? h("select", { id: `attribute-${field.definition_id}`, "aria-required": field.required || undefined, value: attributes[field.definition_id] ?? "", disabled: pending, onChange: (event: ChangeEvent<HTMLSelectElement>) => setAttributes({ ...attributes, [field.definition_id]: event.target.value }) }, h("option", { value: "" }, "Seleccioná"), field.options.map((option) => h("option", { key: option, value: option }, option))) : h("input", { id: `attribute-${field.definition_id}`, "aria-required": field.required || undefined, type: field.field_type === FIELD_TYPE.NUMBER ? "number" : "text", step: field.field_type === FIELD_TYPE.NUMBER ? "any" : undefined, value: attributes[field.definition_id] ?? "", disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setAttributes({ ...attributes, [field.definition_id]: event.target.value }) });

  return h("main", { "aria-labelledby": "onboarding-heading", "data-ui-onboarding": true },
    h("h1", { id: "onboarding-heading" }, "Alta de productos"), h(Action, { variant: "tertiary", onClick: onBack }, "Volver a ventas"), feedback,
    h("div", { "data-ui-onboarding-layout": true },
      h(Panel, { label: "Crear categoría" } as never, h("form", { onSubmit: submitCategory, "aria-busy": state.categoryStatus === "pending" || undefined },
        h(Field, { kind: "text", label: "Nombre de la categoría", error: fieldError === "category" ? "Ingresá un nombre para la categoría." : undefined, control: h("input", { id: "category-name", value: categoryName, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setCategoryName(event.target.value) }) } as never),
        h("fieldset", null, h("legend", null, "Agregar campo"), h(Field, { kind: "text", label: "Nombre del campo", control: h("input", { id: "field-label", value: pendingField.label, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setPendingField({ ...pendingField, label: event.target.value }) }) } as never), h(Field, { kind: "select", label: "Tipo de campo", control: h("select", { id: "field-type", value: pendingField.field_type, disabled: pending, onChange: (event: ChangeEvent<HTMLSelectElement>) => setPendingField({ ...pendingField, field_type: event.target.value as FieldType }) }, h("option", { value: FIELD_TYPE.TEXT }, "Texto"), h("option", { value: FIELD_TYPE.NUMBER }, "Número"), h("option", { value: FIELD_TYPE.OPTION }, "Opción")) } as never), pendingField.field_type === FIELD_TYPE.OPTION ? h(Field, { kind: "text", label: "Opciones separadas por coma", control: h("input", { id: "field-options", value: pendingField.optionsText, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setPendingField({ ...pendingField, optionsText: event.target.value }) }) } as never) : null, h("label", null, h("input", { type: "checkbox", checked: pendingField.required, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setPendingField({ ...pendingField, required: event.target.checked }) }), " Campo obligatorio"), h(Action, { variant: "secondary", type: "button", disabled: pending || !pendingField.label.trim(), onClick: addField }, "Agregar campo")),
        categoryFields.length ? h("ul", { "aria-label": "Campos pendientes" }, categoryFields.map((field, index) => h("li", { key: `${field.label}-${index}` }, `${field.label} · ${field.required ? "obligatorio" : "opcional"}`))) : null,
        h(Action, { variant: "primary", type: "submit", pending: state.categoryStatus === "pending", pendingLabel: "Creando categoría…" }, "Crear categoría"))),
      state.categoriesStatus === "loading" ? h(Feedback, { kind: "loading" } as never, "Cargando categorías…") : state.categoriesStatus === "empty" ? h(Feedback, { kind: "empty" } as never, "Creá una categoría para habilitar el alta de productos.") : state.categoriesStatus === "error" ? h(Feedback, { kind: "error" } as never, h("span", null, "No se pudieron cargar las categorías. ", h(Action, { variant: "tertiary", onClick: loadCategories }, "Reintentar"))) : h(Panel, { label: "Crear producto activo" } as never, h("form", { onSubmit: submitProduct, noValidate: true, "aria-busy": state.productStatus === "pending" || undefined },
        h(Field, { kind: "select", label: "Categoría", error: fieldError === "category" ? "Seleccioná una categoría." : undefined, control: h("select", { id: "product-category", value: selectedId, disabled: pending, onChange: (event: ChangeEvent<HTMLSelectElement>) => { setSelectedId(event.target.value); setAttributes({}); } }, state.categories.map((category) => h("option", { key: category.category_id, value: category.category_id }, category.name))) } as never),
        h(Field, { kind: "sku", label: "SKU", error: fieldError === "sku" ? "Ingresá el SKU." : undefined, control: h("input", { id: "product-sku", value: sku, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setSku(event.target.value) }) } as never),
        h(Field, { kind: "text", label: "Nombre del producto", error: fieldError === "name" ? "Ingresá el nombre del producto." : undefined, control: h("input", { id: "product-name", value: productName, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setProductName(event.target.value) }) } as never),
        h(Field, { kind: "money", label: "Precio de compra (Bs)", hint: "Obligatorio; usá coma decimal. No se asigna un costo predeterminado.", error: fieldError === "purchase-price" ? "Ingresá un precio de compra positivo y válido en Bs." : undefined, control: h("input", { id: "purchase-price", value: purchasePrice, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setPurchasePrice(event.target.value), "aria-required": true }) } as never),
        h(Field, { kind: "money", label: "Precio de venta (Bs)", hint: "Usá coma decimal; se envían centavos enteros.", error: fieldError === "list-price" ? "Ingresá un precio de venta válido en Bs." : undefined, control: h("input", { id: "list-price", value: listPrice, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setListPrice(event.target.value) }) } as never),
            h(Field, { kind: "money", label: "Precio mínimo de venta (Bs)", hint: "No puede superar el precio de venta.", error: fieldError === "minimum-price" ? "Ingresá un precio mínimo válido y menor o igual al precio de venta." : undefined, control: h("input", { id: "minimum-sale-price", value: minimumSalePrice, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setMinimumSalePrice(event.target.value) }) } as never),
        h(Field, { kind: "quantity", label: "Stock inicial (unidades enteras)", error: fieldError === "stock" ? "Ingresá una cantidad entera mayor que cero." : undefined, control: h("input", { id: "opening-stock", value: stock, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setStock(event.target.value) }) } as never),
        h(Field, { kind: "quantity", label: "Umbral de stock bajo (opcional)", hint: "Dejalo vacío para usar 1 unidad.", error: fieldError === "low-stock-threshold" ? "Ingresá un umbral entero mayor o igual a 1." : undefined, control: h("input", { id: "low-stock-threshold", min: 1, value: lowStockThreshold, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setLowStockThreshold(event.target.value) }) } as never),
        h(LocationPicker, { id: "product-primary-location", label: "Ubicación principal (opcional)", locations: productLocations, segments: locationSegments, selectedId: primaryLocationId, disabled: pending, status: locationsStatus, onChange: setPrimaryLocationId }),
        locationsStatus === "error" ? h("p", null, "Podés asignar la ubicación después desde Catálogo.") : null,
        selected?.fields.map((field) => h(Field, { key: field.definition_id, kind: field.field_type === FIELD_TYPE.OPTION ? "select" : "text", label: field.label, hint: field.required ? "Campo obligatorio." : "Campo opcional.", error: fieldError === `attribute-${field.definition_id}` ? fieldErrorMessage || validateCategoryAttributes(selected!, attributes)?.message : undefined, control: fieldControl(field) } as never)),
        h(Action, { variant: "primary", type: "submit", pending: state.productStatus === "pending", pendingLabel: "Creando producto…" }, "Crear producto"))))
  );
}
