import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { Action, Badge, Feedback, Field, IconAction } from "./controls.ts";

const productionStyle = document.createElement("style");
productionStyle.textContent = await readFile(new URL("../styles.css", import.meta.url), "utf8");
document.head.append(productionStyle);

function styleOf(element: Element, property: string) {
  let value = getComputedStyle(element).getPropertyValue(property).trim();
  for (let depth = 0; depth < 3; depth += 1) {
    const variable = value.match(/^var\((--[^)]+)\)$/)?.[1];
    if (!variable) break;
    value = getComputedStyle(element).getPropertyValue(variable).trim()
      || getComputedStyle(document.documentElement).getPropertyValue(variable).trim();
  }
  return value;
}

function actionStateRule(selector: string, rules = productionStyle.sheet?.cssRules) {
  return [...(rules ?? [])]
    .filter((rule): rule is CSSStyleRule => "selectorText" in rule)
    .find((rule) => rule.selectorText === selector)?.style;
}

function reducedMotionRule(selector: string) {
  const media = [...(productionStyle.sheet?.cssRules ?? [])]
    .find((rule): rule is CSSMediaRule => "media" in rule && rule.media.mediaText === "(prefers-reduced-motion: reduce)");
  return actionStateRule(selector, media?.cssRules);
}

test("actions expose names, production sizing, and complete enabled pointer states", async () => {
  let activations = 0;
  const user = userEvent.setup({ document });
  const { rerender } = render(createElement(Action, { variant: "primary", onClick: () => { activations += 1; } }, "Save"));
  await user.click(screen.getByRole("button", { name: "Save" }));
  assert.equal(activations, 1);

  rerender(createElement(Action, { variant: "primary", pending: true, pendingLabel: "Saving…", onClick: () => { activations += 1; } }, "Save"));
  const pending = screen.getByRole("button", { name: "Saving…" });
  assert.equal((pending as HTMLButtonElement).disabled, true);
  assert.equal(styleOf(pending, "cursor"), "progress");
  await user.click(pending);
  assert.equal(activations, 1);

  render(createElement("div", null,
    createElement(Action, { variant: "primary" }, "Confirm"),
    createElement(Action, { variant: "secondary" }, "Select"),
    createElement(Action, { variant: "tertiary" }, "Back"),
    createElement(Action, { variant: "destructive" }, "Archive"),
    createElement(IconAction, { accessibleName: "Retry", icon: createElement("span", null, "↻"), onClick: () => { activations += 1; } })));
  const actions = ["Confirm", "Select", "Back", "Archive"].map((name) => screen.getByRole("button", { name }));
  for (const action of actions) {
    assert.deepEqual([styleOf(action, "min-height"), styleOf(action, "padding-inline"), styleOf(action, "font-weight")], ["44px", "16px", "600"]);
  }
  for (const variant of ["primary", "secondary", "tertiary", "destructive"]) {
    const enabled = `[data-ui-action="${variant}"]:not(:disabled):not([aria-busy="true"])`;
    assert.ok(actionStateRule(`${enabled}:hover`), `${variant} hover rule`);
    assert.ok(actionStateRule(`${enabled}:active`), `${variant} active rule`);
  }
  const reducedActive = '[data-ui-action]:not(:disabled):not([aria-busy="true"]):active';
  assert.equal(reducedMotionRule(reducedActive)?.getPropertyValue("transform"), "none");
  await user.click(screen.getByRole("button", { name: "Retry" }));
  assert.equal(activations, 2);
  assert.equal(screen.getByText("↻").closest("[aria-hidden]")?.getAttribute("aria-hidden"), "true");
  assert.throws(() => IconAction({ accessibleName: " " as never, icon: "x" }), /non-empty/);
});

