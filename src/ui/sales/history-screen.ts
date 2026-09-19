import {
  createElement,
  Fragment,
  type ChangeEvent,
  type DependencyList,
  type EffectCallback,
  type FormEvent,
  type MouseEvent,
  type RefObject,
  useEffect,
  useMemo,
  useReducer,
  useRef,
  useState,
} from "react";

import { postSaleCommands, type PostSaleErrorCode } from "../../commands/post-sale.ts";
import {
  salesHistoryCommands,
  type SalesHistorySummary,
} from "../../commands/sales-history.ts";
import {
  canOpenCancellation,
  canOpenReturn,
  correctionFocusTarget,
  createHistoryFlow,
  initialHistoryState,
  type CancellationIntent,
  type HistoryAction,
  type HistoryState,
  type ReturnIntent,
} from "./history-flow.ts";
import { historyListError, historySummaryCells, projectCorrectionHistory, projectHistoryDetail } from "./history-presentation.ts";
import { Action, Badge, Feedback, Field } from "../visual-system/controls.ts";
import { AlignedData } from "../visual-system/structure.ts";
import { ConfirmationDialog, FormDialog } from "../visual-system/confirmation-dialog.ts";

const originalItemColumns = [
  { label: "Producto", align: "start", kind: "text" },
  { label: "SKU", align: "start", kind: "sku" },
  { label: "Cantidad", align: "end", kind: "numeric" },
  { label: "Precio unitario", align: "end", kind: "money" },
  { label: "Subtotal", align: "end", kind: "money" },
] as const;
const originalPaymentColumns = [
  { label: "Dato de pago", align: "start", kind: "text" },
  { label: "Importe", align: "end", kind: "money" },
] as const;
const returnedLineColumns = [
  { label: "Línea de venta", align: "end", kind: "numeric" },
  { label: "Producto", align: "end", kind: "numeric" },
  { label: "Cantidad devuelta", align: "end", kind: "numeric" },
] as const;
const restoredLineColumns = [
  { label: "Línea de venta", align: "end", kind: "numeric" },
  { label: "Producto", align: "end", kind: "numeric" },
  { label: "Cantidad restaurada", align: "end", kind: "numeric" },
] as const;

function CorrectionHistory({ detail }: { detail: HistoryState["detail"] }) {
  if (!detail) return null;
  const corrections = projectCorrectionHistory(detail);
  const facts = (record: { requestId: string; occurredAt: string; status: string }, reason?: string) =>
    createElement("dl", { "data-ui-history-correction-facts": true },
      createElement("dt", null, "Fecha y hora registrada"), createElement("dd", { "data-ui-type": "numeric" }, record.occurredAt),
      createElement("dt", null, "Estado"), createElement("dd", null, record.status),
      createElement("dt", null, "ID de solicitud"), createElement("dd", { "data-ui-type": "mono" }, record.requestId),
      reason === undefined ? null : createElement("dt", null, "Motivo"),
      reason === undefined ? null : createElement("dd", null, reason));
  return createElement("section", { "aria-labelledby": "inventory-correction-history-heading", "data-ui-history-corrections": true },
    createElement("h2", { id: "inventory-correction-history-heading" }, "Historial de correcciones de inventario"),
    corrections.returns.length === 0 && corrections.cancellation === null
      ? createElement("p", null, "No hay correcciones de inventario registradas.")
      : null,
    corrections.returns.map((record) => createElement("section", { key: record.identity, "aria-label": record.identity, "data-ui-history-correction-record": true },
      createElement("h3", null, record.identity), facts(record),
      createElement(AlignedData, { caption: "Artículos devueltos", columns: returnedLineColumns, rows: record.lines }))),
    corrections.cancellation ? createElement("section", { "aria-label": corrections.cancellation.identity, "data-ui-history-correction-record": true },
      createElement("h3", null, corrections.cancellation.identity), facts(corrections.cancellation, corrections.cancellation.reason),
      createElement(AlignedData, { caption: "Unidades restauradas por cancelación", columns: restoredLineColumns, rows: corrections.cancellation.lines })) : null);
}
const localToday = () => {
  const date = new Date();
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
};

export const correctionControlStyle = { minWidth: 44, minHeight: 44 } as const;

type Focusable = Pick<HTMLElement, "focus">;
export type FocusFinder = (id: string) => Focusable | null;
type EffectRunner = (
  effect: EffectCallback,
  dependencies?: DependencyList,
) => void;

const findFocusableById: FocusFinder = (id) =>
  globalThis.document?.getElementById(id) ?? null;

