import assert from "node:assert/strict";
import test from "node:test";

import { createLocationManagementFlow, generateLocationCode, initialLocationManagementState, LOCATION_TEMPLATES, validateLocationSchema, validateLocationValues } from "./location-management-flow.ts";

const schema = { revision: 3, segments: [{ id: 1, label: "Piso", position: 0 }, { id: 2, label: "Estante", position: 1 }] };
const location = { location_id: 5, code: "PB-12", values: ["PB", "12"], active: true, revision: 0 };

test("offers editable templates and scratch and previews the canonical generated code", () => {
  assert.deepEqual(LOCATION_TEMPLATES.map(({ name }) => name), ["Piso · Estante · Nivel", "Sector · Gaveta", "Mostrador · Posición", "Desde cero"]);
  assert.deepEqual(LOCATION_TEMPLATES[0].segments, ["Piso", "Estante", "Nivel"]);
  assert.equal(generateLocationCode(["p.b.", "12"]), "PB-12");
  assert.equal(generateLocationCode(["Piso", ""]), "");
});

test("validates ordered schema labels and concrete values before invoking commands", () => {
  assert.deepEqual(validateLocationSchema([" Piso ", "Estante"]), {});
  assert.equal(validateLocationSchema(["Piso", " piso "])['segment-1'], "Cada segmento necesita un nombre distinto.");
  assert.equal(validateLocationSchema([" "])['segment-0'], "Ingresá un nombre para este segmento.");
  assert.equal(validateLocationSchema(Array.from({ length: 9 }, (_, index) => `S${index}`)).segments, "Configurá entre 1 y 8 segmentos.");
  assert.deepEqual(validateLocationValues(["PB", "12"], 2), {});
  assert.equal(validateLocationValues(["PB", "  "], 2)['value-1'], "Completá este valor.");
  assert.equal(validateLocationValues(["PB"], 2).values, "La cantidad de valores debe coincidir con los segmentos.");
  assert.equal(validateLocationValues(["---"], 1)['value-0'], "Incluí al menos una letra o un número para generar el código.");
});

test("models load failure, successful creation, lifecycle updates, and recoverable failures", () => {
  const loaded = createLocationManagementFlow(initialLocationManagementState, { type: "loaded", schema, locations: [location] });
  const unavailable = createLocationManagementFlow(initialLocationManagementState, { type: "load_failed" });
  const created = createLocationManagementFlow(loaded, { type: "succeeded", notice: "Ubicación PB-12 creada.", location: { ...location, location_id: 6, code: "PA-3" } });
  const deactivated = createLocationManagementFlow(created, { type: "succeeded", notice: "Ubicación PB-12 desactivada.", location: { ...location, active: false, revision: 1 } });
  const blocked = createLocationManagementFlow(deactivated, { type: "failed", message: "Assigned locations are protected." });
  const deleted = createLocationManagementFlow(blocked, { type: "succeeded", notice: "Ubicación eliminada.", location_id: 6 });
  assert.equal(unavailable.status, "unavailable");
  assert.equal(created.locations.length, 2);
  assert.equal(deactivated.locations.find((item) => item.location_id === 5)?.active, false);
  assert.equal(blocked.error, "Assigned locations are protected.");
  assert.deepEqual(deleted.locations.map((item) => item.location_id), [5]);
});
