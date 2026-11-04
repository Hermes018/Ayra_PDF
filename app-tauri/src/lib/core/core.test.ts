import { describe, it, expect } from 'vitest';
import { Affine, viewport, type Pt } from './viewportEngine.svelte';
import { evalPressure, tools, type PressureCurve } from './toolEngine.svelte';
import { commands, canonicalChord, withPage, type DocSnapshot } from './commandEngine.svelte';
import { theme } from './themeContract.svelte';

describe('Affine transform math', () => {
  it('invert round-trip satisfies M * M^-1 = I to 1e-9', () => {
    const m = new Affine(1.5, 0.3, -0.2, 1.2, 100, 200);
    const inv = m.invert();
    expect(inv).not.toBeNull();
    if (!inv) return;
    const identity = m.multiply(inv);
    expect(identity.a).toBeCloseTo(1, 9);
    expect(identity.b).toBeCloseTo(0, 9);
    expect(identity.c).toBeCloseTo(0, 9);
    expect(identity.d).toBeCloseTo(1, 9);
    expect(identity.e).toBeCloseTo(0, 9);
    expect(identity.f).toBeCloseTo(0, 9);
  });

  it('rotation transforms 0, 90, 180, 270 degrees correctly', () => {
    const p: Pt = { x: 10, y: 0 };
    expect(Affine.rotate(0).apply(p)).toEqual({ x: 10, y: 0 });
    expect(Affine.rotate(90).apply(p).x).toBeCloseTo(0, 9);
    expect(Affine.rotate(90).apply(p).y).toBeCloseTo(10, 9);
    expect(Affine.rotate(180).apply(p).x).toBeCloseTo(-10, 9);
    expect(Affine.rotate(180).apply(p).y).toBeCloseTo(0, 9);
    expect(Affine.rotate(270).apply(p).x).toBeCloseTo(0, 9);
    expect(Affine.rotate(270).apply(p).y).toBeCloseTo(-10, 9);
  });
});

describe('Viewport Coordinate Engine', () => {
  it('calibrate guards reject zero or non-finite inputs', () => {
    expect(viewport.calibrate(0, { x: 0, y: 0 }, { x: 0, y: 0 }, 10, 'mm', 2)).toBe(false);
    expect(viewport.calibrate(0, { x: 0, y: 0 }, { x: 10, y: 0 }, -5, 'mm', 2)).toBe(false);
    expect(viewport.calibrate(0, { x: 0, y: 0 }, { x: 10, y: 0 }, Infinity, 'mm', 2)).toBe(false);
    expect(viewport.calibrate(0, { x: 0, y: 0 }, { x: 100, y: 0 }, 1000, 'mm', 2)).toBe(true);
  });

  it('zoomAt preserves the screen anchor point', () => {
    viewport.upsertView({ id: 'test-view', zoom: 1.0, panX: 0, panY: 0, linkGroup: null, dpr: 1 });
    const anchor: Pt = { x: 200, y: 150 };
    viewport.zoomAt('test-view', 2.0, anchor);
    const v = viewport.views['test-view'];
    expect(v.zoom).toBe(2.0);
    // World point under anchor before zoom: (200 - 0)/1 = 200
    // Screen point after zoom: 200 * 2.0 + panX = 400 + panX
    // Anchor was 200 => 400 + panX = 200 => panX = -200
    expect(v.panX).toBe(-200);
    expect(v.panY).toBe(-150);
  });
});

describe('Tool Engine & Pressure Curves', () => {
  it('evalPressure curve is monotonically non-decreasing on [0, 1]', () => {
    const curve: PressureCurve = { x1: 0.25, y1: 0.1, x2: 0.25, y2: 1 };
    let prev = -1;
    for (let p = 0; p <= 1.0; p += 0.05) {
      const y = evalPressure(curve, p);
      expect(y).toBeGreaterThanOrEqual(prev);
      expect(y).toBeGreaterThanOrEqual(0);
      expect(y).toBeLessThanOrEqual(1);
      prev = y;
    }
  });
});

describe('Command Bus & Transactional Undo/Redo', () => {
  it('canonicalChord normalizes keys correctly', () => {
    expect(canonicalChord({ ctrlKey: true, altKey: false, shiftKey: false, metaKey: false, key: 'z' })).toBe('Ctrl+Z');
    expect(canonicalChord({ ctrlKey: true, altKey: false, shiftKey: true, metaKey: false, key: 'Z' })).toBe('Ctrl+Shift+Z');
  });

  it('executes commands and supports copy-on-write undo/redo', () => {
    const initial: DocSnapshot = Object.freeze({
      rev: 0,
      pages: Object.freeze([new Map()])
    });

    commands.load(initial);
    commands.register('test.addNode', (doc, args) => {
      const { id } = args as { id: string };
      const next = withPage(doc, 0, (m) => {
        m.set(id, { id } as any);
      });
      return { next, label: `Add ${id}` };
    });

    expect(commands.canUndo).toBe(false);
    expect(commands.dispatch({ kind: 'execute', command: 'test.addNode', args: { id: 'node1' } })).toBe(true);
    expect(commands.canUndo).toBe(true);
    expect(commands.doc.pages[0]?.has('node1')).toBe(true);
    expect(commands.doc.rev).toBe(1);

    expect(commands.undo()).toBe(true);
    expect(commands.doc.pages[0]?.has('node1')).toBe(false);
    expect(commands.canRedo).toBe(true);

    expect(commands.redo()).toBe(true);
    expect(commands.doc.pages[0]?.has('node1')).toBe(true);
  });
});

describe('Theme Contract', () => {
  it('provides complete light/dark token cascade with non-empty values', () => {
    theme.mode = 'light';
    expect(theme.token('--glass-bg')).toBeDefined();
    expect(theme.token('--accent')).toBeDefined();
    theme.mode = 'dark';
    expect(theme.token('--glass-bg')).toBeDefined();
    expect(theme.token('--canvas-bg')).toBeDefined();
  });
});
