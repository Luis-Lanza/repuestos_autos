import { createElement, type ReactElement, type ReactNode, type RefObject } from "react";

import { ConfirmationDialog, type ConfirmationDialogProps } from "./confirmation-dialog.ts";

/** A routine, non-destructive dialog for completing an in-progress checkout. */
export type CheckoutDialogProps = Omit<ConfirmationDialogProps, "purpose"> & {
  description: string | ReactElement;
  confirmLabel: ReactNode;
  initialFocusRef?: RefObject<HTMLElement>;
};

export function CheckoutDialog(props: CheckoutDialogProps) {
  return createElement(ConfirmationDialog, { ...props, purpose: "routine" });
}
