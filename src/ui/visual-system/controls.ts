import {
  Children,
  cloneElement,
  createElement,
  isValidElement,
  useId,
  type ButtonHTMLAttributes,
  type InputHTMLAttributes,
  type ReactElement,
  type ReactNode,
  type SelectHTMLAttributes,
} from "react";

export type ActionVariant = "primary" | "secondary" | "tertiary" | "destructive";
export type ActionProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant: ActionVariant;
  pending?: boolean;
  pendingLabel?: ReactNode;
};

export function Action({ variant, pending = false, pendingLabel = "Working…", disabled, children, ...button }: ActionProps) {
  return createElement("button", {
    ...button,
    type: button.type ?? "button",
    disabled: disabled || pending,
    "aria-busy": pending || undefined,
    "data-ui-action": variant,
  }, pending ? pendingLabel : children);
}

export type FieldKind = "text" | "search" | "date" | "select" | "checkbox" | "quantity" | "money" | "sku";
type InputControl = ReactElement<InputHTMLAttributes<HTMLInputElement>, "input">;
type SelectControl = ReactElement<SelectHTMLAttributes<HTMLSelectElement>, "select">;
export type FieldProps = {
  kind: FieldKind;
  label: ReactNode;
  hint?: ReactNode;
  error?: ReactNode;
  control: InputControl | SelectControl;
};

export function Field({ kind, label, hint, error, control }: FieldProps) {
  if (Children.count(control) !== 1 || !isValidElement(control)) throw new TypeError("Field requires exactly one native control");
  const expected = kind === "select" ? "select" : "input";
  if (control.type !== expected) throw new TypeError(`Field kind ${kind} requires a native ${expected}`);
  const generated = useId().replace(/:/g, "");
  const id = control.props.id ?? `field-${generated}`;
  const hintId = hint ? `${id}-hint` : undefined;
  const errorId = error ? `${id}-error` : undefined;
  const describedBy = [control.props["aria-describedby"], hintId, errorId].filter(Boolean).join(" ") || undefined;
  const association = { id, "aria-describedby": describedBy, "aria-invalid": error ? true : undefined };
  const native = kind === "select"
    ? cloneElement(control as SelectControl, association)
    : (() => {
        const input = control as InputControl;
        const props: InputHTMLAttributes<HTMLInputElement> & Record<`data-ui-${string}`, boolean | undefined> = {
          ...association,
          type: kind === "search" ? "search" : kind === "date" ? "date" : kind === "checkbox" ? "checkbox" : kind === "quantity" ? "number" : input.props.type,
          inputMode: kind === "quantity" ? "numeric" : kind === "money" ? "decimal" : input.props.inputMode,
          step: kind === "quantity" ? 1 : input.props.step,
          "data-ui-quantity": kind === "quantity" || undefined,
          "data-ui-money": kind === "money" || undefined,
          "data-ui-sku": kind === "sku" || undefined,
        };
        return cloneElement(input, props);
      })();
  return createElement("div", { "data-ui-field": kind },
    createElement("label", { htmlFor: id }, label),
    createElement("div", { "data-ui-field-control": kind }, kind === "money" ? createElement("span", { "aria-hidden": "true" }, "Bs") : null, native),
    hint ? createElement("small", { id: hintId }, hint) : null,
    error ? createElement("p", { id: errorId, "data-ui-field-error": true }, error) : null);
}

export type FeedbackKind = "initial" | "loading" | "empty" | "success" | "advisory" | "error" | "unavailable" | "stale";
export function Feedback({ kind, children }: { kind: FeedbackKind; children: ReactNode }) {
  const role = kind === "error" || kind === "unavailable" ? "alert" : kind === "initial" ? undefined : "status";
  return createElement("div", {
    role,
    "aria-busy": kind === "loading" || undefined,
    "data-ui-feedback": kind,
  }, children);
}

export type BadgeKind = "available" | "low-stock" | "out-of-stock" | "active" | "archived" | "stale" | "unavailable" | "confirmed" | "cancelled";
export function Badge<Text extends string>({ kind, text }: { kind: BadgeKind; text: NonEmpty<Text> }) {
  if (!text.trim()) throw new TypeError("Badge requires explicit visible text");
  return createElement("span", { "data-ui-badge": kind }, text);
}

type NonEmpty<Name extends string> = Name extends "" ? never : Name;
type IconActionProps<Name extends string> = Omit<ActionProps, "variant" | "children"> & {
  accessibleName: NonEmpty<Name>;
  icon: ReactNode;
};

export function IconAction<Name extends string>({ accessibleName, icon, ...button }: IconActionProps<Name>) {
  if (!accessibleName.trim()) throw new TypeError("IconAction accessibleName must be non-empty");
  return createElement(Action, { ...button, variant: "tertiary", "aria-label": accessibleName },
    createElement("span", { "aria-hidden": "true" }, icon));
}
