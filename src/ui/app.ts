import { createElement, useEffect, useRef, useState } from "react";

import { inventoryCommands } from "../commands/inventory.ts";
import { AppShell } from "./app-shell.ts";
import { DashboardScreen } from "./dashboard/dashboard-screen.ts";
import { OnboardingScreen } from "./onboarding/onboarding-screen.ts";
import { InventoryScreen } from "./inventory/inventory-screen.ts";
import { BackupScreen } from "./backup/backup-screen.ts";
import { SaleScreen } from "./sales/sale-screen.ts";
import { SalesHistoryScreen } from "./sales/history-screen.ts";
import { CatalogMaintenanceScreen } from "./catalog/catalog-maintenance-screen.ts";

export const SCREEN = {
  DASHBOARD: "dashboard", SALES: "sales", ONBOARDING: "onboarding", INVENTORY: "inventory", BACKUP: "backup", CATALOG: "catalog", SALES_HISTORY: "sales_history",
} as const;
export const NAVIGATION_ACTION = {
  OPEN_DASHBOARD: "open_dashboard", START_ONBOARDING: "start_onboarding", RETURN_TO_SALES: "return_to_sales", OPEN_INVENTORY: "open_inventory", OPEN_BACKUP: "open_backup", OPEN_CATALOG: "open_catalog", OPEN_SALES_HISTORY: "open_sales_history",
} as const;
export type Screen = (typeof SCREEN)[keyof typeof SCREEN];
export type NavigationAction = (typeof NAVIGATION_ACTION)[keyof typeof NAVIGATION_ACTION];
export function screenAfter(_current: Screen, action: NavigationAction): Screen {
  return action === NAVIGATION_ACTION.OPEN_DASHBOARD ? SCREEN.DASHBOARD : action === NAVIGATION_ACTION.OPEN_SALES_HISTORY ? SCREEN.SALES_HISTORY : action === NAVIGATION_ACTION.OPEN_INVENTORY ? SCREEN.INVENTORY : action === NAVIGATION_ACTION.OPEN_BACKUP ? SCREEN.BACKUP : action === NAVIGATION_ACTION.OPEN_CATALOG ? SCREEN.CATALOG : action === NAVIGATION_ACTION.START_ONBOARDING ? SCREEN.ONBOARDING : SCREEN.SALES;
}
function screenContent(screen: Screen, onNavigate: (action: NavigationAction) => void, refreshInventoryCount: () => void, inventoryFilter: "all" | "alerts", openInventoryAlerts: () => void) {
  if (screen === SCREEN.DASHBOARD) return createElement(DashboardScreen, { onOpenInventoryAlerts: openInventoryAlerts });
  if (screen === SCREEN.ONBOARDING) return createElement(OnboardingScreen, { onBack: () => onNavigate(NAVIGATION_ACTION.RETURN_TO_SALES) });
  if (screen === SCREEN.INVENTORY) return createElement(InventoryScreen, { key: inventoryFilter, initialStockState: inventoryFilter, onInventoryAlertsRefresh: refreshInventoryCount });
  if (screen === SCREEN.BACKUP) return createElement(BackupScreen);
  if (screen === SCREEN.CATALOG) return createElement(CatalogMaintenanceScreen);
  if (screen === SCREEN.SALES_HISTORY) return createElement(SalesHistoryScreen);
  return createElement(SaleScreen, { onInventoryAlertsRefresh: refreshInventoryCount });
}
export function App() {
  const [screen, setScreen] = useState<Screen>(SCREEN.DASHBOARD);
  const [inventoryCount, setInventoryCount] = useState<number | null>(null);
  const [inventoryFilter, setInventoryFilter] = useState<"all" | "alerts">("all");
  const alertAttempt = useRef(0);
  const mounted = useRef(true);
  useEffect(() => () => { mounted.current = false; alertAttempt.current += 1; }, []);
  const refreshInventoryCount = () => {
    const attempt = ++alertAttempt.current;
    setInventoryCount(null);
    void inventoryCommands.listAlerts().then((response) => {
      if (!mounted.current || attempt !== alertAttempt.current) return;
      setInventoryCount(response.kind === "success" ? response.alerts.length : null);
    }).catch(() => {
      if (mounted.current && attempt === alertAttempt.current) setInventoryCount(null);
    });
  };
  useEffect(() => { refreshInventoryCount(); }, []);
  const navigate = (action: NavigationAction) => {
    setInventoryFilter("all");
    setScreen((current) => screenAfter(current, action));
    refreshInventoryCount();
  };
  const openInventoryAlerts = () => {
    setInventoryFilter("alerts");
    setScreen(SCREEN.INVENTORY);
    refreshInventoryCount();
  };
  const inventoryCue = inventoryCount && inventoryCount > 0 ? `${inventoryCount} ${inventoryCount === 1 ? "alerta" : "alertas"} de stock` : null;
  return createElement(AppShell, { screen, onNavigate: navigate, onInventoryAlerts: openInventoryAlerts, inventoryCue }, screenContent(screen, navigate, refreshInventoryCount, inventoryFilter, openInventoryAlerts));
}
