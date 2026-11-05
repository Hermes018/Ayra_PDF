import { describe, it, expect, beforeEach } from 'vitest';
import { ink, type ActiveStroke } from './inkEngine.svelte';
import {
  shapes,
  computeStrokeBBox,
  generateRevisionCloudPath,
  generatePolylineSvgPath,
  generateSquiggleSvgPath,
  generateLineEndingPath,
  getStrokeDashArray,
} from './shapeAndHatchEngine.svelte';
import { selection, pointInPolygon } from './selectionEngine.svelte';
import { commands } from './commandEngine.svelte';
import { tools } from './toolEngine.svelte';

function makeStroke(
  id: string,
  x0: number,
  y0: number,
  x1: number,
  y1: number,
  tool: ActiveStroke['tool'] = 'shape.rect'
): ActiveStroke {
  return {
    id,
    tool,
    points: [
      { x: x0, y: y0, p: 0.5 },
      { x: x1, y: y0, p: 0.5 },
      { x: x1, y: y1, p: 0.5 },
      { x: x0, y: y1, p: 0.5 },
    ],
    preset: tools.presets[tool] ?? tools.presets['pen.a'],
    color: 'var(--glass-text)',
    widthPt: 2,
    opacity: 1,
    bbox: [x0, y0, x1, y1],
  };
}

