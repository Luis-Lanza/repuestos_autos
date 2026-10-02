import { createElement, Fragment, useEffect, useRef, useState } from "react";

import { inventoryCommands } from "../commands/inventory.ts";
import { licenseCommands } from "../commands/license.ts";
import { AppShell } from "./app-shell.ts";
import { DashboardScreen } from "./dashboard/dashboard-screen.ts";
import { OnboardingScreen } from "./onboarding/onboarding-screen.ts";
import { InventoryScreen } from "./inventory/inventory-screen.ts";
import { BackupScreen } from "./backup/backup-screen.ts";
import { SaleScreen } from "./sales/sale-screen.ts";
import { SalesHistoryScreen } from "./sales/history-screen.ts";
import { CatalogMaintenanceScreen } from "./catalog/catalog-maintenance-screen.ts";
import { MovementLedgerScreen } from "./reports/movement-ledger-screen.ts";
import { ActivationScreen } from "./licensing/activation-screen.ts";

export const SCREEN = {
  DASHBOARD: "dashboard", SALES: "sales", ONBOARDING: "onboarding", INVENTORY: "inventory", BACKUP: "backup", CATALOG: "catalog", SALES_HISTORY: "sales_history", REPORTS: "reports",
} as const;
export const NAVIGATION_ACTION = {
  OPEN_DASHBOARD: "open_dashboard", START_ONBOARDING: "start_onboarding", RETURN_TO_SALES: "return_to_sales", OPEN_INVENTORY: "open_inventory", OPEN_BACKUP: "open_backup", OPEN_CATALOG: "open_catalog", OPEN_SALES_HISTORY: "open_sales_history", OPEN_REPORTS: "open_reports",
} as const;
export type Screen = (typeof SCREEN)[keyof typeof SCREEN];
export type NavigationAction = (typeof NAVIGATION_ACTION)[keyof typeof NAVIGATION_ACTION];
export function screenAfter(_current: Screen, action: NavigationAction): Screen {
  return action === NAVIGATION_ACTION.OPEN_DASHBOARD ? SCREEN.DASHBOARD : action === NAVIGATION_ACTION.OPEN_SALES_HISTORY ? SCREEN.SALES_HISTORY : action === NAVIGATION_ACTION.OPEN_REPORTS ? SCREEN.REPORTS : action === NAVIGATION_ACTION.OPEN_INVENTORY ? SCREEN.INVENTORY : action === NAVIGATION_ACTION.OPEN_BACKUP ? SCREEN.BACKUP : action === NAVIGATION_ACTION.OPEN_CATALOG ? SCREEN.CATALOG : action === NAVIGATION_ACTION.START_ONBOARDING ? SCREEN.ONBOARDING : SCREEN.SALES;
}
function screenContent(screen: Screen, onNavigate: (action: NavigationAction) => void, refreshInventoryCount: () => void, inventoryFilter: "all" | "alerts", openInventoryAlerts: () => void, canRestore: boolean) {
  if (screen === SCREEN.DASHBOARD) return createElement(DashboardScreen, { onOpenInventoryAlerts: openInventoryAlerts });
  if (screen === SCREEN.ONBOARDING) return createElement(OnboardingScreen, { onBack: () => onNavigate(NAVIGATION_ACTION.RETURN_TO_SALES) });
  if (screen === SCREEN.INVENTORY) return createElement(InventoryScreen, { key: inventoryFilter, initialStockState: inventoryFilter, onInventoryAlertsRefresh: refreshInventoryCount });
  if (screen === SCREEN.BACKUP) return createElement(BackupScreen, { canRestore });
  if (screen === SCREEN.CATALOG) return createElement(CatalogMaintenanceScreen);
  if (screen === SCREEN.SALES_HISTORY) return createElement(SalesHistoryScreen);
  if (screen === SCREEN.REPORTS) return createElement(MovementLedgerScreen);
  return createElement(SaleScreen, { onInventoryAlertsRefresh: refreshInventoryCount });
}
export function App() {
  const [screen, setScreen] = useState<Screen>(SCREEN.DASHBOARD);
  const [accessMode, setAccessMode] = useState<"loading" | "activation" | "active" | "recovery">("loading");
  const [activationNotice, setActivationNotice] = useState<string | null>(null);
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
  useEffect(() => {
    let current = true;
    void licenseCommands.status().then((response) => {
      if (current) setAccessMode(response.kind === "status" && response.code === "active" ? "active" : "activation");
    });
    return () => { current = false; };
  }, []);
  useEffect(() => { if (accessMode === "active" || accessMode === "recovery") refreshInventoryCount(); }, [accessMode]);
  const navigate = (action: NavigationAction) => {
    if (accessMode === "recovery" && ![NAVIGATION_ACTION.OPEN_DASHBOARD, NAVIGATION_ACTION.OPEN_SALES_HISTORY, NAVIGATION_ACTION.OPEN_REPORTS, NAVIGATION_ACTION.OPEN_BACKUP].includes(action)) return;
    setInventoryFilter("all");
    setScreen((current) => screenAfter(current, action));
    refreshInventoryCount();
  };
  const openInventoryAlerts = () => {
    if (accessMode === "recovery") return;
    setInventoryFilter("alerts");
    setScreen(SCREEN.INVENTORY);
    refreshInventoryCount();
  };
  const inventoryCue = inventoryCount && inventoryCount > 0 ? `${inventoryCount} ${inventoryCount === 1 ? "alerta" : "alertas"} de stock` : null;
  const content = accessMode === "loading"
    ? createElement("main", { role: "status", "aria-live": "polite" }, "Verificando el estado de la licencia…")
    : accessMode === "activation"
      ? createElement(ActivationScreen, {
        onRecovery: () => { setScreen(SCREEN.DASHBOARD); setAccessMode("recovery"); },
        onActivated: (notice) => {
          if (notice) setActivationNotice(notice);
          setAccessMode("active");
        },
      })
      : createElement(AppShell, { screen, onNavigate: navigate, onInventoryAlerts: openInventoryAlerts, inventoryCue, recoveryMode: accessMode === "recovery" }, screenContent(screen, navigate, refreshInventoryCount, inventoryFilter, openInventoryAlerts, accessMode === "active"));
  return createElement(Fragment, null,
    createElement("div", { role: "status", "aria-live": "polite", "aria-atomic": true, "data-ui-activation-notice": true }, activationNotice),
    content,
  );
}
