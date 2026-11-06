import { expect, test, vi, beforeEach, afterEach, beforeAll, afterAll, describe } from 'vitest';
import { render, screen, cleanup, fireEvent, within } from '@testing-library/svelte';
import { flushSync } from 'svelte';
import { appState, LABS_STORAGE_KEY } from '$lib/state.svelte';
import { IpcClient } from '$lib/ipc';
import type { AppSettings } from '$lib/bindings/AppSettings';
import { tools } from '$lib/core/toolEngine.svelte';
import { commands } from '$lib/core/commandEngine.svelte';
import type { ActiveStroke } from '$lib/core/inkEngine.svelte';
import { inserts, recognizeHandwritingStrokesToText } from '$lib/core/insertEngine.svelte';
import { viewport } from '$lib/core/viewportEngine.svelte';
import {
    INK_GUESS_CHARS,
    INK_GUESS_LABS,
    MATH_TEXT_LABS,
    attachLabsGate,
    resetLabsGateForTests,
    selectToolIfAllowed,
    setLabs,
    shortcutsForSave,
} from '$lib/components/studio/labs.svelte';
import DockableToolbar from '$lib/components/studio/DockableToolbar.svelte';
import AnnotationLedger from '$lib/components/studio/AnnotationLedger.svelte';
import PropertyPill from '$lib/components/studio/PropertyPill.svelte';
import FavoritesBar from '$lib/components/studio/FavoritesBar.svelte';
import ShortcutSettings from '$lib/components/studio/ShortcutSettings.svelte';
import SettingsView from '../SettingsView.svelte';
import SpatialCanvas from '../studio/SpatialCanvas.svelte';

// PROTOTYPE_SCOPE.md: LaTeX insert (2.16) and ink-to-text (8.10) sit behind Labs, OFF by default.
// Overclaiming copy that must never come back, with Labs on or off.
const OVERCLAIMS = [
    /LaTeX Mathematical Vector Stamp/i,
    /Offline Engine/i,
    /Stamp Vector Equation/i,
    /LaTeX Math Equation/i,
    /editable text/i,
    /handwritten/i,
    /Ink → Text/,
];

const LABS_CHORD = 'Alt+Shift+M';

function expectNoOverclaims() {
    for (const re of OVERCLAIMS) {
        expect(document.body.textContent ?? '').not.toMatch(re);
        expect(document.body.innerHTML).not.toMatch(re);
    }
}

function openInsertDrawer() {
    fireEvent.click(screen.getByTestId('category-btn-insert'));
    flushSync();
    return screen.getByRole('menu', { name: /Insert Suite & Media tools/i });
}

function openLedger() {
    inserts.ledgerOpen = true;
    render(AnnotationLedger);
    flushSync();
}

function stroke(points: Array<[number, number]>, text?: string): ActiveStroke {
    return { id: `s-${Math.random()}`, points: points.map(([x, y]) => ({ x, y, p: 0.5 })), text } as unknown as ActiveStroke;
}

/** Characters the classifier emitted for strokes without text (the "Ink Note: " prefix removed). */
function guessChars(strokes: ActiveStroke[]): string[] {
    const out = recognizeHandwritingStrokesToText(strokes);
    expect(out.startsWith('Ink Note')).toBe(true);
    return [...out.replace(/^Ink Note(: )?/, '')];
}

function settingsFixture(): AppSettings {
    return {
        tier_override: null,
        output_policy: 'SuffixNewFile',
        backups_enabled: true,
        language: 'en',
        theme: 'System',
        local_ai_model_path: null,
        ai_local_enabled: false,
        ai_license_acknowledged: false,
        ai_model_sha256: null,
        ai_model_license: null,
    };
}

