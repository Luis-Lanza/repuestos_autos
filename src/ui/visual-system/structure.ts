import { createElement, useId, type ReactNode } from "react";

export function Panel({ label, children }: { label: ReactNode; children: ReactNode }) {
  const headingId = `panel-${useId().replace(/:/g, "")}`;
  return createElement("section", { "aria-labelledby": headingId, "data-ui-panel": true },
    createElement("h2", { id: headingId }, label),
    children);
}

export type DataValueKind = "text" | "sku" | "numeric" | "money";
export type DataColumn = {
  label: string;
  align: "start" | "end";
  kind: DataValueKind;
};
export type AlignedDataProps = {
  caption: ReactNode;
  columns: readonly DataColumn[];
  rows: readonly (readonly ReactNode[])[];
};

export function AlignedData({ caption, columns, rows }: AlignedDataProps) {
  rows.forEach((row, index) => {
    if (row.length !== columns.length) {
      throw new TypeError(`AlignedData row ${index + 1} requires exactly ${columns.length} cells`);
    }
  });
  return createElement("div", { "data-ui-data-scroll": true },
    createElement("table", { "data-ui-aligned-data": true },
      createElement("caption", null, caption),
      createElement("thead", null, createElement("tr", null,
        columns.map((column, index) => createElement("th", {
          key: index,
          scope: "col",
          "data-ui-align": column.align,
          "data-ui-kind": column.kind,
        }, column.label)))),
      createElement("tbody", null, rows.map((row, rowIndex) => createElement("tr", { key: rowIndex },
        row.map((value, columnIndex) => {
          const column = columns[columnIndex];
          return createElement("td", {
            key: columnIndex,
            "data-label": column.label,
            "data-ui-align": column.align,
            "data-ui-kind": column.kind,
          }, value);
        }))))));
}
