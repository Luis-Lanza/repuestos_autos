import { createElement, useState } from "react";

import { OnboardingScreen } from "./onboarding/onboarding-screen.ts";
import { InventoryScreen } from "./inventory/inventory-screen.ts";
import { BackupScreen } from "./backup/backup-screen.ts";
import { SaleScreen } from "./sales/sale-screen.ts";
import { CatalogMaintenanceScreen } from "./catalog/catalog-maintenance-screen.ts";

export const SCREEN = {
  SALES: "sales",
  ONBOARDING: "onboarding",
  INVENTORY: "inventory",
  BACKUP: "backup",
  CATALOG: "catalog",
} as const;

export const NAVIGATION_ACTION = {
  START_ONBOARDING: "start_onboarding",
  RETURN_TO_SALES: "return_to_sales",
  OPEN_INVENTORY: "open_inventory",
  OPEN_BACKUP: "open_backup",
  OPEN_CATALOG: "open_catalog",
} as const;

type Screen = (typeof SCREEN)[keyof typeof SCREEN];
type NavigationAction =
  (typeof NAVIGATION_ACTION)[keyof typeof NAVIGATION_ACTION];

export function screenAfter(
  _current: Screen,
  action: NavigationAction,
): Screen {
  return action === NAVIGATION_ACTION.OPEN_INVENTORY ? SCREEN.INVENTORY : action === NAVIGATION_ACTION.OPEN_BACKUP ? SCREEN.BACKUP : action === NAVIGATION_ACTION.OPEN_CATALOG ? SCREEN.CATALOG : action === NAVIGATION_ACTION.START_ONBOARDING
    ? SCREEN.ONBOARDING
    : SCREEN.SALES;
}

export function App() {
  const [screen, setScreen] = useState<Screen>(SCREEN.SALES);

  if (screen === SCREEN.ONBOARDING) {
    return createElement(OnboardingScreen, {
      onBack: () =>
        setScreen((current) =>
          screenAfter(current, NAVIGATION_ACTION.RETURN_TO_SALES),
        ),
    });
  }
  if (screen === SCREEN.INVENTORY) return createElement("div", null, createElement("button", { type: "button", onClick: () => setScreen(SCREEN.SALES) }, "Sales"), createElement(InventoryScreen));
  if (screen === SCREEN.BACKUP) return createElement("div", null, createElement("button", { type: "button", onClick: () => setScreen(SCREEN.SALES) }, "Sales"), createElement(BackupScreen));
  if (screen === SCREEN.CATALOG) return createElement("div", null, createElement("button", { type: "button", onClick: () => setScreen(SCREEN.SALES) }, "Sales"), createElement(CatalogMaintenanceScreen));

  return createElement(
    "div",
    null,
    createElement(
      "nav",
      { "aria-label": "Application" },
      createElement(
        "button",
        {
          type: "button",
          onClick: () =>
            setScreen((current) =>
              screenAfter(current, NAVIGATION_ACTION.START_ONBOARDING),
            ),
        },
        "Onboard product",
      ),
      createElement("button", { type: "button", onClick: () => setScreen(SCREEN.INVENTORY) }, "Inventory"),
       createElement("button", { type: "button", onClick: () => setScreen(SCREEN.BACKUP) }, "Backup and restore"),
       createElement("button", { type: "button", onClick: () => setScreen(SCREEN.CATALOG) }, "Catalog maintenance"),
    ),
    createElement(SaleScreen),
  );
}
