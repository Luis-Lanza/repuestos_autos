import type {
  SalesHistoryDetail,
  SalesHistorySummary,
} from "../../commands/sales-history.ts";

export type CorrectionValidation = {
  message: string;
  focus_target: string | null;
};

export type ReturnIntent = {
  sale_id: number;
  request_id: string;
  lines: Record<number, string>;
  status: "open" | "pending" | "error" | "reload_requested";
  error: string | null;
  validation: CorrectionValidation | null;
};

export type CancellationIntent = {
  sale_id: number;
  request_id: string;
  reason: string;
  confirmed: boolean;
  status: "open" | "pending" | "error" | "reload_requested";
  error: string | null;
  validation: CorrectionValidation | null;
};

export type HistoryState = {
  view: "list" | "detail";
  status: "idle" | "loading" | "ready" | "empty" | "error";
  sales: SalesHistorySummary[];
  has_more: boolean;
  selected_id: number | null;
  detail: SalesHistoryDetail | null;
  message: string | null;
  return_intent: ReturnIntent | null;
  cancellation_intent: CancellationIntent | null;
};

export const initialHistoryState: HistoryState = {
  view: "list",
  status: "idle",
  sales: [],
  has_more: false,
  selected_id: null,
  detail: null,
  message: null,
  return_intent: null,
  cancellation_intent: null,
};

export type HistoryAction =
  | { type: "list_started" }
  | { type: "list_loaded"; sales: SalesHistorySummary[]; has_more: boolean }
  | { type: "list_failed"; message: string }
  | { type: "detail_started"; sale_id: number }
  | { type: "detail_loaded"; detail: SalesHistoryDetail }
  | { type: "detail_failed"; message: string }
  | { type: "return_intent_opened"; request_id: string }
  | { type: "return_line_selected"; sale_line_id: number; selected: boolean }
  | { type: "return_quantity_changed"; sale_line_id: number; value: string }
  | { type: "return_submit_started" }
  | { type: "return_submit_failed"; message: string }
  | { type: "return_submit_succeeded" }
  | { type: "cancellation_intent_opened"; request_id: string }
  | { type: "cancellation_reason_changed"; value: string }
  | { type: "cancellation_confirmation_changed"; confirmed: boolean }
  | { type: "cancellation_submit_started" }
  | { type: "cancellation_submit_failed"; message: string }
  | { type: "cancellation_submit_succeeded" }
  | { type: "back_to_list" };

const returnableLine = (detail: SalesHistoryDetail, saleLineId: number) =>
  detail.lines.find((line) => line.sale_line_id === saleLineId);

export function canOpenReturn(state: HistoryState): boolean {
  return (
    state.return_intent === null &&
    state.detail?.status === "confirmed" &&
    state.detail.lines.some(
      (line) =>
        Number.isSafeInteger(line.remaining_returnable_quantity) &&
        line.remaining_returnable_quantity > 0,
    )
  );
}

export function canOpenCancellation(state: HistoryState): boolean {
  return (
    state.cancellation_intent === null && state.detail?.status === "confirmed"
  );
}

function returnFocusTarget(
  state: HistoryState,
  intent: ReturnIntent,
): string | null {
  const selectedLineId = Object.keys(intent.lines)
    .map(Number)
    .filter(Number.isSafeInteger)
    .sort((left, right) => left - right)[0];
  if (selectedLineId !== undefined) return `return-quantity-${selectedLineId}`;
  const firstReturnableLineId = state.detail?.lines
    .filter(
      (line) =>
        Number.isSafeInteger(line.remaining_returnable_quantity) &&
        line.remaining_returnable_quantity > 0,
    )
    .map((line) => line.sale_line_id)
    .sort((left, right) => left - right)[0];
  return firstReturnableLineId === undefined
    ? null
    : `return-line-${firstReturnableLineId}`;
}

