import { LICENSE_STATUS, type InstallationCodeResponse, type LicenseImportResponse, type LicenseSelectionResponse, type LicenseStatusResponse } from "../../commands/license.ts";

export type ActivationOutcome = "loading" | "active" | "activation-required" | "import-pending" | "import-succeeded" | "import-cancelled" | "import-failed" | "identity-unavailable";
export interface ActivationState { outcome: ActivationOutcome; installationCode: string | null; message: string | null; }
export const initialActivationState: ActivationState = { outcome: "loading", installationCode: null, message: null };
export interface ActivationCommands { status: () => Promise<LicenseStatusResponse>; installationCode: () => Promise<InstallationCodeResponse>; chooseFile: () => Promise<LicenseSelectionResponse>; importFile: () => Promise<LicenseImportResponse>; }

export function activationErrorMessage(code: string): string {
  if (code === LICENSE_STATUS.IDENTITY_UNAVAILABLE) return "No se pudo obtener el código de instalación en este equipo.";
  if (code === LICENSE_STATUS.MACHINE_MISMATCH) return "El archivo de licencia no corresponde a este equipo.";
  if (code === LICENSE_STATUS.UNSUPPORTED_VERSION) return "El archivo de licencia no es compatible.";
  if (code === LICENSE_STATUS.UNKNOWN_KEY || code === LICENSE_STATUS.SIGNATURE_INVALID || code === LICENSE_STATUS.WRONG_PRODUCT || code === LICENSE_STATUS.FILE_INVALID) return "No se pudo validar el archivo de licencia. Revisá el archivo e intentá nuevamente.";
  return "No se pudo completar la activación. Intentá nuevamente.";
}

export function createActivationFlow(commands: ActivationCommands, onChange: (state: ActivationState) => void, onActive: (notice?: string) => void) {
  let mounted = true;
  let generation = 0;
  let importing = false;
  let state = initialActivationState;
  const publish = (next: ActivationState) => { if (mounted) { state = next; onChange(next); } };
  const load = async () => {
    const id = ++generation;
    publish({ outcome: "loading", installationCode: null, message: null });
    const license = await commands.status();
    if (!mounted || id !== generation) return;
    if (license.kind === "status" && license.code === LICENSE_STATUS.ACTIVE) {
      publish({ outcome: "active", installationCode: null, message: null }); onActive(); return;
    }
    const code = await commands.installationCode();
    if (!mounted || id !== generation) return;
    if (code.kind !== "code") {
      publish({ outcome: "identity-unavailable", installationCode: null, message: activationErrorMessage(LICENSE_STATUS.IDENTITY_UNAVAILABLE) });
      return;
    }
    const detail = license.kind === "status" && license.code !== LICENSE_STATUS.ACTIVATION_REQUIRED && license.code !== "license_missing" ? activationErrorMessage(license.code) : null;
    publish({ outcome: "activation-required", installationCode: code.code, message: detail });
  };
  const importLicense = async () => {
    if (!mounted || importing || !state.installationCode) return;
    importing = true;
    const id = generation;
    publish({ outcome: "import-pending", installationCode: state.installationCode, message: null });
    try {
      const selected = await commands.chooseFile();
      if (!mounted || id !== generation) return;
      if (selected.kind === "cancelled") { publish({ outcome: "import-cancelled", installationCode: state.installationCode, message: "No se seleccionó ningún archivo." }); return; }
      if (selected.kind !== "selected") { publish({ outcome: "import-failed", installationCode: state.installationCode, message: activationErrorMessage(selected.code) }); return; }
      const result = await commands.importFile();
      if (!mounted || id !== generation) return;
      if (result.kind === "imported") {
        const notice = "Licencia activada correctamente.";
        publish({ outcome: "import-succeeded", installationCode: state.installationCode, message: notice });
        onActive(notice);
      }
      else if (result.kind === "cancelled") publish({ outcome: "import-cancelled", installationCode: state.installationCode, message: "No se seleccionó ningún archivo." });
      else publish({ outcome: "import-failed", installationCode: state.installationCode, message: activationErrorMessage(result.code) });
    } catch {
      if (mounted && id === generation) publish({ outcome: "import-failed", installationCode: state.installationCode, message: activationErrorMessage(LICENSE_STATUS.FILE_INVALID) });
    } finally { importing = false; }
  };
  return { load, importLicense, dispose: () => { mounted = false; generation++; } };
}
