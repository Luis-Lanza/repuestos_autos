import { createContext, createElement, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";

export type ActionNoticeSeverity = "success" | "error" | "warning" | "info";
export type ActionNoticeInput = { severity: ActionNoticeSeverity; message: string };
export type ActionNotifications = { publish: (notice: ActionNoticeInput) => string; dismiss: (id: string) => void };
type Notice = ActionNoticeInput & { id: string };
const Context = createContext<ActionNotifications | null>(null);
const labels = { success: "Éxito", error: "Error", warning: "Advertencia", info: "Información" };
const VISIBLE_LIMIT = 3;

/** Null outside the provider: isolated screens retain their inline outcome fallback. */
export function useActionNotifications(): ActionNotifications | null { return useContext(Context); }

function NoticeCard({ notice, dismiss }: { notice: Notice; dismiss: ActionNotifications["dismiss"] }) {
  const [hovered, setHovered] = useState(false);
  const [focused, setFocused] = useState(false);
  const remaining = useRef(6000);
  const transient = notice.severity === "success" || notice.severity === "info";
  useEffect(() => {
    if (!transient || hovered || focused) return;
    const started = Date.now();
    const timer = setTimeout(() => dismiss(notice.id), remaining.current);
    return () => {
      clearTimeout(timer);
      remaining.current = Math.max(0, remaining.current - (Date.now() - started));
    };
  }, [dismiss, notice.id, transient, hovered, focused]);
  return createElement("div", {
    "data-ui-action-notice": true, "data-severity": notice.severity,
    onMouseEnter: () => setHovered(true), onMouseLeave: () => setHovered(false),
    onFocus: () => setFocused(true),
    onBlur: (event) => { if (!event.currentTarget.contains(event.relatedTarget as Node | null)) setFocused(false); },
  }, createElement("strong", null, labels[notice.severity]), createElement("p", null, notice.message),
  createElement("button", { type: "button", "aria-label": `Cerrar notificación: ${labels[notice.severity]}`, onClick: () => dismiss(notice.id) }, "Cerrar"));
}

/** The same portal node/live regions are reparented, not remounted, for modal accessibility. */
function NotificationHost({ queue, dismiss, announcements }: { queue: Notice[]; dismiss: ActionNotifications["dismiss"]; announcements: { polite: string; assertive: string } }) {
  const [target, setTarget] = useState<HTMLElement | null>(null);
  useEffect(() => {
    const node = document.createElement("div");
    node.setAttribute("data-ui-action-notification-portal", "true");
    const position = () => {
      const dialogs = [...document.querySelectorAll<HTMLElement>('[aria-modal="true"]')]
        .filter((dialog) => !dialog.closest('[hidden], [aria-hidden="true"], [inert]'));
      const focusedDialog = document.activeElement?.closest<HTMLElement>('[aria-modal="true"]');
      const parent = focusedDialog && dialogs.includes(focusedDialog) ? focusedDialog : dialogs.at(-1) ?? document.body;
      if (node.parentElement !== parent) parent.appendChild(node);
    };
    position();
    const observer = new window.MutationObserver(position);
    observer.observe(document.body, { childList: true, subtree: true, attributes: true, attributeFilter: ["aria-modal", "hidden", "aria-hidden", "inert"] });
    document.addEventListener("focusin", position);
    setTarget(node);
    return () => { observer.disconnect(); document.removeEventListener("focusin", position); node.remove(); };
  }, []);
  if (!target) return null;
  return createPortal(createElement("section", { "aria-label": "Notificaciones de acciones", "data-ui-action-notifications": true },
    createElement("div", { role: "status", "aria-live": "polite", "aria-atomic": true, "data-ui-notification-live": true }, announcements.polite),
    createElement("div", { role: "alert", "aria-live": "assertive", "aria-atomic": true, "data-ui-notification-live": true }, announcements.assertive),
    queue.slice(0, VISIBLE_LIMIT).map((notice) => createElement(NoticeCard, { key: notice.id, notice, dismiss })),
    queue.length > VISIBLE_LIMIT ? createElement("p", { "data-ui-notification-overflow": true }, `${queue.length - VISIBLE_LIMIT} notificaciones en espera`) : null,
  ), target);
}

/** FIFO, no eviction: queued notices have no expiry clock until displayed. */
export function ActionNotificationProvider({ children }: { children?: ReactNode }) {
  const [queue, setQueue] = useState<Notice[]>([]);
  const [announcements, setAnnouncements] = useState({ polite: "", assertive: "" });
  const sequence = useRef(0);
  const dismiss = useCallback((id: string) => setQueue((current) => current.filter((notice) => notice.id !== id)), []);
  const publish = useCallback((input: ActionNoticeInput) => {
    const id = String(++sequence.current);
    setQueue((current) => [...current, { ...input, id }]);
    const channel = input.severity === "error" || input.severity === "warning" ? "assertive" : "polite";
    setAnnouncements((current) => ({ ...current, [channel]: `${labels[input.severity]} (${id}): ${input.message}` }));
    return id;
  }, []);
  const api = useMemo(() => ({ publish, dismiss }), [publish, dismiss]);
  return createElement(Context.Provider, { value: api }, children, createElement(NotificationHost, { queue, dismiss, announcements }));
}