describe('Labs gating (2.16 math text, 8.10 shape guess)', () => {
    let detachGate: () => void = () => {};
    const defaultFavorites = [...tools.favorites];
    const defaultShortcuts = { ...commands.shortcuts };

    beforeAll(() => {
        detachGate = attachLabsGate(); // as routes/+page.svelte does
    });
    afterAll(() => {
        detachGate();
    });

    beforeEach(() => {
        localStorage.clear();
        appState.initPreferences();
        flushSync();
        tools.selectTool('pan');
        inserts.ledgerOpen = false;
        inserts.latexModalOpen = false;
    });

    afterEach(() => {
        cleanup();
        vi.restoreAllMocks();
        setLabs(false);
        localStorage.clear();
        inserts.ledgerOpen = false;
        inserts.latexModalOpen = false;
        tools.favorites = [...defaultFavorites];
        commands.shortcuts = { ...defaultShortcuts };
        resetLabsGateForTests();
        tools.selectTool('pan');
        flushSync();
    });

    describe('(a) default and persistence', () => {
        test('Labs is off when nothing was ever saved', () => {
            expect(localStorage.getItem(LABS_STORAGE_KEY)).toBeNull();
            expect(appState.labsEnabled).toBe(false);
        });

        test('a corrupt or non-boolean stored value reads as off', () => {
            for (const bad of ['garbage{', '"true"', '1', 'null']) {
                localStorage.setItem(LABS_STORAGE_KEY, bad);
                appState.initPreferences();
                expect(appState.labsEnabled).toBe(false);
            }
        });

        test('setLabs persists and is read back on next start', () => {
            setLabs(true);
            expect(localStorage.getItem(LABS_STORAGE_KEY)).toBe('true');
            appState.labsEnabled = false;
            appState.initPreferences();
            expect(appState.labsEnabled).toBe(true);
        });

        test('Settings shows the Labs toggle unchecked and turning it on enables Labs', async () => {
            vi.spyOn(IpcClient, 'getSettings').mockResolvedValue(settingsFixture());
            vi.spyOn(IpcClient, 'getRecentLogs').mockResolvedValue([]);
            render(SettingsView);
            await screen.findByTestId('settings-loaded');

            const toggle = screen.getByTestId('labs-toggle');
            expect(toggle).toHaveAttribute('aria-checked', 'false');
            const list = screen.getByTestId('labs-feature-list');
            expect(list).toHaveTextContent(MATH_TEXT_LABS.label);
            expect(list).toHaveTextContent('not a TeX renderer');
            expect(list).toHaveTextContent(INK_GUESS_LABS.label);
            expect(list).toHaveTextContent(INK_GUESS_LABS.note);

            await fireEvent.click(toggle);
            expect(appState.labsEnabled).toBe(true);
            expect(localStorage.getItem(LABS_STORAGE_KEY)).toBe('true');
        });
    });

    describe('honest 8.10 copy matches the classifier', () => {
        // One synthetic stroke per classifier branch (core/insertEngine.svelte.ts).
        const circle = Array.from({ length: 9 }, (_, i): [number, number] => [
            50 + 40 * Math.cos((i / 8) * 2 * Math.PI),
            50 + 40 * Math.sin((i / 8) * 2 * Math.PI),
        ]);
        const branches: Array<[string, ActiveStroke]> = [
            ['.', stroke([[10, 10]])],
            ['O', stroke(circle)],
            ['I', stroke([[0, 0], [2, 100]])],
            ['-', stroke([[0, 0], [100, 2]])],
            ['L', stroke([[0, 0], [0, 50], [50, 50]])],
            ['A', stroke([[0, 50], [12, 0], [25, 50], [37, 0], [50, 50]])],
            ['e', stroke([[0, 0], [25, 50], [50, 0], [75, 50], [100, 0]])],
        ];

        test('every listed character is produced by some stroke shape', () => {
            const produced = branches.map(([, s]) => guessChars([s]).join(''));
            expect(produced).toEqual(branches.map(([c]) => c));
            expect(new Set(produced)).toEqual(new Set(INK_GUESS_CHARS));
        });

        test('random strokes never produce a character outside the list', () => {
            let seed = 0x2f6e2b1;
            const rand = () => ((seed = (seed * 1103515245 + 12345) & 0x7fffffff) / 0x7fffffff);
            const seen = new Set<string>();
            for (let i = 0; i < 4000; i++) {
                const n = 1 + Math.floor(rand() * 12);
                const pts = Array.from({ length: n }, (): [number, number] => [rand() * 300, rand() * 300]);
                for (const c of guessChars([stroke(pts)])) seen.add(c);
            }
            for (const c of seen) expect(INK_GUESS_CHARS).toContain(c);
        });

        test('the note lists exactly the classifier outputs and its other behaviour', () => {
            const quoted = new Set([...INK_GUESS_LABS.note.matchAll(/"(.)"/g)].map((m) => m[1]));
            const fromClassifier = new Set(branches.flatMap(([, s]) => guessChars([s])));
            expect(quoted).toEqual(fromClassifier);
            expect(INK_GUESS_LABS.note).toContain('"Ink Note: "');
            expect(INK_GUESS_LABS.note).toContain('or the last stroke, if none is selected');
            expect(INK_GUESS_LABS.note).toContain('already has text keeps that text');
            expect(INK_GUESS_LABS.note).toContain('It does not recognise handwriting.');
            // Behaviours the note claims, checked against the classifier.
            expect(recognizeHandwritingStrokesToText([stroke([[0, 0], [5, 5]], 'kept')])).toBe('Ink Note: kept');
            expect(recognizeHandwritingStrokesToText([branches[2][1], branches[3][1]])).toBe('Ink Note: I-');
        });
    });

    describe('(b) Labs off: entry points are absent or inert', () => {
        test('toolbar Insert drawer has no math-text / LaTeX tool', () => {
            render(DockableToolbar);
            const drawer = openInsertDrawer();
            expect(within(drawer).getByText('Sticky Note')).toBeInTheDocument();
            expect(within(drawer).queryByText(MATH_TEXT_LABS.name)).not.toBeInTheDocument();
            expect(within(drawer).queryByText(/latex/i)).not.toBeInTheDocument();
            expect(within(drawer).queryByTestId('labs-badge')).not.toBeInTheDocument();
            expectNoOverclaims();
        });

        test('property pill insert picker has no math-text option', () => {
            tools.selectTool('callout.single');
            render(PropertyPill);
            const picker = screen.getByRole('combobox', { name: 'Insert or callout tool' });
            expect(within(picker).getByText('Sticky note')).toBeInTheDocument();
            expect(within(picker).queryByText(MATH_TEXT_LABS.label)).not.toBeInTheDocument();
            expect(within(picker).queryByText(/latex/i)).not.toBeInTheDocument();
        });

        test('ledger has no shape-guess button and never opens the math-text dialog', () => {
            inserts.latexModalOpen = true;
            openLedger();
            expect(screen.getByRole('button', { name: /Detect tables/ })).toBeInTheDocument();
            expect(screen.queryByTestId('ledger-ink-guess-btn')).not.toBeInTheDocument();
            expect(screen.queryByText(INK_GUESS_LABS.name)).not.toBeInTheDocument();
            expect(screen.queryByTestId('math-text-dialog')).not.toBeInTheDocument();
            expect(screen.queryByRole('dialog', { name: MATH_TEXT_LABS.label })).not.toBeInTheDocument();
            expectNoOverclaims();
        });

        test('ledger vault hides the built-in equation item', async () => {
            openLedger();
            await fireEvent.click(screen.getByRole('button', { name: /^Vault/ }));
            expect(screen.getByText('APPROVED STAMP')).toBeInTheDocument();
            expect(screen.queryByText('Euler Buckling Load')).not.toBeInTheDocument();
            expect(screen.queryByText(/insert\.latex/)).not.toBeInTheDocument();
        });

        test('call-site selection refuses a Labs tool', () => {
            expect(selectToolIfAllowed('insert.latex')).toBe(false);
            expect(selectToolIfAllowed('preset.insert.latex')).toBe(false);
            expect(tools.currentToolId).toBe('pan');
            expect(selectToolIfAllowed('insert.note')).toBe(true);
            expect(tools.currentToolId).toBe('insert.note');
        });

        test('a restored favourite for a Labs tool is not shown and cannot be selected', () => {
            // What SpatialCanvas.loadStudioPrefs does with saved favourites.
            tools.favorites = ['preset.pen.a', 'preset.insert.latex'];
            render(FavoritesBar);
            flushSync();
            const bar = screen.getByRole('toolbar', { name: 'Pinned tool presets' });
            const pins = within(bar).getAllByRole('button', { pressed: false });
            expect(pins).toHaveLength(1);
            expect(bar.innerHTML).not.toContain(MATH_TEXT_LABS.label);
            expect(tools.favorites).toContain('preset.insert.latex'); // kept for when Labs is on
        });

        test('a saved shortcut for a Labs tool is parked and its chord does nothing', () => {
            commands.bind(LABS_CHORD, 'tool.insert.latex'); // e.g. restored from prefs
            flushSync();
            expect(commands.shortcuts[LABS_CHORD]).toBeUndefined();
            expect(commands.dispatch({ kind: 'ui.key', chord: LABS_CHORD, phase: 'down' })).toBe(false);
            expect(tools.currentToolId).toBe('pan');
            expect(shortcutsForSave()[LABS_CHORD]).toBe('tool.insert.latex'); // not lost on save

            render(ShortcutSettings, { props: { visible: true } });
            expect(document.body.innerHTML).not.toMatch(/insert › latex/);
        });

        test('a Labs tool that is somehow active falls back to Pan without any Studio mounted', () => {
            tools.selectTool('insert.latex'); // engine-level path that skips the call-site guard
            inserts.latexModalOpen = true;
            flushSync();
            expect(tools.currentToolId).toBe('pan');
            expect(inserts.latexModalOpen).toBe(false);
        });

        test('turning Labs off while its tool is active drops back to Pan in the Studio', () => {
            viewport.pages = [];
            setLabs(true);
            render(SpatialCanvas);
            tools.selectTool('insert.latex');
            flushSync();
            expect(tools.currentToolId).toBe('insert.latex');
            setLabs(false);
            expect(tools.currentToolId).toBe('pan');
        });
    });

    describe('(c) Labs on: entry points appear with honest labels', () => {
        beforeEach(() => {
            setLabs(true);
            flushSync();
        });

        test('toolbar shows "Math text" with a Labs badge and an honest tooltip', () => {
            render(DockableToolbar);
            const drawer = openInsertDrawer();
            const label = within(drawer).getByText(MATH_TEXT_LABS.name);
            const row = label.closest('[role="menuitem"]') as HTMLElement;
            expect(within(row).getByTestId('labs-badge')).toHaveTextContent('Labs');
            expect(row.getAttribute('title')).toContain('not a TeX renderer');
            expectNoOverclaims();
        });

        test('property pill lists and labels the tool as "Math text (Labs)"', () => {
            tools.selectTool('callout.single');
            render(PropertyPill);
            const picker = screen.getByRole('combobox', { name: 'Insert or callout tool' });
            expect(within(picker).getByText(MATH_TEXT_LABS.label)).toBeInTheDocument();

            tools.selectTool('insert.latex');
            flushSync();
            expect(screen.getByTestId('active-tool-label')).toHaveTextContent(MATH_TEXT_LABS.label);
            expect(screen.getByTestId('active-tool-label')).not.toHaveTextContent(/latex/i);
        });

        test('ledger shows the shape-guess button with a Labs badge and the honest note', () => {
            openLedger();
            const btn = screen.getByTestId('ledger-ink-guess-btn');
            expect(btn).toHaveAccessibleName(INK_GUESS_LABS.label);
            expect(within(btn).getByTestId('labs-badge')).toHaveTextContent('Labs');
            expect(btn.getAttribute('title')).toBe(`${INK_GUESS_LABS.label}: ${INK_GUESS_LABS.note}`);
        });

        test('math-text dialog uses honest wording', () => {
            inserts.latexModalOpen = true;
            openLedger();
            const dialog = screen.getByRole('dialog', { name: MATH_TEXT_LABS.label });
            expect(within(dialog).getByTestId('labs-badge')).toHaveTextContent('Labs');
            expect(within(dialog).getByTestId('math-text-note')).toHaveTextContent(
                'Converts \\frac, \\sqrt, Greek letters and ^/_ into Unicode text'
            );
            expect(within(dialog).getByTestId('math-text-note')).toHaveTextContent('It is not a TeX renderer.');
            expect(within(dialog).getByRole('button', { name: MATH_TEXT_LABS.insertAction })).toBeInTheDocument();
            expect(within(dialog).getByRole('textbox', { name: MATH_TEXT_LABS.inputLabel })).toBeInTheDocument();
        });

        test('ledger vault shows the built-in equation item', async () => {
            openLedger();
            await fireEvent.click(screen.getByRole('button', { name: /^Vault/ }));
            expect(screen.getByText('Euler Buckling Load')).toBeInTheDocument();
        });

        test('a restored Labs favourite is shown with the honest label', () => {
            tools.favorites = ['preset.pen.a', 'preset.insert.latex'];
            render(FavoritesBar);
            flushSync();
            const bar = screen.getByRole('toolbar', { name: 'Pinned tool presets' });
            expect(within(bar).getAllByRole('button', { pressed: false })).toHaveLength(2);
            expect(bar.innerHTML).toContain(MATH_TEXT_LABS.label);
        });

        test('a parked Labs shortcut comes back and fires when Labs is turned on', () => {
            setLabs(false);
            commands.bind(LABS_CHORD, 'tool.insert.latex');
            flushSync();
            expect(commands.shortcuts[LABS_CHORD]).toBeUndefined();
            setLabs(true);
            expect(commands.shortcuts[LABS_CHORD]).toBe('tool.insert.latex');
            expect(commands.dispatch({ kind: 'ui.key', chord: LABS_CHORD, phase: 'down' })).toBe(true);
            expect(tools.currentToolId).toBe('insert.latex');
        });
    });
});
