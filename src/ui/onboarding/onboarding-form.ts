import type { AttributeValueInput, Category } from "../../commands/onboarding.ts";

export function parseBsToCentavos(value: string): number | null {
  const match = /^(\d+)(?:[.,](\d{1,2}))?$/.exec(value.trim());
  if (!match) return null;
  const whole = Number(match[1]);
  const cents = Number((match[2] ?? "").padEnd(2, "0") || 0);
  const centavos = whole * 100 + cents;
  return Number.isSafeInteger(whole) && whole > 0 && Number.isSafeInteger(centavos) ? centavos : null;
}

export function parsePositiveWhole(value: string): number | null {
  if (!/^\d+$/.test(value.trim())) return null;
  const quantity = Number(value);
  return Number.isSafeInteger(quantity) && quantity > 0 ? quantity : null;
}

export interface AttributeValidationError { definitionId: number; message: string }

export function validateCategoryAttributes(category: Category, values: Readonly<Record<number, string>>): AttributeValidationError | null {
  for (const field of category.fields) {
    const value = values[field.definition_id] ?? "";
    if (field.required && !value.trim()) return { definitionId: field.definition_id, message: "Completá este campo." };
    if (!value.trim()) continue;
    if (field.field_type === "number" && (!/^[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?$/.test(value.trim()) || !Number.isFinite(Number(value)))) return { definitionId: field.definition_id, message: "Ingresá un número válido y finito." };
    if (field.field_type === "option" && !field.options.includes(value)) return { definitionId: field.definition_id, message: "Seleccioná una opción válida." };
  }
  return null;
}

export function attributeValuesFor(category: Category, values: Readonly<Record<number, string>>): AttributeValueInput[] {
  return category.fields.filter((field) => field.required || (values[field.definition_id] ?? "") !== "").map((field) => ({
    definition_id: field.definition_id, value: values[field.definition_id] ?? "",
  }));
}
