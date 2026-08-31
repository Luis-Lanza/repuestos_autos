import { invoke } from "@tauri-apps/api/core";

type Invoke = (
  command: string,
  payload: { request: unknown },
) => Promise<unknown>;
type RecordValue = Record<string, unknown>;

export type PostSaleErrorCode =
  | "invalid_request"
  | "invalid_quantity"
  | "duplicate_sale_line"
  | "sale_not_found"
  | "sale_not_confirmed"
  | "sale_cancelled"
  | "sale_line_not_found"
  | "quantity_exceeds_remaining"
  | "cancellation_reason_required"
  | "cancellation_already_recorded"
  | "request_conflict"
  | "persistence_failure";

export type PostSaleError = {
  kind: "error";
  code: PostSaleErrorCode;
  message: string;
};

export type ReturnLine = {
  sale_line_id: number;
  product_id: number;
  quantity: number;
};
export type ReturnResult = {
  request_id: string;
  return_id: number;
  sale_id: number;
  status: "confirmed";
  occurred_at: string;
  lines: ReturnLine[];
};
export type CancellationLine = {
  sale_line_id: number;
  product_id: number;
  restored_quantity: number;
};
export type CancellationResult = {
  request_id: string;
  cancellation_id: number;
  sale_id: number;
  status: "cancelled";
  occurred_at: string;
  reason: string;
  lines: CancellationLine[];
};
export type ReturnResponse =
  { kind: "success"; result: ReturnResult } | PostSaleError;
export type CancellationResponse =
  { kind: "success"; result: CancellationResult } | PostSaleError;

export type ReturnRequest = {
  request_id: string;
  sale_id: number;
  lines: { sale_line_id: number; quantity: number }[];
};
export type CancellationRequest = {
  request_id: string;
  sale_id: number;
  reason: string;
};

type ReturnIntent = {
  request_id: string;
  submit: () => Promise<ReturnResponse>;
};
type CancellationIntent = {
  request_id: string;
  submit: () => Promise<CancellationResponse>;
};

const errorCodes = new Set<PostSaleErrorCode>([
  "invalid_request",
  "invalid_quantity",
  "duplicate_sale_line",
  "sale_not_found",
  "sale_not_confirmed",
  "sale_cancelled",
  "sale_line_not_found",
  "quantity_exceeds_remaining",
  "cancellation_reason_required",
  "cancellation_already_recorded",
  "request_conflict",
  "persistence_failure",
]);

const failure = (): PostSaleError => ({
  kind: "error",
  code: "persistence_failure",
  message: "The inventory correction could not be completed.",
});

const record = (value: unknown): RecordValue | undefined =>
  typeof value === "object" && value !== null
    ? (value as RecordValue)
    : undefined;
const string = (value: unknown): value is string => typeof value === "string";
const number = (value: unknown): value is number => typeof value === "number";

const returnLine = (value: unknown): ReturnLine | undefined => {
  const line = record(value);
  if (
    !line ||
    !number(line.sale_line_id) ||
    !number(line.product_id) ||
    !number(line.quantity)
  )
    return;
  return {
    sale_line_id: line.sale_line_id,
    product_id: line.product_id,
    quantity: line.quantity,
  };
};
const cancellationLine = (value: unknown): CancellationLine | undefined => {
  const line = record(value);
  if (
    !line ||
    !number(line.sale_line_id) ||
    !number(line.product_id) ||
    !number(line.restored_quantity)
  )
    return;
  return {
    sale_line_id: line.sale_line_id,
    product_id: line.product_id,
    restored_quantity: line.restored_quantity,
  };
};
const lines = <Line>(
  value: unknown,
  decode: (line: unknown) => Line | undefined,
): Line[] | undefined => {
  if (!Array.isArray(value)) return;
  const decoded = value.map(decode);
  return decoded.every((line): line is Line => line !== undefined)
    ? decoded
    : undefined;
};

const postSaleError = (value: RecordValue): PostSaleError =>
  string(value.code) && errorCodes.has(value.code as PostSaleErrorCode)
    ? {
        kind: "error",
        code: value.code as PostSaleErrorCode,
        message: failure().message,
      }
    : failure();

const returnResponse = (value: unknown): ReturnResponse => {
  const response = record(value);
  if (!response) return failure();
  if (response.kind === "error") return postSaleError(response);
  const result = record(response.result);
  const decodedLines = result && lines(result.lines, returnLine);
  if (
    response.kind !== "success" ||
    !result ||
    !string(result.request_id) ||
    !number(result.return_id) ||
    !number(result.sale_id) ||
    result.status !== "confirmed" ||
    !string(result.occurred_at) ||
    !decodedLines
  )
    return failure();
  return {
    kind: "success",
    result: {
      request_id: result.request_id,
      return_id: result.return_id,
      sale_id: result.sale_id,
      status: "confirmed",
      occurred_at: result.occurred_at,
      lines: decodedLines,
    },
  };
};

const cancellationResponse = (value: unknown): CancellationResponse => {
  const response = record(value);
  if (!response) return failure();
  if (response.kind === "error") return postSaleError(response);
  const result = record(response.result);
  const decodedLines = result && lines(result.lines, cancellationLine);
  if (
    response.kind !== "success" ||
    !result ||
    !string(result.request_id) ||
    !number(result.cancellation_id) ||
    !number(result.sale_id) ||
    result.status !== "cancelled" ||
    !string(result.occurred_at) ||
    !string(result.reason) ||
    !decodedLines
  )
    return failure();
  return {
    kind: "success",
    result: {
      request_id: result.request_id,
      cancellation_id: result.cancellation_id,
      sale_id: result.sale_id,
      status: "cancelled",
      occurred_at: result.occurred_at,
      reason: result.reason,
      lines: decodedLines,
    },
  };
};

export function createPostSaleCommands(
  command: Invoke,
  newRequestId: () => string = () => crypto.randomUUID(),
) {
  const createReturn = (request: ReturnRequest) =>
    command("create_sale_return_command", { request }).then(
      returnResponse,
      failure,
    );
  const cancelSale = (request: CancellationRequest) =>
    command("cancel_sale_command", { request }).then(
      cancellationResponse,
      failure,
    );
  const beginReturn = ({
    sale_id,
    lines,
  }: Omit<ReturnRequest, "request_id">): ReturnIntent => {
    const request_id = newRequestId();
    return {
      request_id,
      submit: () => createReturn({ request_id, sale_id, lines }),
    };
  };
  const beginCancellation = ({
    sale_id,
    reason,
  }: Omit<CancellationRequest, "request_id">): CancellationIntent => {
    const request_id = newRequestId();
    return {
      request_id,
      submit: () => cancelSale({ request_id, sale_id, reason }),
    };
  };
  return { createReturn, cancelSale, beginReturn, beginCancellation };
}

export const postSaleCommands = createPostSaleCommands(invoke);
