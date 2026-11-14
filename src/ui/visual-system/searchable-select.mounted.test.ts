import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement, useState } from "react";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { SearchableSelect, type SearchableSelectOption } from "./searchable-select.ts";

const options: SearchableSelectOption[] = [
  { value: "", label: "Todas las categorías" },
  { value: "1", label: "Frenos" },
  { value: "2", label: "Suspensión" },
  { value: "3", label: "Eléctrico" },
  { value: "4", label: "Filtros de aceite" },
];

function renderSelect(props: { value?: string; disabled?: boolean } = {}) {
  const changes: string[] = [];
  function Harness() {
    const [value, setValue] = useState(props.value ?? "");
    return createElement(SearchableSelect, {
      label: "Categoría",
      value,
      options,
      disabled: props.disabled,
      onChange: (next: string) => { changes.push(next); setValue(next); },
    });
  }
  render(createElement(Harness));
  return { changes, combobox: screen.getByRole("combobox", { name: "Categoría" }) as HTMLInputElement };
}

function openOptions() {
  return within(screen.getByRole("listbox", { name: "Categoría" })).queryAllByRole("option").map((option) => option.textContent);
}

test("opens every option on focus and narrows the list while typing, ignoring case and accents", async () => {
  const user = userEvent.setup({ document });
  const { combobox } = renderSelect({ value: "2" });

  assert.equal(combobox.getAttribute("aria-expanded"), "false");
  assert.equal(combobox.getAttribute("aria-autocomplete"), "list");
  assert.equal(screen.queryByRole("listbox"), null);
  assert.equal(combobox.value, "Suspensión");

  await user.click(combobox);
  assert.equal(combobox.getAttribute("aria-expanded"), "true");
  assert.equal(combobox.getAttribute("aria-controls"), screen.getByRole("listbox", { name: "Categoría" }).id);
  assert.equal(combobox.value, "");
  assert.equal(combobox.placeholder, "Suspensión");
  assert.deepEqual(openOptions(), ["Todas las categorías", "Frenos", "Suspensión", "Eléctrico", "Filtros de aceite"]);

  await user.type(combobox, "electrico");
  assert.deepEqual(openOptions(), ["Eléctrico"]);

  await user.clear(combobox);
  await user.type(combobox, "TOD");
  assert.deepEqual(openOptions(), ["Todas las categorías"]);

  await user.clear(combobox);
  await user.type(combobox, "zzz");
  assert.deepEqual(openOptions(), []);
  assert.match(screen.getByRole("listbox", { name: "Categoría" }).textContent ?? "", /Sin coincidencias/);
});

test("reports the chosen option and the closed control shows its label", async () => {
  const user = userEvent.setup({ document });
  const { changes, combobox } = renderSelect();

  await user.click(combobox);
  await user.type(combobox, "sus");
  await user.click(within(screen.getByRole("listbox", { name: "Categoría" })).getByRole("option", { name: "Suspensión" }));

  assert.deepEqual(changes, ["2"]);
  assert.equal(combobox.getAttribute("aria-expanded"), "false");
  assert.equal(combobox.value, "Suspensión");
  assert.equal(screen.queryByRole("listbox"), null);
});

test("moves the active option with the keyboard and commits it with Enter", async () => {
  const user = userEvent.setup({ document });
  const { changes, combobox } = renderSelect();

  await user.click(combobox);
  await user.type(combobox, "f");
  await user.keyboard("{ArrowDown}");
  const active = screen.getByRole("option", { name: "Frenos" });
  assert.equal(combobox.getAttribute("aria-activedescendant"), active.id);

  await user.keyboard("{ArrowDown}");
  assert.equal(combobox.getAttribute("aria-activedescendant"), screen.getByRole("option", { name: "Filtros de aceite" }).id);

  await user.keyboard("{Enter}");
  assert.deepEqual(changes, ["4"]);
  assert.equal(combobox.getAttribute("aria-expanded"), "false");
  assert.equal(combobox.value, "Filtros de aceite");
});

test("cancels with Escape and never reports a change", async () => {
  const user = userEvent.setup({ document });
  const { changes, combobox } = renderSelect({ value: "2" });

  await user.click(combobox);
  await user.type(combobox, "fre");
  await user.keyboard("{Escape}");

  assert.deepEqual(changes, []);
  assert.equal(combobox.getAttribute("aria-expanded"), "false");
  assert.equal(combobox.value, "Suspensión");
  assert.equal(screen.queryByRole("listbox"), null);
});

test("a disabled control never opens the list", async () => {
  const user = userEvent.setup({ document });
  const { combobox } = renderSelect({ disabled: true });

  await user.click(combobox);

  assert.equal(combobox.disabled, true);
  assert.equal(combobox.getAttribute("aria-expanded"), "false");
  assert.equal(screen.queryByRole("listbox"), null);
});

test("caps the option list height so a long category list scrolls instead of growing", async () => {
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");

  assert.match(css, /\[data-ui-combobox-list\]\s*\{[^}]*max-block-size:[^}]*overflow-y:\s*auto[^}]*\}/s);
  assert.match(css, /\[data-ui-combobox-option\]\[data-ui-combobox-option-active="true"\]/);
});
