import { createElement, type FormEvent } from "react";

import type { ProductActivityState, ProductBrowsePage, ProductBrowseResult, ProductStockState } from "../../commands/catalog.ts";
import { Action, Badge, Feedback, Field } from "../visual-system/controls.ts";

export type CatalogViewMode = "table" | "gallery";
const CATALOG_VIEW_MODE_KEY = "catalog.product-browser.view-mode";
export function readCatalogViewMode(storage?: Pick<Storage, "getItem">): CatalogViewMode {
  try { return (storage ?? globalThis.localStorage).getItem(CATALOG_VIEW_MODE_KEY) === "gallery" ? "gallery" : "table"; }
  catch { return "table"; }
}
export function writeCatalogViewMode(mode: CatalogViewMode, storage?: Pick<Storage, "setItem">): void {
  try { (storage ?? globalThis.localStorage).setItem(CATALOG_VIEW_MODE_KEY, mode); } catch { /* Preferences must not block Catalog. */ }
}

export type ProductBrowserStatus = "initial" | "loading" | "results" | "empty" | "error";
export interface ProductBrowserState {
  status: ProductBrowserStatus;
  query: string;
  category_id: number | null;
  stock_state: ProductStockState;
  activity: ProductActivityState;
  page: number;
  request_id: number;
  result: ProductBrowsePage | null;
  error: string | null;
}
export const initialProductBrowserState: ProductBrowserState = {
  status: "initial", query: "", category_id: null, stock_state: "all", activity: "active", page: 1, request_id: 0, result: null, error: null,
};
export type ProductBrowserAction =
  | { type: "query_changed"; value: string }
  | { type: "category_changed"; value: number | null }
  | { type: "stock_state_changed"; value: ProductStockState }
  | { type: "activity_changed"; value: ProductActivityState }
  | { type: "browse_started"; query: string; category_id: number | null; stock_state: ProductStockState; activity: ProductActivityState; page: number; request_id: number }
  | { type: "browse_succeeded"; request_id: number; result: ProductBrowsePage }
  | { type: "browse_failed"; request_id: number; message: string };
export function createProductBrowserFlow(state: ProductBrowserState, action: ProductBrowserAction): ProductBrowserState {
  switch (action.type) {
    case "query_changed": return { ...state, query: action.value, page: 1, request_id: state.request_id + 1 };
    case "category_changed": return { ...state, category_id: action.value, page: 1, request_id: state.request_id + 1 };
    case "stock_state_changed": return { ...state, stock_state: action.value, page: 1, request_id: state.request_id + 1 };
    case "activity_changed": return { ...state, activity: action.value, page: 1, request_id: state.request_id + 1 };
    case "browse_started":
      if (action.request_id <= state.request_id) return state;
      return { ...state, status: "loading", query: action.query, category_id: action.category_id, stock_state: action.stock_state, activity: action.activity, page: action.page, request_id: action.request_id, error: null };
    case "browse_succeeded":
      if (action.request_id !== state.request_id) return state;
      return { ...state, status: action.result.products.length ? "results" : "empty", result: action.result, error: null };
    case "browse_failed":
      if (action.request_id !== state.request_id) return state;
      return { ...state, status: "error", result: null, error: action.message };
  }
}

