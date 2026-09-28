import { createElement as h, useEffect, useState, type ChangeEvent } from "react";
import type { ProductLocationRecord, ProductLocationSegment } from "../../commands/catalog.ts";
import { guidedLocationId, guidedLocationOptions, searchActiveLocations } from "./location-picker-flow.ts";

export interface LocationPickerProps {
  id: string;
  label: string;
  locations: ProductLocationRecord[];
  segments: ProductLocationSegment[];
  selectedId: string;
  disabled?: boolean;
  status?: "loading" | "ready" | "error";
  onChange: (locationId: string) => void;
}

export function LocationPicker({ id, label, locations, segments, selectedId, disabled = false, status = "ready", onChange }: LocationPickerProps) {
  const [query, setQuery] = useState("");
  const selected = locations.find((location) => String(location.location_id) === selectedId && location.active);
  const [guidedValues, setGuidedValues] = useState<string[]>(selected?.values ?? []);
  useEffect(() => { setGuidedValues(selected?.values ?? []); }, [selectedId, locations]);
  const results = searchActiveLocations(locations, query);
  const updateGuided = (index: number, value: string) => {
    const next = [...guidedValues.slice(0, index), value];
    setGuidedValues(next);
    onChange(guidedLocationId(locations, segments, next));
  };
  const stateText = status === "loading" ? "Cargando ubicaciones activas…" : status === "error" ? "No se pudieron cargar ubicaciones activas." : results.length ? `${results.length} ubicaciones activas disponibles.` : "No hay ubicaciones activas que coincidan.";

  return h("div", { "data-ui-location-picker": true },
    h("label", { htmlFor: `${id}-search` }, "Buscar ubicaciones"),
    h("input", { id: `${id}-search`, type: "search", value: query, disabled: disabled || status !== "ready", onChange: (event: ChangeEvent<HTMLInputElement>) => setQuery(event.target.value), placeholder: "Código o valor de segmento" }),
    h("p", { role: "status", "aria-live": "polite", "data-ui-location-picker-status": true }, stateText),
    segments.map((segment, index) => {
      const options = guidedLocationOptions(locations, segments, guidedValues, index);
      return h("div", { key: segment.id },
        h("label", { htmlFor: `${id}-segment-${segment.id}` }, segment.label),
        h("select", { id: `${id}-segment-${segment.id}`, value: guidedValues[index] ?? "", disabled: disabled || status !== "ready" || (index > 0 && !guidedValues[index - 1]), onChange: (event: ChangeEvent<HTMLSelectElement>) => updateGuided(index, event.target.value) },
          h("option", { value: "" }, `Seleccioná ${segment.label.toLocaleLowerCase()}`),
          options.map((value) => h("option", { key: value, value }, value))));
    }),
    h("label", { htmlFor: id }, label),
    h("select", { id, value: selectedId, disabled: disabled || status !== "ready", onChange: (event: ChangeEvent<HTMLSelectElement>) => onChange(event.target.value) },
      h("option", { value: "" }, "Sin ubicación asignada"),
      results.map((location) => h("option", { key: location.location_id, value: location.location_id }, location.code))));
}