test("Field associates every native control family without owning its value", async () => {
  let textValue = "";
  const user = userEvent.setup({ document });
  render(createElement("form", null,
    createElement(Field, { kind: "text", label: "Name", hint: "Public name", control: createElement("input", { onChange: (event) => { textValue = event.currentTarget.value; } }) }),
    createElement(Field, { kind: "search", label: "Search", control: createElement("input") }),
    createElement(Field, { kind: "date", label: "From", control: createElement("input") }),
    createElement(Field, { kind: "select", label: "Operation", control: createElement("select", null, createElement("option", null, "Entry")) }),
    createElement(Field, { kind: "checkbox", label: "Accept", control: createElement("input") }),
    createElement(Field, { kind: "quantity", label: "Quantity", error: "Whole units only", control: createElement("input") }),
    createElement(Field, { kind: "money", label: "Price", control: createElement("input") }),
    createElement(Field, { kind: "sku", label: "SKU", control: createElement("input") }),
  ));

  await user.type(screen.getByRole("textbox", { name: "Name" }), "Filter");
  assert.equal(textValue, "Filter");
  assert.match(screen.getByRole("textbox", { name: "Name" }).getAttribute("aria-describedby") ?? "", /hint/);
  assert.equal(screen.getByRole("textbox", { name: "Name" }).hasAttribute("aria-invalid"), false);
  screen.getByRole("searchbox", { name: "Search" });
  assert.equal(screen.getByLabelText("From").matches('input[type="date"]'), true);
  screen.getByRole("combobox", { name: "Operation" });
  const checkbox = screen.getByRole("checkbox", { name: "Accept" });
  const checkboxLabel = screen.getByText("Accept");
  assert.deepEqual([styleOf(checkbox, "inline-size"), styleOf(checkbox, "block-size")], ["44px", "44px"]);
  assert.equal(styleOf(checkboxLabel, "min-height"), "44px");
  const quantity = screen.getByRole("spinbutton", { name: "Quantity" });
  assert.deepEqual([quantity.getAttribute("step"), quantity.getAttribute("inputmode"), quantity.getAttribute("aria-invalid")], ["1", "numeric", "true"]);
  assert.match(quantity.getAttribute("aria-describedby") ?? "", /error/);
  assert.equal(screen.getByText("Whole units only").textContent, "Whole units only");
  const money = screen.getByRole("textbox", { name: "Price" });
  assert.equal(money.getAttribute("inputmode"), "decimal");
  assert.equal(screen.getByText("Bs").getAttribute("aria-hidden"), "true");
  assert.equal(screen.getByRole("textbox", { name: "SKU" }).hasAttribute("data-ui-sku"), true);
});

test("Feedback derives live intent and Badge keeps state meaning visible", async () => {
  let retries = 0;
  const user = userEvent.setup({ document });
  render(createElement("main", null,
    createElement(Feedback, { kind: "initial" }, "Start here"),
    createElement(Feedback, { kind: "loading" }, "Loading"),
    createElement(Feedback, { kind: "empty" }, "No records"),
    createElement(Feedback, { kind: "success" }, "Saved"),
    createElement(Feedback, { kind: "advisory" }, "Review"),
    createElement(Feedback, { kind: "error" }, "Failed"),
    createElement(Feedback, { kind: "unavailable" }, "Unavailable", createElement("button", { onClick: () => { retries += 1; } }, "Retry")),
    createElement(Feedback, { kind: "stale" }, "Stale"),
    ...(["available", "low-stock", "out-of-stock", "active", "archived", "stale", "unavailable", "confirmed", "cancelled"] as const)
      .map((kind) => createElement(Badge, { key: kind, kind, text: `Visible ${kind}` })),
  ));

  assert.equal(screen.getByText("Loading").closest("[aria-busy]")?.getAttribute("aria-busy"), "true");
  assert.equal(screen.getAllByRole("status").length, 5);
  assert.equal(screen.getAllByRole("alert").length, 2);
  await user.click(screen.getByRole("button", { name: "Retry" }));
  assert.equal(retries, 1);
  assert.equal(screen.getAllByText(/^Visible /).length, 9);
  assert.throws(() => Badge({ kind: "active", text: " " as never }), /visible text/);
});