const focusCorrectionTarget = (target: string | null, find: FocusFinder) => {
  if (target) find(target)?.focus();
};

function ReturnForm({ state, onAction, onSubmit, onReloadDetail, invokerRef }: {
  state: HistoryState;
  onAction?: (action: HistoryAction) => void;
  onSubmit?: () => void;
  onReloadDetail?: (saleId: number) => void;
  invokerRef?: RefObject<HTMLElement>;
}) {
  const intent = state.return_intent!;
  const locked = intent.status === "pending" || intent.status === "reload_requested";
  const errorTarget = intent.validation?.focus_target;
  const errorId = errorTarget ? `${errorTarget}-error` : undefined;
  return createElement(FormDialog, {
    open: intent.modal_open,
    title: "Devolución de artículos",
    description: "Seleccioná los artículos originales y las cantidades que volverán al inventario.",
    pending: locked,
    invokerRef,
    onCancel: () => onAction?.({ type: "return_modal_closed" }),
  },
  createElement("form", { "aria-label": "Devolución de artículos", "aria-busy": locked,
    "data-ui-history-return": true, onSubmit: (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      if (onSubmit) onSubmit();
      else onAction?.({ type: "return_submit_started" });
    } },
  state.detail === null
    ? createElement("p", { role: "status" }, "Cargando el detalle guardado…")
    : null,
  (state.detail?.lines ?? []).map((line) => {
    const selected = String(line.sale_line_id) in intent.lines;
    const eligible = line.remaining_returnable_quantity > 0;
    const quantityId = `return-quantity-${line.sale_line_id}`;
    const selectionId = `return-line-${line.sale_line_id}`;
    const fieldError = errorTarget === quantityId || errorTarget === selectionId;
    return createElement("fieldset", { key: line.sale_line_id, disabled: locked || !eligible },
      createElement("legend", null, `Línea de venta ${line.sale_line_id} · Máximo disponible: ${line.remaining_returnable_quantity} ${line.remaining_returnable_quantity === 1 ? "unidad" : "unidades"}`),
      eligible ? createElement("label", { htmlFor: selectionId }, "Incluir este artículo",
        createElement("input", { id: selectionId, name: selectionId, type: "checkbox", checked: selected,
          disabled: locked, style: correctionControlStyle, "aria-invalid": fieldError || undefined,
          "aria-describedby": fieldError ? errorId : undefined,
          onChange: (event: ChangeEvent<HTMLInputElement>) => onAction?.({ type: "return_line_selected", sale_line_id: line.sale_line_id, selected: event.target.checked }) }))
        : createElement("p", null, "Sin unidades disponibles para devolver"),
      selected ? createElement("label", { htmlFor: quantityId }, "Cantidad a devolver",
        createElement("input", { id: quantityId, name: quantityId, type: "text", inputMode: "numeric", pattern: "[0-9]*",
          value: intent.lines[line.sale_line_id], disabled: locked, style: correctionControlStyle,
          "aria-invalid": fieldError || undefined, "aria-describedby": fieldError ? errorId : undefined,
          onChange: (event: ChangeEvent<HTMLInputElement>) => onAction?.({ type: "return_quantity_changed", sale_line_id: line.sale_line_id, value: event.target.value }) })) : null,
      fieldError ? createElement("p", { id: errorId, role: "alert" }, intent.validation!.message) : null);
  }),
  intent.error && !intent.validation ? createElement("p", { role: "alert" }, intent.error) : null,
  intent.error && !intent.validation && onReloadDetail
    ? createElement("button", { type: "button", disabled: locked, onClick: () => onReloadDetail(intent.sale_id), style: correctionControlStyle }, "Recargar detalle de venta") : null,
  createElement("button", { type: "submit", disabled: locked, style: correctionControlStyle },
    locked ? "Registrando devolución…" : "Registrar devolución")));
}

