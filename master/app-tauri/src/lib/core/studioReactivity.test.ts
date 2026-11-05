import { describe, it, expect, vi } from 'vitest';
import { tools } from './toolEngine.svelte';
import { commands } from './commandEngine.svelte';
import { ink, generateSvgPathFromPoints, rdp, type Point } from './inkEngine.svelte';
import {
  resolveDock,
  CATEGORIES,
  type CategoryGroup,
  type ToolItem
} from '$lib/components/studio/DockableToolbar.svelte';

describe('ToolStateEngine Reactivity & Preset Scoping', () => {
  it('selectTool("pen.a") updates activeToolId, activeToolLabel and stroke support', () => {
    tools.selectTool('pen.a');
    expect(tools.activeToolId).toBe('pen.a');
    expect(tools.activeToolLabel).toBe('Pen A');
    expect(tools.supportsStrokeProperties).toBe(true);
    expect(tools.supportsEraserMode).toBe(false);
    expect(tools.activePreset).not.toBeNull();
    expect(tools.activePreset?.tool).toBe('pen.a');
  });

  it('selectTool("highlighter") updates activeToolId and activeToolLabel', () => {
    tools.selectTool('highlighter');
    expect(tools.activeToolId).toBe('highlighter');
    expect(tools.activeToolLabel).toBe('Highlighter');
    expect(tools.supportsStrokeProperties).toBe(true);
    expect(tools.supportsEraserMode).toBe(false);
  });

  it('selectTool("eraser.partial") updates eraser mode flag', () => {
    tools.selectTool('eraser.partial');
    expect(tools.activeToolId).toBe('eraser.partial');
    expect(tools.activeToolLabel).toBe('Partial Eraser');
    expect(tools.supportsEraserMode).toBe(true);
    expect(tools.supportsStrokeProperties).toBe(false);

    tools.selectTool('eraser.whole');
    expect(tools.activeToolId).toBe('eraser.whole');
    expect(tools.activeToolLabel).toBe('Whole Eraser');
    expect(tools.supportsEraserMode).toBe(true);
  });

  it('selectTool("pan") and selectTool("select") set neutral stroke states', () => {
    tools.selectTool('pan');
    expect(tools.activeToolId).toBe('pan');
    expect(tools.activeToolLabel).toBe('Pan Canvas');
    expect(tools.supportsStrokeProperties).toBe(false);
    expect(tools.supportsEraserMode).toBe(false);

    tools.selectTool('select');
    expect(tools.activeToolId).toBe('select');
    expect(tools.activeToolLabel).toBe('Select Object');
    expect(tools.supportsStrokeProperties).toBe(false);
    expect(tools.supportsEraserMode).toBe(false);
  });

  it('updateActivePreset mutates current preset without corrupting sibling presets', () => {
    tools.selectTool('pen.a');
    const originalPenBWidth = tools.presets['pen.b'].stroke.widthPt;

    // Mutate pen.a preset
    tools.updateActivePreset({
      stroke: {
        ...tools.activePreset!.stroke,
        widthPt: 9.75
      }
    });

    expect(tools.activePreset?.stroke.widthPt).toBe(9.75);
    expect(tools.presets['pen.a'].stroke.widthPt).toBe(9.75);

    // Verify pen.b preset was not affected
    expect(tools.presets['pen.b'].stroke.widthPt).toBe(originalPenBWidth);

    // Switch to pen.b and verify its preset is active
    tools.selectTool('pen.b');
    expect(tools.activePreset?.stroke.widthPt).toBe(originalPenBWidth);
  });
});

describe('CommandBus Engine Message Routing', () => {
  it('routes kind: "tool.select" message directly to tools state engine', () => {
    const handled = commands.dispatch({
      kind: 'tool.select',
      toolId: 'pen.b'
    });

    expect(handled).toBe(true);
    expect(tools.activeToolId).toBe('pen.b');
    expect(tools.activeToolLabel).toBe('Pen B');
  });

  it('routes kind: "execute" command: "tool.select" to tools state engine', () => {
    const handled = commands.dispatch({
      kind: 'execute',
      command: 'tool.select',
      args: 'highlighter'
    });

    expect(handled).toBe(true);
    expect(tools.activeToolId).toBe('highlighter');
    expect(tools.activeToolLabel).toBe('Highlighter');
  });

  it('routes kind: "style.apply" to update active preset', () => {
    tools.selectTool('pen.a');
    const handled = commands.dispatch({
      kind: 'style.apply',
      patch: {
        stroke: {
          ...tools.activePreset!.stroke,
          opacity: 0.42
        }
      }
    });

    expect(handled).toBe(true);
    expect(tools.activePreset?.stroke.opacity).toBe(0.42);
  });
});

