import { createElement, type FormEvent, useReducer, useState } from "react";

import { searchProducts } from "../../commands/catalog.ts";
import {
  confirmSale,
  type ConfirmSaleRequest,
} from "../../commands/confirm-sale.ts";
import { catalogResultDetails } from "./catalog-result.ts";
import { createSaleFlow, initialSaleState } from "./sale-flow.ts";
import { persistedSummaryDetails } from "./persisted-summary";

function requestId(): string {
  return crypto.randomUUID();
}

function optionalCentavos(value: string): number | null {
  return value === "" ? null : Number(value);
}

export function SaleScreen() {
  const [state, dispatch] = useReducer(createSaleFlow, initialSaleState);
  const [query, setQuery] = useState("");

  async function search(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    try {
      dispatch({
        type: "search_succeeded",
        results: await searchProducts(query),
      });
    } catch {
      dispatch({
        type: "confirmation_failed",
        message: "Unable to search the local catalog.",
      });
    }
  }

  async function confirm() {
    const currentRequestId = state.request_id ?? requestId();
    dispatch({ type: "confirmation_started", request_id: currentRequestId });
    const request: ConfirmSaleRequest = {
      request_id: currentRequestId,
      lines: state.lines.map(({ product_id, quantity, captured_unit_price_centavos, captured_revision, acknowledged_price_centavos, acknowledged_revision }) => ({
        product_id,
        quantity,
        captured_unit_price_centavos,
        captured_revision,
        acknowledged_price_centavos,
        acknowledged_revision,
      })),
      payment: {
        amount_tendered_centavos: optionalCentavos(
          state.payment.amount_tendered_centavos,
        ),
        qr_applied_centavos: optionalCentavos(
          state.payment.qr_applied_centavos,
        ),
      },
    };
    try {
      const response = await confirmSale(request);
      if (response.kind === "success")
        dispatch({ type: "confirmation_succeeded", summary: response });
      else if (response.kind === "stale_catalog_record")
        dispatch({ type: "stale_price_detected", ...response });
      else
        dispatch({
          type: "confirmation_failed",
          message: `${response.code}: ${response.message}`,
        });
    } catch (error) {
      dispatch({
        type: "confirmation_failed",
        message:
          error instanceof Error ? error.message : "Confirmation failed.",
      });
    }
  }

  if (state.persisted_summary) {
    const details = persistedSummaryDetails(state.persisted_summary);
    return createElement(
      "main",
      { "aria-labelledby": "sale-summary-heading" },
      createElement("h1", { id: "sale-summary-heading" }, "Sale confirmed"),
      createElement(
        "p",
        null,
        `Sale ${details.saleId} · ${details.status} · ${details.outcome}`,
      ),
      createElement(
        "dl",
        null,
        createElement("dt", null, "Request ID"),
        createElement("dd", null, details.requestId),
        createElement("dt", null, "Confirmed at"),
        createElement("dd", null, details.confirmedAt),
        createElement("dt", null, "Total"),
        createElement("dd", null, details.total),
      ),
      createElement("h2", null, "Products"),
      createElement(
        "ul",
        null,
        details.lines.map((line) => createElement("li", { key: line }, line)),
      ),
      createElement("h2", null, "Payments"),
      createElement(
        "ul",
        null,
        details.payments.map((payment) =>
          createElement("li", { key: payment }, payment),
        ),
      ),
      createElement(
        "button",
        { type: "button", onClick: () => dispatch({ type: "discard" }) },
        "New sale",
      ),
    );
  }

  return createElement(
    "main",
    { "aria-labelledby": "sale-heading" },
    createElement("h1", { id: "sale-heading" }, "Confirm sale"),
    createElement(
      "form",
      { onSubmit: search },
      createElement("label", { htmlFor: "catalog-search" }, "Search catalog"),
      createElement("input", {
        id: "catalog-search",
        value: query,
        onChange: (event) => setQuery(event.target.value),
      }),
      createElement("button", { type: "submit" }, "Search"),
    ),
    createElement(
      "ul",
      { "aria-label": "Catalog results" },
      state.search_results.map((product) =>
        createElement(
          "li",
          { key: product.product_id },
          catalogResultDetails(product),
          createElement(
            "button",
            {
              type: "button",
              onClick: () => dispatch({ type: "add_product", product }),
            },
            "Add",
          ),
        ),
      ),
    ),
    createElement("h2", null, "Cart"),
    createElement(
      "ul",
      { "aria-label": "Draft cart" },
      state.lines.map((line) =>
        createElement(
          "li",
          { key: line.product_id },
          createElement("strong", null, `${line.sku} — ${line.product_name}`),
          createElement(
            "label",
            null,
            " Quantity ",
            createElement("input", {
              type: "number",
              min: 1,
              step: 1,
              value: line.quantity,
              onChange: (event) =>
                dispatch({
                  type: "line_quantity_changed",
                  product_id: line.product_id,
                  value: event.target.value,
                }),
            }),
          ),
          createElement(
            "button",
            {
              type: "button",
              onClick: () =>
                dispatch({
                  type: "remove_product",
                  product_id: line.product_id,
                }),
            },
            "Remove",
          ),
        ),
      ),
    ),
    createElement(
      "fieldset",
      null,
      createElement("legend", null, "Payment (centavos)"),
      createElement(
        "label",
        null,
        "Cash tendered",
        createElement("input", {
          type: "number",
          min: 0,
          step: 1,
          value: state.payment.amount_tendered_centavos,
          onChange: (event) =>
            dispatch({
              type: "payment_changed",
              field: "amount_tendered_centavos",
              value: event.target.value,
            }),
        }),
      ),
      createElement(
        "label",
        null,
        "QR applied",
        createElement("input", {
          type: "number",
          min: 0,
          step: 1,
          value: state.payment.qr_applied_centavos,
          onChange: (event) =>
            dispatch({
              type: "payment_changed",
              field: "qr_applied_centavos",
              value: event.target.value,
            }),
        }),
      ),
    ),
    state.feedback
      ? createElement("p", { role: "alert" }, state.feedback)
      : null,
    state.stale_price
      ? createElement("button", { type: "button", onClick: () => dispatch({ type: "acknowledge_stale_price", product_id: state.stale_price!.product_id, current_unit_price_centavos: state.stale_price!.current_unit_price_centavos, current_revision: state.stale_price!.current_revision }) }, `Acknowledge current price (${state.stale_price.current_unit_price_centavos} centavos)`)
      : null,
    createElement(
      "button",
      {
        type: "button",
        disabled: state.confirmation === "pending" || state.lines.length === 0 || state.stale_price !== null,
        onClick: confirm,
      },
      state.confirmation === "pending" ? "Confirming…" : "Confirm sale",
    ),
    createElement(
      "button",
      { type: "button", onClick: () => dispatch({ type: "discard" }) },
      "Discard draft",
    ),
  );
}
