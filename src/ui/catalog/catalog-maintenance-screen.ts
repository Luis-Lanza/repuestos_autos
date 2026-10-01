import { createElement, type ChangeEvent, type FormEvent, useEffect, useReducer, useRef, useState } from "react";

import { CATALOG_INTENT, CATALOG_TARGET, browseProducts, catalogAccessCommands, catalogMaintenanceCommands, catalogProductImageCommands, productLocationCommands, type CatalogAccessResponse, type CatalogAccessStatus, type CatalogMaintenanceRecord, type CatalogMetadataDetail, type ProductLocationRecord } from "../../commands/catalog.ts";
import { Action, Feedback } from "../visual-system/controls.ts";
import { CatalogEditDialog, CatalogMetadataEditor } from "../visual-system/catalog-edit-dialog.ts";
import { ConfirmationDialog } from "../visual-system/confirmation-dialog.ts";
import { Panel } from "../visual-system/structure.ts";
import { createCatalogEditRequest, createCategorySchemaEditRequest, createCatalogMaintenanceFlow, fieldErrorsForCatalogEdit, filterCatalogCategories, formForCatalogDetail, initialCatalogMaintenanceState, type CatalogEditFieldErrors, type CatalogEditForm, type CatalogMaintenanceAction } from "./catalog-maintenance-flow.ts";
import { createProductBrowserFlow, initialProductBrowserState, ProductBrowser, readCatalogViewMode, writeCatalogViewMode, type CatalogViewMode, type ProductBrowserState } from "./product-browser.ts";
import { LocationManagementScreen } from "./location-management-screen.ts";

export { CatalogMetadataEditor } from "../visual-system/catalog-edit-dialog.ts";

type Dispatch = (action: CatalogMaintenanceAction) => void;
type SetForm = (form: CatalogEditForm) => void;
type CatalogLoadCommands = Pick<typeof catalogMaintenanceCommands, "detail" | "listCategories">;
type BrowserSnapshot = Pick<ProductBrowserState, "query" | "category_id" | "stock_state" | "activity" | "page" | "request_id">;

const accessErrorMessage = (response: CatalogAccessResponse) => response.kind === "error" && response.code === "invalid_credentials" ? "La contraseña o el código de recuperación no es válido." : response.kind === "error" && response.code === "license_required" ? "Se requiere una licencia válida para configurar el acceso al catálogo." : response.kind === "error" && response.code === "validation_error" ? "La contraseña debe tener entre 8 y 1024 caracteres." : "No se pudo completar la solicitud de acceso al catálogo.";

