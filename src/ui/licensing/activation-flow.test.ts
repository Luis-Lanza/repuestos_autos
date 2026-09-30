import assert from "node:assert/strict";
import test from "node:test";
import { createActivationFlow, initialActivationState, type ActivationCommands, type ActivationState } from "./activation-flow.ts";

const code = "a".repeat(64);
function harness(overrides: Partial<ActivationCommands> = {}) {
  const states: ActivationState[] = [];
  let activated = 0;
  const commands: ActivationCommands = {
    status: async () => ({ kind: "status", code: "activation_required" }),
    installationCode: async () => ({ kind: "code", code }),
    chooseFile: async () => ({ kind: "selected" }),
    importFile: async () => ({ kind: "imported", status: "active" }),
    ...overrides,
  };
  return { states, commands, flow: createActivationFlow(commands, (state) => states.push(state), () => activated++), active: () => activated };
}

test("loads activation code and completes an import", async () => {
  const h = harness();
  assert.deepEqual(initialActivationState, { outcome: "loading", installationCode: null, message: null });
  await h.flow.load();
  assert.equal(h.states.at(-1)?.outcome, "activation-required");
  await h.flow.importLicense();
  assert.equal(h.states.at(-1)?.outcome, "import-succeeded");
  assert.equal(h.active(), 1);
});

test("cancellation is not failure and duplicate imports call IPC once", async () => {
  let resolve!: (value: { kind: "cancelled" }) => void; let calls = 0;
  const h = harness({ chooseFile: () => { calls++; return new Promise((done) => { resolve = done; }); } });
  await h.flow.load();
  const first = h.flow.importLicense(); const second = h.flow.importLicense();
  assert.equal(calls, 1);
  resolve({ kind: "cancelled" }); await Promise.all([first, second]);
  assert.equal(h.states.at(-1)?.outcome, "import-cancelled");
  assert.equal(h.active(), 0);
});

test("ignores stale loads and completions after disposal", async () => {
  let resolveStatus!: (value: { kind: "status"; code: "active" }) => void;
  const h = harness({ status: () => new Promise((resolve) => { resolveStatus = resolve; }) });
  const loading = h.flow.load(); h.flow.dispose(); resolveStatus({ kind: "status", code: "active" }); await loading;
  assert.equal(h.states.length, 1);
  assert.equal(h.states[0].outcome, "loading");
  assert.equal(h.active(), 0);
});

test("reports identity unavailable without exposing native diagnostics", async () => {
  const h = harness({ installationCode: async () => ({ kind: "error", code: "machine_identity_unavailable" }) });
  await h.flow.load();
  assert.equal(h.states.at(-1)?.outcome, "identity-unavailable");
  assert.doesNotMatch(h.states.at(-1)?.message ?? "", /machine|path|registry/i);
});
