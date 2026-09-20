import { createElement, type ChangeEvent, type FormEvent, useEffect, useReducer, useRef, useState } from "react";

import { CATALOG_INTENT, browseProducts, catalogMaintenanceCommands, type CatalogMaintenanceRecord, type CatalogMetadataDetail } from "../../commands/catalog.ts";
import { Action, Feedback } from "../visual-system/controls.ts";
import { CatalogEditDialog, CatalogMetadataEditor } from "../visual-system/catalog-edit-dialog.ts";
import { Panel } from "../visual-system/structure.ts";
import { createCatalogEditRequest, createCatalogMaintenanceFlow, fieldErrorsForCatalogEdit, formForCatalogDetail, initialCatalogMaintenanceState, type CatalogEditFieldErrors, type CatalogEditForm, type CatalogMaintenanceAction } from "./catalog-maintenance-flow.ts";
import { createProductBrowserFlow, initialProductBrowserState, ProductBrowser, type ProductBrowserState } from "./product-browser.ts";

export { CatalogMetadataEditor } from "../visual-system/catalog-edit-dialog.ts";

type Dispatch = (action: CatalogMaintenanceAction) => void;
type SetForm = (form: CatalogEditForm) => void;
type CatalogLoadCommands = Pick<typeof catalogMaintenanceCommands, "detail" | "listCategories">;
type BrowserSnapshot = Pick<ProductBrowserState, "query" | "category_id" | "stock_state" | "activity" | "page" | "request_id">;

export function CatalogMaintenanceRecovery({ required, onReload }: { required: boolean; onReload: () => void }) {
  return required ? createElement(Action, { variant: "secondary", onClick: onReload }, "Recargar registros del catálogo") : null;
}
export function CatalogSuccessNotice({ notice }: { notice: string | null }) {
  return notice ? createElement(Feedback, { kind: "success" } as never, notice) : null;
}
export async function loadCatalogRecords(commands: CatalogLoadCommands, dispatch: Dispatch) {
  dispatch({ type: "load_started" });
  const response = await commands.listCategories();
  dispatch(response.kind === "success" ? { type: "loaded", records: response.records } : { type: "load_failed" });
}
export async function loadCatalogDetail(commands: CatalogLoadCommands, dispatch: Dispatch, setForm: SetForm, record: CatalogMaintenanceRecord) {
  dispatch({ type: "detail_started", record });
  const response = await commands.detail(record);
  if (response.kind === "success") { setForm(formForCatalogDetail(response.detail)); dispatch({ type: "detail_loaded", detail: response.detail }); }
  else dispatch({ type: "detail_failed", code: response.code });
}
export async function reloadCatalogRecords(commands: CatalogLoadCommands, dispatch: Dispatch, setForm: SetForm, selected: CatalogMaintenanceRecord | null) {
  await loadCatalogRecords(commands, dispatch);
  if (selected) await loadCatalogDetail(commands, dispatch, setForm, selected);
}

