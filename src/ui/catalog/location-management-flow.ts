import type { ProductLocationRecord, ProductLocationSchema } from "../../commands/catalog.ts";

export const LOCATION_TEMPLATES = [
  { name: "Piso · Estante · Nivel", segments: ["Piso", "Estante", "Nivel"] },
  { name: "Sector · Gaveta", segments: ["Sector", "Gaveta"] },
  { name: "Mostrador · Posición", segments: ["Mostrador", "Posición"] },
  { name: "Desde cero", segments: ["Segmento 1"] },
] as const;

export function generateLocationCode(values: string[]): string {
  const parts = values.map((value) => value.trim().replace(/[^a-zA-Z0-9]/g, "").toUpperCase());
  return parts.every(Boolean) ? parts.join("-") : "";
}

export function validateLocationSchema(segments: string[]): Record<string, string> {
  const errors: Record<string, string> = {};
  if (segments.length < 1 || segments.length > 8) errors.segments = "Configurá entre 1 y 8 segmentos.";
  segments.forEach((label, index) => {
    const normalized = label.trim();
    if (!normalized) errors[`segment-${index}`] = "Ingresá un nombre para este segmento.";
    else if ([...normalized].length > 40) errors[`segment-${index}`] = "Usá hasta 40 caracteres.";
    else if (segments.slice(0, index).some((earlier) => earlier.trim().toLocaleLowerCase() === normalized.toLocaleLowerCase())) errors[`segment-${index}`] = "Cada segmento necesita un nombre distinto.";
  });
  return errors;
}

export function validateLocationValues(values: string[], segmentCount: number): Record<string, string> {
  const errors: Record<string, string> = {};
  values.slice(0, segmentCount).forEach((value, index) => {
    const normalized = value.trim();
    if (!normalized) errors[`value-${index}`] = "Completá este valor.";
    else if ([...normalized].length > 40) errors[`value-${index}`] = "Usá hasta 40 caracteres.";
    else if (!/[a-zA-Z0-9]/.test(normalized)) errors[`value-${index}`] = "Incluí al menos una letra o un número para generar el código.";
  });
  if (values.length !== segmentCount) errors.values = "La cantidad de valores debe coincidir con los segmentos.";
  return errors;
}

export interface LocationManagementState { status: "loading" | "ready" | "unavailable"; schema: ProductLocationSchema | null; locations: ProductLocationRecord[]; error: string | null; notice: string | null; }
export const initialLocationManagementState: LocationManagementState = { status: "loading", schema: null, locations: [], error: null, notice: null };
export type LocationManagementAction =
  | { type: "loaded"; schema: ProductLocationSchema; locations: ProductLocationRecord[] }
  | { type: "load_failed" }
  | { type: "started" }
  | { type: "failed"; message: string }
  | { type: "succeeded"; notice: string; schema?: ProductLocationSchema; location?: ProductLocationRecord; location_id?: number };
export function createLocationManagementFlow(state: LocationManagementState, action: LocationManagementAction): LocationManagementState {
  switch (action.type) {
    case "loaded": return { status: "ready", schema: action.schema, locations: action.locations, error: null, notice: null };
    case "load_failed": return { ...state, status: "unavailable", error: "No se pudieron cargar el esquema y las ubicaciones. Reintentá.", notice: null };
    case "started": return { ...state, error: null, notice: null };
    case "failed": return { ...state, status: "ready", error: action.message, notice: null };
    case "succeeded": return { ...state, status: "ready", schema: action.schema ?? state.schema, locations: action.location ? [...state.locations.filter((item) => item.location_id !== action.location!.location_id), action.location].sort((a, b) => a.code.localeCompare(b.code)) : action.location_id ? state.locations.filter((item) => item.location_id !== action.location_id) : state.locations, error: null, notice: action.notice };
  }
}
