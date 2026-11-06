// CalibrationModal: Escape / backdrop must not silently drop values the user has entered.
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { render, screen, cleanup, fireEvent } from '@testing-library/svelte';
import { tick } from 'svelte';
import CalibrationModal from './CalibrationModal.svelte';
import { measureEngine } from '$lib/core/measureEngine.svelte';
import { commands } from '$lib/core/commandEngine.svelte';

let detach: () => void = () => {};
beforeEach(() => {
  detach = commands.attachKeyboard(window);
  measureEngine.openCalibrationModal(0, 400);
});
afterEach(() => {
  detach();
  cleanup();
  measureEngine.closeCalibrationModal();
});

async function escape() {
  window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
  await tick();
  await tick();
}
const backdrop = () => (screen.getByTestId('calibration-dialog').parentElement as HTMLElement);

describe('CalibrationModal discard confirm', () => {
  it('nothing entered: Escape closes at once', async () => {
    render(CalibrationModal);
    await tick();
    expect(screen.getByTestId('calibration-dialog')).toBeInTheDocument();
    await escape();
    expect(measureEngine.calibrationModal).toBeNull();
  });

  it('a value entered: Escape asks "Discard calibration?" (alertdialog) and keeps the modal open', async () => {
    render(CalibrationModal);
    await tick();
    await fireEvent.input(screen.getByTestId('calibration-distance'), { target: { value: '250' } });
    await escape();
    expect(measureEngine.calibrationModal).not.toBeNull();
    const alert = screen.getByRole('alertdialog', { name: 'Discard calibration?' });
    expect(alert).toBeInTheDocument();
  });

  it('Keep editing (and Escape on the confirm) returns to the form with the values intact', async () => {
    render(CalibrationModal);
    await tick();
    await fireEvent.input(screen.getByTestId('calibration-distance'), { target: { value: '250' } });
    await escape();
    await escape(); // Escape on the confirm = keep editing
    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect((screen.getByTestId('calibration-distance') as HTMLInputElement).value).toBe('250');
    expect(measureEngine.calibrationModal).not.toBeNull();
  });

  it('a backdrop click asks too; Discard calibration closes without saving', async () => {
    render(CalibrationModal);
    await tick();
    const before = JSON.stringify(measureEngine.scales);
    await fireEvent.input(screen.getByTestId('calibration-depth'), { target: { value: '3' } });
    await fireEvent.click(backdrop());
    await tick();
    expect(screen.getByRole('alertdialog', { name: 'Discard calibration?' })).toBeInTheDocument();
    await fireEvent.click(screen.getByRole('button', { name: 'Discard calibration' }));
    expect(measureEngine.calibrationModal).toBeNull();
    expect(JSON.stringify(measureEngine.scales)).toBe(before);
  });

  it('the Cancel button still closes at once', async () => {
    render(CalibrationModal);
    await tick();
    await fireEvent.input(screen.getByTestId('calibration-distance'), { target: { value: '250' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(measureEngine.calibrationModal).toBeNull();
  });
});
