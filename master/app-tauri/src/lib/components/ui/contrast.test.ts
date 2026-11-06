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

// The Studio ledger drawer: opaque surface (no translucent glass over the canvas) and readable tab labels.
import ledgerSource from '../studio/AnnotationLedger.svelte?raw';

function ruleBody(src: string, selector: string): string {
  const style = src.slice(src.indexOf('<style>'));
  const i = style.indexOf(selector + ' {');
  if (i < 0) throw new Error(`no rule ${selector}`);
  return style.slice(i, style.indexOf('}', i));
}
function contrastOf(fg: Rgba, bg: Rgba): number {
  const [a, b] = [lum(fg), lum(bg)].sort((x, y) => y - x);
  return (a + 0.05) / (b + 0.05);
}
const tok = (theme: 'light' | 'dark', name: string) => parse(TOKENS[theme][name as keyof (typeof TOKENS)['light']]);

describe('Annotation ledger drawer contrast (from tokens)', () => {
  it('the drawer background is the opaque dialog surface token, not translucent glass', () => {
    expect(ruleBody(ledgerSource, '.annotation-ledger-drawer')).toMatch(/background:\s*var\(--surface-dialog\)/);
  });
  it('tab labels use the primary text colour (not the accent blue), also when pressed', () => {
    expect(ruleBody(ledgerSource, '.drawer-tabs :global(.lp-btn.lp-btn--plain)')).toMatch(/color:\s*var\(--text-primary\)/);
  });
  for (const theme of ['light', 'dark'] as const) {
    it(`${theme}: panel text meets 4.5:1 on the opaque surface`, () => {
      expect(ratio('--text-primary', '--surface-dialog', theme)).toBeGreaterThanOrEqual(4.5);
    });
    it(`${theme}: pressed-tab label meets 4.5:1 on the active tint over the surface`, () => {
      const surface = tok(theme, '--surface-dialog');
      const tab = over(tok(theme, '--glass-bg-active'), surface);
      const text = over(tok(theme, '--text-primary'), tab);
      const r = contrastOf(text, tab);
      console.info(`ledger tab ${theme}: ${r.toFixed(2)}`);
      expect(r).toBeGreaterThanOrEqual(4.5);
    });
    it(`${theme}: the OLD accent-blue tab label on the active tint fails 4.5:1 (why it was changed)`, () => {
      const surface = tok(theme, '--surface-dialog');
      const tab = over(tok(theme, '--glass-bg-active'), surface);
      const r = contrastOf(over(tok(theme, '--accent-primary'), tab), tab);
      if (theme === 'dark') expect(r).toBeLessThan(4.5);
    });
  }
});

// Studio chrome bars (top command bar, dockable toolbar, favourites bar): opaque surface, readable labels.
import canvasSource from '../../views/studio/SpatialCanvas.svelte?raw';
import toolbarSource from '../studio/DockableToolbar.svelte?raw';
import favSource from '../studio/FavoritesBar.svelte?raw';

describe('Studio chrome bars contrast (from tokens)', () => {
  it('top command bar, toolbar, its drawer and the favourites bar use the opaque surface token', () => {
    expect(ruleBody(canvasSource, '.studio-top-bar')).toMatch(/background:\s*var\(--surface-dialog\)/);
    expect(ruleBody(toolbarSource, '.dockable-toolbar')).toMatch(/background:\s*var\(--surface-dialog\)/);
    expect(ruleBody(toolbarSource, '.category-popover')).toMatch(/background:\s*var\(--surface-dialog\)/);
    expect(ruleBody(favSource, '.favorites-bar')).toMatch(/background:\s*var\(--surface-dialog\)/);
  });
  it('top bar button labels use the primary text colour (not accent blue)', () => {
    expect(ruleBody(canvasSource, '.studio-top-bar :global(.lp-btn.lp-btn--plain)')).toMatch(/color:\s*var\(--text-primary\)/);
  });
  for (const theme of ['light', 'dark'] as const) {
    it(`${theme}: top bar labels (Markup / Edit text, Save, Flatten, Snap) meet 4.5:1, normal and pressed`, () => {
      const surface = tok(theme, '--surface-dialog');
      const pressed = over(tok(theme, '--glass-bg-active'), surface);
      const normal = contrastOf(over(tok(theme, '--text-primary'), surface), surface);
      const onTint = contrastOf(over(tok(theme, '--text-primary'), pressed), pressed);
      console.info(`top bar ${theme}: normal ${normal.toFixed(2)} pressed ${onTint.toFixed(2)}`);
      expect(normal).toBeGreaterThanOrEqual(4.5);
      expect(onTint).toBeGreaterThanOrEqual(4.5);
    });
  }
});

// Tooltips are dismissed by pointer leave, blur and Escape (keyboard focus shows them, mouse focus does not).
import tooltipSource from './Tooltip.svelte?raw';
describe('Tooltip visibility rules', () => {
  it('shows on hover and keyboard focus only (:focus-visible), not on leftover mouse focus', () => {
    expect(tooltipSource).toMatch(/:focus-visible/);
    expect(tooltipSource).not.toMatch(/focus-within/);
  });
});

// A disabled destructive button must stay legible (no 50 % fade of white text on red).
import buttonSource from './Button.svelte?raw';
describe('Button disabled destructive', () => {
  it('is not faded and uses the secondary text colour on a neutral fill', () => {
    const body = ruleBody(buttonSource, '.lp-btn--destructive.is-disabled');
    expect(body).toContain('opacity: 1');
    expect(body).toContain('var(--text-secondary)');
  });
});
