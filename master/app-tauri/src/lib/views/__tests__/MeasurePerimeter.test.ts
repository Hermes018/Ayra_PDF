import { expect, test, beforeEach, describe } from 'vitest';
import { measureEngine } from '$lib/core/measureEngine.svelte';
import { commands } from '$lib/core/commandEngine.svelte';

// BUG-022: a perimeter measurement is a closed shape. A 72 pt square must measure 288 pt
// (4 sides), not 216 pt (the 3 drawn segments).

function measurePerimeter(pts: Array<[number, number]>) {
    measureEngine.cancelDraft();
    for (const [x, y] of pts) measureEngine.startOrAddPoint('measure.perimeter', 0, { x, y });
    return measureEngine.commitActiveDraft();
}

describe('Perimeter closes the shape (BUG-022)', () => {
    beforeEach(() => {
        measureEngine.measurements = [];
        commands.load(commands.doc);
    });

    test('72 pt square → 288 pt', () => {
        const rec = measurePerimeter([[0, 0], [72, 0], [72, 72], [0, 72]]);
        expect(rec?.kind).toBe('perimeter');
        expect(rec?.rawPtValue).toBeCloseTo(288, 6);
        expect(rec?.perimeterCalibrated).toBeCloseTo(rec!.calibratedValue, 9);
    });

    test('3-4-5 triangle → 12 (closing side included)', () => {
        const rec = measurePerimeter([[0, 0], [30, 0], [30, 40]]);
        expect(rec?.rawPtValue).toBeCloseTo(120, 6);
    });

    test('polylength stays open', () => {
        measureEngine.cancelDraft();
        for (const [x, y] of [[0, 0], [72, 0], [72, 72], [0, 72]] as Array<[number, number]>) {
            measureEngine.startOrAddPoint('measure.polylength', 0, { x, y });
        }
        expect(measureEngine.commitActiveDraft()?.rawPtValue).toBeCloseTo(216, 6);
    });

    test('an OPEN polyline does not include a closing segment: (0,0)→(72,0)→(72,72) is 144 pt, not 144 + 101.8', () => {
        measureEngine.cancelDraft();
        for (const [x, y] of [[0, 0], [72, 0], [72, 72]] as Array<[number, number]>) {
            measureEngine.startOrAddPoint('measure.polylength', 0, { x, y });
        }
        const rec = measureEngine.commitActiveDraft();
        expect(rec?.kind).toBe('polylength');
        expect(rec?.rawPtValue).toBeCloseTo(144, 6);
        expect(rec?.rawPtValue).not.toBeCloseTo(144 + Math.hypot(72, 72), 1);
    });
});
