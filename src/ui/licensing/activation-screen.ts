import { createElement as h, useEffect, useRef, useState } from "react";

import { licenseCommands } from "../../commands/license.ts";
import { createActivationFlow, initialActivationState, type ActivationState } from "./activation-flow.ts";

export function ActivationScreen({ onRecovery, onActivated }: { onRecovery: () => void; onActivated: (notice?: string) => void }) {
  const [state, setState] = useState<ActivationState>(initialActivationState);
  const flow = useRef<ReturnType<typeof createActivationFlow> | null>(null);
  if (!flow.current) flow.current = createActivationFlow(licenseCommands, setState, onActivated);
  useEffect(() => { void flow.current?.load(); return () => flow.current?.dispose(); }, []);
  const busy = state.outcome === "loading" || state.outcome === "import-pending";
  return h("main", { "aria-labelledby": "activation-heading", "data-ui-activation": true },
    h("h1", { id: "activation-heading" }, "Activá Repuestos Autos"),
    state.outcome === "loading" ? h("p", { role: "status", "aria-live": "polite" }, "Verificando el estado de la licencia…") : null,
    state.installationCode ? h("section", { "aria-labelledby": "installation-code-heading" },
      h("h2", { id: "installation-code-heading" }, "Código de instalación"),
      h("p", null, "Compartí este código con el proveedor para recibir tu archivo de licencia."),
      h("label", { htmlFor: "installation-code" }, "Tu código de instalación"),
      h("input", { id: "installation-code", type: "text", readOnly: true, value: state.installationCode, autoComplete: "off", spellCheck: false }),
    ) : null,
    state.outcome === "identity-unavailable" ? h("p", { role: "alert" }, state.message) : null,
    state.outcome === "import-pending" ? h("p", { role: "status", "aria-live": "polite" }, "Importando archivo de licencia…") : null,
    state.message && state.outcome !== "identity-unavailable" && state.outcome !== "import-pending" ? h("p", { role: state.outcome === "import-failed" ? "alert" : "status", "aria-live": "polite" }, state.message) : null,
    h("button", { type: "button", disabled: busy || !state.installationCode, onClick: () => void flow.current?.importLicense() }, "Importar archivo de licencia"),
    h("p", null, "También podés continuar en modo de recuperación para consultar la información disponible y crear copias de seguridad."),
    h("button", { type: "button", disabled: state.outcome === "loading", onClick: onRecovery }, "Continuar en modo de recuperación"),
  );
}