function CancellationForm({ state, onAction, onSubmit, onReloadDetail, invokerRef }: {
  state: HistoryState;
  onAction?: (action: HistoryAction) => void;
  onSubmit?: () => void;
  onReloadDetail?: (saleId: number) => void;
  invokerRef?: RefObject<HTMLElement>;
}) {
  const intent = state.cancellation_intent!;
  const locked = intent.status === "pending" || intent.status === "reload_requested";
  const fieldError = intent.validation?.focus_target;
  const errorId = fieldError ? `${fieldError}-error` : undefined;
  const preparation = createElement(FormDialog, {
    open: intent.modal_open,
    title: "Preparar cancelación de venta",
    description: "Ingresá el motivo y reconocé la corrección de inventario antes de revisar la cancelación.",
    pending: locked,
    invokerRef,
    onCancel: () => onAction?.({ type: "cancellation_intent_closed" }),
  },
  createElement("form", { "aria-label": "Preparar cancelación de venta", "aria-busy": locked,
    "data-ui-history-cancellation": true, onSubmit: (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      onAction?.({ type: "cancellation_confirmation_requested" });
    } },
  createElement("label", { htmlFor: "cancellation-reason" }, "Motivo de cancelación",
    createElement("input", { id: "cancellation-reason", name: "cancellation-reason", value: intent.reason,
      disabled: locked, style: correctionControlStyle, "aria-invalid": fieldError === "cancellation-reason" || undefined,
      "aria-describedby": fieldError === "cancellation-reason" ? errorId : undefined,
      onChange: (event: ChangeEvent<HTMLInputElement>) => onAction?.({ type: "cancellation_reason_changed", value: event.target.value }) })),
  createElement("label", { htmlFor: "cancellation-confirmation" },
    "Confirmo esta corrección de inventario. Los pagos originales no cambian.",
    createElement("input", { id: "cancellation-confirmation", name: "cancellation-confirmation", type: "checkbox",
      checked: intent.confirmed, disabled: locked, style: correctionControlStyle,
      "aria-invalid": fieldError === "cancellation-confirmation" || undefined,
      "aria-describedby": fieldError === "cancellation-confirmation" ? errorId : undefined,
      onChange: (event: ChangeEvent<HTMLInputElement>) => onAction?.({ type: "cancellation_confirmation_changed", confirmed: event.target.checked }) })),
  intent.validation ? createElement("p", { id: errorId, role: "alert" }, intent.validation.message) : null,
  intent.error && !intent.validation ? createElement("p", { role: "alert" }, intent.error) : null,
  intent.error && !intent.validation && onReloadDetail
    ? createElement("button", { type: "button", disabled: locked, style: correctionControlStyle,
      onClick: () => onReloadDetail(intent.sale_id) }, "Recargar detalle de venta") : null,
  createElement("button", { type: "submit", disabled: locked, style: correctionControlStyle }, "Continuar con la cancelación")));
  const confirmation = createElement(ConfirmationDialog, {
    open: !intent.modal_open,
    purpose: "cancellation",
    title: `Cancelar venta #${intent.sale_id}`,
    description: "Se cancelará la venta y se restaurarán únicamente las unidades todavía no devueltas. La venta y los pagos originales seguirán visibles en el historial.",
    confirmLabel: "Cancelar venta",
    pending: locked,
    pendingLabel: "Cancelando venta…",
    onCancel: () => onAction?.({ type: "cancellation_modal_closed" }),
    onConfirm: () => onSubmit ? onSubmit() : onAction?.({ type: "cancellation_submit_started" }),
  },
  createElement("p", null, `Motivo: ${intent.reason}`),
  intent.error ? createElement("p", { role: "alert" }, intent.error) : null,
  intent.error && onReloadDetail ? createElement("button", { type: "button", disabled: locked,
    onClick: () => onReloadDetail(intent.sale_id) }, "Recargar detalle de venta") : null);
  return createElement(Fragment, null, preparation, confirmation);
}

const isSparseHistoryList = (state: HistoryState) =>
  state.status === "empty" || state.status === "ready";

const isSparseHistoryDetail = (state: HistoryState) => {
  const detail = state.detail;
  return state.status === "ready" && detail !== null &&
    detail.lines.length === 1 &&
    detail.payments.length === 1 &&
    detail.returns.length === 0 &&
    detail.cancellation === null &&
    state.return_intent === null &&
    state.cancellation_intent === null;
};

export type HistoryScreenProps = {
  state: HistoryState;
  onReload: (from: string, to: string) => void;
  onSelect: (sale: SalesHistorySummary) => void;
  onBack: () => void;
  onAction?: (action: HistoryAction) => void;
  onReturnSubmit?: () => void;
  onCancellationSubmit?: () => void;
  onReloadDetail?: (saleId: number) => void;
  runEffect?: EffectRunner;
  findFocusable?: FocusFinder;
};

type HistoryCommands = Pick<typeof salesHistoryCommands, "list" | "detail">;
type InteractionCommands = HistoryCommands & Partial<typeof postSaleCommands>;
type Dispatch = (action: HistoryAction) => void;
type CorrectionIntent = ReturnIntent | CancellationIntent;
type CorrectionResponse =
  | { kind: "success"; result: { request_id: string; sale_id: number } }
  | { kind: "error"; code?: PostSaleErrorCode };

