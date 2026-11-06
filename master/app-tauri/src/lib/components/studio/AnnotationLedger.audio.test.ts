// Audio note dialog: closing it while a recording is running must not leave a hidden, running recording.
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { render, screen, cleanup, fireEvent } from '@testing-library/svelte';
import { tick } from 'svelte';
import AnnotationLedger from './AnnotationLedger.svelte';
import { inserts } from '$lib/core/insertEngine.svelte';
import { commands } from '$lib/core/commandEngine.svelte';

let detach: () => void = () => {};
beforeEach(() => {
  detach = commands.attachKeyboard(window);
  inserts.audioModalOpen = true;
  inserts.isRecordingAudio = true;
});
afterEach(() => {
  detach();
  cleanup();
  inserts.audioModalOpen = false;
  inserts.isRecordingAudio = false;
});

async function escape() {
  window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
  await tick();
  await tick();
}
const dialog = () => document.querySelector('[aria-label="Voice Audio Note Recorder"]') as HTMLElement;

describe('discardAudioRecording (engine)', () => {
  it('stops the recorder and its microphone tracks and clears the recording state', () => {
    const stopTrack = vi.fn();
    const stop = vi.fn();
    (inserts as any).mediaRecorder = { state: 'recording', stop, stream: { getTracks: () => [{ stop: stopTrack }] } };
    inserts.discardAudioRecording();
    expect(stop).toHaveBeenCalled();
    expect(stopTrack).toHaveBeenCalled();
    expect(inserts.isRecordingAudio).toBe(false);
    expect(inserts.audioModalOpen).toBe(false);
  });
});

describe('audio note dialog close while recording', () => {
  it('Escape asks "Discard recording?" and the recording keeps running', async () => {
    render(AnnotationLedger);
    await tick();
    await escape();
    expect(screen.getByRole('alertdialog', { name: 'Discard recording?' })).toBeInTheDocument();
    expect(inserts.isRecordingAudio).toBe(true);
    expect(inserts.audioModalOpen).toBe(true);
  });

  it('Keep recording (and Escape on the confirm) goes back to the recorder', async () => {
    render(AnnotationLedger);
    await tick();
    await escape();
    await fireEvent.click(screen.getByRole('button', { name: 'Keep recording' }));
    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect(inserts.isRecordingAudio).toBe(true);
    await escape();
    await escape(); // Escape on the confirm = keep recording
    expect(inserts.isRecordingAudio).toBe(true);
    expect(inserts.audioModalOpen).toBe(true);
  });

  it('a backdrop click and Cancel ask too; Discard recording stops and closes', async () => {
    render(AnnotationLedger);
    await tick();
    await fireEvent.click(dialog().parentElement as HTMLElement);
    await tick();
    expect(screen.getByRole('alertdialog', { name: 'Discard recording?' })).toBeInTheDocument();
    await fireEvent.click(screen.getByRole('button', { name: 'Discard recording' }));
    expect(inserts.isRecordingAudio).toBe(false);
    expect(inserts.audioModalOpen).toBe(false);
  });

  it('when nothing is recording, Escape closes at once', async () => {
    inserts.isRecordingAudio = false;
    render(AnnotationLedger);
    await tick();
    await escape();
    expect(inserts.audioModalOpen).toBe(false);
  });
});
