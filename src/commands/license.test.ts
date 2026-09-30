import assert from "node:assert/strict";
import test from "node:test";
import { createLicenseCommands, LICENSE_STATUS } from "./license.ts";

test("decodes each license IPC response without trusting the invoke type", async () => {
  const values: unknown[] = [
    { kind: "status", code: "active" },
    { kind: "code", code: "a".repeat(64) },
    { kind: "selected", path: "C:\\private\\license.lic" },
    { kind: "imported", status: "active" },
  ];
  const commands = createLicenseCommands(async () => values.shift());
  assert.deepEqual(await commands.status(), { kind: "status", code: "active" });
  assert.deepEqual(await commands.installationCode(), { kind: "code", code: "a".repeat(64) });
  assert.deepEqual(await commands.chooseFile(), { kind: "selected" });
  assert.deepEqual(await commands.importFile(), { kind: "imported", status: "active" });
});

test("bounds malformed IPC, preserves discriminants, and treats cancellation as harmless", async () => {
  const values: unknown[] = [null, { kind: "code", code: "raw machine id" }, { kind: "cancelled", path: "C:\\secret" }, { kind: "cancelled" }, { kind: "error", code: "native-path" }];
  const commands = createLicenseCommands(async () => values.shift());
  assert.deepEqual(await commands.status(), { kind: "error", code: LICENSE_STATUS.STORAGE_UNAVAILABLE });
  assert.deepEqual(await commands.installationCode(), { kind: "error", code: LICENSE_STATUS.IDENTITY_UNAVAILABLE });
  assert.deepEqual(await commands.chooseFile(), { kind: "cancelled" });
  assert.deepEqual(await commands.importFile(), { kind: "cancelled" });
  assert.deepEqual(await commands.status(), { kind: "error", code: LICENSE_STATUS.STORAGE_UNAVAILABLE });
});

test("maps rejected invoke promises to bounded errors", async () => {
  const commands = createLicenseCommands(async () => { throw new Error("C:\\private\\diagnostic"); });
  assert.deepEqual(await commands.status(), { kind: "error", code: LICENSE_STATUS.STORAGE_UNAVAILABLE });
  assert.deepEqual(await commands.chooseFile(), { kind: "error", code: LICENSE_STATUS.FILE_INVALID });
});