const correctionError = (code?: PostSaleErrorCode) =>
  code === "request_conflict" || code === "quantity_exceeds_remaining"
    ? "La corrección entró en conflicto con el detalle guardado. Recargá e intentá nuevamente."
    : "No se pudo guardar la corrección de inventario. Recargá e intentá nuevamente.";

export function createSalesHistoryInteraction(commands: InteractionCommands) {
  const submitting = new Set<string>();
  let requestIdentity = 0;
  let active = true;
  const beginIntent = () => ++requestIdentity;
  const isCurrent = (identity: number) => active && identity === requestIdentity;
  const invalidate = () => { requestIdentity += 1; };
  const reloadDetailFor = async (
    saleId: number,
    dispatch: Dispatch,
    identity: number,
  ) => {
    if (!isCurrent(identity)) return;
    dispatch({ type: "detail_started", sale_id: saleId });
    const response = await commands.detail(saleId);
    if (!isCurrent(identity)) return;
    dispatch(
      response.kind === "success"
        ? { type: "detail_loaded", detail: response.detail }
        : { type: "detail_failed", message: response.message },
    );
  };
  const reloadDetail = (saleId: number, dispatch: Dispatch) =>
    reloadDetailFor(saleId, dispatch, beginIntent());
  const submit = async (
    state: HistoryState,
    dispatch: Dispatch,
    isReturn: boolean,
    intent: CorrectionIntent | null,
    command: (
      intent: CorrectionIntent,
    ) => Promise<CorrectionResponse> | undefined,
  ) => {
    const started: HistoryAction = {
      type: isReturn ? "return_submit_started" : "cancellation_submit_started",
    };
    const pending = createHistoryFlow(state, started);
    dispatch(started);
    const pendingIntent = isReturn
      ? pending.return_intent
      : pending.cancellation_intent;
    if (
      !intent ||
      pendingIntent?.status !== "pending" ||
      submitting.has(intent.request_id)
    )
      return;
    submitting.add(intent.request_id);
    const identity = beginIntent();
    try {
      const response = await command(intent);
      if (!isCurrent(identity)) return;
      if (
        !response ||
        response.kind === "error" ||
        response.result.request_id !== intent.request_id ||
        response.result.sale_id !== intent.sale_id
      ) {
        dispatch({
          type: isReturn
            ? "return_submit_failed"
            : "cancellation_submit_failed",
          request_id: intent.request_id,
          message: correctionError(response?.kind === "error" ? response.code : undefined),
        });
        return;
      }
      dispatch({
        type: isReturn
          ? "return_submit_succeeded"
          : "cancellation_submit_succeeded",
        request_id: intent.request_id,
      });
      const reloaded = await commands.detail(intent.sale_id);
      if (!isCurrent(identity)) return;
      if (reloaded.kind === "success") {
        dispatch({ type: "detail_loaded", detail: reloaded.detail });
      } else {
        dispatch({
          type: isReturn ? "return_submit_failed" : "cancellation_submit_failed",
          request_id: intent.request_id,
          message: correctionError(),
        });
      }
    } finally {
      submitting.delete(intent.request_id);
    }
  };
  const submitReturn = (state: HistoryState, dispatch: Dispatch) =>
    submit(state, dispatch, true, state.return_intent, (intent) =>
      commands.createReturn?.({
        request_id: intent.request_id,
        sale_id: intent.sale_id,
        lines: Object.entries((intent as ReturnIntent).lines).map(
          ([sale_line_id, quantity]) => ({
            sale_line_id: Number(sale_line_id),
            quantity: Number(quantity),
          }),
        ),
      }),
    );
  const submitCancellation = (state: HistoryState, dispatch: Dispatch) =>
    submit(state, dispatch, false, state.cancellation_intent, (intent) =>
      commands.cancelSale?.({
        request_id: intent.request_id,
        sale_id: intent.sale_id,
        reason: (intent as CancellationIntent).reason.trim(),
      }),
    );
  return {
    reload: async (from: string, to: string, dispatch: Dispatch) => {
      const identity = beginIntent();
      dispatch({ type: "list_started" });
      const response = await commands.list(from, to);
      if (!isCurrent(identity)) return;
      dispatch(
        response.kind === "success"
          ? {
              type: "list_loaded",
              sales: response.sales,
              has_more: response.has_more,
            }
          : { type: "list_failed", message: historyListError(response.code) },
      );
    },
    select: async (sale: SalesHistorySummary, dispatch: Dispatch) => {
      await reloadDetail(sale.sale_id, dispatch);
    },
    reloadDetail,
    submitReturn,
    submitCancellation,
    invalidate,
    activate: () => { active = true; },
    dispose: () => { active = false; invalidate(); },
  };
}

