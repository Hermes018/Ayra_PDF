// BUG-045 & BUG-064: the property pill title stays on one line (sentence case, tooltip with the full name),
// the pill sits above the favourites bar and the Help button, and the favourites bar is lifted above
// the 28px bottom status bar.
import { describe, it, expect } from 'vitest';
import pill from './PropertyPill.svelte?raw';
import fav from './FavoritesBar.svelte?raw';

function rule(src: string, selector: string): string {
  const style = src.slice(src.indexOf('<style>'));
  const i = style.indexOf(selector + ' {');
  if (i < 0) throw new Error(`no rule ${selector}`);
  return style.slice(i, style.indexOf('}', i));
}

describe('property pill layout (BUG-045 & BUG-064)', () => {
  it('the tool label is one line, not upper-cased, with an ellipsis and a tooltip', () => {
    const r = rule(pill, '.tool-label');
    expect(r).toMatch(/white-space:\s*nowrap/);
    expect(r).toMatch(/text-overflow:\s*ellipsis/);
    expect(r).not.toMatch(/text-transform:\s*uppercase/);
    expect(pill).toMatch(/data-testid="active-tool-label"\s+title=/);
  });
  it('the pill is lifted above the favourites bar row (layout tokens, no bare px)', () => {
    const bottom = /bottom:\s*(calc\([^;]+\)|[^;]+);/.exec(rule(pill, '.property-pill'))?.[1] ?? '';
    const favBottom = /bottom:\s*([^;]+);/.exec(rule(fav, '.favorites-bar'))?.[1] ?? '';
    expect(bottom).toContain('var(--control-height-lg)');
    expect(favBottom).toContain('status-bar-height');
  });
});