function returnIntentValidation(
  state: HistoryState,
  intent: ReturnIntent,
): CorrectionValidation | null {
  if (
    state.detail?.sale_id !== intent.sale_id ||
    state.detail.status !== "confirmed"
  )
    return {
      message: "This sale is no longer eligible for returns. Reload history.",
      focus_target: returnFocusTarget(state, intent),
    };
  const entries = Object.entries(intent.lines).sort(
    ([left], [right]) => Number(left) - Number(right),
  );
  if (!entries.length)
    return {
      message: "Select at least one return line.",
      focus_target: returnFocusTarget(state, intent),
    };
  for (const [saleLineId, value] of entries) {
    const line = returnableLine(state.detail, Number(saleLineId));
    const focusTarget = `return-quantity-${saleLineId}`;
    if (
      !line ||
      !Number.isSafeInteger(line.remaining_returnable_quantity) ||
      line.remaining_returnable_quantity <= 0
    )
      return {
        message: "This line is no longer eligible for return.",
        focus_target: focusTarget,
      };
    if (!/^[1-9]\d*$/.test(value))
      return {
        message: "Return quantities must be positive whole numbers.",
        focus_target: focusTarget,
      };
    const quantity = Number(value);
    if (!Number.isSafeInteger(quantity))
      return {
        message: "Return quantities must be positive whole numbers.",
        focus_target: focusTarget,
      };
    if (quantity > line.remaining_returnable_quantity)
      return {
        message:
          "Return quantity exceeds the persisted remaining availability.",
        focus_target: focusTarget,
      };
  }
  return null;
}

function editableReturnIntent(intent: ReturnIntent): boolean {
  return intent.status === "open" || intent.status === "error";
}

function cancellationIntentValidation(
  state: HistoryState,
  intent: CancellationIntent,
): CorrectionValidation | null {
  if (
    state.detail?.sale_id !== intent.sale_id ||
    state.detail.status !== "confirmed"
  )
    return {
      message:
        "This sale is no longer eligible for cancellation. Reload history.",
      focus_target: "cancellation-reason",
    };
  if (!intent.reason)
    return {
      message: "A cancellation reason is required.",
      focus_target: "cancellation-reason",
    };
  if (!intent.confirmed)
    return {
      message: "Confirm the cancellation before submitting.",
      focus_target: "cancellation-confirmation",
    };
  return null;
}

export function correctionFocusTarget(state: HistoryState): string | null {
  if (state.return_intent?.status === "error")
    return state.return_intent.validation?.focus_target ?? null;
  if (state.cancellation_intent?.status === "error")
    return state.cancellation_intent.validation?.focus_target ?? null;
  return null;
}

function editableCancellationIntent(intent: CancellationIntent): boolean {
  return intent.status === "open" || intent.status === "error";
}

