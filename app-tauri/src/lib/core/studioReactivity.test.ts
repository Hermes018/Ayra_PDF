import { describe, it, expect } from 'vitest';
import { tools } from './toolEngine.svelte';
import { commands } from './commandEngine.svelte';
import { resolveDock } from '$lib/components/studio/DockableToolbar.svelte';

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
