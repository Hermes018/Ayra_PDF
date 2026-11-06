/**
 * Parse a number field: empty, blank, null or invalid means `fallback`; a real 0 stays 0 (never
 * `Number(x) || fallback`). Accepts a number too: Svelte's `bind:value` on `<input type="number">` yields a number
 * (or null when the field is empty), even though ui/Input types the value as a string.
 */
export function parseNumberOr(value: string | number | null | undefined, fallback: number): number {
  if (value === null || value === undefined) return fallback;
  if (typeof value === 'number') return Number.isFinite(value) ? value : fallback;
  if (value.trim() === '') return fallback;
  const n = Number(value);
  return Number.isFinite(n) ? n : fallback;
}