export function createHistoryFlow(
  state: HistoryState,
  action: HistoryAction,
): HistoryState {
  switch (action.type) {
    case "list_started":
      return {
        ...state,
        view: "list",
        status: "loading",
        detail: null,
        message: null,
        return_intent: null,
        cancellation_intent: null,
      };
    case "list_loaded":
      return {
        ...state,
        view: "list",
        status: action.sales.length ? "ready" : "empty",
        sales: action.sales,
        has_more: action.has_more,
        selected_id: null,
        detail: null,
        message: null,
        return_intent: null,
        cancellation_intent: null,
      };
    case "list_failed":
      return {
        ...state,
        view: "list",
        status: "error",
        sales: [],
        has_more: false,
        detail: null,
        message: action.message,
        return_intent: null,
        cancellation_intent: null,
      };
    case "detail_started":
      return {
        ...state,
        view: "detail",
        status: "loading",
        selected_id: action.sale_id,
        detail: null,
        message: null,
        return_intent: null,
        cancellation_intent: null,
      };
    case "detail_loaded":
      return {
        ...state,
        view: "detail",
        status: "ready",
        selected_id: action.detail.sale_id,
        detail: action.detail,
        message: null,
        return_intent: null,
        cancellation_intent: null,
      };
    case "detail_failed":
      return {
        ...state,
        view: "detail",
        status: "error",
        detail: null,
        message: action.message,
      };
    case "return_intent_opened":
      return canOpenReturn(state)
        ? {
            ...state,
            return_intent: {
              sale_id: state.detail!.sale_id,
              request_id: action.request_id,
              lines: {},
              status: "open",
              error: null,
              validation: null,
            },
          }
        : state;
    case "return_line_selected": {
      const intent = state.return_intent;
      const line =
        state.detail && returnableLine(state.detail, action.sale_line_id);
      if (
        !intent ||
        !editableReturnIntent(intent) ||
        !line ||
        line.remaining_returnable_quantity <= 0
      )
        return state;
      const lines = { ...intent.lines };
      if (action.selected) lines[action.sale_line_id] ??= "";
      else delete lines[action.sale_line_id];
      return {
        ...state,
        return_intent: {
          ...intent,
          lines,
          status: "open",
          error: null,
          validation: null,
        },
      };
    }
    case "return_quantity_changed": {
      const intent = state.return_intent;
      if (
        !intent ||
        !editableReturnIntent(intent) ||
        !(action.sale_line_id in intent.lines)
      )
        return state;
      return {
        ...state,
        return_intent: {
          ...intent,
          lines: { ...intent.lines, [action.sale_line_id]: action.value },
          status: "open",
          error: null,
          validation: null,
        },
      };
    }
    case "return_submit_started": {
      const intent = state.return_intent;
      if (
        !intent ||
        intent.status === "pending" ||
        intent.status === "reload_requested"
      )
        return state;
      const validation = returnIntentValidation(state, intent);
      return {
        ...state,
        return_intent: validation
          ? {
              ...intent,
              status: "error",
              error: validation.message,
              validation,
            }
          : { ...intent, status: "pending", error: null, validation: null },
      };
    }
    case "return_submit_failed":
      return state.return_intent?.status === "pending"
        ? {
            ...state,
            return_intent: {
              ...state.return_intent,
              status: "error",
              error: action.message,
              validation: null,
            },
          }
        : state;
    case "return_submit_succeeded":
      return state.return_intent?.status === "pending"
        ? {
            ...state,
            return_intent: {
              ...state.return_intent,
              status: "reload_requested",
              error: null,
              validation: null,
            },
          }
        : state;
    case "cancellation_intent_opened":
      return canOpenCancellation(state)
        ? {
            ...state,
            cancellation_intent: {
              sale_id: state.detail!.sale_id,
              request_id: action.request_id,
              reason: "",
              confirmed: false,
              status: "open",
              error: null,
              validation: null,
            },
          }
        : state;
    case "cancellation_reason_changed": {
      const intent = state.cancellation_intent;
      if (!intent || !editableCancellationIntent(intent)) return state;
      return {
        ...state,
        cancellation_intent: {
          ...intent,
          reason: action.value.trim(),
          status: "open",
          error: null,
          validation: null,
        },
      };
    }
    case "cancellation_confirmation_changed": {
      const intent = state.cancellation_intent;
      if (!intent || !editableCancellationIntent(intent)) return state;
      return {
        ...state,
        cancellation_intent: {
          ...intent,
          confirmed: action.confirmed,
          status: "open",
          error: null,
          validation: null,
        },
      };
    }
    case "cancellation_submit_started": {
      const intent = state.cancellation_intent;
      if (!intent || !editableCancellationIntent(intent)) return state;
      const validation = cancellationIntentValidation(state, intent);
      return {
        ...state,
        cancellation_intent: validation
          ? {
              ...intent,
              status: "error",
              error: validation.message,
              validation,
            }
          : { ...intent, status: "pending", error: null, validation: null },
      };
    }
    case "cancellation_submit_failed":
      return state.cancellation_intent?.status === "pending"
        ? {
            ...state,
            cancellation_intent: {
              ...state.cancellation_intent,
              status: "error",
              error: action.message,
              validation: null,
            },
          }
        : state;
    case "cancellation_submit_succeeded":
      return state.cancellation_intent?.status === "pending"
        ? {
            ...state,
            cancellation_intent: {
              ...state.cancellation_intent,
              status: "reload_requested",
              error: null,
              validation: null,
            },
          }
        : state;
    case "back_to_list":
      return {
        ...state,
        view: "list",
        status: state.sales.length ? "ready" : "empty",
        selected_id: null,
        detail: null,
        message: null,
        return_intent: null,
        cancellation_intent: null,
      };
  }
}
