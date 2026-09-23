import { createElement, type ChangeEvent, type FormEvent, useEffect, useReducer, useRef, useState } from "react";

import { CATALOG_INTENT, browseProducts, catalogMaintenanceCommands, catalogProductImageCommands, type CatalogMaintenanceRecord, type CatalogMetadataDetail } from "../../commands/catalog.ts";
import { Action, Feedback } from "../visual-system/controls.ts";
import { CatalogEditDialog, CatalogMetadataEditor } from "../visual-system/catalog-edit-dialog.ts";
import { ConfirmationDialog } from "../visual-system/confirmation-dialog.ts";
import { Panel } from "../visual-system/structure.ts";
import { createCatalogEditRequest, createCatalogMaintenanceFlow, fieldErrorsForCatalogEdit, filterCatalogCategories, formForCatalogDetail, initialCatalogMaintenanceState, type CatalogEditFieldErrors, type CatalogEditForm, type CatalogMaintenanceAction } from "./catalog-maintenance-flow.ts";
import { createProductBrowserFlow, initialProductBrowserState, ProductBrowser, readCatalogViewMode, writeCatalogViewMode, type CatalogViewMode, type ProductBrowserState } from "./product-browser.ts";

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
  const [imageThumbnail, setImageThumbnail] = useState<string | null>(null);
  const [imagePending, setImagePending] = useState(false);
  const [imageFeedback, setImageFeedback] = useState<string | null>(null);
  const [browseThumbnails, setBrowseThumbnails] = useState<Record<number, string>>({});
  const mounted = useRef(true);
  const attempt = useRef(0);
  const browseAttempt = useRef(0);
  const browseThumbnailAttempt = useRef(0);
  const detailThumbnailAttempt = useRef(0);
  const imageMutationLocked = useRef(false);
  const refreshDetailAfterRecovery = useRef(false);
  const [catalogViewMode, setCatalogViewMode] = useState<CatalogViewMode>(readCatalogViewMode);
  const [catalogSubview, setCatalogSubview] = useState<"products" | "categories">("products");
  const [categoryQuery, setCategoryQuery] = useState("");
  const [categoryActionPending, setCategoryActionPending] = useState<number | null>(null);
  const [categoryActionFeedback, setCategoryActionFeedback] = useState<Record<number, string>>({});
  const [archiveConfirmation, setArchiveConfirmation] = useState<{ record: CatalogMaintenanceRecord; context: "category-list" | "detail" } | null>(null);
  const [archivePending, setArchivePending] = useState(false);
  const catalogMainHeading = useRef<HTMLHeadingElement>(null);
  const categoryManagementHeading = useRef<HTMLHeadingElement>(null);
  const [browser, browserDispatch] = useReducer(createProductBrowserFlow, { ...initialProductBrowserState, activity: "all" });
  const browserRef = useRef(browser);
  browserRef.current = browser;
  const mutationLocked = useRef(false);
  useEffect(() => () => { mounted.current = false; attempt.current += 1; browseAttempt.current += 1; browseThumbnailAttempt.current += 1; detailThumbnailAttempt.current += 1; mutationLocked.current = true; imageMutationLocked.current = true; }, []);
  useEffect(() => { (catalogSubview === "categories" ? categoryManagementHeading.current : catalogMainHeading.current)?.focus(); }, [catalogSubview]);

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
    detailThumbnailAttempt.current += 1;
    setImageThumbnail(null);
    setImageFeedback(null);
    setForm(null);
    dispatch({ type: "detail_started", record });
    const response = await catalogMaintenanceCommands.detail(record);
    if (!mounted.current || current !== attempt.current) return;
    if (response.kind === "success") { setForm(formForCatalogDetail(response.detail)); refreshDetailAfterRecovery.current = false; dispatch({ type: "detail_loaded", detail: response.detail }); }
    else dispatch({ type: "detail_failed", code: response.code });
  };
  useEffect(() => {
    const current = ++detailThumbnailAttempt.current;
    const detail = state.detail;
    if (!detail || detail.target !== "product") { setImageThumbnail(null); return; }
    void catalogProductImageCommands.thumbnail({ product_id: detail.entity_id, expected_revision: detail.revision }).then((response) => {
      if (!mounted.current || current !== detailThumbnailAttempt.current) return;
      if (response.kind === "success" && response.product_id === detail.entity_id && response.revision === detail.revision) {
        setImageThumbnail(response.src);
        setImageFeedback(null);
      } else {
        setImageThumbnail(null);
        if (response.kind === "error" && response.code !== "image_unavailable") setImageFeedback("No se pudo cargar la vista previa.");
      }
    });
  }, [state.detail]);
  useEffect(() => {
    const result = browser.result;
    if (!result || browser.status !== "results") { setBrowseThumbnails({}); return; }
    const current = ++browseThumbnailAttempt.current;
    const requestId = browser.request_id;
    setBrowseThumbnails({});
    void Promise.all(result.products.map(async (product) => {
      const response = await catalogProductImageCommands.thumbnail({ product_id: product.product_id, expected_revision: product.revision });
      return response.kind === "success" && response.product_id === product.product_id && response.revision === product.revision ? [product.product_id, response.src] as const : null;
    })).then((thumbnails) => {
      if (!mounted.current || current !== browseThumbnailAttempt.current || browserRef.current.request_id !== requestId) return;
      setBrowseThumbnails(Object.fromEntries(thumbnails.filter((item): item is readonly [number, string] => item !== null)));
    });
  }, [browser.result, browser.status]);

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
  const manageCategoryLifecycle = async (record: CatalogMaintenanceRecord, confirmedArchive = false) => {
    if (record.target !== "category" || mutationLocked.current || state.recovery_required || state.status === "pending") return;
    if (record.activity === "active" && !confirmedArchive) {
      setArchiveConfirmation({ record, context: "category-list" });
      return;
    }
    mutationLocked.current = true;
    setCategoryActionPending(record.entity_id);
    setCategoryActionFeedback((current) => ({ ...current, [record.entity_id]: "" }));
    const response = await catalogMaintenanceCommands.maintain({ target: record.target, entity_id: record.entity_id, intent: record.activity === "active" ? CATALOG_INTENT.ARCHIVE : CATALOG_INTENT.REACTIVATE, expected_revision: record.revision });
    if (!mounted.current) return;
    mutationLocked.current = false;
    setCategoryActionPending(null);
    if (response.kind === "error") {
      const feedback = response.code === "lifecycle_blocked" ? "No se puede archivar esta categoría mientras tenga productos activos." : response.code === "stale_catalog_record" ? "La categoría cambió. Recargá los registros para continuar." : response.code === "catalog_unavailable" ? "La categoría no está disponible. Recargá los registros." : "No se pudo actualizar el estado de la categoría.";
      setCategoryActionFeedback((current) => ({ ...current, [record.entity_id]: feedback }));
      if (response.code === "stale_catalog_record") dispatch({ type: "mutation_failed", code: response.code });
      return;
    }
    dispatch({ type: "mutation_succeeded", record: response });
    await refreshCatalogList();
  };
  const maintain = async (archiveRecord?: CatalogMaintenanceRecord) => {
    const detail = state.detail;
    const target = archiveRecord ?? detail;
    if (!target || mutationLocked.current || state.recovery_required) return;
    mutationLocked.current = true;
    dispatch({ type: "mutation_started" });
    const response = await catalogMaintenanceCommands.maintain({ target: target.target, entity_id: target.entity_id, intent: archiveRecord ? CATALOG_INTENT.ARCHIVE : target.activity === "active" ? CATALOG_INTENT.ARCHIVE : CATALOG_INTENT.REACTIVATE, expected_revision: target.revision });
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
  const requestDetailLifecycle = () => {
    const detail = state.detail;
    if (!detail) return;
    if (detail.activity === "active") {
      setArchiveConfirmation({ record: { entity_id: detail.entity_id, target: detail.target, label: detail.target === "product" ? `${detail.sku} — ${detail.name}` : detail.name, activity: detail.activity, revision: detail.revision }, context: "detail" });
      return;
    }
    void maintain();
  };
  const confirmArchive = async () => {
    const confirmation = archiveConfirmation;
    if (!confirmation || archivePending || mutationLocked.current) return;
    setArchivePending(true);
    try {
      if (confirmation.context === "category-list") await manageCategoryLifecycle(confirmation.record, true);
      else await maintain(confirmation.record);
    } finally {
      if (mounted.current) {
        setArchivePending(false);
        setArchiveConfirmation(null);
      }
    }
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
  const mutateImage = async (operation: "choose" | "remove") => {
    const detail = state.detail;
    if (!detail || detail.target !== "product" || imageMutationLocked.current || state.recovery_required || mutationLocked.current) return;
    imageMutationLocked.current = true;
    setImagePending(true);
    setImageFeedback(null);
    const response = await catalogProductImageCommands[operation]({ product_id: detail.entity_id, expected_revision: detail.revision });
    if (!mounted.current) return;
    imageMutationLocked.current = false;
    setImagePending(false);
    if (response.kind === "cancelled") { setImageFeedback("No se modificó la imagen."); return; }
    if (response.kind === "error") {
      refreshDetailAfterRecovery.current = response.code === "stale_catalog_record";
      if (response.code === "stale_catalog_record") dispatch({ type: "mutation_failed", code: response.code });
      else setImageFeedback("No se pudo actualizar la imagen del producto.");
      return;
    }
    if (response.product_id !== detail.entity_id) { setImageFeedback("No se pudo actualizar la imagen del producto."); return; }
    setImageThumbnail(null);
    dispatch({ type: "mutation_succeeded", record: { entity_id: detail.entity_id, target: detail.target, label: `${detail.sku} — ${detail.name}`, activity: detail.activity, revision: response.revision } });
    refreshDetailAfterRecovery.current = true;
    const refreshed = await refreshCatalogList(true);
    if (refreshed && mounted.current) await loadDetail({ target: detail.target, entity_id: detail.entity_id, label: `${detail.sku} — ${detail.name}`, activity: detail.activity, revision: response.revision });
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
  const visibleCategories = filterCatalogCategories(state.records, categoryQuery);

  return createElement(
    "main",
    { "aria-labelledby": "catalog-maintenance-heading", "data-ui-catalog": true },
    createElement("h1", { id: "catalog-maintenance-heading", ref: catalogMainHeading, tabIndex: -1 }, "Catálogo"),
    createElement("p", null, "Editá metadatos desde el detalle de categorías y productos."),
    createElement("nav", { "aria-label": "Vistas del catálogo", "data-ui-catalog-navigation": true },
      createElement(Action, { variant: catalogSubview === "products" ? "secondary" : "tertiary", "aria-current": catalogSubview === "products" ? "page" : undefined, onClick: () => setCatalogSubview("products") }, "Productos"),
      createElement(Action, { variant: catalogSubview === "categories" ? "secondary" : "tertiary", "aria-current": catalogSubview === "categories" ? "page" : undefined, onClick: () => setCatalogSubview("categories") }, "Gestionar categorías")),
    catalogSubview === "categories" ? createElement("section", { "aria-labelledby": "catalog-category-management-heading", "data-ui-category-management": true },
      createElement("div", { "data-ui-category-management-header": true },
        createElement("h2", { id: "catalog-category-management-heading", ref: categoryManagementHeading, tabIndex: -1 }, "Gestión de categorías"),
        createElement(Action, { variant: "tertiary", onClick: () => setCatalogSubview("products") }, "Volver a productos")),
      createElement("label", { "data-ui-category-search": true }, "Buscar categorías", createElement("input", { type: "search", value: categoryQuery, onChange: (event: ChangeEvent<HTMLInputElement>) => setCategoryQuery(event.currentTarget.value), "aria-label": "Buscar categorías" })),
      state.status === "loading" && !state.selected ? createElement(Feedback, { kind: "loading" } as never, "Cargando categorías…") : null,
      state.status === "unavailable" && !state.selected ? createElement(Feedback, { kind: "unavailable" } as never, createElement("span", null, "Las categorías no están disponibles. ", createElement(Action, { variant: "tertiary", onClick: () => void load() }, "Reintentar categorías"))) : null,
      state.status === "ready" && visibleCategories.length === 0 ? createElement(Feedback, { kind: "empty" } as never, categoryQuery ? "No hay categorías que coincidan con la búsqueda." : "Todavía no hay categorías.") : null,
      state.lifecycle_feedback ? createElement(Feedback, { kind: state.recovery_required ? state.status === "loading" ? "loading" : "error" : "success" } as never, state.lifecycle_feedback) : null,
      state.recovery_required ? createElement(Action, { variant: "secondary", disabled: pending, onClick: () => void refreshCatalogList() }, "Reintentar categorías") : null,
      createElement("ul", { "aria-label": "Categorías", "data-ui-category-list": true }, visibleCategories.map((record) => createElement("li", { key: record.entity_id, "aria-label": `${record.label}, ${record.activity === "active" ? "Activa" : "Archivada"}`, "data-ui-category-row": true },
        createElement("div", { "data-ui-category-identity": true }, createElement("strong", null, record.label), createElement("span", null, `${record.active_product_count} productos activos`), createElement("span", { "data-ui-category-state": record.activity }, record.activity === "active" ? "Activa" : "Archivada")),
        createElement("div", { "data-ui-category-actions": true },
          createElement(Action, { variant: "secondary", disabled: state.status === "pending" || state.recovery_required || categoryActionPending !== null, onClick: () => void loadDetail(record), "aria-label": `Editar ${record.label}` }, "Editar"),
          createElement(Action, { variant: record.activity === "active" ? "destructive" : "secondary", disabled: state.status === "pending" || state.recovery_required || categoryActionPending !== null, onClick: () => void manageCategoryLifecycle(record), "aria-label": `${record.activity === "active" ? "Archivar" : "Reactivar"} ${record.label}`, "data-ui-catalog-lifecycle-action": record.activity }, record.activity === "active" ? "Archivar" : "Reactivar")),
        categoryActionPending === record.entity_id ? createElement(Feedback, { kind: "loading" } as never, "Actualizando categoría…") : null,
        categoryActionFeedback[record.entity_id] ? createElement(Feedback, { kind: "error" } as never, categoryActionFeedback[record.entity_id]) : null)))
      ) : null,
    createElement(CatalogSuccessNotice, { notice: state.success_notice }),
    state.status === "loading" && !state.selected ? createElement(Feedback, { kind: "loading" } as never, "Cargando registros del catálogo…") : null,
    state.status === "unavailable" && !state.selected ? createElement(Feedback, { kind: "unavailable" } as never, createElement("span", null, "El catálogo no está disponible. ", createElement(Action, { variant: "tertiary", onClick: reload }, "Reintentar catálogo"))) : null,
    createElement(
      "div",
      { "data-ui-catalog-layout": true, hidden: catalogSubview !== "products" },
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
            presentation: "catalog",
            catalogViewMode,
            onCatalogViewModeChange: (mode) => { setCatalogViewMode(mode); writeCatalogViewMode(mode); },
            onQueryChange: (value) => browserDispatch({ type: "query_changed", value }),
            onCategoryChange: (value) => browserDispatch({ type: "category_changed", value }),
            onActivityChange: (value) => browserDispatch({ type: "activity_changed", value }),
            showActivity: true,
            onSubmit: submitBrowse,
            onPageChange: (page) => void browseCatalogProducts(browserSnapshot(), page),
            onSelect: (product) => void loadDetail({ target: "product", entity_id: product.product_id, label: `${product.sku} — ${product.name}`, activity: "active", revision: product.revision }),
            allowUnavailableSelection: true,
            actionLabel: "Editar",
            thumbnails: browseThumbnails,
            disabled: interactionLocked,
          }),
        ),
      ),
    ),
    state.selected ? createElement(CatalogEditDialog, { record: state.selected, detail: state.detail, form, loading: state.status === "loading", pending: state.status === "pending", feedback: state.feedback, lifecycleFeedback: state.lifecycle_feedback, recoveryRequired: state.recovery_required, fieldErrors: state.field_errors, imageThumbnail, imagePending, imageFeedback, onChooseImage: () => void mutateImage("choose"), onRemoveImage: () => void mutateImage("remove"), onChange: change, onSubmit: edit, onLifecycle: requestDetailLifecycle, onReload: state.recovery_required ? retryRefresh : reload, onCancel: close }) : null,
    archiveConfirmation ? createElement(ConfirmationDialog, {
      open: true,
      purpose: "cancellation",
      title: `Archivar ${archiveConfirmation.record.label}`,
      description: `¿Querés archivar ${archiveConfirmation.record.label}? Podés reactivarlo más adelante.`,
      confirmLabel: "Confirmar archivo",
      pending: archivePending,
      pendingLabel: "Archivando…",
      dialogId: "catalog-archive-confirmation-dialog",
      onCancel: () => { if (!archivePending) setArchiveConfirmation(null); },
      onConfirm: () => void confirmArchive(),
    }) : null,
  );
}
