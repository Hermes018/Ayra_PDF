import { describe, it, expect, afterEach } from 'vitest';
import { render, cleanup, fireEvent, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { placeTooltip } from './tooltipPlacement';
import Button from './Button.svelte';

afterEach(cleanup);
const rect = (x: number, y: number, w: number, h: number) => ({ left: x, top: y, right: x + w, bottom: y + h, width: w, height: h });
const VP = { width: 1000, height: 800 };

describe('placeTooltip', () => {
  it('centres under the anchor when there is room', () => {
    const p = placeTooltip(rect(400, 100, 100, 28), { width: 200, height: 30 }, VP, 4);
    expect(p.left).toBe(350);
    expect(p.top).toBe(100 + 28 + 4);
    expect(p.side).toBe('below');
  });
  it('clamps to the left edge for an anchor at x = 0', () => {
    const p = placeTooltip(rect(0, 100, 40, 28), { width: 240, height: 30 }, VP, 4);
    expect(p.left).toBeGreaterThanOrEqual(4);
  });
  it('clamps to the right edge', () => {
    const p = placeTooltip(rect(960, 100, 40, 28), { width: 240, height: 30 }, VP, 4);
    expect(p.left + 240).toBeLessThanOrEqual(1000 - 4);
  });
  it('flips above when there is no room below', () => {
    const p = placeTooltip(rect(400, 780, 100, 28), { width: 200, height: 30 }, VP, 4);
    expect(p.side).toBe('above');
    expect(p.top + 30).toBeLessThanOrEqual(780);
  });
});

describe('Tooltip in a component', () => {
  it('positions itself inside the viewport when the anchor sits at x = 0', async () => {
    const orig = Element.prototype.getBoundingClientRect;
    Element.prototype.getBoundingClientRect = function (this: Element) {
      return (this.getAttribute('role') === 'tooltip' ? rect(0, 0, 240, 30) : rect(0, 100, 60, 28)) as DOMRect;
    };
    try {
      render(Button, { variant: 'primary', onclick: () => {}, disabled: true, disabledReason: 'Open a PDF to enable Save', children: createRawSnippet(() => ({ render: () => '<span>Save</span>' })) });
      await fireEvent.pointerEnter(screen.getByRole('button').parentElement as HTMLElement);
      const tip = screen.getByRole('tooltip');
      expect(parseFloat(tip.style.left)).toBeGreaterThanOrEqual(0);
    } finally {
      Element.prototype.getBoundingClientRect = orig;
    }
  });
});