describe('Toolbar resolveDock Math & Clamping', () => {
  const PW = 1200;
  const PH = 800;
  const SW = 50;
  const SH = 300;
  const THRESHOLD = 40;

  it('detects top docking when y <= threshold', () => {
    expect(resolveDock(300, 20, SW, SH, PW, PH, THRESHOLD)).toBe('top');
    expect(resolveDock(300, 40, SW, SH, PW, PH, THRESHOLD)).toBe('top');
  });

  it('detects left docking when x <= threshold and y > threshold', () => {
    expect(resolveDock(20, 200, SW, SH, PW, PH, THRESHOLD)).toBe('left');
    expect(resolveDock(40, 200, SW, SH, PW, PH, THRESHOLD)).toBe('left');
  });

  it('detects right docking when remaining right margin <= threshold', () => {
    // Right margin = PW - (x + SW) = 1200 - (1130 + 50) = 20 <= 40
    expect(resolveDock(1130, 200, SW, SH, PW, PH, THRESHOLD)).toBe('right');
    // Exact threshold boundary: 1200 - (1110 + 50) = 40
    expect(resolveDock(1110, 200, SW, SH, PW, PH, THRESHOLD)).toBe('right');
  });

  it('returns null (floating) when outside docking margins', () => {
    expect(resolveDock(200, 200, SW, SH, PW, PH, THRESHOLD)).toBeNull();
    expect(resolveDock(600, 400, SW, SH, PW, PH, THRESHOLD)).toBeNull();
  });

  it('correctly clamps coordinate ranges within parent bounds', () => {
    const minX = 8;
    const maxX = Math.max(minX, PW - SW - 8); // 1200 - 50 - 8 = 1142
    const minY = 8;
    const maxY = Math.max(minY, PH - SH - 8); // 800 - 300 - 8 = 492

    const clampX = (x: number) => Math.min(Math.max(minX, x), maxX);
    const clampY = (y: number) => Math.min(Math.max(minY, y), maxY);

    expect(clampX(-50)).toBe(8);
    expect(clampX(0)).toBe(8);
    expect(clampX(500)).toBe(500);
    expect(clampX(2000)).toBe(1142);

    expect(clampY(-100)).toBe(8);
    expect(clampY(0)).toBe(8);
    expect(clampY(250)).toBe(250);
    expect(clampY(1000)).toBe(492);
  });
});

