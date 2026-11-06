import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

// BUG-020: Fit Width / Fit Page must use the real page size (not A4), honour rotation, and
// account for the page's world offset (layoutPages puts page 1 at 32 px), so the page lands
// fully inside the canvas.

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async (cmd: string) => (cmd === 'read_file_bytes' ? [37, 80, 68, 70] : null)),
}));

import SpatialCanvas from '../studio/SpatialCanvas.svelte';
import { appState } from '$lib/state.svelte';
import { viewport } from '$lib/core/viewportEngine.svelte';

const PT_TO_CSS = 96 / 72;
const VIEW = 'fit-test';

function pageScreenRect(viewId: string, index: number) {
    const g = viewport.pages[index];
    const swap = g.rotation === 90 || g.rotation === 270;
    const w = (swap ? g.heightPt : g.widthPt) * PT_TO_CSS;
    const h = (swap ? g.widthPt : g.heightPt) * PT_TO_CSS;
    const m = viewport.worldToScreen(viewId);
    const tl = m.apply({ x: g.worldX, y: g.worldY });
    const br = m.apply({ x: g.worldX + w, y: g.worldY + h });
    return { left: tl.x, top: tl.y, right: br.x, bottom: br.y };
}

describe('viewport.fitPageInView (BUG-020)', () => {
    beforeEach(() => {
        viewport.layoutPages([
            { widthPt: 612, heightPt: 792 }, // US Letter
            { widthPt: 1224, heightPt: 792 }, // Tabloid landscape
            { widthPt: 612, heightPt: 792, rotation: 90 },
        ]);
        viewport.upsertView({ id: VIEW, zoom: 1, panX: 0, panY: 0, linkGroup: null, dpr: 1 });
    });

    test.each([0, 1, 2])('Fit Width puts page %i edge to edge inside a 990 px canvas', (index) => {
        viewport.fitPageInView(VIEW, 990, 700, index, 'width');
        const r = pageScreenRect(VIEW, index);
        expect(r.left).toBeGreaterThanOrEqual(0);
        expect(r.right).toBeLessThanOrEqual(990);
        expect(r.left).toBeCloseTo(990 - r.right, 3); // centred
        expect(r.right - r.left).toBeGreaterThan(990 * 0.9); // actually fills the width
        expect(r.top).toBeGreaterThanOrEqual(0);
        expect(r.top).toBeLessThan(40);
    });

    test.each([0, 1, 2])('Fit Page shows all of page %i, centred', (index) => {
        viewport.fitPageInView(VIEW, 990, 700, index, 'page');
        const r = pageScreenRect(VIEW, index);
        expect(r.left).toBeGreaterThanOrEqual(0);
        expect(r.top).toBeGreaterThanOrEqual(0);
        expect(r.right).toBeLessThanOrEqual(990);
        expect(r.bottom).toBeLessThanOrEqual(700);
        expect(r.left).toBeCloseTo(990 - r.right, 3);
        expect(r.top).toBeCloseTo(700 - r.bottom, 3);
        // Touches the limiting dimension (minus margins), not shrunk arbitrarily.
        expect(Math.max((r.right - r.left) / 990, (r.bottom - r.top) / 700)).toBeGreaterThan(0.9);
    });
});

describe('Studio zoom preset uses the active page (BUG-020)', () => {
    beforeEach(() => {
        (window as any).pdfjsLib = {
            GlobalWorkerOptions: {},
            getDocument: () => ({
                promise: Promise.resolve({
                    numPages: 1,
                    getPage: async () => ({
                        getViewport: ({ scale }: { scale: number }) => ({ width: 1224 * scale, height: 792 * scale }),
                        render: () => ({ promise: Promise.resolve() }),
                    }),
                    destroy: async () => {},
                }),
            }),
        };
        vi.spyOn(HTMLElement.prototype, 'clientWidth', 'get').mockReturnValue(990);
        vi.spyOn(HTMLElement.prototype, 'clientHeight', 'get').mockReturnValue(700);
        appState.clear();
        viewport.pages = [];
        appState.addFiles(['/docs/tabloid.pdf']);
    });

    afterEach(() => {
        cleanup();
        vi.restoreAllMocks();
        delete (window as any).pdfjsLib;
        appState.clear();
        viewport.pages = [];
    });

    test('Fit Width on a tabloid page keeps it inside the 990 px canvas', async () => {
        render(SpatialCanvas);
        await waitFor(() => expect(viewport.pages).toHaveLength(1));
        const preset = screen.getByTestId('zoom-preset') as HTMLSelectElement;
        await fireEvent.change(preset, { target: { value: 'fit-width' } });
        const r = pageScreenRect('main-canvas', 0);
        expect(r.left).toBeGreaterThanOrEqual(0);
        expect(r.right).toBeLessThanOrEqual(990);
        expect(r.right - r.left).toBeGreaterThan(900);
    });

    test('Fit Page on a tabloid page keeps all of it visible', async () => {
        render(SpatialCanvas);
        await waitFor(() => expect(viewport.pages).toHaveLength(1));
        const preset = screen.getByTestId('zoom-preset') as HTMLSelectElement;
        await fireEvent.change(preset, { target: { value: 'fit-page' } });
        const r = pageScreenRect('main-canvas', 0);
        expect(r.left).toBeGreaterThanOrEqual(0);
        expect(r.top).toBeGreaterThanOrEqual(0);
        expect(r.right).toBeLessThanOrEqual(990);
        expect(r.bottom).toBeLessThanOrEqual(700);
    });
});
