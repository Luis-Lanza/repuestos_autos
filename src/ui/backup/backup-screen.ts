import { createElement as h, useEffect, useReducer, useRef, useState } from "react";

import { useActionNotifications } from "../visual-system/action-notifications.ts";
import { backupCommands } from "../../commands/backup.ts";
import { Action, Feedback, Field } from "../visual-system/controls.ts";
import { ConfirmationDialog } from "../visual-system/confirmation-dialog.ts";
import { Panel } from "../visual-system/structure.ts";
import { canConfirmRestore, createBackupFlow, createBackupInteraction, initialBackupState, type BackupAction } from "./backup-flow.ts";

const date = (seconds: number) => { const value = new Date(seconds * 1000); return Number.isNaN(value.getTime()) ? "Fecha no disponible" : `${String(value.getUTCDate()).padStart(2, "0")}/${String(value.getUTCMonth() + 1).padStart(2, "0")}/${value.getUTCFullYear()}, ${String(value.getUTCHours()).padStart(2, "0")}:${String(value.getUTCMinutes()).padStart(2, "0")}`; };
const backupState = (status: string) => status === "failure" || status === "unavailable" || status === "success";
const restoreState = (status: string) => ["invalid", "expired", "unavailable", "failure", "recovery", "success"].includes(status);

