import assert from "node:assert/strict";
import test from "node:test";
import { guidedLocationId, guidedLocationOptions } from "./location-picker-flow.ts";

const segments = [{ id: 1, label: "Piso", position: 0 }, { id: 2, label: "Estante", position: 1 }];
const locations = [
  { location_id: 1, code: "PB-12", values: ["PB", "12"], active: true, revision: 0 },
  { location_id: 2, code: "PA-12", values: ["PA", "12"], active: true, revision: 0 },
  { location_id: 3, code: "PB-9", values: ["PB", "9"], active: false, revision: 0 },
];

test("cascades distinct active values by preceding segments and resolves only complete paths", () => {
  assert.deepEqual(guidedLocationOptions(locations, segments, [], 0), ["PA", "PB"]);
  assert.deepEqual(guidedLocationOptions(locations, segments, ["PB"], 1), ["12"]);
  assert.deepEqual(guidedLocationOptions(locations, segments, ["PA"], 1), ["12"]);
  assert.equal(guidedLocationId(locations, segments, ["PB"]), "");
  assert.equal(guidedLocationId(locations, segments, ["PB", "12"]), "1");
});
