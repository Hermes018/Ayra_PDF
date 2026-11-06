import { expect, test, describe } from 'vitest';
import { ink } from '$lib/core/inkEngine.svelte';
import { inserts } from '$lib/core/insertEngine.svelte';
import { measureEngine } from '$lib/core/measureEngine.svelte';
import { shapes } from '$lib/core/shapeAndHatchEngine.svelte';
import { selection } from '$lib/core/selectionEngine.svelte';
import { snapEngine } from '$lib/core/snapEngine.svelte';
import { viewport } from '$lib/core/viewportEngine.svelte';
import { tools } from '$lib/core/toolEngine.svelte';
import { commands } from '$lib/core/commandEngine.svelte';
import { activate, clearDocumentSessions, ENGINE_STATE } from '../studio/documentSessions';

// Review r5: switching documents must reset EVERY per-document engine field. This test lists each
// engine's reactive fields; a new field that is neither reset per document nor declared app-level
// fails here instead of leaking silently between documents.

const ENGINES: Record<string, object> = {
    ink,
    inserts,
    measureEngine,
    shapes,
    selection,
    snapEngine,
    viewport,
    tools,
    commands,
};

/** Svelte 5 compiles `x = $state(…)` class fields to get/set accessors on the prototype. */
function stateFields(obj: object): string[] {
    const proto = Object.getPrototypeOf(obj);
    return Object.getOwnPropertyNames(proto).filter((name) => {
        const d = Object.getOwnPropertyDescriptor(proto, name);
        return !!d?.get && !!d?.set;
    });
}

describe('document switch resets every per-document engine field', () => {
    test('every reactive engine field is classified (per-document or app-level)', () => {
        for (const [name, engine] of Object.entries(ENGINES)) {
            const classified = [
                ...(ENGINE_STATE[name]?.perDocument ?? []),
                ...(ENGINE_STATE[name]?.app ?? []),
                ...(ENGINE_STATE[name]?.derived ?? []), // computed from the fields above
            ];
            for (const field of stateFields(engine)) {
                expect(classified, `${name}.${field} is not classified in ENGINE_STATE`).toContain(field);
            }
        }
    });

    test('activate() puts every per-document field back to its reset value', () => {
        clearDocumentSessions();
        activate('/reset/one.pdf');
        const snapshot = () =>
            Object.fromEntries(
                Object.entries(ENGINE_STATE).flatMap(([name, cls]) =>
                    cls.perDocument.map((f) => [`${name}.${f}`, JSON.stringify((ENGINES[name] as any)[f])])
                )
            );
        const clean = snapshot();
        // Dirty every per-document field.
        for (const [name, cls] of Object.entries(ENGINE_STATE)) {
            for (const f of cls.perDocument) {
                const cur = (ENGINES[name] as any)[f];
                (ENGINES[name] as any)[f] =
                    Array.isArray(cur) ? [{ id: 'junk', points: [] }] : typeof cur === 'boolean' ? !cur : typeof cur === 'number' ? cur + 7 : { junk: true, x: 1, y: 2, pageIndex: 3 };
            }
        }
        activate('/reset/two.pdf');
        expect(snapshot()).toEqual(clean);
        clearDocumentSessions();
    });

    test('activate() resets plain (non-getter/setter) engine state', () => {
        clearDocumentSessions();
        activate('/reset/one.pdf');
        // Dirty plain engine fields on inserts (resetDocumentState resets recordedChunks)
        (inserts as any).recordedChunks = [new Blob()];
        expect((inserts as any).recordedChunks.length).toBe(1);

        activate('/reset/two.pdf');
        expect((inserts as any).recordedChunks).toEqual([]);
        // Note: commands.journal is plain and not cleared by activate (recorded in checkpoint per AT-06).
        clearDocumentSessions();
    });
});