function stockText(product: ProductBrowseResult) {
  return product.available_quantity === 0 ? "Sin stock: 0" : product.available_quantity === 1 ? "Stock bajo: 1" : `Disponible: ${product.available_quantity}`;
}
function priceText(centavos: number) { return `Bs ${Math.trunc(centavos / 100)},${String(centavos % 100).padStart(2, "0")}`; }
function stockKind(product: ProductBrowseResult) {
  return product.available_quantity === 0 ? "out-of-stock" : product.available_quantity === 1 ? "low-stock" : "available";
}
export interface ProductBrowserProps {
  state: ProductBrowserState;
  onQueryChange: (value: string) => void;
  onCategoryChange: (value: number | null) => void;
  onStockStateChange?: (value: ProductStockState) => void;
  onActivityChange?: (value: ProductActivityState) => void;
  showActivity?: boolean;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  onPageChange: (page: number) => void;
  onSelect?: (product: ProductBrowseResult) => void;
  actionLabel?: string;
  searchLabel?: string;
  initialMessage?: string;
  loadingMessage?: string;
  allowUnavailableSelection?: boolean;
  disabledProductIds?: ReadonlySet<number>;
  thumbnails?: Readonly<Record<number, string>>;
  disabled?: boolean;
  presentation?: "sales" | "catalog";
  catalogViewMode?: CatalogViewMode;
  onCatalogViewModeChange?: (mode: CatalogViewMode) => void;
}
export function ProductBrowser(props: ProductBrowserProps) {
  const { state } = props;
  const page = state.result;
  const feedback = state.status === "initial" ? createElement(Feedback, { kind: "initial" } as never, props.initialMessage ?? "Buscá un producto para comenzar.")
    : state.status === "loading" ? createElement(Feedback, { kind: "loading", "aria-label": props.loadingMessage ?? "Cargando productos…" } as never, props.loadingMessage ?? "Cargando productos…")
    : state.status === "error" ? createElement(Feedback, { kind: "error" } as never, state.error ?? "No se pudo cargar el catálogo.")
      : state.status === "empty" ? createElement(Feedback, { kind: "empty" } as never, `No encontramos productos para “${state.query}”.`) : null;
  const salesPresentation = props.presentation === "sales";
  const catalogPresentation = props.presentation === "catalog";
  const galleryPresentation = catalogPresentation && props.catalogViewMode === "gallery";
  return createElement("div", { "data-ui-product-browser": salesPresentation ? "sales" : catalogPresentation ? "catalog" : true },
    createElement("form", { onSubmit: props.onSubmit, "aria-busy": state.status === "loading", "data-ui-sale-search": true },
      createElement(Field, { kind: "search", label: props.searchLabel ?? "Buscar en el catálogo", control: createElement("input", { value: state.query, disabled: props.disabled, onChange: (event) => { if (!props.disabled) props.onQueryChange(event.target.value); } }) } as never),
      createElement(Field, { kind: "select", label: "Categoría", control: createElement("select", { value: state.category_id ?? "", disabled: props.disabled, onChange: (event) => { if (!props.disabled) props.onCategoryChange(event.target.value ? Number(event.target.value) : null); } }, createElement("option", { value: "" }, "Todas las categorías"), (page?.categories ?? []).map((category) => createElement("option", { key: category.category_id, value: category.category_id }, category.name))) } as never),
      props.showActivity && props.onActivityChange ? createElement(Field, { kind: "select", label: "Actividad", control: createElement("select", { value: state.activity, disabled: props.disabled, onChange: (event) => { if (!props.disabled) props.onActivityChange?.(event.target.value as ProductActivityState); } }, createElement("option", { value: "all" }, "Todas"), createElement("option", { value: "active" }, "Activos"), createElement("option", { value: "archived" }, "Archivados")) } as never) : null,
      props.onStockStateChange ? createElement(Field, { kind: "select", label: "Estado del stock", control: createElement("select", { value: state.stock_state, disabled: props.disabled, onChange: (event) => { if (!props.disabled) props.onStockStateChange?.(event.target.value as ProductStockState); } }, createElement("option", { value: "all" }, "Todo el stock"), createElement("option", { value: "available" }, "Disponible"), createElement("option", { value: "low_stock" }, "Stock bajo"), createElement("option", { value: "out_of_stock" }, "Sin stock"), createElement("option", { value: "alerts" }, "Alertas")) } as never) : null,
      catalogPresentation ? createElement("div", { "data-ui-catalog-search-actions": true },
        createElement("div", { role: "group", "aria-label": "Presentación del catálogo", "data-ui-catalog-view-toggle": true },
          createElement(Action, { variant: props.catalogViewMode === "gallery" ? "tertiary" : "secondary", type: "button", "aria-label": "Vista de tabla", "aria-pressed": props.catalogViewMode !== "gallery", "data-ui-catalog-view-toggle": true, disabled: props.disabled, onClick: () => props.onCatalogViewModeChange?.("table") }, "Tabla"),
          createElement(Action, { variant: props.catalogViewMode === "gallery" ? "secondary" : "tertiary", type: "button", "aria-label": "Vista de galería", "aria-pressed": props.catalogViewMode === "gallery", "data-ui-catalog-view-toggle": true, disabled: props.disabled, onClick: () => props.onCatalogViewModeChange?.("gallery") }, "Galería")),
        createElement(Action, { variant: "secondary", type: "submit", disabled: props.disabled }, "Buscar"))
        : createElement(Action, { variant: "secondary", type: "submit", disabled: props.disabled }, "Buscar")),
    feedback,
    salesPresentation && page && state.status === "results" ? createElement("div", { role: "status", "data-ui-sales-catalog-status": true },
      createElement("span", null, `Resultados del catálogo: ${page.total} ${page.total === 1 ? "repuesto" : "repuestos"}`),
      createElement("span", null, "Filtro: Stock activo")) : null,
    page && state.status !== "error" && page.products.length ? createElement("ul", { "aria-label": "Resultados del catálogo", "data-ui-product-browser-list": true, "data-ui-sale-list": true, "data-ui-catalog-gallery": galleryPresentation || undefined }, page.products.map((product) => {
      const selected = props.disabledProductIds?.has(product.product_id) ?? false;
      const actionText = salesPresentation && selected ? "Agregado" : props.actionLabel ?? "Seleccionar";
      const action = props.onSelect ? createElement(Action, {
        variant: "secondary",
        "aria-label": actionText === "Seleccionar" ? `Seleccionar ${product.name} (SKU: ${product.sku})` : undefined,
        disabled: props.disabled || selected || !props.allowUnavailableSelection && product.available_quantity < 1,
        onClick: () => props.onSelect?.(product),
      }, actionText) : null;
      const thumbnail = props.thumbnails?.[product.product_id];
      const safeThumbnail = thumbnail && thumbnail.length <= 2_796_227 && /^data:image\/jpeg;base64,(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(thumbnail) ? createElement("img", { src: thumbnail, alt: product.name, loading: "lazy", "data-ui-product-thumbnail": true }) : null;
      return galleryPresentation ? createElement("li", { key: product.product_id, "data-ui-catalog-product-card": true },
        safeThumbnail,
        createElement("div", { "data-ui-catalog-product-card-identity": true }, createElement("strong", null, product.name), createElement("span", { "data-ui-sku": true }, product.sku), createElement("span", null, product.category_name)),
        createElement("span", { "data-ui-money": true }, priceText(product.list_price_centavos)),
        createElement(Badge, { kind: stockKind(product), text: stockText(product) }),
        action) : salesPresentation ? createElement("li", { key: product.product_id, "data-ui-sales-product": true },
        createElement("div", { "data-ui-product-identity": true },
          createElement("strong", null, product.name),
          createElement("div", { "data-ui-product-facts": true },
            createElement("span", { "data-ui-sku": true }, product.sku),
            createElement("span", { "data-ui-product-category": true }, product.category_name),
            createElement(Badge, { kind: stockKind(product), text: stockText(product) }))),
        createElement("div", { "data-ui-product-action": true },
          createElement("span", { "data-ui-unit-price": true },
            createElement("span", { "data-ui-unit-price-caption": true }, "Precio unitario"),
            createElement("span", { "data-ui-money": true }, priceText(product.list_price_centavos))),
          action)) : createElement("li", { key: product.product_id },
            safeThumbnail,
            createElement("div", null, createElement("strong", null, product.name), createElement("span", { "data-ui-sku": true }, product.sku), createElement("span", null, product.category_name)),
            createElement("span", { "data-ui-money": true }, priceText(product.list_price_centavos)),
            createElement(Badge, { kind: stockKind(product), text: stockText(product) }),
            action);
    })) : null,
    page && page.total_pages > 1 ? createElement("nav", { "aria-label": "Páginas de productos", "data-ui-product-browser-pages": true }, createElement("span", null, `Página ${page.page} de ${page.total_pages}`), createElement(Action, { variant: "tertiary", disabled: props.disabled || page.page <= 1, onClick: () => props.onPageChange(page.page - 1) }, "Anterior"), createElement(Action, { variant: "tertiary", disabled: props.disabled || page.page >= page.total_pages, onClick: () => props.onPageChange(page.page + 1) }, "Siguiente")) : null);
}