function CatalogAccessGate({ status, onUnlocked }: { status: CatalogAccessStatus | "loading"; onUnlocked: () => void }) {
  const [password, setPassword] = useState("");
  const [confirmation, setConfirmation] = useState("");
  const [recoveryCode, setRecoveryCode] = useState("");
  const [shownCode, setShownCode] = useState<string | null>(null);
  const [mode, setMode] = useState<"setup" | "unlock" | "recovery" | "confirm_setup" | "confirm_recovery">(status === "setup_required" ? "setup" : "unlock");
  const [feedback, setFeedback] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  useEffect(() => { setMode(status === "setup_required" ? "setup" : "unlock"); }, [status]);
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (pending) return;
    if ((mode === "setup" || mode === "recovery") && (password.length < 8 || password !== confirmation)) { setFeedback("Ingresá una contraseña de al menos 8 caracteres y confirmala correctamente."); return; }
    setPending(true); setFeedback(null);
    try {
      const response = mode === "setup" ? await catalogAccessCommands.beginSetup(password)
        : mode === "unlock" ? await catalogAccessCommands.unlock(password)
          : mode === "recovery" ? await catalogAccessCommands.beginRecovery(recoveryCode, password)
            : mode === "confirm_setup" ? await catalogAccessCommands.finishSetup()
              : await catalogAccessCommands.finishRecovery();
      setPassword(""); setConfirmation(""); setRecoveryCode("");
      if (response.kind === "recovery_code") { setShownCode(response.recovery_code); setMode(mode === "setup" ? "confirm_setup" : "confirm_recovery"); }
      else if (response.kind === "success") { setShownCode(null); onUnlocked(); }
      else setFeedback(accessErrorMessage(response));
    } finally { setPending(false); }
  };
  const isConfirmation = mode === "confirm_setup" || mode === "confirm_recovery";
  const recoveryNotice = isConfirmation && shownCode ? createElement("section", { "aria-label": "Código de recuperación" },
    createElement("p", null, "Guardá este código de recuperación en un lugar seguro. Solo se muestra ahora."),
    createElement("output", { "data-ui-catalog-recovery-code": true }, shownCode),
    createElement("label", null,
      createElement("input", { type: "checkbox", checked: confirmation === "stored", onChange: (event: ChangeEvent<HTMLInputElement>) => setConfirmation(event.currentTarget.checked ? "stored" : "") }),
      "Confirmo que guardé el código de recuperación")) : null;
  const form = status === "locked" || status === "setup_required" ? createElement("form", { onSubmit: (event: FormEvent) => { void submit(event); } },
    (mode === "setup" || mode === "recovery") && createElement("label", null, "Nueva contraseña", createElement("input", { type: "password", autoComplete: "new-password", value: password, onChange: (event: ChangeEvent<HTMLInputElement>) => setPassword(event.currentTarget.value), required: true, minLength: 8, maxLength: 1024 })),
    (mode === "setup" || mode === "recovery") && createElement("label", null, "Confirmar contraseña", createElement("input", { type: "password", autoComplete: "new-password", value: confirmation, onChange: (event: ChangeEvent<HTMLInputElement>) => setConfirmation(event.currentTarget.value), required: true, minLength: 8, maxLength: 1024 })),
    mode === "unlock" && createElement("label", null, "Contraseña", createElement("input", { type: "password", autoComplete: "current-password", value: password, onChange: (event: ChangeEvent<HTMLInputElement>) => setPassword(event.currentTarget.value), required: true, maxLength: 1024 })),
    mode === "recovery" && createElement("label", null, "Código de recuperación", createElement("input", { type: "text", autoComplete: "off", value: recoveryCode, onChange: (event: ChangeEvent<HTMLInputElement>) => setRecoveryCode(event.currentTarget.value), required: true, maxLength: 48 })),
    recoveryNotice,
    feedback && createElement(Feedback, { kind: "error" } as never, feedback),
    createElement(Action, { type: "submit", disabled: pending || isConfirmation && confirmation !== "stored" }, pending ? "Procesando…" : isConfirmation ? "Continuar al catálogo" : mode === "setup" ? "Configurar catálogo" : mode === "unlock" ? "Desbloquear catálogo" : "Restablecer contraseña"),
    mode === "unlock" && createElement(Action, { type: "button", variant: "tertiary", onClick: () => { setMode("recovery"); setFeedback(null); } }, "Usar código de recuperación"),
    mode === "recovery" && createElement(Action, { type: "button", variant: "tertiary", onClick: () => { setMode("unlock"); setFeedback(null); } }, "Volver al ingreso de contraseña")) : null;
  return createElement("main", { "aria-labelledby": "catalog-access-heading", "data-ui-catalog-access": true },
    createElement("h1", { id: "catalog-access-heading" }, "Acceso al catálogo"),
    createElement("p", null, status === "setup_required" ? "Configurá una contraseña para proteger el catálogo de este dispositivo." : "Ingresá tu contraseña para desbloquear el catálogo durante esta sesión."),
    (status === "loading" || status === "unavailable") && createElement(Feedback, { kind: status === "loading" ? "loading" : "error" } as never, status === "loading" ? "Verificando acceso…" : "No se pudo leer la configuración local del catálogo."),
    form);
}

