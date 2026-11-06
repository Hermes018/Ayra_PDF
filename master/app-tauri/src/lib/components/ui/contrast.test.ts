// WCAG contrast computed from the theme tokens for the Dialog card and Tooltip (opaque surface, glass only on the backdrop).
import { describe, it, expect } from 'vitest';
import { TOKENS } from '$lib/core/themeContract.svelte';

type Rgba = [number, number, number, number];
function parse(c: string): Rgba {
  const hex = /^#([0-9a-f]{6})$/i.exec(c.trim());
  if (hex) {
    const n = parseInt(hex[1], 16);
    return [(n >> 16) & 255, (n >> 8) & 255, n & 255, 1];
  }
  const m = /^rgba?\(([^)]+)\)$/.exec(c.trim());
  if (!m) throw new Error(`unparseable colour ${c}`);
  const p = m[1].split(',').map((x) => parseFloat(x));
  return [p[0], p[1], p[2], p[3] ?? 1];
}
function over(fg: Rgba, bg: Rgba): Rgba {
  const a = fg[3];
  return [0, 1, 2].map((i) => fg[i] * a + bg[i] * (1 - a)).concat(1) as Rgba;
}
function lum([r, g, b]: Rgba): number {
  const f = (v: number) => {
    const s = v / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
}
export function ratio(fgTok: string, bgTok: string, theme: 'light' | 'dark'): number {
  const bg = parse(TOKENS[theme][bgTok as keyof (typeof TOKENS)['light']]);
  expect(bg[3], `${bgTok} must be opaque`).toBe(1);
  const fg = over(parse(TOKENS[theme][fgTok as keyof (typeof TOKENS)['light']]), bg);
  const [a, b] = [lum(fg), lum(bg)].sort((x, y) => y - x);
  return (a + 0.05) / (b + 0.05);
}

describe('Dialog and Tooltip contrast (from tokens)', () => {
  for (const theme of ['light', 'dark'] as const) {
    it(`${theme}: dialog title and body meet 4.5:1 on the opaque surface`, () => {
      const t = ratio('--text-primary', '--surface-dialog', theme);
      const b = ratio('--text-primary', '--surface-dialog', theme); // Dialog body uses text-primary (secondary is 4.0:1 in light)
      console.info(`contrast ${theme}: title ${t.toFixed(2)} body ${b.toFixed(2)}`);
      expect(t).toBeGreaterThanOrEqual(4.5);
      expect(b).toBeGreaterThanOrEqual(4.5);
    });
    it(`${theme}: the dialog scrim is a translucent token`, () => {
      expect(parse(TOKENS[theme]['--scrim' as keyof (typeof TOKENS)['light']])[3]).toBeLessThan(1);
    });
  }
});
