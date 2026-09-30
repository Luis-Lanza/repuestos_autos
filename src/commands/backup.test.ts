import assert from "node:assert/strict";
import test from "node:test";

import { createBackupCommands } from "./backup.ts";

test("uses opaque native picker tokens for backup and restore", async () => {
  const calls: unknown[] = [];
  const commands = createBackupCommands(async (name, payload) => {
    calls.push({ name, payload });
    return name === "choose_backup_destination_command" ? { kind: "selected", token: "opaque-destination-token" } : name === "choose_restore_source_command" ? { kind: "selected", token: "opaque-source-token" } : name === "create_backup_command" ? { kind: "created", file_name: "backup-10-id.sqlite3", created_at_unix_seconds: 10, size_bytes: 20, schema_version: 6, durability_warning: false, cleanup_warning: false } : name === "prepare_restore_command" ? { kind: "prepared", token: "token", size_bytes: 20, schema_version: 6 } : { kind: "restored" };
  });
  assert.deepEqual(await commands.chooseBackupDestination(), { kind: "selected", token: "opaque-destination-token" });
  assert.deepEqual(await commands.chooseRestoreSource(), { kind: "selected", token: "opaque-source-token" });
  assert.deepEqual(await commands.createBackup("opaque-destination-token"), { kind: "created", summary: { file_name: "backup-10-id.sqlite3", created_at_unix_seconds: 10, size_bytes: 20, schema_version: 6, durability_warning: false, cleanup_warning: false } });
  assert.equal((await commands.prepareRestore("opaque-source-token")).kind, "prepared");
  assert.equal((await commands.confirmRestore("token")).kind, "restored");
  assert.deepEqual(calls, [{ name: "choose_backup_destination_command", payload: {} }, { name: "choose_restore_source_command", payload: {} }, { name: "create_backup_command", payload: { request: { destination_token: "opaque-destination-token" } } }, { name: "prepare_restore_command", payload: { request: { source_token: "opaque-source-token" } } }, { name: "confirm_restore_command", payload: { request: { token: "token", confirmed: true } } }]);
});

test("decodes picker error envelopes returned as resolved Tauri responses", async () => {
  const commands = createBackupCommands(async (name) => name === "choose_backup_destination_command"
    ? { kind: "error", code: "unsupported_destination", message: "native detail" }
    : { kind: "error", code: "storage_unavailable", message: "native detail" });

  assert.deepEqual(await commands.chooseBackupDestination(), {
    kind: "error",
    code: "unsupported_destination",
    message: "Backup storage is unavailable.",
  });
  assert.deepEqual(await commands.chooseRestoreSource(), {
    kind: "error",
    code: "storage_unavailable",
    message: "Backup storage is unavailable.",
  });
});

test("keeps cancellation distinct and rejects picker paths or path-bearing backup responses", async () => {
  const cancelled = createBackupCommands(async () => ({ kind: "cancelled" }));
  assert.deepEqual(await cancelled.chooseBackupDestination(), { kind: "cancelled" });
  assert.deepEqual(await cancelled.chooseRestoreSource(), { kind: "cancelled" });

  const forgedPath = createBackupCommands(async () => ({ kind: "selected", path: "C:\\secret" }));
  assert.deepEqual(await forgedPath.chooseBackupDestination(), { kind: "error", code: "storage_unavailable", message: "Backup storage is unavailable." });
  assert.deepEqual(await forgedPath.chooseRestoreSource(), { kind: "error", code: "storage_unavailable", message: "Backup storage is unavailable." });
  assert.equal((await createBackupCommands(async () => ({ kind: "created", path: "C:\\secret", created_at_unix_seconds: 1, size_bytes: 1, schema_version: 6, durability_warning: false, cleanup_warning: false })).createBackup("token")).kind, "error");
});

test("maps license-required restore denial to accurate safe feedback", async () => {
  const commands = createBackupCommands(async () => ({ kind: "error", code: "license_required", message: "native detail that must not be trusted" }));
  assert.deepEqual(await commands.prepareRestore("opaque-source-token"), {
    kind: "error",
    code: "license_required",
    message: "A valid license is required to restore a backup.",
  });
});

test("maps malformed and failed IPC responses to stable backup errors", async () => {
  const commands = createBackupCommands(async () => { throw new Error("sqlite path /internal"); });
  assert.deepEqual(await commands.createBackup("opaque-token"), { kind: "error", code: "storage_unavailable", message: "Backup storage is unavailable." });
});