export function BackupScreen({ canRestore = true }: { canRestore?: boolean } = {}) {
  const [state, dispatch] = useReducer(createBackupFlow, initialBackupState);
  const [dialogOpen, setDialogOpen] = useState(false);
  const notifications = useActionNotifications();
  const [sharedOutcome, setSharedOutcome] = useState(false);
  // The interaction dispatches completions only after its mounted/generation guards.
  const accept = (action: BackupAction) => {
    let message: string | null = null;
    let severity: "success" | "error" | "warning" = "error";
    if (action.type === "backup_succeeded") {
      const warning = action.summary.durability_warning || action.summary.cleanup_warning;
      severity = warning ? "warning" : "success";
      message = action.summary.durability_warning ? "La copia fue publicada, pero no se pudo confirmar la durabilidad del directorio final." : action.summary.cleanup_warning ? "La copia fue creada; falló la limpieza de un archivo interno temporal." : "Copia creada correctamente.";
    } else if (action.type === "restore_succeeded") {
      severity = "success"; message = "Restauración completada correctamente.";
    } else if (action.type === "backup_failed" || (action.type === "restore_failed" && ["failure", "unavailable"].includes(action.status))) {
      message = action.message;
    }
    if (message) notifications?.publish({ severity, message });
    setSharedOutcome(Boolean(notifications && message));
    // Unknown recovery must never invite a blind destructive retry.
    dispatch(action.type === "restore_failed" && action.status === "recovery"
      ? { ...action, message: "No se pudo recuperar la restauración. El estado local es incierto; conservá el respaldo y solicitá asistencia antes de restaurar nuevamente." }
      : action);
  };
  const dispatchRef = useRef<(action: BackupAction) => void>(accept); dispatchRef.current = accept;
  const interactionRef = useRef<ReturnType<typeof createBackupInteraction> | null>(null);
  if (!interactionRef.current) interactionRef.current = createBackupInteraction(backupCommands, (action) => dispatchRef.current(action));
  useEffect(() => () => interactionRef.current?.dispose(), []);
  useEffect(() => { if (state.restore_status === "success" || state.restore_status === "failure" || state.restore_status === "recovery") setDialogOpen(false); }, [state.restore_status]);
  const interaction = interactionRef.current;
  const pending = state.backup_status === "pending" || state.restore_status === "pending";
  const backupMessage = backupState(state.backup_status) ? state.feedback : null;
  const restoreMessage = restoreState(state.restore_status) ? state.feedback : null;
  const cancelDialog = () => { if (!pending) { setDialogOpen(false); dispatch({ type: "restore_cancelled" }); } };
  return h("main", { "aria-labelledby": "backup-heading", "data-ui-backup": true },
    h("h1", { id: "backup-heading" }, "Copia y restauración"),
    h("p", null, "Creá una copia local o restaurá un respaldo validado."),
    h("div", { "data-ui-backup-layout": true },
      h(Panel, { label: "Copia de seguridad" } as never,
        h(Action, { variant: "primary", pending: state.backup_status === "pending", pendingLabel: "Creando copia…", disabled: pending, onClick: () => void interaction.backup() }, "Elegir destino de la copia"),
        backupMessage && !sharedOutcome && !(state.backup_status === "success" && (state.backup?.durability_warning || state.backup?.cleanup_warning)) ? h(Feedback, { kind: state.backup_status === "success" ? "success" : state.backup_status === "unavailable" ? "unavailable" : "error" } as never, backupMessage) : null,
        state.backup?.durability_warning ? h(Feedback, { kind: "error" } as never, "La copia fue publicada, pero no se pudo confirmar la durabilidad del directorio final.") : null,
        state.backup?.cleanup_warning ? h(Feedback, { kind: "advisory" } as never, "La copia fue creada, pero falló la limpieza de un archivo interno temporal.") : null,
        state.backup ? h("section", { "aria-labelledby": "backup-summary-heading", "data-ui-backup-summary": true },
          h("h3", { id: "backup-summary-heading" }, "Última copia creada"),
          h("p", null, `Guardada en el destino elegido como ${state.backup.file_name}.`),
          h("dl", null, h("dt", null, "Archivo"), h("dd", null, state.backup.file_name), h("dt", null, "Fecha"), h("dd", null, date(state.backup.created_at_unix_seconds)), h("dt", null, "Tamaño"), h("dd", null, `${state.backup.size_bytes} bytes`), h("dt", null, "Esquema"), h("dd", null, state.backup.schema_version))) : null),
      h(Panel, { label: "Restauración" } as never,
        canRestore ? h(Action, { variant: "secondary", pending: state.restore_status === "pending" && !state.summary, pendingLabel: "Preparando restauración…", disabled: pending, onClick: () => void interaction.prepareRestore() }, "Elegir archivo de respaldo") : h("p", { role: "note" }, "La restauración está disponible con una licencia activa."),
        canRestore ? null : h(Action, { variant: "secondary", disabled: true }, "Elegir archivo de respaldo"),
        restoreMessage && !sharedOutcome ? h(Feedback, { kind: state.restore_status === "success" ? "success" : state.restore_status === "unavailable" ? "unavailable" : "error" } as never, restoreMessage) : null,
        canRestore && state.summary ? h("section", { "aria-labelledby": "restore-summary-heading", "data-ui-restore-candidate": true },
          h("h3", { id: "restore-summary-heading" }, "Candidato de restauración"),
          h("p", null, `Tamaño: ${state.summary.size_bytes} bytes · Esquema: ${state.summary.schema_version}`),
          h("p", null, "Esta acción reemplazará los datos locales actuales."),
          h(Action, { variant: "secondary", disabled: state.restore_status !== "prepared", onClick: () => setDialogOpen(true) }, "Revisar restauración")) : null),
    ),
    canRestore ? h(ConfirmationDialog, { open: dialogOpen && state.summary !== null, purpose: "restore", title: "Restaurar datos locales", description: "Esta acción reemplazará los datos locales actuales", pending: state.restore_status === "pending", confirmLabel: "Restaurar datos", onCancel: cancelDialog, onConfirm: () => { if (canConfirmRestore(state)) void interaction.restore(state.summary!.token); } },
      h(Field, { kind: "checkbox", label: "Entiendo que la restauración reemplaza los datos locales.", control: h("input", { id: "restore-acknowledgement", type: "checkbox", checked: state.confirmed, disabled: state.restore_status === "pending", onChange: (event) => dispatch({ type: "restore_confirmation_changed", confirmed: event.target.checked }) }) } as never)) : null,
  );
}