function CatalogPasswordChange() {
  const [current, setCurrent] = useState(""); const [next, setNext] = useState(""); const [confirm, setConfirm] = useState(""); const [feedback, setFeedback] = useState<string | null>(null); const [pending, setPending] = useState(false);
  const pendingRef = useRef(false); const mounted = useRef(true); const requestId = useRef(0);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; requestId.current += 1; }; }, []);
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (pendingRef.current) return;
    if (next.length < 8 || next !== confirm) { setFeedback("La nueva contraseña debe tener al menos 8 caracteres y coincidir con la confirmación."); return; }
    pendingRef.current = true; setPending(true); setFeedback(null);
    const request = ++requestId.current;
    try {
      const response = await catalogAccessCommands.changePassword(current, next);
      if (!mounted.current || request !== requestId.current) return;
      if (response.kind === "success") { setCurrent(""); setNext(""); setConfirm(""); setFeedback("Contraseña actualizada."); }
      else setFeedback(accessErrorMessage(response));
    } catch {
      if (mounted.current && request === requestId.current) setFeedback("No se pudo completar la solicitud de acceso al catálogo.");
    } finally {
      if (mounted.current && request === requestId.current) { pendingRef.current = false; setPending(false); }
    }
  };
  return createElement("details", { "data-ui-catalog-password-change": true }, createElement("summary", null, "Cambiar contraseña del catálogo"), createElement("form", { onSubmit: (event: FormEvent) => void submit(event) }, createElement("label", null, "Contraseña actual", createElement("input", { type: "password", autoComplete: "current-password", value: current, onChange: (event: ChangeEvent<HTMLInputElement>) => setCurrent(event.currentTarget.value), required: true, maxLength: 1024, disabled: pending })), createElement("label", null, "Nueva contraseña", createElement("input", { type: "password", autoComplete: "new-password", value: next, onChange: (event: ChangeEvent<HTMLInputElement>) => setNext(event.currentTarget.value), required: true, minLength: 8, maxLength: 1024, disabled: pending })), createElement("label", null, "Confirmar nueva contraseña", createElement("input", { type: "password", autoComplete: "new-password", value: confirm, onChange: (event: ChangeEvent<HTMLInputElement>) => setConfirm(event.currentTarget.value), required: true, minLength: 8, maxLength: 1024, disabled: pending })), feedback ? createElement(Feedback, { kind: feedback === "Contraseña actualizada." ? "success" : "error" } as never, feedback) : null, createElement(Action, { type: "submit", disabled: pending }, pending ? "Guardando…" : "Guardar contraseña")));
}

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
  const [accessStatus, setAccessStatus] = useState<CatalogAccessStatus | "loading">("loading");
  const [lockPending, setLockPending] = useState(false);
  const [lockFeedback, setLockFeedback] = useState<string | null>(null);
  const lockPendingRef = useRef(false);
  const [form, setForm] = useState<CatalogEditForm | null>(null);
  const [productLocations, setProductLocations] = useState<ProductLocationRecord[]>([]);
  const [productLocationsStatus, setProductLocationsStatus] = useState<"loading" | "ready" | "error">("loading");
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
  const [catalogSubview, setCatalogSubview] = useState<"products" | "categories" | "locations">("products");
  const [categoryQuery, setCategoryQuery] = useState("");
  const [categoryActionPending, setCategoryActionPending] = useState<number | null>(null);
  const [categoryActionFeedback, setCategoryActionFeedback] = useState<Record<number, string>>({});
  const [archiveConfirmation, setArchiveConfirmation] = useState<{ record: CatalogMaintenanceRecord; context: "category-list" | "detail" } | null>(null);
  const [archivePending, setArchivePending] = useState(false);
  const [retirementConfirmation, setRetirementConfirmation] = useState<{ index: number; label: string } | null>(null);
  const catalogMainHeading = useRef<HTMLHeadingElement>(null);
  const categoryManagementHeading = useRef<HTMLHeadingElement>(null);
  const [browser, browserDispatch] = useReducer(createProductBrowserFlow, { ...initialProductBrowserState, activity: "all" });
  const browserRef = useRef(browser);
  browserRef.current = browser;
  const mutationLocked = useRef(false);
  useEffect(() => () => { mounted.current = false; attempt.current += 1; browseAttempt.current += 1; browseThumbnailAttempt.current += 1; detailThumbnailAttempt.current += 1; mutationLocked.current = true; imageMutationLocked.current = true; }, []);
  useEffect(() => { (catalogSubview === "categories" ? categoryManagementHeading.current : catalogSubview === "products" ? catalogMainHeading.current : null)?.focus(); }, [catalogSubview]);

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
    setProductLocations([]);
    setProductLocationsStatus("loading");
    dispatch({ type: "detail_started", record });
    const response = await catalogMaintenanceCommands.detail(record);
    if (!mounted.current || current !== attempt.current) return;
    if (response.kind === "success") {
      if (response.detail.target === CATALOG_TARGET.PRODUCT) {
        const locations = await productLocationCommands.list(false);
        if (!mounted.current || current !== attempt.current) return;
        if (locations.kind === "locations_success") { setProductLocations(locations.locations.filter((location) => location.active)); setProductLocationsStatus("ready"); }
        else setProductLocationsStatus("error");
      } else setProductLocationsStatus("ready");
      setForm(formForCatalogDetail(response.detail)); refreshDetailAfterRecovery.current = false; dispatch({ type: "detail_loaded", detail: response.detail });
    } else dispatch({ type: "detail_failed", code: response.code });
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
  useEffect(() => { void catalogAccessCommands.status().then((response) => setAccessStatus(response.kind === "status" ? response.status : "unavailable")); }, []);
  useEffect(() => { if (accessStatus === "unlocked") void load(); }, [accessStatus]);
  useEffect(() => {
    const first = Object.keys(state.field_errors)[0];
    if (first) {
      const id = first.startsWith("attribute-") ? `catalog-${first}` : first === "minimum_sale_price_centavos" ? "catalog-edit-minimum-price" : `catalog-edit-${first.replaceAll("_", "-")}`;
      document.getElementById(id)?.focus();
    }
  }, [state.field_errors]);

  const lockCatalog = async () => {
    if (lockPendingRef.current) return;
    lockPendingRef.current = true;
    setLockPending(true);
    setLockFeedback(null);
    const response = await catalogAccessCommands.lock();
    lockPendingRef.current = false;
    setLockPending(false);
    if (response.kind === "success") {
      attempt.current += 1; browseAttempt.current += 1; browseThumbnailAttempt.current += 1; detailThumbnailAttempt.current += 1;
      mutationLocked.current = true; imageMutationLocked.current = true;
      setForm(null); setProductLocations([]); setImageThumbnail(null); setBrowseThumbnails({});
      dispatch({ type: "selection_cleared" });
      setAccessStatus("locked");
    } else setLockFeedback(accessErrorMessage(response));
  };
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
    if (response.kind === "error") {
      mutationLocked.current = false;
      refreshDetailAfterRecovery.current = response.code === "stale_catalog_record" || response.code === "stale_category_schema";
      dispatch({ type: "edit_failed", code: response.code });
      return;
    }
    let updatedRevision = response.revision;
    if (detail.target === CATALOG_TARGET.PRODUCT && (form.primary_location_id ?? null) !== detail.primary_location_id) {
      const assignment = await productLocationCommands.assignPrimary({ product_id: detail.entity_id, expected_revision: response.revision, location_id: form.primary_location_id ?? null });
      if (!mounted.current) return;
      if (assignment.kind !== "assignment_success" || assignment.product_id !== detail.entity_id || assignment.location_id !== (form.primary_location_id ?? null)) {
        mutationLocked.current = false;
        refreshDetailAfterRecovery.current = true;
        dispatch({ type: "location_assignment_failed" });
        return;
      }
      updatedRevision = assignment.revision;
    }
    mutationLocked.current = false;
    const updatedRecord = { ...response, revision: updatedRevision };
    dispatch({ type: "edit_succeeded", record: updatedRecord });
    refreshDetailAfterRecovery.current = true;
    const refreshed = await refreshCatalogList(true);
    if (refreshed && mounted.current) await loadDetail({ target: response.target, entity_id: response.entity_id, label: response.label, activity: response.activity, revision: updatedRevision });
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
  const change = (field: string, value: string) => setForm((current) => !current ? current : field.startsWith("attribute-") ? { ...current, attribute_values: { ...current.attribute_values, [Number(field.slice(10))]: value } } : field === "primary_location_id" ? { ...current, primary_location_id: value === "" ? null : Number(value) } : { ...current, [field]: value });
  const addCategoryField = () => setForm((current) => current?.category_fields ? { ...current, category_fields: [...current.category_fields, { definition_id: null, label: "", field_type: "text", required: false, options: "", active: true }] } : current);
  const removeCategoryFieldDraft = (index: number) => setForm((current) => {
    const field = current?.category_fields?.[index];
    if (!current?.category_fields || field?.definition_id !== null) return current;
    return { ...current, category_fields: current.category_fields.filter((_, fieldIndex) => fieldIndex !== index) };
  });
  const changeCategoryField = (index: number, field: string, value: string | boolean) => setForm((current) => current?.category_fields ? { ...current, category_fields: current.category_fields.map((item, itemIndex) => itemIndex === index ? { ...item, [field]: value } : item) } : current);
  const requestCategoryFieldRetirement = (index: number) => {
    const field = form?.category_fields?.[index];
    if (field?.definition_id !== null && field) setRetirementConfirmation({ index, label: field.label });
  };
  const confirmCategoryFieldRetirement = () => {
    const retirement = retirementConfirmation;
    if (!retirement) return;
    setForm((current) => current?.category_fields ? { ...current, category_fields: current.category_fields.map((field, index) => index === retirement.index ? { ...field, active: false } : field) } : current);
    setRetirementConfirmation(null);
  };
  const saveCategorySchema = async () => {
    const detail = state.detail;
    if (!detail || detail.target !== "category" || !form || mutationLocked.current || state.recovery_required) return;
    const request = createCategorySchemaEditRequest(detail, form);
    if (!request) { dispatch({ type: "edit_validation_failed", field_errors: { category_fields: "Completá el nombre y al menos una opción para cada campo de opciones." } }); return; }
    mutationLocked.current = true;
    dispatch({ type: "edit_started" });
    const currentFields = detail.attribute_definitions.filter((field) => field.active !== false).map((field) => ({ definition_id: field.definition_id, label: field.label, field_type: field.field_type, required: field.required, options: field.field_type === "option" ? field.options : [] }));
    const schemaChanged = JSON.stringify(request.fields) !== JSON.stringify(currentFields);
    const schemaResponse = schemaChanged ? await catalogMaintenanceCommands.editCategorySchema(request) : null;
    if (!mounted.current) return;
    if (schemaResponse?.kind === "error") {
      mutationLocked.current = false;
      refreshDetailAfterRecovery.current = schemaResponse.code === "stale_category_schema" || schemaResponse.code === "stale_catalog_record";
      dispatch({ type: "edit_failed", code: schemaResponse.code });
      return;
    }
    const response = !schemaChanged
      ? await catalogMaintenanceCommands.edit({ target: "category", entity_id: detail.entity_id, expected_revision: detail.revision, name: form.name })
      : form.name === detail.name
        ? schemaResponse!
        : await catalogMaintenanceCommands.edit({ target: "category", entity_id: detail.entity_id, expected_revision: schemaResponse!.revision, name: form.name });
    if (!mounted.current) return;
    mutationLocked.current = false;
    if (response.kind === "error") {
      refreshDetailAfterRecovery.current = response.code === "stale_category_schema" || response.code === "stale_catalog_record";
      dispatch({ type: "edit_failed", code: response.code });
      return;
    }
    dispatch({ type: "edit_succeeded", record: response });
    refreshDetailAfterRecovery.current = true;
    const refreshed = await refreshCatalogList(true);
    if (refreshed && mounted.current) await loadDetail({ target: response.target, entity_id: response.entity_id, label: response.label, activity: response.activity, revision: response.revision });
  };
  const pending = state.status === "pending" || state.status === "loading" && !!state.selected;
  const interactionLocked = pending || state.recovery_required;
  const submitBrowse = (event: FormEvent) => {
    event.preventDefault();
    if (!interactionLocked) {
      const snapshot = browserSnapshot();
      void browseCatalogProducts(snapshot, snapshot.page ?? 1);
    }
  };
  const visibleCategories = filterCatalogCategories(state.records, categoryQuery);

  if (accessStatus !== "unlocked") return createElement(CatalogAccessGate, { status: accessStatus, onUnlocked: () => setAccessStatus("unlocked") });

  return createElement(
    "main",
    { "aria-labelledby": "catalog-maintenance-heading", "data-ui-catalog": true },
    createElement("h1", { id: "catalog-maintenance-heading", ref: catalogMainHeading, tabIndex: -1 }, "Catálogo"),
    createElement("p", null, "Editá metadatos desde el detalle de categorías y productos."),
    createElement(Action, { variant: "secondary", disabled: lockPending, "aria-busy": lockPending, onClick: () => void lockCatalog() }, lockPending ? "Bloqueando…" : "Bloquear catálogo"),
    lockFeedback ? createElement(Feedback, { kind: "error" } as never, lockFeedback) : null,
    createElement(CatalogPasswordChange),
    createElement("nav", { "aria-label": "Vistas del catálogo", "data-ui-catalog-navigation": true },
      createElement(Action, { variant: catalogSubview === "products" ? "secondary" : "tertiary", "aria-current": catalogSubview === "products" ? "page" : undefined, onClick: () => setCatalogSubview("products") }, "Productos"),
      createElement(Action, { variant: catalogSubview === "categories" ? "secondary" : "tertiary", "aria-current": catalogSubview === "categories" ? "page" : undefined, onClick: () => setCatalogSubview("categories") }, "Gestionar categorías"),
      createElement(Action, { variant: catalogSubview === "locations" ? "secondary" : "tertiary", "aria-current": catalogSubview === "locations" ? "page" : undefined, onClick: () => setCatalogSubview("locations") }, "Gestionar ubicaciones")),
    catalogSubview === "locations" ? createElement(LocationManagementScreen) : null,
    catalogSubview === "categories" ? createElement("section", { "aria-labelledby": "catalog-category-management-heading", "data-ui-category-management": true },
      createElement("div", { "data-ui-category-management-header": true },
        createElement("h2", { id: "catalog-category-management-heading", ref: categoryManagementHeading, tabIndex: -1 }, "Gestión de categorías")),
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
    state.selected ? createElement(CatalogEditDialog, { record: state.selected, detail: state.detail, form, loading: state.status === "loading", pending: state.status === "pending", feedback: state.feedback, lifecycleFeedback: state.lifecycle_feedback, recoveryRequired: state.recovery_required, fieldErrors: state.field_errors, locations: productLocations, locationsStatus: productLocationsStatus, imageThumbnail, imagePending, imageFeedback, onChooseImage: () => void mutateImage("choose"), onRemoveImage: () => void mutateImage("remove"), onChange: change, onSubmit: edit, onLifecycle: requestDetailLifecycle, onReload: state.recovery_required ? retryRefresh : reload, onAddCategoryField: addCategoryField, onChangeCategoryField: changeCategoryField, onRetireCategoryField: requestCategoryFieldRetirement, onRemoveCategoryFieldDraft: removeCategoryFieldDraft, onSaveCategorySchema: () => void saveCategorySchema(), onCancel: close }) : null,
    retirementConfirmation ? createElement(ConfirmationDialog, {
      open: true,
      purpose: "cancellation",
      title: `Retirar ${retirementConfirmation.label || "campo"}`,
      description: `¿Querés retirar ${retirementConfirmation.label || "este campo"}? La definición y sus valores históricos se conservarán; no se podrá reactivar en esta versión.`,
      confirmLabel: "Confirmar retiro",
      pending: false,
      dialogId: "catalog-field-retirement-confirmation-dialog",
      onCancel: () => setRetirementConfirmation(null),
      onConfirm: confirmCategoryFieldRetirement,
    }) : null,
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
