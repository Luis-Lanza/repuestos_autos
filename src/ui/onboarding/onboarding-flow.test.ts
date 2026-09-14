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
test("maps category and product failures to localized recovery feedback", () => {
  let state = createOnboardingFlow(initialOnboardingState, { type: "category_started", requestId: 1 });
  state = createOnboardingFlow(state, { type: "category_failed", requestId: 1 }); assert.equal(state.feedback, "No se pudo crear la categoría.");
  state = createOnboardingFlow(state, { type: "product_started", requestId: 2 }); state = createOnboardingFlow(state, { type: "product_failed", requestId: 2 });
  assert.equal(state.feedback, "No se pudo crear el producto.");
});
