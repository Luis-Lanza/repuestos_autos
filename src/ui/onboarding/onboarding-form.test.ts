import assert from "node:assert/strict";
import test from "node:test";
import type { Category } from "../../commands/onboarding.ts";
import { attributeValuesFor, parseBsToCentavos, parsePositiveWhole, validateCategoryAttributes } from "./onboarding-form.ts";

const category: Category = { category_id: 1, name: "Filtros", fields: [
  { definition_id: 10, label: "Marca", field_type: "text", required: true, options: [] },
  { definition_id: 11, label: "Material", field_type: "option", required: false, options: ["Goma"] },
] };

test("preserves required blank attributes and omits optional blank attributes", () => {
  assert.deepEqual(attributeValuesFor(category, { 10: "", 11: "" }), [{ definition_id: 10, value: "" }]);
});
test("validates required attributes, finite numbers, and exact option membership", () => {
  const typedCategory: Category = { ...category, fields: [
    ...category.fields,
    { definition_id: 12, label: "Cantidad", field_type: "number", required: false, options: [] },
  ] };
  assert.deepEqual(validateCategoryAttributes(typedCategory, { 10: "  " }), { definitionId: 10, message: "Completá este campo." });
  assert.deepEqual(validateCategoryAttributes(typedCategory, { 10: "ACDelco", 11: "Metal" }), { definitionId: 11, message: "Seleccioná una opción válida." });
  assert.deepEqual(validateCategoryAttributes(typedCategory, { 10: "ACDelco", 11: "Goma", 12: "Infinity" }), { definitionId: 12, message: "Ingresá un número válido y finito." });
  assert.deepEqual(validateCategoryAttributes(typedCategory, { 10: "ACDelco", 11: "Goma", 12: "12.5" }), null);
});
test("parses comma and dot decimals into integer centavos", () => {
  assert.equal(parseBsToCentavos("125,50"), 12550); assert.equal(parseBsToCentavos("125.5"), 12550);
});
test("rejects blank, zero, invalid precision, and non-numeric money", () => {
  for (const value of ["", " ", "0", "0,00", "125,999", "12.3.4", "Bs 12"]) assert.equal(parseBsToCentavos(value), null, value);
});
test("accepts only positive safe whole-unit stock", () => {
  assert.equal(parsePositiveWhole("8"), 8); for (const value of ["", "0", "  ", "1.5", "-1", "2,0"]) assert.equal(parsePositiveWhole(value), null, value);
});
