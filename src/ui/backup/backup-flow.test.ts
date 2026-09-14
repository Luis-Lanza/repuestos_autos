import assert from "node:assert/strict";
import test from "node:test";

import { backupFeedback, canConfirmRestore, createBackupFlow, initialBackupState, restoreErrorStatus } from "./backup-flow.ts";

test("keeps picker cancellation harmless and renders prepared restore confirmation", () => {
  const cancelled = createBackupFlow(createBackupFlow(initialBackupState, { type: "backup_started" }), { type: "backup_cancelled" });
  const prepared = createBackupFlow(cancelled, { type: "restore_prepared", summary: { token: "token", size_bytes: 2048, schema_version: 6 } });
  assert.equal(cancelled.feedback, "Selección de copia cancelada.");
  assert.equal(prepared.restore_status, "prepared");
  assert.equal(canConfirmRestore(prepared), false);
  assert.equal(canConfirmRestore(createBackupFlow(prepared, { type: "restore_confirmation_changed", confirmed: true })), true);
});

test("reports loading, stable failure, and restore success", () => {
  const loading = createBackupFlow(initialBackupState, { type: "restore_started" });
  const failed = createBackupFlow(loading, { type: "failed", message: "restore_failed: The restore could not be completed." });
  const restored = createBackupFlow(createBackupFlow(initialBackupState, { type: "restore_prepared", summary: { token: "token", size_bytes: 2048, schema_version: 6 } }), { type: "restore_succeeded" });
  assert.equal(loading.restore_status, "pending");
  assert.equal(failed.feedback, "restore_failed: The restore could not be completed.");
  assert.equal(restored.feedback, "Restauración completada correctamente.");
});

test("keeps restore status meanings and acknowledgement gating distinct", () => {
  const candidate = createBackupFlow(initialBackupState, { type: "restore_prepared", summary: { token: "t", size_bytes: 1, schema_version: 6 } });
  assert.equal(createBackupFlow(candidate, { type: "restore_failed", status: "invalid", message: "inválido" }).restore_status, "invalid");
  assert.equal(createBackupFlow(candidate, { type: "restore_failed", status: "expired", message: "venció" }).restore_status, "expired");
  assert.equal(createBackupFlow(candidate, { type: "restore_failed", status: "unavailable", message: "no disponible" }).restore_status, "unavailable");
  assert.equal(createBackupFlow(candidate, { type: "restore_failed", status: "recovery", message: "reintentar" }).restore_status, "recovery");
  const confirmed = createBackupFlow(candidate, { type: "restore_confirmation_changed", confirmed: true });
  assert.equal(canConfirmRestore(confirmed), true);
  assert.equal(canConfirmRestore(createBackupFlow(confirmed, { type: "restore_confirm_started" })), false);
});

test("maps native error codes to bounded Spanish recovery copy", () => {
  assert.equal(backupFeedback("invalid_backup", true), "El archivo de respaldo no es válido.");
  assert.equal(backupFeedback("token_expired", true), "La preparación de restauración venció. Elegí el archivo nuevamente.");
  assert.equal(backupFeedback("destination_exists"), "Ya existe una copia en ese destino. Elegí otro.");
  assert.equal(restoreErrorStatus("unsupported_schema"), "invalid");
});
