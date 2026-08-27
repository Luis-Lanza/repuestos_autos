import type { BackupSummary, RestoreSummary } from "../../commands/backup.ts";

const STATUS = { IDLE: "idle", PENDING: "pending", SUCCESS: "success", ERROR: "error" } as const;
export interface BackupState { backup_status: (typeof STATUS)[keyof typeof STATUS]; restore_status: (typeof STATUS)[keyof typeof STATUS]; summary: RestoreSummary | null; confirmed: boolean; backup: BackupSummary | null; feedback: string | null; }
export const initialBackupState: BackupState = { backup_status: STATUS.IDLE, restore_status: STATUS.IDLE, summary: null, confirmed: false, backup: null, feedback: null };
export type BackupAction = { type: "backup_started" } | { type: "backup_cancelled" } | { type: "backup_succeeded"; summary: BackupSummary } | { type: "restore_prepared"; summary: RestoreSummary } | { type: "restore_confirmation_changed"; confirmed: boolean } | { type: "restore_started" } | { type: "restore_succeeded" } | { type: "failed"; message: string };
export function createBackupFlow(state: BackupState, action: BackupAction): BackupState {
  switch (action.type) {
    case "backup_started": return { ...state, backup_status: STATUS.PENDING, feedback: null };
    case "backup_cancelled": return { ...state, backup_status: STATUS.IDLE, feedback: "Backup selection was cancelled." };
    case "backup_succeeded": return { ...state, backup_status: STATUS.SUCCESS, backup: action.summary, feedback: "Backup completed successfully." };
    case "restore_prepared": return { ...state, restore_status: STATUS.IDLE, summary: action.summary, confirmed: false, feedback: null };
    case "restore_confirmation_changed": return { ...state, confirmed: action.confirmed };
    case "restore_started": return { ...state, restore_status: STATUS.PENDING, feedback: null };
    case "restore_succeeded": return { ...state, restore_status: STATUS.SUCCESS, summary: null, confirmed: false, feedback: "Restore completed successfully." };
    case "failed": return { ...state, backup_status: state.backup_status === STATUS.PENDING ? STATUS.ERROR : state.backup_status, restore_status: state.backup_status === STATUS.PENDING ? state.restore_status : STATUS.ERROR, feedback: action.message };
  }
}
export const canConfirmRestore = (state: BackupState) => state.summary !== null && state.confirmed && state.restore_status !== STATUS.PENDING;
