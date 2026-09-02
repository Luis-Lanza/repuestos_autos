import { createElement, type ReactNode } from "react";

import type { NavigationAction, Screen } from "./app.ts";

interface AppShellProps {
  screen: Screen;
  onNavigate: (action: NavigationAction) => void;
  children?: ReactNode;
}

const items = [
  ["Ventas", "sales", "return_to_sales"],
  ["Inventario", "inventory", "open_inventory"],
  ["Catálogo", "catalog", "open_catalog"],
  ["Alta de productos", "onboarding", "start_onboarding"],
  ["Historial de ventas", "sales_history", "open_sales_history"],
  ["Copia y restauración", "backup", "open_backup"],
] as const satisfies ReadonlyArray<readonly [string, Screen, NavigationAction]>;

export function AppShell({ screen, onNavigate, children }: AppShellProps) {
  return createElement("div", { "data-ui-app-shell": true },
    createElement("aside", { "data-ui-shell-sidebar": true },
      createElement("div", { "data-ui-shell-identity": true }, "Repuestos Autos"),
      createElement("nav", { "aria-label": "Navegación principal", "data-ui-shell-navigation": true },
        items.map(([label, destination, action]) => createElement("button", {
          key: destination,
          type: "button",
          "aria-current": screen === destination ? "page" : undefined,
          onClick: () => onNavigate(action),
        }, label)),
      ),
    ),
    createElement("div", { "data-ui-shell-content": true }, children),
  );
}
