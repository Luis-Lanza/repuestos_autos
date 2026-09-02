import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { render, screen, within } from "@testing-library/react";

import { AlignedData, Panel } from "./structure.ts";

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

const columns = [
  { label: "Producto", align: "start", kind: "text" },
  { label: "SKU", align: "start", kind: "sku" },
  { label: "Stock", align: "end", kind: "numeric" },
  { label: "Venta", align: "start", kind: "text" },
  { label: "Fecha", align: "start", kind: "text" },
  { label: "Total", align: "end", kind: "money" },
  { label: "Estado", align: "start", kind: "text" },
  { label: "Corrección", align: "start", kind: "text" },
] as const;

const values = [
  "Filtro premium", "fil-Ace-001", "02", "#184", "14/08/2026",
  "Bs 125,50", "Cancelada", "Devolución: 1",
] as const;

test("Panel names its region with its internal heading", () => {
  render(createElement(Panel, { label: "Detalle persistido" }, createElement("p", null, "Contenido del caller")));

  const region = screen.getByRole("region", { name: "Detalle persistido" });
  const heading = within(region).getByRole("heading", { level: 2, name: "Detalle persistido" });
  assert.equal(region.getAttribute("aria-labelledby"), heading.id);
  assert.equal(within(region).getByText("Contenido del caller").textContent, "Contenido del caller");
});

test("AlignedData preserves native semantics, caller values, order, and column presentation intent", () => {
  render(createElement(AlignedData, { caption: "Artículos originales", columns, rows: [values] }));

  const table = screen.getByRole("table", { name: "Artículos originales" });
  assert.equal(table.querySelector("caption")?.textContent, "Artículos originales");
  const headers = within(table).getAllByRole("columnheader");
  assert.deepEqual(headers.map((header) => header.textContent), columns.map(({ label }) => label));
  assert.ok(headers.every((header) => header.getAttribute("scope") === "col"));

  const cells = within(table).getAllByRole("cell");
  assert.deepEqual(cells.map((cell) => cell.textContent), values);
  assert.deepEqual(cells.map((cell) => [
    cell.getAttribute("data-label"),
    cell.getAttribute("data-ui-align"),
    cell.getAttribute("data-ui-kind"),
  ]), columns.map(({ label, align, kind }) => [label, align, kind]));
  assert.deepEqual(cells.map((cell) => styleOf(cell, "text-align")),
    ["start", "start", "end", "start", "start", "end", "start", "start"]);
  assert.match(styleOf(cells[1], "font-family"), /Cascadia Mono/);
  assert.equal(styleOf(cells[2], "font-variant-numeric"), "tabular-nums");
  assert.equal(styleOf(cells[5], "font-variant-numeric"), "tabular-nums");

  const responsiveSource = productionStyle.textContent ?? "";
  assert.match(responsiveSource, /@media \(max-width: 960px\)[\s\S]*td::before/);
  assert.match(responsiveSource, /content: attr\(data-label\)/);
});

test("AlignedData fails before rendering a row with a missing or extra cell", () => {
  assert.throws(() => AlignedData({ caption: "Faltante", columns, rows: [["solo una"]] }), /row 1 requires exactly 8 cells/);
  assert.throws(() => AlignedData({ caption: "Extra", columns: columns.slice(0, 1), rows: [["uno", "dos"]] }), /row 1 requires exactly 1 cells/);
});
