import { createElement, isValidElement, useEffect, useId, useRef, type ReactElement, type ReactNode, type RefObject } from "react";

import { Action } from "./controls.ts";

export type DestructivePurpose = "restore" | "cancellation";
export type ConfirmationDialogProps = {
  open: boolean;
  purpose: DestructivePurpose;
  title: string;
  description: string | ReactElement;
  confirmLabel: ReactNode;
  pending?: boolean;
  pendingLabel?: ReactNode;
  initialFocusRef?: RefObject<HTMLElement>;
  onCancel: () => void;
  onConfirm: () => void;
  children?: ReactNode;
};

const focusableSelector = [
  "button", "input", "select", "textarea", "a[href]", "area[href]", "[contenteditable]", "[tabindex]",
].join(",");
const openDialogs = new WeakMap<Document, Set<HTMLElement>>();

function currentDialog(owner: Document) {
  const connected = [...(openDialogs.get(owner) ?? [])].filter((node) => node.isConnected);
  return connected[connected.length - 1] ?? null;
}

function isEligible(element: HTMLElement | null, dialog: HTMLElement): element is HTMLElement {
  if (!element?.isConnected || !dialog.contains(element) || element.tabIndex < 0) return false;
  if (element.matches(":disabled") || element.getAttribute("aria-disabled") === "true") return false;
  if (element.tagName === "INPUT" && (element as HTMLInputElement).type === "hidden") return false;
  for (let node: HTMLElement | null = element; node; node = node.parentElement) {
    const computed = node.ownerDocument.defaultView?.getComputedStyle(node);
    if (node.hidden || node.getAttribute("aria-hidden") === "true" || node.hasAttribute("inert")
      || computed?.display === "none" || computed?.visibility === "hidden" || computed?.visibility === "collapse") return false;
    if (node === dialog) break;
  }
  const browserHasLayout = dialog.ownerDocument.documentElement.getClientRects().length > 0;
  return !browserHasLayout || element.getClientRects().length > 0;
}

function restore(invoker: HTMLElement | null) {
  if (invoker?.isConnected) invoker.focus();
}

export function ConfirmationDialog({
  open, purpose, title, description, confirmLabel, pending = false,
  pendingLabel = "Procesando…", initialFocusRef, onCancel, onConfirm, children,
}: ConfirmationDialogProps) {
  if (typeof title !== "string" || !title.trim()) throw new TypeError("ConfirmationDialog requires a nonblank title");
  if (!(typeof description === "string" && description.trim()) && !isValidElement(description)) {
    throw new TypeError("ConfirmationDialog requires a valid description");
  }
  const generated = useId().replace(/:/g, "");
  const titleId = `confirmation-title-${generated}`;
  const descriptionId = `confirmation-description-${generated}`;
  const dialogRef = useRef<HTMLDivElement>(null);
  const ownedDialogRef = useRef<HTMLElement | null>(null);
  const invokerRef = useRef<HTMLElement | null>(null);
  const wasOpen = useRef(false);
  const deferredRestore = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (deferredRestore.current) clearTimeout(deferredRestore.current);
    deferredRestore.current = null;
    const container = dialogRef.current;
    if (open && container) {
      const owner = container.ownerDocument;
      if (!wasOpen.current) {
        const active = owner.activeElement as HTMLElement | null;
        invokerRef.current = active && active !== owner.body ? active : null;
      }
      ownedDialogRef.current = container;
      const dialogs = openDialogs.get(owner) ?? new Set<HTMLElement>();
      dialogs.delete(container); dialogs.add(container); openDialogs.set(owner, dialogs);
      const contain = (event: globalThis.KeyboardEvent) => {
        if (currentDialog(owner) !== container) return;
        if (event.key === "Escape") {
          event.preventDefault();
          if (!pending) onCancel();
          return;
        }
        if (event.key !== "Tab") return;
        const focusable = [...container.querySelectorAll<HTMLElement>(focusableSelector)].filter((node) => isEligible(node, container));
        const activeIndex = focusable.indexOf(owner.activeElement as HTMLElement);
        if (!focusable.length || activeIndex < 0 || (!event.shiftKey && activeIndex === focusable.length - 1) || (event.shiftKey && activeIndex === 0)) {
          event.preventDefault();
          (focusable[event.shiftKey ? focusable.length - 1 : 0] ?? container).focus();
        }
      };
      owner.addEventListener("keydown", contain, true);
      const active = owner.activeElement as HTMLElement | null;
      if (!isEligible(active, container)) {
        const explicit = initialFocusRef?.current ?? null;
        const back = container.querySelector<HTMLElement>("[data-ui-dialog-actions] button");
        (isEligible(explicit, container) ? explicit : isEligible(back, container) ? back : container).focus();
      }
      wasOpen.current = true;
      return () => {
        owner.removeEventListener("keydown", contain, true);
        deferredRestore.current = setTimeout(() => {
          dialogs.delete(container);
          if (!currentDialog(owner)) restore(invokerRef.current);
          invokerRef.current = null; wasOpen.current = false;
        }, 0);
      };
    }
    if (!open && wasOpen.current) {
      const owned = ownedDialogRef.current;
      if (owned) openDialogs.get(owned.ownerDocument)?.delete(owned);
      restore(invokerRef.current);
      invokerRef.current = null; wasOpen.current = false;
    }
  });

  if (!open) return null;

  return createElement("div", { "data-ui-dialog-backdrop": true },
    createElement("div", {
      ref: dialogRef, role: "dialog", tabIndex: -1,
      "aria-modal": "true", "aria-labelledby": titleId, "aria-describedby": descriptionId,
      "aria-busy": pending || undefined, "data-ui-confirmation-dialog": true,
      "data-ui-purpose": purpose,
    },
    createElement("h2", { id: titleId }, title),
    createElement("p", { id: descriptionId }, description),
    children,
    createElement("div", { "data-ui-dialog-actions": true },
      createElement(Action, { variant: "secondary", disabled: pending, onClick: () => { if (!pending) onCancel(); } }, "Volver"),
      createElement(Action, { variant: "destructive", pending, pendingLabel, onClick: () => { if (!pending) onConfirm(); } }, confirmLabel))));
}