describe('Phase D4 — Shape, Polyline, Cloud, Hatch & Selection Engine', () => {
  beforeEach(() => {
    ink.completedStrokes = [];
    selection.clearSelection();
    shapes.cancelActiveShape();
  });

  it('flipSelected("horizontal") and flipSelected("vertical") reflect coordinates across selection center', () => {
    // Create an asymmetric triangle inside [100..200, 100..300] -> center = (150, 200)
    const tri: ActiveStroke = {
      id: 'tri-1',
      tool: 'shape.polygon',
      points: [
        { x: 100, y: 100, p: 0.5 },
        { x: 200, y: 120, p: 0.5 },
        { x: 120, y: 300, p: 0.5 },
      ],
      preset: tools.presets['pen.a'],
      color: 'var(--glass-text)',
      widthPt: 2,
      opacity: 1,
    };
    ink.completedStrokes = [tri];
    selection.selectedIds = ['tri-1'];

    // Horizontal flip: x' = 2*150 - x = 300 - x
    selection.flipSelected('horizontal');
    const flippedH = ink.completedStrokes[0];
    expect(flippedH.points[0].x).toBeCloseTo(200, 5);
    expect(flippedH.points[1].x).toBeCloseTo(100, 5);
    expect(flippedH.points[2].x).toBeCloseTo(180, 5);
    expect(flippedH.flipH).toBe(true);

    // Vertical flip: y' = 2*200 - y = 400 - y
    selection.flipSelected('vertical');
    const flippedHV = ink.completedStrokes[0];
    expect(flippedHV.points[0].y).toBeCloseTo(300, 5);
    expect(flippedHV.points[1].y).toBeCloseTo(280, 5);
    expect(flippedHV.points[2].y).toBeCloseTo(100, 5);
    expect(flippedHV.flipV).toBe(true);
  });

  it('alignSelected("left" | "center" | "right" | "top" | "middle" | "bottom") aligns multiple objects accurately', () => {
    const s1 = makeStroke('s1', 20, 40, 60, 80);   // width 40, height 40, minX 20, maxX 60
    const s2 = makeStroke('s2', 120, 140, 200, 220); // width 80, height 80, minX 120, maxX 200
    ink.completedStrokes = [s1, s2];
    selection.selectedIds = ['s1', 's2'];

    // Align left -> both minX should become 20
    selection.alignSelected('left');
    expect(computeStrokeBBox(ink.completedStrokes[0]).minX).toBeCloseTo(20, 5);
    expect(computeStrokeBBox(ink.completedStrokes[1]).minX).toBeCloseTo(20, 5);

    // Undo restores s2 to minX = 120
    commands.undo();
    expect(computeStrokeBBox(ink.completedStrokes[1]).minX).toBeCloseTo(120, 5);

    // Align right -> both maxX should become 200
    selection.alignSelected('right');
    expect(computeStrokeBBox(ink.completedStrokes[0]).maxX).toBeCloseTo(200, 5);
    expect(computeStrokeBBox(ink.completedStrokes[1]).maxX).toBeCloseTo(200, 5);

    // Align top -> both minY should become 40
    selection.alignSelected('top');
    expect(computeStrokeBBox(ink.completedStrokes[0]).minY).toBeCloseTo(40, 5);
    expect(computeStrokeBBox(ink.completedStrokes[1]).minY).toBeCloseTo(40, 5);

    // Align bottom -> both maxY should become 120 (since s2 was moved to y=40..120)
    selection.alignSelected('bottom');
    expect(computeStrokeBBox(ink.completedStrokes[0]).maxY).toBeCloseTo(120, 5);
    expect(computeStrokeBBox(ink.completedStrokes[1]).maxY).toBeCloseTo(120, 5);
  });

  it('groupSelected() and ungroupSelected() assign and clear shared groupId and select together', () => {
    const s1 = makeStroke('g1', 10, 10, 50, 50);
    const s2 = makeStroke('g2', 80, 80, 120, 120);
    ink.completedStrokes = [s1, s2];
    selection.selectedIds = ['g1', 'g2'];

    const gid = selection.groupSelected();
    expect(gid).toBeTruthy();
    expect(ink.completedStrokes[0].groupId).toBe(gid);
    expect(ink.completedStrokes[1].groupId).toBe(gid);

    // Selecting one member of the group automatically selects all members
    selection.clearSelection();
    selection.selectAtPoint(25, 25, false);
    expect(selection.selectedIds).toEqual(['g1', 'g2']);

    // Ungroup clears groupId
    selection.ungroupSelected();
    expect(ink.completedStrokes[0].groupId).toBeUndefined();
    expect(ink.completedStrokes[1].groupId).toBeUndefined();
  });

  it('generates valid SVG paths for revision cloud (scalloped cubic Bézier arcs), polylines, squiggles, and line endings', () => {
    const cloudPath = generateRevisionCloudPath([
      { x: 0, y: 0, p: 0.5 },
      { x: 100, y: 0, p: 0.5 },
      { x: 100, y: 80, p: 0.5 },
      { x: 0, y: 80, p: 0.5 },
    ]);
    expect(cloudPath.startsWith('M ')).toBe(true);
    expect(cloudPath.includes(' C ')).toBe(true);
    expect(cloudPath.endsWith(' Z')).toBe(true);

    const polyOpen = generatePolylineSvgPath([
      { x: 10, y: 10, p: 0.5 },
      { x: 40, y: 25, p: 0.5 },
      { x: 80, y: 10, p: 0.5 },
    ], false);
    expect(polyOpen).toBe('M 10.00 10.00 L 40.00 25.00 L 80.00 10.00');

    const squiggle = generateSquiggleSvgPath({
      minX: 10,
      minY: 10,
      maxX: 90,
      maxY: 30,
      width: 80,
      height: 20,
      cx: 50,
      cy: 20,
    });
    expect(squiggle.includes(' Q ')).toBe(true);

    const arrow = generateLineEndingPath(
      { x: 100, y: 50, p: 0.5 },
      { x: 50, y: 50, p: 0.5 },
      'closed-arrow',
      2
    );
    expect(arrow).not.toBeNull();
    expect(arrow?.filled).toBe(true);
    expect(getStrokeDashArray('dashed', 2)).toBe('8 5');
    expect(getStrokeDashArray('dash-dot', 2)).toBe('9 4 2 4');
  });

  it('point-in-polygon lasso hit test selects only objects inside the freeform lasso polygon', () => {
    const insideStroke = makeStroke('in-1', 40, 40, 60, 60);
    const outsideStroke = makeStroke('out-1', 200, 200, 240, 240);
    ink.completedStrokes = [insideStroke, outsideStroke];

    const lassoPolygon = [
      { x: 20, y: 20 },
      { x: 100, y: 20 },
      { x: 100, y: 100 },
      { x: 20, y: 100 },
    ];
    expect(pointInPolygon({ x: 50, y: 50 }, lassoPolygon)).toBe(true);
    expect(pointInPolygon({ x: 220, y: 220 }, lassoPolygon)).toBe(false);

    selection.startLasso(20, 20);
    selection.updateLasso(100, 20);
    selection.updateLasso(100, 100);
    selection.updateLasso(20, 100);
    const selected = selection.commitLasso(false);
    expect(selected).toEqual(['in-1']);
  });

  it('supports Z-order reordering, duplicate (Ctrl+D), format painter, and snapshot stamp creation', () => {
    const s1 = makeStroke('z1', 10, 10, 30, 30);
    const s2 = makeStroke('z2', 40, 40, 60, 60);
    const s3 = makeStroke('z3', 70, 70, 90, 90);
    s1.hatchPattern = 'diagonal-steel';
    ink.completedStrokes = [s1, s2, s3];

    // Bring z1 to front
    selection.selectedIds = ['z1'];
    selection.reorderSelected('front');
    expect(ink.completedStrokes.map((s) => s.id)).toEqual(['z2', 'z3', 'z1']);

    // Send z1 to back
    selection.reorderSelected('back');
    expect(ink.completedStrokes.map((s) => s.id)).toEqual(['z1', 'z2', 'z3']);

    // Format painter copies hatchPattern from z1 to z2
    selection.copyStyleFromSelected();
    selection.applyFormatPainterToIds(['z2']);
    expect(ink.completedStrokes[1].hatchPattern).toBe('diagonal-steel');

    // Duplicate z1
    const dupIds = selection.duplicateSelected(20, 20);
    expect(dupIds.length).toBe(1);
    expect(ink.completedStrokes.length).toBe(4);

    // Snapshot stamp creation
    const snap = selection.createSnapshotStamp({ minX: 10, minY: 10, maxX: 90, maxY: 70 }, 'data:image/png;base64,iVBORw0KGgo=');
    expect(snap.tool).toBe('select.snapshot');
    expect(ink.completedStrokes.length).toBe(5);
  });
});
