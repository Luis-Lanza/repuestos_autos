import { createElement as h, useEffect, useReducer, useRef, useState, type ChangeEvent, type FormEvent } from "react";
import { createCategory, createProduct, FIELD_TYPE, listCategories, type Category, type CategoryFieldInput, type FieldType } from "../../commands/onboarding.ts";
import { Action, Feedback, Field } from "../visual-system/controls.ts";
import { Panel } from "../visual-system/structure.ts";
import { attributeValuesFor, parseBsToCentavos, parsePositiveWhole } from "./onboarding-form.ts";
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
  const [attributes, setAttributes] = useState<Record<number, string>>({});
  const [fieldError, setFieldError] = useState("");
  const mounted = useRef(true), request = useRef(0), mutation = useRef(0), categoryLock = useRef(false), productLock = useRef(false);
  const selected = state.categories.find((category) => category.category_id === Number(selectedId));
  const focus = (id: string) => document.getElementById(id)?.focus();

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
  useEffect(() => { void loadCategories(); return () => { mounted.current = false; request.current++; mutation.current++; }; }, []);

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
    const productErrors = !selected ? "category" : !sku.trim() ? "sku" : !productName.trim() ? "name" : parseBsToCentavos(purchasePrice) === null ? "purchase-price" : parseBsToCentavos(listPrice) === null ? "list-price" : parseBsToCentavos(minimumSalePrice) === null ? "minimum-price" : parseBsToCentavos(minimumSalePrice)! > parseBsToCentavos(listPrice)! ? "minimum-price" : parsePositiveWhole(stock) === null ? "stock" : (selected.fields.find((field) => field.required && !(attributes[field.definition_id] ?? "").trim()) ? "attribute" : "");
    if (productErrors) { setFieldError(productErrors); focus(productErrors === "category" ? "product-category" : productErrors === "sku" ? "product-sku" : productErrors === "name" ? "product-name" : productErrors === "purchase-price" ? "purchase-price" : productErrors === "list-price" ? "list-price" : productErrors === "minimum-price" ? "minimum-sale-price" : productErrors === "stock" ? "opening-stock" : `attribute-${selected?.fields.find((field) => field.required && !(attributes[field.definition_id] ?? "").trim())?.definition_id}`); return; }
    const id = ++mutation.current; productLock.current = true; setFieldError(""); dispatch({ type: "product_started", requestId: id });
    try {
      const response = await createProduct({ sku: sku.trim(), name: productName.trim(), category_id: selected!.category_id, purchase_price_centavos: parseBsToCentavos(purchasePrice)!, sale_price_centavos: parseBsToCentavos(listPrice)!, minimum_sale_price_centavos: parseBsToCentavos(minimumSalePrice)!, opening_quantity: parsePositiveWhole(stock)!, attribute_values: attributeValuesFor(selected!, attributes) });
      if (!mounted.current || id !== mutation.current) return;
      if (response.kind === "success") { dispatch({ type: "product_succeeded", requestId: id, message: `Producto creado: ${response.sku}. Stock inicial: ${response.available_quantity} unidades.` }); setSku(""); setProductName(""); setPurchasePrice(""); setListPrice(""); setMinimumSalePrice(""); setStock(""); setAttributes({}); }
      else dispatch({ type: "product_failed", requestId: id });
    } catch { if (mounted.current && id === mutation.current) dispatch({ type: "product_failed", requestId: id }); }
    finally { productLock.current = false; }
  };
  const pending = state.categoryStatus === "pending" || state.productStatus === "pending";
  const feedback = state.feedback && (state.categoryStatus === "error" || state.productStatus === "error" || state.categoryStatus === "success" || state.productStatus === "success") ? h(Feedback, { kind: state.categoryStatus === "error" || state.productStatus === "error" ? "error" : "success" } as never, state.feedback) : null;
  const fieldControl = (field: Category["fields"][number]) => field.field_type === FIELD_TYPE.OPTION ? h("select", { id: `attribute-${field.definition_id}`, "aria-required": field.required || undefined, value: attributes[field.definition_id] ?? "", disabled: pending, onChange: (event: ChangeEvent<HTMLSelectElement>) => setAttributes({ ...attributes, [field.definition_id]: event.target.value }) }, h("option", { value: "" }, "Seleccioná"), field.options.map((option) => h("option", { key: option, value: option }, option))) : h("input", { id: `attribute-${field.definition_id}`, "aria-required": field.required || undefined, type: field.field_type === FIELD_TYPE.NUMBER ? "number" : "text", value: attributes[field.definition_id] ?? "", disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setAttributes({ ...attributes, [field.definition_id]: event.target.value }) });

  return h("main", { "aria-labelledby": "onboarding-heading", "data-ui-onboarding": true },
    h("h1", { id: "onboarding-heading" }, "Alta de productos"), h(Action, { variant: "tertiary", onClick: onBack }, "Volver a ventas"), feedback,
    h("div", { "data-ui-onboarding-layout": true },
      h(Panel, { label: "Crear categoría" } as never, h("form", { onSubmit: submitCategory, "aria-busy": state.categoryStatus === "pending" || undefined },
        h(Field, { kind: "text", label: "Nombre de la categoría", error: fieldError === "category" ? "Ingresá un nombre para la categoría." : undefined, control: h("input", { id: "category-name", value: categoryName, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setCategoryName(event.target.value) }) } as never),
        h("fieldset", null, h("legend", null, "Agregar campo"), h(Field, { kind: "text", label: "Nombre del campo", control: h("input", { id: "field-label", value: pendingField.label, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setPendingField({ ...pendingField, label: event.target.value }) }) } as never), h(Field, { kind: "select", label: "Tipo de campo", control: h("select", { id: "field-type", value: pendingField.field_type, disabled: pending, onChange: (event: ChangeEvent<HTMLSelectElement>) => setPendingField({ ...pendingField, field_type: event.target.value as FieldType }) }, h("option", { value: FIELD_TYPE.TEXT }, "Texto"), h("option", { value: FIELD_TYPE.NUMBER }, "Número"), h("option", { value: FIELD_TYPE.OPTION }, "Opción")) } as never), pendingField.field_type === FIELD_TYPE.OPTION ? h(Field, { kind: "text", label: "Opciones separadas por coma", control: h("input", { id: "field-options", value: pendingField.optionsText, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setPendingField({ ...pendingField, optionsText: event.target.value }) }) } as never) : null, h("label", null, h("input", { type: "checkbox", checked: pendingField.required, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setPendingField({ ...pendingField, required: event.target.checked }) }), " Campo obligatorio"), h(Action, { variant: "secondary", type: "button", disabled: pending || !pendingField.label.trim(), onClick: addField }, "Agregar campo")),
        categoryFields.length ? h("ul", { "aria-label": "Campos pendientes" }, categoryFields.map((field, index) => h("li", { key: `${field.label}-${index}` }, `${field.label} · ${field.required ? "obligatorio" : "opcional"}`))) : null,
        h(Action, { variant: "primary", type: "submit", pending: state.categoryStatus === "pending", pendingLabel: "Creando categoría…" }, "Crear categoría"))),
      state.categoriesStatus === "loading" ? h(Feedback, { kind: "loading" } as never, "Cargando categorías…") : state.categoriesStatus === "empty" ? h(Feedback, { kind: "empty" } as never, "Creá una categoría para habilitar el alta de productos.") : state.categoriesStatus === "error" ? h(Feedback, { kind: "error" } as never, h("span", null, "No se pudieron cargar las categorías. ", h(Action, { variant: "tertiary", onClick: loadCategories }, "Reintentar"))) : h(Panel, { label: "Crear producto activo" } as never, h("form", { onSubmit: submitProduct, "aria-busy": state.productStatus === "pending" || undefined },
        h(Field, { kind: "select", label: "Categoría", error: fieldError === "category" ? "Seleccioná una categoría." : undefined, control: h("select", { id: "product-category", value: selectedId, disabled: pending, onChange: (event: ChangeEvent<HTMLSelectElement>) => { setSelectedId(event.target.value); setAttributes({}); } }, state.categories.map((category) => h("option", { key: category.category_id, value: category.category_id }, category.name))) } as never),
        h(Field, { kind: "sku", label: "SKU", error: fieldError === "sku" ? "Ingresá el SKU." : undefined, control: h("input", { id: "product-sku", value: sku, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setSku(event.target.value) }) } as never),
        h(Field, { kind: "text", label: "Nombre del producto", error: fieldError === "name" ? "Ingresá el nombre del producto." : undefined, control: h("input", { id: "product-name", value: productName, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setProductName(event.target.value) }) } as never),
        h(Field, { kind: "money", label: "Precio de compra (Bs)", hint: "Obligatorio; usá coma decimal. No se asigna un costo predeterminado.", error: fieldError === "purchase-price" ? "Ingresá un precio de compra positivo y válido en Bs." : undefined, control: h("input", { id: "purchase-price", value: purchasePrice, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setPurchasePrice(event.target.value), "aria-required": true }) } as never),
        h(Field, { kind: "money", label: "Precio de lista (Bs)", hint: "Usá coma decimal; se envían centavos enteros.", error: fieldError === "list-price" ? "Ingresá un precio de lista válido en Bs." : undefined, control: h("input", { id: "list-price", value: listPrice, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setListPrice(event.target.value) }) } as never),
            h(Field, { kind: "money", label: "Precio mínimo de venta (Bs)", hint: "No puede superar el precio de lista.", error: fieldError === "minimum-price" ? "Ingresá un precio mínimo válido y menor o igual al precio de lista." : undefined, control: h("input", { id: "minimum-sale-price", value: minimumSalePrice, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setMinimumSalePrice(event.target.value) }) } as never),
        h(Field, { kind: "quantity", label: "Stock inicial (unidades enteras)", error: fieldError === "stock" ? "Ingresá una cantidad entera mayor que cero." : undefined, control: h("input", { id: "opening-stock", value: stock, disabled: pending, onChange: (event: ChangeEvent<HTMLInputElement>) => setStock(event.target.value) }) } as never),
        selected?.fields.map((field) => h(Field, { key: field.definition_id, kind: field.field_type === FIELD_TYPE.OPTION ? "select" : "text", label: field.label, hint: field.required ? "Campo obligatorio." : "Campo opcional.", error: fieldError === "attribute" && field.required && !(attributes[field.definition_id] ?? "").trim() ? "Completá este campo." : undefined, control: fieldControl(field) } as never)),
        h(Action, { variant: "primary", type: "submit", pending: state.productStatus === "pending", pendingLabel: "Creando producto…" }, "Crear producto"))))
  );
}
