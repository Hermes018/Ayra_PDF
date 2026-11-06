import { expect, test, beforeEach, afterEach, describe } from 'vitest';
import { render, cleanup, fireEvent } from '@testing-library/svelte';
import SpatialCanvas from '../studio/SpatialCanvas.svelte';
import { appState } from '$lib/state.svelte';
import { viewport } from '$lib/core/viewportEngine.svelte';
import { tools } from '$lib/core/toolEngine.svelte';
import { commands } from '$lib/core/commandEngine.svelte';
import { ink } from '$lib/core/inkEngine.svelte';
import { shapes } from '$lib/core/shapeAndHatchEngine.svelte';

// BUG-019: an eraser gesture must be one undo step. Before the fix erasing was not recorded, so
// Undo reverted the previous pen stroke instead (removing a stroke the user still wanted).

function drawCommittedStroke(y: number): string {
    // Same commit path as SpatialCanvas.handlePointerUp for a pen stroke.
    const before = [...ink.completedStrokes];
    ink.startStroke(100, y, 0.5, 'pen.a');
    for (const x of [150, 200, 250, 300]) ink.addPoint(x, y, 0.5);
    const committed = ink.endStroke();
    if (!committed) throw new Error('stroke not committed');
    shapes.recordStrokeHistory(before, [...ink.completedStrokes], `Ink ${committed.tool}`);
    return committed.id;
}

function ids(): string[] {
    return ink.completedStrokes.map((s) => s.id);
}

async function eraseAt(world: { x: number; y: number }) {
    const wet = document.getElementById('wet-layer') as Element;
    const pt = viewport.worldToScreen('main-canvas').apply(world);
    await fireEvent.pointerDown(wet, { clientX: pt.x, clientY: pt.y, button: 0, buttons: 1, pointerId: 7 });
    await fireEvent.pointerUp(wet, { clientX: pt.x, clientY: pt.y, button: 0, buttons: 0, pointerId: 7 });
}

describe('Eraser gestures are undoable (BUG-019)', () => {
    let a = '';
    let b = '';

    beforeEach(() => {
        appState.clear();
        viewport.pages = [];
        ink.completedStrokes = [];
        commands.load(commands.doc);
        render(SpatialCanvas);
        a = drawCommittedStroke(100);
        b = drawCommittedStroke(300);
    });

    afterEach(() => {
        cleanup();
        ink.completedStrokes = [];
        commands.load(commands.doc);
        tools.selectTool('pan');
    });

    test('whole eraser: undo restores the erased stroke and keeps the other one', async () => {
        tools.selectTool('eraser.whole');
        await eraseAt({ x: 200, y: 100 });
        expect(ids()).toEqual([b]);

        expect(commands.undo()).toBe(true);
        expect(ids()).toEqual([a, b]);

        expect(commands.redo()).toBe(true);
        expect(ids()).toEqual([b]);
    });

    test('partial eraser: undo restores the exact original stroke geometry', async () => {
        const original = JSON.parse(JSON.stringify(ink.completedStrokes));
        tools.selectTool('eraser.partial');
        await eraseAt({ x: 200, y: 100 });
        expect(ids()).toEqual([`${a}-part-0`, `${a}-part-1`, b]);

        expect(commands.undo()).toBe(true);
        expect(JSON.parse(JSON.stringify(ink.completedStrokes))).toEqual(original);

        // One more undo removes the last pen stroke, as the user expects.
        expect(commands.undo()).toBe(true);
        expect(ids()).toEqual([a]);
    });

    test('an eraser click that hits nothing adds no history entry', async () => {
        tools.selectTool('eraser.whole');
        await eraseAt({ x: 600, y: 600 });
        expect(ids()).toEqual([a, b]);
        expect(commands.undo()).toBe(true);
        expect(ids()).toEqual([a]); // undid stroke B, not a phantom erase
    });
});
