import {
  createElement,
  useId,
  useRef,
  useState,
  type ChangeEvent,
  type FocusEvent,
  type KeyboardEvent,
  type MouseEvent,
} from "react";

/**
 * Filter text comparison for the searchable select: case and accent insensitive, so typing
 * "electrico" finds "Eléctrico" and "SUS" finds "Suspensión".
 */
export function normalizeSelectText(text: string): string {
  return text.normalize("NFD").replace(/[\u0300-\u036f]/g, "").toLocaleLowerCase().trim();
}

export function filterSelectOptions(options: SearchableSelectOption[], text: string): SearchableSelectOption[] {
  const needle = normalizeSelectText(text);
  return needle ? options.filter((option) => normalizeSelectText(option.label).includes(needle)) : options;
}

export interface SearchableSelectOption {
  value: string;
  label: string;
}

export interface SearchableSelectProps {
  label: string;
  value: string;
  options: SearchableSelectOption[];
  onChange: (value: string) => void;
  disabled?: boolean;
  emptyMessage?: string;
}

/**
 * Dropdown whose open list is filtered by typing, so a long option list stays usable.
 * Closed, it shows the selected option label. Open, the field is empty for filtering,
 * the current selection stays visible as placeholder, and the list is capped and scrolls.
 */
export function SearchableSelect({ label, value, options, onChange, disabled = false, emptyMessage = "Sin coincidencias" }: SearchableSelectProps) {
  const generated = useId().replace(/:/g, "");
  const inputId = `combobox-${generated}`;
  const listId = `${inputId}-list`;
  const container = useRef<HTMLDivElement | null>(null);
  const [open, setOpen] = useState(false);
  const [typed, setTyped] = useState("");
  const [activeIndex, setActiveIndex] = useState(-1);

  const selectedLabel = options.find((option) => option.value === value)?.label ?? "";
  const visible = open ? filterSelectOptions(options, typed) : [];

  const close = () => { setOpen(false); setTyped(""); setActiveIndex(-1); };
  const openList = () => {
    if (disabled) return;
    setOpen(true);
    setTyped("");
    setActiveIndex(-1);
  };
  const commit = (option: SearchableSelectOption) => {
    close();
    if (option.value !== value) onChange(option.value);
  };
  const move = (delta: number) => {
    if (!open) {
      openList();
      setActiveIndex(delta > 0 ? 0 : Math.max(options.length - 1, -1));
      return;
    }
    if (!visible.length) return;
    const next = activeIndex < 0
      ? (delta > 0 ? 0 : visible.length - 1)
      : Math.min(Math.max(activeIndex + delta, 0), visible.length - 1);
    setActiveIndex(next);
  };

  const onInput = (event: ChangeEvent<HTMLInputElement>) => {
    const next = event.target.value;
    setOpen(true);
    setTyped(next);
    setActiveIndex(-1);
  };
  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (disabled) return;
    if (event.key === "ArrowDown") { event.preventDefault(); move(1); return; }
    if (event.key === "ArrowUp") { event.preventDefault(); move(-1); return; }
    if (event.key === "Escape") { if (open) { event.preventDefault(); close(); } return; }
    if (event.key === "Tab") { if (open) close(); return; }
    if (event.key !== "Enter" || !open) return;
    // Enter closes the list instead of submitting the surrounding search form.
    event.preventDefault();
    const active = visible[activeIndex];
    if (active) commit(active);
    else if (visible.length === 1) commit(visible[0]);
    else close();
  };
  const onBlur = (event: FocusEvent<HTMLDivElement>) => {
    if (container.current?.contains(event.relatedTarget as Node | null)) return;
    close();
  };

  return createElement("div", { "data-ui-field": "combobox", "data-ui-searchable-select": true, ref: container, onBlur },
    createElement("label", { htmlFor: inputId }, label),
    createElement("div", { "data-ui-field-control": "combobox" },
      createElement("div", { "data-ui-combobox": true },
        createElement("input", {
          id: inputId,
          type: "text",
          role: "combobox",
          autoComplete: "off",
          disabled,
          value: open ? typed : selectedLabel,
          placeholder: open ? selectedLabel : undefined,
          "aria-expanded": open,
          "aria-controls": listId,
          "aria-autocomplete": "list",
          "aria-activedescendant": open && visible[activeIndex] ? `${listId}-option-${activeIndex}` : undefined,
          onFocus: openList,
          onClick: () => { if (!open) openList(); },
          onChange: onInput,
          onKeyDown,
        }),
        open ? createElement("ul", { id: listId, role: "listbox", "aria-label": label, "data-ui-combobox-list": true },
          visible.length
            ? visible.map((option, index) => createElement("li", {
                key: option.value || "__all",
                id: `${listId}-option-${index}`,
                role: "option",
                "aria-selected": option.value === value,
                "data-ui-combobox-option": true,
                "data-ui-combobox-option-active": index === activeIndex || undefined,
                onMouseDown: (event: MouseEvent) => event.preventDefault(),
                onClick: () => commit(option),
              }, option.label))
            : createElement("li", { role: "presentation", "data-ui-combobox-empty": true }, emptyMessage))
          : null)));
}