export function CatalogMaintenanceScreen() {
  const [state, dispatch] = useReducer(createCatalogMaintenanceFlow, initialCatalogMaintenanceState);
  const [form, setForm] = useState<CatalogEditForm | null>(null);
  const mounted = useRef(true);
  const attempt = useRef(0);
  const browseAttempt = useRef(0);
  const refreshDetailAfterRecovery = useRef(false);
  const [browser, browserDispatch] = useReducer(createProductBrowserFlow, { ...initialProductBrowserState, activity: "all" });
  const browserRef = useRef(browser);
  browserRef.current = browser;
  const mutationLocked = useRef(false);
  useEffect(() => () => { mounted.current = false; attempt.current += 1; browseAttempt.current += 1; mutationLocked.current = true; }, []);

  const browserSnapshot = (): BrowserSnapshot => {
    const current = browserRef.current;
    return { query: current.query, category_id: current.category_id, stock_state: current.stock_state, activity: current.activity, page: current.page, request_id: current.request_id };
  };
  const browseCatalogProducts = async (snapshot: BrowserSnapshot, page = 1) => {
    const current = Math.max(++browseAttempt.current, snapshot.request_id + 1);
    browseAttempt.current = current;
    browserDispatch({ type: "browse_started", query: snapshot.query, category_id: snapshot.category_id, stock_state: snapshot.stock_state, activity: snapshot.activity, page, request_id: current });
    try {
      const result = await browseProducts({ query: snapshot.query, category_id: snapshot.category_id, activity: snapshot.activity, page, page_size: 20 });
      if (!mounted.current || current !== browseAttempt.current) return false;
      browserDispatch({ type: "browse_succeeded", request_id: current, result });
      return true;
    } catch {
      if (!mounted.current || current !== browseAttempt.current) return false;
      browserDispatch({ type: "browse_failed", request_id: current, message: "La navegación paginada de productos no está disponible en esta versión." });
      return false;
    }
  };
  const load = async () => {
    const current = ++attempt.current;
    const initialBrowseSnapshot = browserSnapshot();
    dispatch({ type: "load_started" });
    const response = await catalogMaintenanceCommands.listCategories();
    if (mounted.current && current === attempt.current) {
      dispatch(response.kind === "success" ? { type: "loaded", records: response.records } : { type: "load_failed" });
      if (response.kind === "success" && browserRef.current.request_id === initialBrowseSnapshot.request_id) await browseCatalogProducts(initialBrowseSnapshot);
    }
  };
  const loadDetail = async (record: CatalogMaintenanceRecord) => {
    const current = ++attempt.current;
    setForm(null);
    dispatch({ type: "detail_started", record });
    const response = await catalogMaintenanceCommands.detail(record);
    if (!mounted.current || current !== attempt.current) return;
    if (response.kind === "success") { setForm(formForCatalogDetail(response.detail)); refreshDetailAfterRecovery.current = false; dispatch({ type: "detail_loaded", detail: response.detail }); }
    else dispatch({ type: "detail_failed", code: response.code });
  };
  const refreshCatalogList = async (keepRecoveryLocked = false) => {
    const current = ++attempt.current;
    dispatch({ type: "refresh_started" });
    const response = await catalogMaintenanceCommands.listCategories();
    if (!mounted.current || current !== attempt.current) return false;
    if (response.kind !== "success") { dispatch({ type: "refresh_failed" }); return false; }
    dispatch({ type: "refresh_list_succeeded", records: response.records });
    const browsed = await browseCatalogProducts(browserSnapshot());
    if (!browsed || !mounted.current || current !== attempt.current) {
      if (mounted.current && current === attempt.current) dispatch({ type: "refresh_failed" });
      return false;
    }
    dispatch({ type: "refresh_succeeded", records: response.records, keep_recovery_locked: keepRecoveryLocked });
    return true;
  };
  useEffect(() => { void load(); }, []);
  useEffect(() => {
    const first = Object.keys(state.field_errors)[0];
    if (first) {
      const id = first.startsWith("attribute-") ? `catalog-${first}` : first === "list_price_centavos" ? "catalog-edit-list-price" : first === "minimum_sale_price_centavos" ? "catalog-edit-minimum-price" : `catalog-edit-${first}`;
      document.getElementById(id)?.focus();
    }
  }, [state.field_errors]);

  const reload = async () => {
    const selected = state.selected;
    const selectionAttempt = attempt.current;
    await load();
    if (mounted.current && selected && attempt.current === selectionAttempt + 1) await loadDetail(selected);
  };
  const close = () => {
    attempt.current += 1;
    browseAttempt.current += 1;
    refreshDetailAfterRecovery.current = false;
    setForm(null);
    dispatch({ type: "selection_cleared" });
  };
  const retryRefresh = async () => {
    if (!state.selected) return;
    const selected = state.selected;
    const reloadDetail = refreshDetailAfterRecovery.current;
    const refreshed = await refreshCatalogList(reloadDetail);
    if (refreshed && reloadDetail && mounted.current) await loadDetail(selected);
  };
  const maintain = async () => {
    const detail = state.detail;
    if (!detail || mutationLocked.current || state.recovery_required) return;
    mutationLocked.current = true;
    dispatch({ type: "mutation_started" });
    const response = await catalogMaintenanceCommands.maintain({ target: detail.target, entity_id: detail.entity_id, intent: detail.activity === "active" ? CATALOG_INTENT.ARCHIVE : CATALOG_INTENT.REACTIVATE, expected_revision: detail.revision });
    if (!mounted.current) return;
    mutationLocked.current = false;
    if (response.kind === "error") {
      refreshDetailAfterRecovery.current = response.code === "stale_catalog_record";
      dispatch({ type: "mutation_failed", code: response.code });
      return;
    }
    dispatch({ type: "mutation_succeeded", record: response });
    refreshDetailAfterRecovery.current = false;
    await refreshCatalogList();
  };
  const edit = async () => {
    const detail = state.detail;
    if (!detail || !form || mutationLocked.current || state.recovery_required) return;
    const request = createCatalogEditRequest(detail, form);
    if (!request) { dispatch({ type: "edit_validation_failed", field_errors: fieldErrorsForCatalogEdit(detail, form) }); return; }
    mutationLocked.current = true;
    dispatch({ type: "edit_started" });
    const response = await catalogMaintenanceCommands.edit(request);
    if (!mounted.current) return;
    mutationLocked.current = false;
    if (response.kind === "error") {
      refreshDetailAfterRecovery.current = response.code === "stale_catalog_record";
      dispatch({ type: "edit_failed", code: response.code });
      return;
    }
    dispatch({ type: "edit_succeeded", record: response });
    refreshDetailAfterRecovery.current = true;
    const refreshed = await refreshCatalogList(true);
    if (refreshed && mounted.current) await loadDetail({ target: response.target, entity_id: response.entity_id, label: response.label, activity: response.activity, revision: response.revision });
  };
  const change = (field: string, value: string) => setForm((current) => !current ? current : field.startsWith("attribute-") ? { ...current, attribute_values: { ...current.attribute_values, [Number(field.slice(10))]: value } } : { ...current, [field]: value });
  const pending = state.status === "pending" || state.status === "loading" && !!state.selected;
  const interactionLocked = pending || state.recovery_required;
  const submitBrowse = (event: FormEvent) => {
    event.preventDefault();
    if (!interactionLocked) {
      const snapshot = browserSnapshot();
      void browseCatalogProducts(snapshot, snapshot.page ?? 1);
    }
  };
  const visibleRecords = state.records;

  return createElement(
    "main",
    { "aria-labelledby": "catalog-maintenance-heading", "data-ui-catalog": true },
    createElement("h1", { id: "catalog-maintenance-heading" }, "Catálogo"),
    createElement("p", null, "Editá metadatos desde el detalle de categorías y productos."),
    createElement(CatalogSuccessNotice, { notice: state.success_notice }),
    state.status === "loading" && !state.selected ? createElement(Feedback, { kind: "loading" } as never, "Cargando registros del catálogo…") : null,
    state.status === "unavailable" && !state.selected ? createElement(Feedback, { kind: "unavailable" } as never, createElement("span", null, "El catálogo no está disponible. ", createElement(Action, { variant: "tertiary", onClick: reload }, "Reintentar catálogo"))) : null,
    createElement(
      "div",
      { "data-ui-catalog-layout": true },
      createElement(
        Panel,
        { label: "Registros del catálogo" } as never,
        createElement("h2", null, "Categorías"),
        state.status === "ready" && state.records.length === 0 ? createElement(Feedback, { kind: "empty" } as never, "Todavía no hay registros del catálogo.") : null,
        createElement("ul", { "data-ui-catalog-master": true }, visibleRecords.map((record) => createElement(
          "li",
          { key: `${record.target}-${record.entity_id}`, "data-ui-selected": state.selected?.target === record.target && state.selected.entity_id === record.entity_id || undefined },
          createElement(
            Action,
            { variant: "tertiary", disabled: state.status === "pending" || state.recovery_required, onClick: () => void loadDetail(record), "aria-label": `Editar ${record.label}` },
            createElement("span", null,
              createElement("strong", null, record.label),
              createElement("small", null, record.target === "product" ? "Producto" : "Categoría"),
              createElement("span", null, "Editar"),
              createElement("span", null, record.activity === "active" ? "Activo" : "Archivado")),
          ),
        ))),
      ),
      createElement(
        "div",
        { "data-ui-catalog-workspace": true },
        createElement(
          Panel,
          { label: "Productos" } as never,
          createElement(ProductBrowser, {
            state: browser,
            onQueryChange: (value) => browserDispatch({ type: "query_changed", value }),
            onCategoryChange: (value) => browserDispatch({ type: "category_changed", value }),
            onActivityChange: (value) => browserDispatch({ type: "activity_changed", value }),
            showActivity: true,
            onSubmit: submitBrowse,
            onPageChange: (page) => void browseCatalogProducts(browserSnapshot(), page),
            onSelect: (product) => void loadDetail({ target: "product", entity_id: product.product_id, label: `${product.sku} — ${product.name}`, activity: "active", revision: product.revision }),
            allowUnavailableSelection: true,
            actionLabel: "Editar",
            disabled: interactionLocked,
          }),
        ),
      ),
    ),
    state.selected ? createElement(CatalogEditDialog, { record: state.selected, detail: state.detail, form, loading: state.status === "loading", pending: state.status === "pending", feedback: state.feedback, lifecycleFeedback: state.lifecycle_feedback, recoveryRequired: state.recovery_required, fieldErrors: state.field_errors, onChange: change, onSubmit: edit, onLifecycle: maintain, onReload: state.recovery_required ? retryRefresh : reload, onCancel: close }) : null,
  );
}
