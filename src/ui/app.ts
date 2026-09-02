import { createElement, useState } from "react";

import { AppShell } from "./app-shell.ts";
import { OnboardingScreen } from "./onboarding/onboarding-screen.ts";
import { InventoryScreen } from "./inventory/inventory-screen.ts";
import { BackupScreen } from "./backup/backup-screen.ts";
import { SaleScreen } from "./sales/sale-screen.ts";
import { SalesHistoryScreen } from "./sales/history-screen.ts";
import { CatalogMaintenanceScreen } from "./catalog/catalog-maintenance-screen.ts";

export const SCREEN = {
  SALES: "sales",
  ONBOARDING: "onboarding",
  INVENTORY: "inventory",
  BACKUP: "backup",
  CATALOG: "catalog",
  SALES_HISTORY: "sales_history",
} as const;

export const NAVIGATION_ACTION = {
  START_ONBOARDING: "start_onboarding",
  RETURN_TO_SALES: "return_to_sales",
  OPEN_INVENTORY: "open_inventory",
  OPEN_BACKUP: "open_backup",
  OPEN_CATALOG: "open_catalog",
  OPEN_SALES_HISTORY: "open_sales_history",
} as const;

export type Screen = (typeof SCREEN)[keyof typeof SCREEN];
export type NavigationAction =
  (typeof NAVIGATION_ACTION)[keyof typeof NAVIGATION_ACTION];

export function screenAfter(
  _current: Screen,
  action: NavigationAction,
): Screen {
  return action === NAVIGATION_ACTION.OPEN_SALES_HISTORY ? SCREEN.SALES_HISTORY : action === NAVIGATION_ACTION.OPEN_INVENTORY ? SCREEN.INVENTORY : action === NAVIGATION_ACTION.OPEN_BACKUP ? SCREEN.BACKUP : action === NAVIGATION_ACTION.OPEN_CATALOG ? SCREEN.CATALOG : action === NAVIGATION_ACTION.START_ONBOARDING
    ? SCREEN.ONBOARDING
    : SCREEN.SALES;
}

function screenContent(screen: Screen, onNavigate: (action: NavigationAction) => void) {
  if (screen === SCREEN.ONBOARDING) {
    return createElement(OnboardingScreen, {
      onBack: () => onNavigate(NAVIGATION_ACTION.RETURN_TO_SALES),
    });
  }
  if (screen === SCREEN.INVENTORY) return createElement(InventoryScreen);
  if (screen === SCREEN.BACKUP) return createElement(BackupScreen);
  if (screen === SCREEN.CATALOG) return createElement(CatalogMaintenanceScreen);
  if (screen === SCREEN.SALES_HISTORY) return createElement(SalesHistoryScreen);
  return createElement(SaleScreen);
}

export function App() {
  const [screen, setScreen] = useState<Screen>(SCREEN.SALES);
  const navigate = (action: NavigationAction) =>
    setScreen((current) => screenAfter(current, action));

  return createElement(AppShell, { screen, onNavigate: navigate }, screenContent(screen, navigate));
}