describe('Phase D3 — InkEngine, Spline Math, Laser Pointer, & Smart Shapes', () => {
  it('generates a valid variable-width SVG path using Catmull-Rom spline envelope', () => {
    const pts: Point[] = [
      { x: 10, y: 10, p: 0.2 },
      { x: 30, y: 15, p: 0.8 },
      { x: 60, y: 40, p: 0.5 },
      { x: 90, y: 60, p: 0.3 }
    ];
    const path = generateSvgPathFromPoints(pts);
    expect(path).toBeTruthy();
    expect(path.startsWith('M ')).toBe(true);
    expect(path.includes(' C ')).toBe(true);
    expect(path.includes(' A ')).toBe(true);
    expect(path.endsWith(' Z')).toBe(true);
  });

  it('manages stroke lifecycle: startStroke -> addPoint -> endStroke', () => {
    ink.completedStrokes = [];
    ink.startStroke(50, 50, 0.4, 'pen.a');
    expect(ink.isDrawing).toBe(true);
    expect(ink.currentStroke).not.toBeNull();
    expect(ink.currentStroke?.points.length).toBe(1);

    ink.addPoint(60, 65, 0.6);
    ink.addPoint(75, 80, 0.7);
    expect(ink.currentStroke?.points.length).toBe(3);

    const committed = ink.endStroke();
    expect(committed).not.toBeNull();
    expect(ink.isDrawing).toBe(false);
    expect(ink.completedStrokes.length).toBe(1);
    expect(ink.completedStrokes[0].id).toBe(committed?.id);
  });

  it('prunes expired points in laser pointer trail with 1800ms decay', () => {
    ink.laserPoints = [];
    const now = Date.now();
    ink.laserPoints.push({ x: 10, y: 10, p: 1.0, t: now - 2000 }); // expired
    ink.laserPoints.push({ x: 20, y: 20, p: 1.0, t: now - 500 });  // active
    ink.laserPoints.push({ x: 30, y: 30, p: 1.0, t: now });        // active

    ink.updateLaser();
    expect(ink.laserPoints.length).toBe(2);
    expect(ink.laserPoints[0].x).toBe(20);
    expect(ink.laserPoints[1].x).toBe(30);
  });

  it('smart shapes: recognizes a straight line during dwell', () => {
    ink.startStroke(0, 0, 0.5, 'pen.a');
    for (let x = 10; x <= 100; x += 10) {
      // Add slight jitter along y
      ink.addPoint(x, (x % 20 === 0 ? 0.5 : -0.5), 0.5);
    }
    const recognized = ink.recognizeShape();
    expect(recognized).toBe(true);
    expect(ink.currentStroke?.points.length).toBe(2);
    expect(ink.currentStroke?.points[0].x).toBe(0);
    expect(ink.currentStroke?.points[1].x).toBe(100);
    ink.endStroke();
  });

  it('smart shapes: recognizes a rectangle during dwell', () => {
    ink.startStroke(10, 10, 0.5, 'pen.a');
    // Draw top edge
    ink.addPoint(50, 10, 0.5);
    ink.addPoint(100, 10, 0.5);
    // Draw right edge
    ink.addPoint(100, 40, 0.5);
    ink.addPoint(100, 80, 0.5);
    // Draw bottom edge
    ink.addPoint(50, 80, 0.5);
    ink.addPoint(10, 80, 0.5);
    // Draw left edge back to start
    ink.addPoint(10, 40, 0.5);
    ink.addPoint(10, 12, 0.5);

    const recognized = ink.recognizeShape();
    expect(recognized).toBe(true);
    expect(ink.currentStroke?.points.length).toBe(5); // 4 corners + closed point
    ink.endStroke();
  });

  it('smart shapes: recognizes a triangle during dwell', () => {
    ink.startStroke(10, 80, 0.5, 'pen.a');
    ink.addPoint(30, 45, 0.5);
    ink.addPoint(50, 10, 0.5); // Apex
    ink.addPoint(70, 45, 0.5);
    ink.addPoint(90, 80, 0.5); // Right base
    ink.addPoint(50, 80, 0.5);
    ink.addPoint(12, 78, 0.5); // Back to start

    const recognized = ink.recognizeShape();
    expect(recognized).toBe(true);
    expect(ink.currentStroke?.points.length).toBe(4); // 3 vertices + closed point
    ink.endStroke();
  });

  it('smart shapes: recognizes an ellipse/circle during dwell', () => {
    ink.startStroke(150, 100, 0.5, 'pen.a');
    // Draw 36 points of a circle with radius 50 centered at (100, 100)
    for (let i = 1; i <= 36; i++) {
      const th = (i / 36) * Math.PI * 2;
      ink.addPoint(100 + Math.cos(th) * 50, 100 + Math.sin(th) * 50, 0.5);
    }

    const recognized = ink.recognizeShape();
    expect(recognized).toBe(true);
    // Snaps to perfect 37-point ellipse path (0 to 36 inclusive)
    expect(ink.currentStroke?.points.length).toBe(37);
    ink.endStroke();
  });

  it('smart shapes: Ramer-Douglas-Peucker simplifies noisy polylines into core vertices', () => {
    const noisyLine: Point[] = [
      { x: 0, y: 0, p: 0.5 },
      { x: 25, y: 0.5, p: 0.5 },
      { x: 50, y: -0.4, p: 0.5 },
      { x: 75, y: 0.3, p: 0.5 },
      { x: 100, y: 0, p: 0.5 },
    ];
    const simplified = rdp(noisyLine, 2);
    expect(simplified.length).toBe(2);
    expect(simplified[0].x).toBe(0);
    expect(simplified[1].x).toBe(100);
  });

  it('boolean eraser: eraseWhole removes intersecting stroke', () => {
    ink.completedStrokes = [];
    ink.startStroke(100, 100, 0.5, 'pen.a');
    ink.addPoint(150, 100, 0.5);
    ink.addPoint(200, 100, 0.5);
    ink.endStroke();
    expect(ink.completedStrokes.length).toBe(1);

    // Eraser misses
    const hitMiss = ink.eraseWhole({ x: 50, y: 50, p: 1 }, 10);
    expect(hitMiss).toBe(false);
    expect(ink.completedStrokes.length).toBe(1);

    // Eraser hits segment at (150, 100)
    const hitSuccess = ink.eraseWhole({ x: 150, y: 105, p: 1 }, 10);
    expect(hitSuccess).toBe(true);
    expect(ink.completedStrokes.length).toBe(0);
  });

  it('boolean eraser: erasePartial splits a single stroke into two', () => {
    ink.completedStrokes = [];
    ink.startStroke(0, 0, 0.5, 'pen.a');
    ink.addPoint(20, 0, 0.5);
    ink.addPoint(40, 0, 0.5);
    ink.addPoint(60, 0, 0.5);
    ink.addPoint(80, 0, 0.5);
    ink.addPoint(100, 0, 0.5);
    ink.endStroke();
    expect(ink.completedStrokes.length).toBe(1);

    // Bisect stroke around (40, 0) with radius 15 (covering point 40,0)
    const bisected = ink.erasePartial({ x: 40, y: 0, p: 1 }, 12);
    expect(bisected).toBe(true);
    // The stroke should be split into 2 remaining segments: [0, 20] and [60, 80, 100]
    expect(ink.completedStrokes.length).toBe(2);
    expect(ink.completedStrokes[0].points.length).toBe(2);
    expect(ink.completedStrokes[1].points.length).toBe(3);
  });

  describe('Phase U4: Grouped 41-Tool Drawer & Shortcut Engine', () => {
    it('covers all tool families and at least 41 tools in CATEGORIES', () => {
      const allTools = CATEGORIES.flatMap((c: CategoryGroup) => c.tools);
      expect(allTools.length).toBeGreaterThanOrEqual(41);
      expect(CATEGORIES.map((c: CategoryGroup) => c.id)).toEqual([
        'nav',
        'ink',
        'erase',
        'shapes',
        'review',
        'measure',
        'insert'
      ]);

      const toolIds = allTools.map((t: ToolItem) => t.id);
      expect(toolIds).toContain('pan');
      expect(toolIds).toContain('select.marquee');
      expect(toolIds).toContain('select.lasso');
      expect(toolIds).toContain('pen.a');
      expect(toolIds).toContain('highlighter');
      expect(toolIds).toContain('laser');
      expect(toolIds).toContain('eraser.partial');
      expect(toolIds).toContain('eraser.whole');
      expect(toolIds).toContain('shape.polyline');
      expect(toolIds).toContain('shape.cloud');
      expect(toolIds).toContain('hatch.area');
      expect(toolIds).toContain('review.squiggle');
      expect(toolIds).toContain('review.redact');
      expect(toolIds).toContain('measure.calibrate');
      expect(toolIds).toContain('measure.protractor');
      expect(toolIds).toContain('insert.latex');
      expect(toolIds).toContain('insert.signature');
      expect(toolIds).toContain('insert.audio');
    });

    it('shortcutFor returns registered chords from commandEngine', () => {
      expect(commands.shortcutFor('pan')).toBe('H');
      expect(commands.shortcutFor('select')).toBe('V');
      expect(commands.shortcutFor('pen.a')).toBe('P');
      expect(commands.shortcutFor('highlighter')).toBe('I');
      expect(commands.shortcutFor('laser')).toBe('L');
      expect(commands.shortcutFor('eraser.partial')).toBe('E');
    });

    it('pinFavorite and isFavorite toggle tool in favorites list', () => {
      expect(tools.isFavorite('shape.rect')).toBe(false);
      tools.pinFavorite('shape.rect');
      expect(tools.isFavorite('shape.rect')).toBe(true);
      tools.pinFavorite('shape.rect');
      expect(tools.isFavorite('shape.rect')).toBe(false);
    });
  });
});