export function HistoryScreen({
  state,
  onReload,
  onSelect,
  onBack,
  onAction,
  onReturnSubmit,
  onCancellationSubmit,
  onReloadDetail,
  runEffect = useEffect,
  findFocusable = findFocusableById,
}: HistoryScreenProps) {
  const correctionInvokerRef = useRef<HTMLButtonElement>(null);
  const [from, setFrom] = useState(localToday);
  const [to, setTo] = useState(localToday);
  const focusTarget = correctionFocusTarget(state);
  runEffect(() => {
    focusCorrectionTarget(focusTarget, findFocusable);
  }, [focusTarget, findFocusable]);
  const submit = (event: FormEvent<HTMLFormElement>) => { event.preventDefault(); onReload(from, to); };
  if (state.view === "detail") {
    const original = state.detail ? projectHistoryDetail(state.detail) : null;
    return createElement("main", {
      "aria-labelledby": "sales-history-detail-heading",
      "aria-busy": state.status === "loading",
      "data-ui-history-detail": true,
      "data-ui-density": isSparseHistoryDetail(state) ? "sparse" : undefined,
    },
      createElement("div", { "data-ui-history-detail-header": true },
        createElement("h1", { id: "sales-history-detail-heading" }, "Detalle de venta"),
        createElement(Action, { variant: "secondary", onClick: onBack }, "Volver al historial"),
      ),
      state.status === "loading" ? createElement(Feedback, { kind: "loading" } as never, "Cargando detalle de venta…") : null,
      state.status === "error" ? createElement(Feedback, { kind: "error" } as never, "No se pudo cargar el detalle de venta.") : null,
      state.detail && original
        ? createElement(
            "section",
            { "data-ui-history-detail-content": true },
            createElement(
              "section",
              { "aria-labelledby": "original-sale-heading", "data-ui-history-original": true },
              createElement("h2", { id: "original-sale-heading" }, "Datos originales de la venta"),
              createElement("dl", { "data-ui-history-identity": true },
                createElement("dt", null, "Venta"), createElement("dd", null, original.identity),
                createElement("dt", null, "Fecha y hora"), createElement("dd", { "data-ui-type": "numeric" }, original.date),
                createElement("dt", null, "Estado"), createElement("dd", null,
                  createElement(Badge, { kind: state.detail.status === "confirmed" ? "confirmed" : "cancelled", text: original.status }))),
              createElement("div", { "data-ui-history-summary-layout": true },
                createElement("section", { "aria-label": "Artículos originales", "data-ui-history-articles": true },
                  createElement(AlignedData, { caption: "Artículos originales", columns: originalItemColumns, rows: original.lines })),
                createElement("aside", { "aria-label": "Resumen de pago", "data-ui-history-rail": true },
                  createElement("section", { "aria-label": "Pagos originales", "data-ui-history-payments": true },
                    createElement(AlignedData, { caption: "Pagos originales", columns: originalPaymentColumns, rows: original.payments })),
                  createElement("p", { "data-ui-history-total": true }, createElement("span", null, "Total original"), createElement("strong", null, original.total)))),
            ),
            createElement(CorrectionHistory, { detail: state.detail }),
            createElement(
              "div",
              { "data-ui-history-correction-actions": true },
              (canOpenReturn(state) || state.return_intent !== null)
                ? createElement("button", { ref: correctionInvokerRef, disabled: state.return_intent !== null, type: "button", onClick: (event: MouseEvent<HTMLButtonElement>) => {
                    correctionInvokerRef.current = event.currentTarget;
                    onAction?.({ type: "return_intent_opened", request_id: crypto.randomUUID() });
                  }, style: correctionControlStyle }, "Iniciar devolución de artículos")
                : null,
              (canOpenCancellation(state) || state.cancellation_intent !== null)
                ? createElement(
                    "button",
                    {
                      ref: correctionInvokerRef,
                      disabled: state.cancellation_intent !== null,
                      type: "button",
                      onClick: (event: MouseEvent<HTMLButtonElement>) => {
                        correctionInvokerRef.current = event.currentTarget;
                        onAction?.({
                          type: "cancellation_intent_opened",
                          request_id: crypto.randomUUID(),
                        });
                      },
                      style: correctionControlStyle,
                    },
                    "Iniciar cancelación de venta",
                  )
                : null,
            ),
          )
        : null,
      state.return_intent
        ? createElement(ReturnForm, { state, onAction, onSubmit: onReturnSubmit, onReloadDetail, invokerRef: correctionInvokerRef })
        : null,
      state.cancellation_intent
        ? createElement(CancellationForm, { state, onAction, onSubmit: onCancellationSubmit, onReloadDetail, invokerRef: correctionInvokerRef })
        : null,
    );
  }
  return createElement("main", {
    "aria-labelledby": "sales-history-heading",
    "aria-busy": state.status === "loading",
    "data-ui-history-list": true,
    "data-ui-density": isSparseHistoryList(state) ? "sparse" : undefined,
  },
    createElement("h1", { id: "sales-history-heading" }, "Historial de ventas"),
    createElement("form", { onSubmit: submit, "data-ui-history-filters": true },
      createElement(Field, { kind: "date", label: "Desde", control: createElement("input", { id: "history-from", value: from, onChange: (event) => setFrom(event.target.value) }) } as never),
      createElement(Field, { kind: "date", label: "Hasta", control: createElement("input", { id: "history-to", value: to, onChange: (event) => setTo(event.target.value) }) } as never),
      createElement(Action, { variant: "primary", type: "submit", pending: state.status === "loading", pendingLabel: "Cargando…" }, "Cargar historial"),
    ),
    state.status === "idle" ? createElement(Feedback, { kind: "initial" } as never, "Elegí un rango de fechas para consultar las ventas.") : null,
    state.status === "loading" ? createElement(Feedback, { kind: "loading" } as never, "Cargando historial de ventas…") : null,
    state.status === "empty" ? createElement(Feedback, { kind: "empty" } as never, "No hay ventas en este rango.") : null,
    state.status === "error" ? createElement(Feedback, { kind: "error" } as never,
      createElement("p", null, state.message ?? "No se pudo cargar el historial de ventas."),
      createElement(Action, { variant: "secondary", onClick: () => onReload(from, to) }, "Reintentar historial"),
    ) : null,
    state.status === "ready" ? createElement(AlignedData, {
      caption: "Ventas del período",
      columns: [
        { label: "Venta", align: "start", kind: "text" },
        { label: "Fecha y hora", align: "start", kind: "text" },
        { label: "Estado", align: "start", kind: "text" },
        { label: "Artículos", align: "end", kind: "numeric" },
        { label: "Pagos", align: "start", kind: "text" },
        { label: "Total", align: "end", kind: "money" },
        { label: "Acción", align: "start", kind: "text" },
      ],
      rows: state.sales.map((sale) => {
        const cells = historySummaryCells(sale);
        return [cells.identity, cells.date,
          createElement(Badge, { kind: sale.status === "confirmed" ? "confirmed" : "cancelled", text: cells.status }),
          cells.items, cells.payments, cells.total,
          createElement(Action, { variant: "secondary", onClick: () => onSelect(sale) }, "Ver detalle")];
      }),
    }) : null,
    state.has_more ? createElement(Feedback, { kind: "advisory" } as never, "Hay más ventas. Reducí el rango de fechas.") : null,
  );
}
export function SalesHistoryScreen() {
  const [state, dispatch] = useReducer(createHistoryFlow, initialHistoryState);
  const interaction = useMemo(
    () =>
      createSalesHistoryInteraction({
        ...salesHistoryCommands,
        ...postSaleCommands,
      }),
    [],
  );
  const reload = (from: string, to: string) =>
    interaction.reload(from, to, dispatch);
  const select = (sale: SalesHistorySummary) =>
    interaction.select(sale, dispatch);
  useEffect(() => {
    interaction.activate();
    const today = localToday();
    void reload(today, today);
    return interaction.dispose;
  }, [interaction]);
  return createElement(HistoryScreen, {
    state,
    onReload: (from, to) => void reload(from, to),
    onSelect: (sale) => void select(sale),
    onBack: () => {
      interaction.invalidate();
      dispatch({ type: "back_to_list" });
    },
    onAction: dispatch,
    onReturnSubmit: () => void interaction.submitReturn(state, dispatch),
    onCancellationSubmit: () =>
      void interaction.submitCancellation(state, dispatch),
    onReloadDetail: (saleId) => void interaction.reloadDetail(saleId, dispatch),
  });
}
