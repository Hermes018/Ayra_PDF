// BUG-043: the global Space binding (pan spring) must not swallow Space on interactive elements.
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { render, screen, cleanup } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { commands } from './commandEngine.svelte';
import Button from '$lib/components/ui/Button.svelte';

let detach: () => void = () => {};
beforeEach(() => {
  detach = commands.attachKeyboard(window);
});
afterEach(() => {
  detach();
  cleanup();
  document.body.innerHTML = '';
  commands.spaceSpringActive = false;
});

function space(target: EventTarget): KeyboardEvent {
  const e = new KeyboardEvent('keydown', { key: ' ', code: 'Space', bubbles: true, cancelable: true });
  target.dispatchEvent(e);
  return e;
}
function keyup(target: EventTarget): void {
  target.dispatchEvent(new KeyboardEvent('keyup', { key: ' ', code: 'Space', bubbles: true, cancelable: true }));
}

describe('Space on the canvas (characterization, unchanged)', () => {
  it('on window and on a non-interactive canvas element it starts the pan spring and is consumed', () => {
    for (const target of [window, (() => { const c = document.createElement('div'); c.className = 'spatial-viewport'; document.body.appendChild(c); return c; })()]) {
      const e = space(target);
      expect(commands.spaceSpringActive).toBe(true);
      expect(e.defaultPrevented).toBe(true);
      keyup(target);
      expect(commands.spaceSpringActive).toBe(false);
    }
  });
});

describe('Space on interactive elements is left to the browser', () => {
  const make = (html: string): HTMLElement => {
    const host = document.createElement('div');
    host.innerHTML = html;
    document.body.appendChild(host);
    return host.firstElementChild as HTMLElement;
  };
  const cases: Record<string, string> = {
    button: '<button>Go</button>',
    select: '<select><option>a</option></select>',
    textarea: '<textarea></textarea>',
    input: '<input type="checkbox" />',
    'role=button': '<div role="button" tabindex="0">Go</div>',
    'role=switch': '<div role="switch" tabindex="0">Go</div>',
    contenteditable: '<div contenteditable="true">x</div>',
    link: '<a href="#x">x</a>',
  };
  for (const [name, html] of Object.entries(cases)) {
    it(`${name}: no pan spring, not preventDefault-ed`, () => {
      const e = space(make(html));
      expect(commands.spaceSpringActive).toBe(false);
      expect(e.defaultPrevented).toBe(false);
    });
  }
  it('a child of a button (icon span) counts as the button', () => {
    const e = space(make('<button><span>icon</span></button>').firstElementChild as HTMLElement);
    expect(e.defaultPrevented).toBe(false);
  });
  it('a focused ui/Button keeps Space for activation (native click on keyup)', () => {
    render(Button, { variant: 'primary', onclick: () => {}, children: createRawSnippet(() => ({ render: () => '<span>Save copy</span>' })) });
    const b = screen.getByRole('button', { name: 'Save copy' });
    b.focus();
    const e = space(b);
    expect(e.defaultPrevented).toBe(false);
    expect(commands.spaceSpringActive).toBe(false);
  });
});
