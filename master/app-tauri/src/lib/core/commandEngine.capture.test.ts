// Chord capture (ShortcutSettings "Record"): Escape must cancel the capture, not become the shortcut.
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { commands } from './commandEngine.svelte';

let detach: () => void = () => {};
beforeEach(() => {
  detach = commands.attachKeyboard(window);
});
afterEach(() => {
  detach();
  commands.cancelCaptureChord();
});

function press(key: string, init: KeyboardEventInit = {}): void {
  window.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true, ...init }));
}

describe('chord capture', () => {
  it('records an ordinary chord', async () => {
    const p = commands.captureNextChord();
    press('k', { ctrlKey: true });
    expect(await p).toBe('Ctrl+K');
  });

  it('Escape during capture resolves null (capture cancelled) and is not recorded', async () => {
    const p = commands.captureNextChord();
    press('Escape');
    expect(await p).toBeNull();
  });

  it('Escape during capture is consumed: escape handlers (an open dialog) do not run', async () => {
    const handler = vi.fn(() => true);
    const off = commands.registerEscape(handler);
    const p = commands.captureNextChord();
    press('Escape');
    await p;
    expect(handler).not.toHaveBeenCalled();
    off();
  });

  it('after the capture is cancelled, Escape reaches the escape handlers again', () => {
    const handler = vi.fn(() => true);
    const off = commands.registerEscape(handler);
    void commands.captureNextChord();
    press('Escape'); // cancels the capture
    press('Escape'); // now an ordinary Escape
    expect(handler).toHaveBeenCalledTimes(1);
    off();
  });

  it('cancelCaptureChord resolves a pending capture with null', async () => {
    const p = commands.captureNextChord();
    commands.cancelCaptureChord();
    expect(await p).toBeNull();
  });

  it('with the keyboard attached twice (as in the app), Escape during capture still reaches no escape handler', async () => {
    const detach2 = commands.attachKeyboard(window);
    const handler = vi.fn(() => true);
    const off = commands.registerEscape(handler);
    const p = commands.captureNextChord();
    press('Escape');
    expect(await p).toBeNull();
    expect(handler).not.toHaveBeenCalled();
    off();
    detach2();
  });
});
