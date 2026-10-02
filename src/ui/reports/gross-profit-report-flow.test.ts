import assert from "node:assert/strict";
import test from "node:test";
import { createGrossProfitReportFlow, initialGrossProfitReportState } from "./gross-profit-report-flow.ts";
const period = { from: "2024-03-10", to: "2024-03-20" };
const summary = { amount_centavos: 0, missing_cost_line_count: 0, activity_count: 1 };
const page = { rows: [], page: 1, page_size: 20, total: 25, total_pages: 2 };
test("applied range clears both result sets and accepts matching summary and first page", () => {
 let state = initialGrossProfitReportState(period);
 state = createGrossProfitReportFlow(state, { type: "range_applied", period, range_id: 1, operations_request_id: 1 });
 assert.equal(state.report, null); assert.equal(state.operations, null);
 state = createGrossProfitReportFlow(state, { type: "summary_succeeded", range_id: 1, report: summary });
 state = createGrossProfitReportFlow(state, { type: "operations_succeeded", range_id: 1, operations_request_id: 1, report: page });
 assert.equal(state.summary_status, "ready"); assert.equal(state.operations_status, "ready");
});
test("ignores stale date and page responses; paging is bounded to the loaded total", () => {
 let state = createGrossProfitReportFlow(initialGrossProfitReportState(period), { type: "range_applied", period, range_id: 1, operations_request_id: 1 });
 state = createGrossProfitReportFlow(state, { type: "range_applied", period: { ...period, to: "2024-03-21" }, range_id: 2, operations_request_id: 2 });
 const current = createGrossProfitReportFlow(state, { type: "summary_succeeded", range_id: 2, report: summary });
 assert.equal(createGrossProfitReportFlow(current, { type: "summary_succeeded", range_id: 1, report: { ...summary, amount_centavos: 999 } }), current);
 const loaded = createGrossProfitReportFlow(current, { type: "operations_succeeded", range_id: 2, operations_request_id: 2, report: page });
 const requested = createGrossProfitReportFlow(loaded, { type: "page_requested", page: 2, operations_request_id: 3 });
 assert.equal(requested.requested_page, 2); assert.equal(requested.operations?.page, 1); assert.equal(requested.operations_status, "loading");
 assert.equal(createGrossProfitReportFlow(requested, { type: "operations_succeeded", range_id: 2, operations_request_id: 2, report: { ...page, page: 1 } }), requested);
 assert.equal(createGrossProfitReportFlow(requested, { type: "page_requested", page: 3, operations_request_id: 4 }), requested);
 assert.equal(createGrossProfitReportFlow(requested, { type: "operations_succeeded", range_id: 2, operations_request_id: 3, report: { ...page, page: 2 } }).operations?.page, 2);
});
test("allows bounded paging while the independent summary is loading or failed and rejects duplicate requests", () => {
 for (const summaryStatus of ["loading", "error"] as const) {
  let state = createGrossProfitReportFlow(initialGrossProfitReportState(period), { type: "range_applied", period, range_id: 7, operations_request_id: 7 });
  state = createGrossProfitReportFlow(state, { type: "operations_succeeded", range_id: 7, operations_request_id: 7, report: page });
  if (summaryStatus === "error") state = createGrossProfitReportFlow(state, { type: "summary_failed", range_id: 7 });
  const requested = createGrossProfitReportFlow(state, { type: "page_requested", page: 2, operations_request_id: 8 });
  assert.equal(requested.requested_page, 2);
  assert.equal(requested.operations_status, "loading");
  assert.equal(requested.operations?.page, 1);
  assert.equal(createGrossProfitReportFlow(requested, { type: "page_requested", page: 2, operations_request_id: 9 }), requested);
 }
});
test("a newly applied range supersedes an in-flight page request", () => {
 let state = createGrossProfitReportFlow(initialGrossProfitReportState(period), { type: "range_applied", period, range_id: 1, operations_request_id: 1 });
 state = createGrossProfitReportFlow(state, { type: "operations_succeeded", range_id: 1, operations_request_id: 1, report: page });
 state = createGrossProfitReportFlow(state, { type: "page_requested", page: 2, operations_request_id: 2 });
 state = createGrossProfitReportFlow(state, { type: "range_applied", period: { ...period, to: "2024-03-21" }, range_id: 2, operations_request_id: 3 });
 assert.equal(createGrossProfitReportFlow(state, { type: "operations_succeeded", range_id: 1, operations_request_id: 2, report: { ...page, page: 2 } }), state);
 assert.equal(state.operations, null);
});
test("keeps empty distinct from zero-profit activity and preserves retryable errors", () => {
 let state = createGrossProfitReportFlow(initialGrossProfitReportState(period), { type: "range_applied", period, range_id: 4, operations_request_id: 4 });
 state = createGrossProfitReportFlow(state, { type: "summary_succeeded", range_id: 4, report: { ...summary, activity_count: 0 } });
 state = createGrossProfitReportFlow(state, { type: "operations_failed", range_id: 4, operations_request_id: 4 });
 assert.equal(state.summary_status, "empty"); assert.equal(state.operations_status, "error");
 state = createGrossProfitReportFlow(state, { type: "range_applied", period, range_id: 5, operations_request_id: 5 });
 assert.equal(state.summary_status, "loading"); assert.equal(state.operations_status, "loading");
});
