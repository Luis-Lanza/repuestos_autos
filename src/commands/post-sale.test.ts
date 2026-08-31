import assert from "node:assert/strict";
import test from "node:test";

import { createPostSaleCommands } from "./post-sale.ts";

test("post-sale commands send snake_case payloads and retain intent UUIDs across retries", async () => {
  const calls: unknown[] = [];
  const commands = createPostSaleCommands(
    async (command, payload) => {
      calls.push({ command, payload });
      return command === "create_sale_return_command"
        ? {
            kind: "success",
            result: {
              request_id: "550e8400-e29b-41d4-a716-446655440001",
              return_id: 5,
              sale_id: 7,
              status: "confirmed",
              occurred_at: "2026-03-08T12:00:00Z",
              lines: [{ sale_line_id: 11, product_id: 1, quantity: 2 }],
            },
          }
        : {
            kind: "success",
            result: {
              request_id: "550e8400-e29b-41d4-a716-446655440002",
              cancellation_id: 6,
              sale_id: 7,
              status: "cancelled",
              occurred_at: "2026-03-08T12:01:00Z",
              reason: "inventory correction",
              lines: [
                { sale_line_id: 11, product_id: 1, restored_quantity: 0 },
              ],
            },
          };
    },
    () => "550e8400-e29b-41d4-a716-446655440001",
  );

  const retry = commands.beginReturn({
    sale_id: 7,
    lines: [{ sale_line_id: 11, quantity: 2 }],
  });
  const returned = await retry.submit();
  const replayed = await retry.submit();
  const cancelled = await commands
    .beginCancellation({ sale_id: 7, reason: "inventory correction" })
    .submit();

  assert.deepEqual(returned, replayed);
  assert.equal(returned.kind, "success");
  assert.equal(cancelled.kind, "success");

  assert.equal(retry.request_id, "550e8400-e29b-41d4-a716-446655440001");
  assert.deepEqual(calls, [
    {
      command: "create_sale_return_command",
      payload: {
        request: {
          request_id: "550e8400-e29b-41d4-a716-446655440001",
          sale_id: 7,
          lines: [{ sale_line_id: 11, quantity: 2 }],
        },
      },
    },
    {
      command: "create_sale_return_command",
      payload: {
        request: {
          request_id: "550e8400-e29b-41d4-a716-446655440001",
          sale_id: 7,
          lines: [{ sale_line_id: 11, quantity: 2 }],
        },
      },
    },
    {
      command: "cancel_sale_command",
      payload: {
        request: {
          request_id: "550e8400-e29b-41d4-a716-446655440001",
          sale_id: 7,
          reason: "inventory correction",
        },
      },
    },
  ]);
});

test("post-sale commands guard result shapes and normalize errors", async () => {
  const commands = createPostSaleCommands(
    async (command) =>
      command === "create_sale_return_command"
        ? { kind: "error", code: "request_conflict", message: "unexpected" }
        : { kind: "success", result: { sale_id: "wrong" } },
    () => "id",
  );

  assert.deepEqual(
    await commands.createReturn({ request_id: "id", sale_id: 7, lines: [] }),
    {
      kind: "error",
      code: "request_conflict",
      message: "The inventory correction could not be completed.",
    },
  );
  assert.deepEqual(
    await commands.cancelSale({ request_id: "id", sale_id: 7, reason: "x" }),
    {
      kind: "error",
      code: "persistence_failure",
      message: "The inventory correction could not be completed.",
    },
  );
  assert.deepEqual(
    await createPostSaleCommands(async () => {
      throw new Error("sqlite refund");
    }).createReturn({ request_id: "id", sale_id: 7, lines: [] }),
    {
      kind: "error",
      code: "persistence_failure",
      message: "The inventory correction could not be completed.",
    },
  );
});

test("post-sale success projections discard untrusted fields and reject malformed lines", async () => {
  const commands = createPostSaleCommands(async (command) =>
    command === "create_sale_return_command"
      ? {
          kind: "success",
          result: {
            request_id: "return-id",
            return_id: 1,
            sale_id: 2,
            status: "confirmed",
            occurred_at: "now",
            refund_amount: 999,
            lines: [
              { sale_line_id: 3, product_id: 4, quantity: 5, sql: "select" },
            ],
          },
          schema: "unexpected",
        }
      : {
          kind: "success",
          result: {
            request_id: "cancel-id",
            cancellation_id: 6,
            sale_id: 2,
            status: "cancelled",
            occurred_at: "later",
            reason: "normalized",
            driver: "unexpected",
            lines: [
              {
                sale_line_id: 3,
                product_id: 4,
                restored_quantity: 0,
                refund_amount: 999,
              },
            ],
          },
        },
  );

  assert.deepEqual(
    await commands.createReturn({ request_id: "id", sale_id: 2, lines: [] }),
    {
      kind: "success",
      result: {
        request_id: "return-id",
        return_id: 1,
        sale_id: 2,
        status: "confirmed",
        occurred_at: "now",
        lines: [{ sale_line_id: 3, product_id: 4, quantity: 5 }],
      },
    },
  );
  assert.deepEqual(
    await commands.cancelSale({ request_id: "id", sale_id: 2, reason: "x" }),
    {
      kind: "success",
      result: {
        request_id: "cancel-id",
        cancellation_id: 6,
        sale_id: 2,
        status: "cancelled",
        occurred_at: "later",
        reason: "normalized",
        lines: [{ sale_line_id: 3, product_id: 4, restored_quantity: 0 }],
      },
    },
  );

  const malformed = createPostSaleCommands(async (command) =>
    command === "create_sale_return_command"
      ? {
          kind: "success",
          result: {
            request_id: "id",
            return_id: 1,
            sale_id: 2,
            status: "confirmed",
            occurred_at: "now",
            lines: [{ sale_line_id: 3, product_id: 4, quantity: "5" }],
          },
        }
      : {
          kind: "success",
          result: {
            request_id: "id",
            cancellation_id: 1,
            sale_id: 2,
            status: "cancelled",
            occurred_at: "now",
            reason: "x",
            lines: [{ sale_line_id: 3, product_id: 4, restored_quantity: "0" }],
          },
        },
  );
  const expected = {
    kind: "error",
    code: "persistence_failure",
    message: "The inventory correction could not be completed.",
  };
  assert.deepEqual(
    await malformed.createReturn({ request_id: "id", sale_id: 2, lines: [] }),
    expected,
  );
  assert.deepEqual(
    await malformed.cancelSale({ request_id: "id", sale_id: 2, reason: "x" }),
    expected,
  );
});
