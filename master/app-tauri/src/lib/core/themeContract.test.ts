// The contract is the single token source (DESIGN_LANGUAGE section 2). app.css must not redefine its values.
import { describe, it, expect } from 'vitest';
import css from '../../app.css?raw';
import { theme, TOKENS, STATIC_TOKENS, NEW_TOKEN_NAMES } from './themeContract.svelte';

describe('theme contract is the single token source', () => {
  it('--glass-blur is a full backdrop-filter value in both themes', () => {
    for (const t of ['light', 'dark'] as const) {
      expect(TOKENS[t]['--glass-blur']).toBe('blur(24px) saturate(180%)');
    }
  });

  it('light and dark define exactly the same token names', () => {
    expect(Object.keys(TOKENS.light).sort()).toEqual(Object.keys(TOKENS.dark).sort());
  });

  it('defines the 11 NEW tokens from DESIGN_LANGUAGE', () => {
    expect(NEW_TOKEN_NAMES).toHaveLength(11);
    for (const n of NEW_TOKEN_NAMES) expect(STATIC_TOKENS[n]).toBeTruthy();
  });

  it('app.css defines no token that the contract owns', () => {
    const defined = new Set([...css.matchAll(/^\s*(--[a-z0-9-]+)\s*:/gm)].map((m) => m[1]));
    const owned = [...Object.keys(TOKENS.light), ...Object.keys(STATIC_TOKENS)];
    expect(owned.filter((n) => defined.has(n))).toEqual([]);
  });

  it('every var(--x) used by app.css resolves to the contract or an app.css alias', () => {
    const defined = new Set([...css.matchAll(/^\s*(--[a-z0-9-]+)\s*:/gm)].map((m) => m[1]));
    const owned = new Set([...Object.keys(TOKENS.light), ...Object.keys(STATIC_TOKENS)]);
    const used = [...css.matchAll(/var\((--[a-z0-9-]+)/g)].map((m) => m[1]);
    expect(used.filter((n) => !defined.has(n) && !owned.has(n))).toEqual([]);
  });

  it('applyToRoot writes the resolved tokens onto the root', () => {
    const el = document.createElement('div');
    theme.mode = 'light';
    theme.applyToRoot(el);
    expect(el.style.getPropertyValue('--glass-blur')).toBe('blur(24px) saturate(180%)');
    expect(el.style.getPropertyValue('--control-height-md')).toBe('28px');
    expect(el.dataset.theme).toBe('light');
    theme.mode = 'system';
  });
});
