import type {
  AttributeValueInput,
  Category,
} from "../../commands/onboarding.ts";

export function attributeValuesFor(
  category: Category,
  values: Readonly<Record<number, string>>,
): AttributeValueInput[] {
  return category.fields
    .filter((field) => field.required || (values[field.definition_id] ?? "") !== "")
    .map((field) => ({
      definition_id: field.definition_id,
      value: values[field.definition_id] ?? "",
    }));
}
