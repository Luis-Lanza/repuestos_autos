import assert from "node:assert/strict";
import test from "node:test";

import { canConfirmRestore, createBackupFlow, initialBackupState } from "./backup-flow.ts";

test("keeps picker cancellation harmless and renders prepared restore confirmation", () => {
  const cancelled = createBackupFlow(createBackupFlow(initialBackupState, { type: "backup_started" }), { type: "backup_cancelled" });
  const prepared = createBackupFlow(cancelled, { type: "restore_prepared", summary: { token: "token", size_bytes: 2048, schema_version: 6 } });
  assert.equal(cancelled.feedback, "Backup selection was cancelled.");
  assert.equal(canConfirmRestore(prepared), false);
  assert.equal(canConfirmRestore(createBackupFlow(prepared, { type: "restore_confirmation_changed", confirmed: true })), true);
});

test("reports loading, stable failure, and restore success", () => {
  const loading = createBackupFlow(initialBackupState, { type: "restore_started" });
  const failed = createBackupFlow(loading, { type: "failed", message: "restore_failed: The restore could not be completed." });
  const restored = createBackupFlow(createBackupFlow(initialBackupState, { type: "restore_prepared", summary: { token: "token", size_bytes: 2048, schema_version: 6 } }), { type: "restore_succeeded" });
  assert.equal(loading.restore_status, "pending");
  assert.equal(failed.feedback, "restore_failed: The restore could not be completed.");
  assert.equal(restored.feedback, "Restore completed successfully.");
});
