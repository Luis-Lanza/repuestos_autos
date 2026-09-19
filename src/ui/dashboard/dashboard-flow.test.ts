import assert from "node:assert/strict";
import test from "node:test";

import { createDashboardFlow, initialDashboardState } from "./dashboard-flow.ts";

test("ignores stale dashboard completions", () => {
  const started = createDashboardFlow(initialDashboardState, { type: "load_started", request_id: 2 });
  const stale = createDashboardFlow(started, { type: "load_failed", request_id: 1 });
  assert.equal(stale.status, "loading");
  assert.equal(stale.request_id, 2);
});
