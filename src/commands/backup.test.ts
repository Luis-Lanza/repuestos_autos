import assert from "node:assert/strict";
import test from "node:test";

import { createBackupCommands } from "./backup.ts";

test("uses only native picker and backup command payloads", async () => {
  const calls: unknown[] = [];
  const commands = createBackupCommands(async (name, payload) => {
    calls.push({ name, payload });
    return name === "choose_backup_destination_command" ? { kind: "cancelled" } : name === "choose_restore_source_command" ? { kind: "selected", path: "C:\\backup.sqlite3" } : name === "create_backup_command" ? { kind: "created", path: "D:\\backup.sqlite3", created_at_unix_seconds: 10, size_bytes: 20, schema_version: 6 } : name === "prepare_restore_command" ? { kind: "prepared", token: "token", size_bytes: 20, schema_version: 6 } : { kind: "restored" };
  });
  assert.deepEqual(await commands.chooseBackupDestination(), { kind: "cancelled" });
  assert.deepEqual(await commands.chooseRestoreSource(), { kind: "selected", path: "C:\\backup.sqlite3" });
  assert.equal((await commands.createBackup("D:\\")).kind, "created");
  assert.equal((await commands.prepareRestore("C:\\backup.sqlite3")).kind, "prepared");
  assert.equal((await commands.confirmRestore("token")).kind, "restored");
  assert.deepEqual(calls, [{ name: "choose_backup_destination_command", payload: {} }, { name: "choose_restore_source_command", payload: {} }, { name: "create_backup_command", payload: { request: { destination: "D:\\" } } }, { name: "prepare_restore_command", payload: { request: { source: "C:\\backup.sqlite3" } } }, { name: "confirm_restore_command", payload: { request: { token: "token", confirmed: true } } }]);
});

test("maps malformed and failed IPC responses to stable backup errors", async () => {
  const commands = createBackupCommands(async () => { throw new Error("sqlite path /internal"); });
  assert.deepEqual(await commands.createBackup("D:\\"), { kind: "error", code: "storage_unavailable", message: "Backup storage is unavailable." });
});
