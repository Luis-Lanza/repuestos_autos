import assert from "node:assert/strict";
import test from "node:test";
import { createOnboardingFlow, initialOnboardingState, canSubmitCategory, canSubmitProduct } from "./onboarding-flow.ts";
const category = { category_id: 1, name: "Filtros", fields: [] };

test("keeps the latest category load and exposes empty state", () => {
  let state = createOnboardingFlow(initialOnboardingState, { type: "categories_started", requestId: 1 });
  state = createOnboardingFlow(state, { type: "categories_started", requestId: 2 });
  state = createOnboardingFlow(state, { type: "categories_succeeded", requestId: 1, categories: [category] });
  assert.equal(state.categoriesStatus, "loading");
  state = createOnboardingFlow(state, { type: "categories_succeeded", requestId: 2, categories: [] }); assert.equal(state.categoriesStatus, "empty");
});
test("locks duplicate category and product activation synchronously", () => {
  assert.equal(canSubmitCategory({ ...initialOnboardingState, categoryStatus: "pending" }), false);
  assert.equal(canSubmitProduct({ ...initialOnboardingState, productStatus: "pending" }), false);
});
test("ignores obsolete mutation completions", () => {
  let state = createOnboardingFlow(initialOnboardingState, { type: "product_started", requestId: 1 });
  state = createOnboardingFlow(state, { type: "product_started", requestId: 2 });
  state = createOnboardingFlow(state, { type: "product_succeeded", requestId: 1, message: "viejo" });
  assert.equal(state.productStatus, "pending");
  state = createOnboardingFlow(state, { type: "product_succeeded", requestId: 2, message: "Producto creado." }); assert.equal(state.feedback, "Producto creado.");
});
test("replaces stale category schemas and gives actionable product recovery feedback", () => {
  let state = createOnboardingFlow(initialOnboardingState, { type: "product_started", requestId: 8 });
  const refreshed = { category_id: 1, name: "Filtros", fields: [{ definition_id: 12, label: "Modelo", field_type: "text" as const, required: true, options: [] }] };
  state = createOnboardingFlow(state, { type: "category_schema_changed", requestId: 8, categories: [refreshed] });
  assert.deepEqual(state.categories, [refreshed]);
  assert.equal(state.productStatus, "error");
  assert.match(state.feedback ?? "", /campos de esta categoría cambiaron.*volvé a intentar/);
});

test("keeps the safe field-specific backend validation reason in product feedback", () => {
  let state = createOnboardingFlow(initialOnboardingState, { type: "product_started", requestId: 4 });
  state = createOnboardingFlow(state, { type: "product_failed", requestId: 4, message: "Seleccioná una de las opciones disponibles." });
  assert.equal(state.productStatus, "error");
  assert.equal(state.feedback, "Seleccioná una de las opciones disponibles.");
});

test("maps category failures to localized recovery feedback and preserves a specific product reason", () => {
  let state = createOnboardingFlow(initialOnboardingState, { type: "category_started", requestId: 1 });
  state = createOnboardingFlow(state, { type: "category_failed", requestId: 1 }); assert.equal(state.feedback, "No se pudo crear la categoría.");
  state = createOnboardingFlow(state, { type: "product_started", requestId: 2 }); state = createOnboardingFlow(state, { type: "product_failed", requestId: 2, message: "El SKU ya existe." });
  assert.equal(state.productStatus, "error");
  assert.equal(state.feedback, "El SKU ya existe.");
});
