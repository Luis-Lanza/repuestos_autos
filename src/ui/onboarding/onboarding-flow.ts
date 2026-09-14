import type { Category } from "../../commands/onboarding.ts";

export type OnboardingStatus = "idle" | "pending" | "success" | "error";
export type CategoriesStatus = "loading" | "ready" | "empty" | "error";
export interface OnboardingState {
  categories: Category[]; categoriesStatus: CategoriesStatus; categoryStatus: OnboardingStatus; productStatus: OnboardingStatus;
  categoryRequestId: number; categoryMutationId: number; productMutationId: number; feedback: string | null;
}
export const initialOnboardingState: OnboardingState = {
  categories: [], categoriesStatus: "loading", categoryStatus: "idle", productStatus: "idle",
  categoryRequestId: 0, categoryMutationId: 0, productMutationId: 0, feedback: null,
};
type Action =
  | { type: "categories_started"; requestId: number } | { type: "categories_succeeded"; requestId: number; categories: Category[] }
  | { type: "categories_failed"; requestId: number } | { type: "category_started"; requestId: number }
  | { type: "category_succeeded"; requestId: number; category: Category } | { type: "category_failed"; requestId: number }
  | { type: "product_started"; requestId: number } | { type: "product_succeeded"; requestId: number; message: string }
  | { type: "product_failed"; requestId: number };
export function createOnboardingFlow(state: OnboardingState, action: Action): OnboardingState {
  switch (action.type) {
    case "categories_started": return { ...state, categoriesStatus: "loading", categoryRequestId: action.requestId, feedback: null };
    case "categories_succeeded": return action.requestId === state.categoryRequestId ? { ...state, categories: action.categories, categoriesStatus: action.categories.length ? "ready" : "empty" } : state;
    case "categories_failed": return action.requestId === state.categoryRequestId ? { ...state, categoriesStatus: "error", feedback: "No se pudieron cargar las categorías." } : state;
    case "category_started": return { ...state, categoryStatus: "pending", categoryMutationId: action.requestId, feedback: null };
    case "category_succeeded": return action.requestId === state.categoryMutationId ? { ...state, categories: [...state.categories, action.category], categoriesStatus: "ready", categoryStatus: "success", feedback: `Categoría creada: ${action.category.name}.` } : state;
    case "category_failed": return action.requestId === state.categoryMutationId ? { ...state, categoryStatus: "error", feedback: "No se pudo crear la categoría." } : state;
    case "product_started": return { ...state, productStatus: "pending", productMutationId: action.requestId, feedback: null };
    case "product_succeeded": return action.requestId === state.productMutationId ? { ...state, productStatus: "success", feedback: action.message } : state;
    case "product_failed": return action.requestId === state.productMutationId ? { ...state, productStatus: "error", feedback: "No se pudo crear el producto." } : state;
  }
}
export const canSubmitCategory = (state: OnboardingState) => state.categoryStatus !== "pending";
export const canSubmitProduct = (state: OnboardingState) => state.productStatus !== "pending";
