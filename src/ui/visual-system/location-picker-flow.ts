import type { ProductLocationRecord, ProductLocationSegment } from "../../commands/catalog.ts";

export function searchActiveLocations(locations: ProductLocationRecord[], query: string): ProductLocationRecord[] {
  const normalized = query.trim().toLocaleLowerCase();
  const active = locations.filter((location) => location.active);
  if (!normalized) return active;
  return active.filter((location) => location.code.toLocaleLowerCase().includes(normalized)
    || location.values.some((value) => value.toLocaleLowerCase().includes(normalized)));
}

export function guidedLocationOptions(locations: ProductLocationRecord[], segments: ProductLocationSegment[], selectedValues: string[], segmentIndex: number): string[] {
  const active = locations.filter((location) => location.active && location.values.length === segments.length);
  return [...new Set(active.filter((location) => segments.slice(0, segmentIndex).every((_, index) => location.values[index] === selectedValues[index]))
    .map((location) => location.values[segmentIndex]).filter((value): value is string => typeof value === "string"))]
    .sort((a, b) => a.localeCompare(b));
}

export function guidedLocationId(locations: ProductLocationRecord[], segments: ProductLocationSegment[], selectedValues: string[]): string {
  if (selectedValues.length !== segments.length || selectedValues.some((value) => !value)) return "";
  const match = locations.find((location) => location.active && location.values.length === segments.length
    && location.values.every((value, index) => value === selectedValues[index]));
  return match ? String(match.location_id) : "";
}
