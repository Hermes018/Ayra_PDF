import { expect, test, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent } from '@testing-library/svelte';
import AnnotationLedger from '$lib/components/studio/AnnotationLedger.svelte';
import { inserts } from '$lib/core/insertEngine.svelte';
import { ink } from '$lib/core/inkEngine.svelte';
import { commands } from '$lib/core/commandEngine.svelte';

// BUG-017: a Draw-mode signature must place the drawn strokes (every pen-down → pen-up), not a
// "Signed: <name>" text box. Strokes are committed as ink (saved as PDF Ink annotations by
// studio_save_annotations) grouped by one groupId, in one undo step.

async function drawStroke(pad: Element, pts: Array<[number, number]>) {
    const [first, ...rest] = pts;
    await fireEvent.pointerDown(pad, { clientX: first[0], clientY: first[1], pointerId: 3, buttons: 1 });
    for (const [x, y] of rest) await fireEvent.pointerMove(pad, { clientX: x, clientY: y, pointerId: 3, buttons: 1 });
    await fireEvent.pointerUp(pad, { clientX: rest.at(-1)![0], clientY: rest.at(-1)![1], pointerId: 3 });
}

describe('Draw-mode signature (BUG-017)', () => {
    beforeEach(() => {
        ink.completedStrokes = [];
        commands.load(commands.doc);
        inserts.pendingInsertPt = { x: 300, y: 400, pageIndex: 1 };
        inserts.signatureMode = 'draw';
        inserts.signatureModalOpen = true;
        render(AnnotationLedger);
    });

    afterEach(() => {
        cleanup();
        inserts.signatureModalOpen = false;
        ink.completedStrokes = [];
        commands.load(commands.doc);
    });

    test('stamps every drawn stroke as grouped ink at the insert point, with no text', async () => {
        expect(screen.getByTestId('signature-tab-draw')).toHaveAttribute('aria-pressed', 'true');
        const pad = screen.getByTestId('signature-pad');
        await drawStroke(pad, [[10, 10], [40, 30], [70, 12]]);
        await drawStroke(pad, [[90, 40], [120, 20]]);
        // Both strokes are previewed, not just the last one.
        expect(pad.querySelectorAll('path')).toHaveLength(2);

        await fireEvent.click(screen.getByTestId('signature-stamp-btn'));

        const placed = ink.completedStrokes;
        expect(placed).toHaveLength(2);
        for (const s of placed) {
            expect(s.tool).toBe('pen.a');
            expect(s.text).toBeUndefined();
            expect(s.pageIndex).toBe(1);
            expect(s.groupId).toMatch(/^sig-/);
        }
        expect(placed[0].groupId).toBe(placed[1].groupId);
        expect(placed[0].points.map((p) => [p.x, p.y])).toEqual([[310, 410], [340, 430], [370, 412]]);
        expect(placed[1].points.map((p) => [p.x, p.y])).toEqual([[390, 440], [420, 420]]);
        expect(JSON.stringify(placed)).not.toContain('Signed:');
        expect(inserts.signatureModalOpen).toBe(false);

        // One undo removes the whole signature.
        expect(commands.undo()).toBe(true);
        expect(ink.completedStrokes).toHaveLength(0);
    });

    test('stamping with an empty pad places nothing and keeps the dialog open', async () => {
        await fireEvent.click(screen.getByTestId('signature-stamp-btn'));
        expect(ink.completedStrokes).toHaveLength(0);
        expect(inserts.signatureModalOpen).toBe(true);
        expect(screen.getByTestId('signature-draw-hint')).toHaveTextContent(/draw your signature/i);
    });

    test('Clear empties the pad', async () => {
        const pad = screen.getByTestId('signature-pad');
        await drawStroke(pad, [[10, 10], [40, 30]]);
        await fireEvent.click(screen.getByTestId('signature-clear-btn'));
        expect(pad.querySelectorAll('path')).toHaveLength(0);
    });

    test('the type and webcam tabs are reachable by test id', async () => {
        await fireEvent.click(screen.getByTestId('signature-tab-type'));
        expect(screen.getByRole('textbox', { name: 'Typed Signature Name' })).toBeInTheDocument();
        await fireEvent.click(screen.getByTestId('signature-tab-webcam'));
        expect(screen.getByTestId('signature-tab-webcam')).toHaveAttribute('aria-pressed', 'true');
    });
});
