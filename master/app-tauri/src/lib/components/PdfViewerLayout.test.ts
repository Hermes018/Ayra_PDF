// The PDF preview must not clip its left edge when the page is wider than the pane: no centring that overflows to
// the left (use auto margins, which fall back to the start edge).
import { describe, it, expect } from 'vitest';
import src from './PdfViewer.svelte?raw';

function rule(selector: string): string {
  const style = src.slice(src.indexOf('<style>'));
  const i = style.indexOf(selector + ' {');
  if (i < 0) throw new Error(`no rule ${selector}`);
  return style.slice(i, style.indexOf('}', i));
}

describe('PdfViewer layout', () => {
  it('the scroll wrapper does not centre its (possibly wider) canvas with justify-content: center', () => {
    expect(rule('.canvas-wrapper')).not.toMatch(/justify-content:\s*center/);
  });
  it('the canvas centres itself with auto margins so an overflowing page starts at the left edge', () => {
    expect(rule('canvas')).toMatch(/margin:\s*0 auto/);
  });
});
