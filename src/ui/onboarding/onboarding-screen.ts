import {
  createElement,
  type ChangeEvent,
  type FormEvent,
  useEffect,
  useState,
} from "react";

import {
  createCategory,
  createProduct,
  FIELD_TYPE,
  listCategories,
  type Category,
  type CategoryFieldInput,
  type FieldType,
} from "../../commands/onboarding.ts";
import { attributeValuesFor } from "./onboarding-form.ts";

interface OnboardingScreenProps {
  onBack: () => void;
}

interface PendingField {
  label: string;
  field_type: FieldType;
  required: boolean;
  options: string;
}

const EMPTY_FIELD: PendingField = {
  label: "",
  field_type: FIELD_TYPE.TEXT,
  required: false,
  options: "",
};

function wholeNumber(value: string): number | null {
  const parsed = Number(value);
  return Number.isSafeInteger(parsed) ? parsed : null;
}

export function OnboardingScreen({ onBack }: OnboardingScreenProps) {
  const [categories, setCategories] = useState<Category[]>([]);
  const [selectedCategoryId, setSelectedCategoryId] = useState("");
  const [categoryName, setCategoryName] = useState("");
  const [pendingField, setPendingField] = useState<PendingField>(EMPTY_FIELD);
  const [categoryFields, setCategoryFields] = useState<CategoryFieldInput[]>([]);
  const [sku, setSku] = useState("");
  const [productName, setProductName] = useState("");
  const [catalogPrice, setCatalogPrice] = useState("");
  const [openingQuantity, setOpeningQuantity] = useState("");
  const [attributeValues, setAttributeValues] = useState<Record<number, string>>({});
  const [feedback, setFeedback] = useState<string | null>(null);

  useEffect(() => {
    listCategories()
      .then((response) => {
        if (response.kind === "error") {
          setFeedback(`${response.code}: ${response.message}`);
          return;
        }
        setCategories(response.categories);
        if (response.categories[0]) {
          setSelectedCategoryId(String(response.categories[0].category_id));
        }
      })
      .catch(() => setFeedback("Unable to load categories."));
  }, []);

  const selectedCategory = categories.find(
    (category) => category.category_id === Number(selectedCategoryId),
  );

  function addField() {
    const options =
      pendingField.field_type === FIELD_TYPE.OPTION
        ? pendingField.options
            .split(",")
            .map((option) => option.trim())
            .filter(Boolean)
        : [];
    setCategoryFields([
      ...categoryFields,
      {
        label: pendingField.label,
        field_type: pendingField.field_type,
        required: pendingField.required,
        options,
      },
    ]);
    setPendingField(EMPTY_FIELD);
  }

  async function submitCategory(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    try {
      const response = await createCategory({ name: categoryName, fields: categoryFields });
      if (response.kind === "error") {
        setFeedback(`${response.code}: ${response.message}`);
        return;
      }
      setCategories([...categories, response]);
      setSelectedCategoryId(String(response.category_id));
      setCategoryName("");
      setCategoryFields([]);
      setFeedback(`Category ${response.name} created.`);
    } catch {
      setFeedback("Unable to create category.");
    }
  }

  async function submitProduct(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const price = wholeNumber(catalogPrice);
    const quantity = wholeNumber(openingQuantity);
    if (!selectedCategory || price === null || quantity === null) {
      setFeedback("Select a category and enter whole-number price and stock values.");
      return;
    }
    try {
      const response = await createProduct({
        sku,
        name: productName,
        category_id: selectedCategory.category_id,
        catalog_unit_price_centavos: price,
        opening_quantity: quantity,
        attribute_values: attributeValuesFor(selectedCategory, attributeValues),
      });
      if (response.kind === "error") {
        setFeedback(`${response.code}: ${response.message}`);
        return;
      }
      setSku("");
      setProductName("");
      setCatalogPrice("");
      setOpeningQuantity("");
      setAttributeValues({});
      setFeedback(
        `${response.sku} created with ${response.available_quantity} units in stock.`,
      );
    } catch {
      setFeedback("Unable to create product.");
    }
  }

  return createElement(
    "main",
    { "aria-labelledby": "onboarding-heading" },
    createElement("h1", { id: "onboarding-heading" }, "Product onboarding"),
    createElement("button", { type: "button", onClick: onBack }, "Back to sales"),
    feedback ? createElement("p", { role: "status" }, feedback) : null,
    createElement("h2", null, "Create category"),
    createElement(
      "form",
      { onSubmit: submitCategory },
      createElement("label", { htmlFor: "category-name" }, "Category name"),
      createElement("input", {
        id: "category-name",
        required: true,
        value: categoryName,
        onChange: (event) => setCategoryName(event.target.value),
      }),
      createElement("fieldset", null,
        createElement("legend", null, "Add category field"),
        createElement("label", { htmlFor: "field-label" }, "Field label"),
        createElement("input", {
          id: "field-label",
          value: pendingField.label,
          onChange: (event) => setPendingField({ ...pendingField, label: event.target.value }),
        }),
        createElement("label", { htmlFor: "field-type" }, "Field type"),
        createElement("select", {
          id: "field-type",
          value: pendingField.field_type,
          onChange: (event: ChangeEvent<HTMLSelectElement>) => setPendingField({ ...pendingField, field_type: event.target.value as FieldType }),
        },
        createElement("option", { value: FIELD_TYPE.TEXT }, "Text"),
        createElement("option", { value: FIELD_TYPE.NUMBER }, "Number"),
        createElement("option", { value: FIELD_TYPE.OPTION }, "Predefined option")),
        pendingField.field_type === FIELD_TYPE.OPTION
          ? createElement("label", null, "Options (comma separated)", createElement("input", {
              value: pendingField.options,
              onChange: (event) => setPendingField({ ...pendingField, options: event.target.value }),
            }))
          : null,
        createElement("label", null, createElement("input", {
          type: "checkbox",
          checked: pendingField.required,
          onChange: (event) => setPendingField({ ...pendingField, required: event.target.checked }),
        }), " Required"),
        createElement("button", {
          type: "button",
          disabled: pendingField.label.trim() === "",
          onClick: addField,
        }, "Add field"),
      ),
      createElement("ul", { "aria-label": "Pending category fields" },
        categoryFields.map((field, index) => createElement("li", { key: `${field.label}-${index}` },
          `${field.label} · ${field.field_type} · ${field.required ? "required" : "optional"}`,
        )),
      ),
      createElement("button", { type: "submit" }, "Create category"),
    ),
    createElement("h2", null, "Create active product"),
    createElement(
      "form",
      { onSubmit: submitProduct },
      createElement("label", { htmlFor: "product-category" }, "Category"),
      createElement("select", {
        id: "product-category",
        required: true,
        value: selectedCategoryId,
        onChange: (event: ChangeEvent<HTMLSelectElement>) => {
          setSelectedCategoryId(event.target.value);
          setAttributeValues({});
        },
      },
      categories.map((category) => createElement("option", { key: category.category_id, value: category.category_id }, category.name))),
      createElement("label", { htmlFor: "product-sku" }, "SKU"),
      createElement("input", { id: "product-sku", required: true, value: sku, onChange: (event) => setSku(event.target.value) }),
      createElement("label", { htmlFor: "product-name" }, "Product name"),
      createElement("input", { id: "product-name", required: true, value: productName, onChange: (event) => setProductName(event.target.value) }),
      createElement("label", { htmlFor: "catalog-price" }, "Catalog price (centavos)"),
      createElement("input", { id: "catalog-price", type: "number", min: 1, step: 1, required: true, value: catalogPrice, onChange: (event) => setCatalogPrice(event.target.value) }),
      createElement("label", { htmlFor: "opening-stock" }, "Opening stock"),
      createElement("input", { id: "opening-stock", type: "number", min: 1, step: 1, required: true, value: openingQuantity, onChange: (event) => setOpeningQuantity(event.target.value) }),
      selectedCategory?.fields.map((field) => createElement("label", { key: field.definition_id },
        `${field.label}${field.required ? " (required)" : ""}`,
        field.field_type === FIELD_TYPE.OPTION
          ? createElement("select", {
              required: field.required,
              value: attributeValues[field.definition_id] ?? "",
              onChange: (event: ChangeEvent<HTMLSelectElement>) => setAttributeValues({ ...attributeValues, [field.definition_id]: event.target.value }),
            }, createElement("option", { value: "" }, "Select"), field.options.map((option) => createElement("option", { key: option, value: option }, option)))
          : createElement("input", {
              type: field.field_type === FIELD_TYPE.NUMBER ? "number" : "text",
              step: field.field_type === FIELD_TYPE.NUMBER ? "any" : undefined,
              required: field.required,
              value: attributeValues[field.definition_id] ?? "",
              onChange: (event) => setAttributeValues({ ...attributeValues, [field.definition_id]: event.target.value }),
            }),
      )),
      createElement("button", { type: "submit" }, "Create product"),
    ),
  );
}
