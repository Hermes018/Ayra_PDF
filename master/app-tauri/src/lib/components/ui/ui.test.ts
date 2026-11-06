// Unit tests for lib/components/ui: render, keyboard, disabled-reason exposure (DESIGN_LANGUAGE sections 1, 3, 4).
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, cleanup, fireEvent } from '@testing-library/svelte';
import { createRawSnippet, tick } from 'svelte';
import { commands } from '$lib/core/commandEngine.svelte';
import Button from './Button.svelte';
import IconButton from './IconButton.svelte';
import Input from './Input.svelte';
import Select from './Select.svelte';
import Toggle from './Toggle.svelte';
import Dialog from './Dialog.svelte';
import EmptyState from './EmptyState.svelte';
import ErrorState from './ErrorState.svelte';
import Progress from './Progress.svelte';
import SidebarSection from './SidebarSection.svelte';

const text = (s: string) => createRawSnippet(() => ({ render: () => `<span>${s}</span>` }));
const btn = (label: string) => createRawSnippet(() => ({ render: () => `<button type="button">${label}</button>` }));

let detach: () => void = () => {};
beforeEach(() => {
  detach = commands.attachKeyboard(window);
});
afterEach(() => {
  detach();
  cleanup();
  vi.restoreAllMocks();
});

/** The tooltip text a control exposes through aria-describedby. */
function describedText(el: HTMLElement): string {
  const ids = (el.getAttribute('aria-describedby') ?? '').split(' ').filter(Boolean);
  return ids.map((id) => document.getElementById(id)?.textContent ?? '').join(' ');
}

describe('Button', () => {
  it('renders each variant and size as a native button', () => {
    for (const variant of ['primary', 'secondary', 'destructive', 'plain'] as const) {
      for (const size of ['sm', 'md', 'lg'] as const) {
        const { unmount } = render(Button, { variant, size, onclick: () => {}, children: text('Save copy') });
        const b = screen.getByRole('button', { name: 'Save copy' });
        expect(b.tagName).toBe('BUTTON');
        expect(b).toHaveClass(`lp-btn--${variant}`, `lp-btn--${size}`);
        unmount();
      }
    }
  });
  it('is activated by click (native button: Enter and Space activate it in the browser) and is in the Tab order', async () => {
    const onclick = vi.fn();
    render(Button, { variant: 'primary', onclick, children: text('Save copy') });
    const b = screen.getByRole('button', { name: 'Save copy' });
    expect(b.tabIndex).toBe(0);
    b.focus();
    expect(b).toHaveFocus();
    await fireEvent.click(b);
    expect(onclick).toHaveBeenCalledTimes(1);
  });
  it('disabled: ignores clicks, stays focusable, exposes the reason via aria-describedby', async () => {
    const onclick = vi.fn();
    render(Button, { variant: 'primary', onclick, disabled: true, disabledReason: 'Open a PDF to enable Save', children: text('Save') });
    const b = screen.getByRole('button', { name: 'Save' });
    expect(b).toHaveAttribute('aria-disabled', 'true');
    expect(b.tabIndex).toBe(0);
    expect(describedText(b)).toBe('Open a PDF to enable Save');
    expect(screen.getByRole('tooltip')).toHaveTextContent('Open a PDF to enable Save');
    await fireEvent.click(b);
    expect(onclick).not.toHaveBeenCalled();
  });
  it('busy: aria-busy, shows progress, ignores clicks', async () => {
    const onclick = vi.fn();
    render(Button, { variant: 'primary', onclick, busy: true, children: text('Compress PDF') });
    const b = screen.getByRole('button', { name: 'Compress PDF' });
    expect(b).toHaveAttribute('aria-busy', 'true');
    await fireEvent.click(b);
    expect(onclick).not.toHaveBeenCalled();
  });
  it('warns in dev when disabled with a blank reason', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});
    // @ts-expect-error disabled without disabledReason is a type error
    render(Button, { variant: 'primary', onclick: () => {}, disabled: true, children: text('Save') });
    expect(warn).toHaveBeenCalled();
  });
});

describe('IconButton', () => {
  it('renders with an accessible name and a tooltip carrying it', () => {
    render(IconButton, { icon: 'undo', ariaLabel: 'Undo', onclick: () => {} });
    const b = screen.getByRole('button', { name: 'Undo' });
    expect(describedText(b)).toBe('Undo');
  });
  it('appends the bound chord when shortcutCmd is given', () => {
    vi.spyOn(commands, 'shortcutFor').mockReturnValue('Ctrl+Z');
    render(IconButton, { icon: 'undo', ariaLabel: 'Undo', onclick: () => {}, shortcutCmd: 'undo' });
    expect(describedText(screen.getByRole('button', { name: 'Undo' }))).toBe('Undo (Ctrl+Z)');
  });
  it('requires ariaLabel as a type error and warns in dev when blank', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});
    // @ts-expect-error ariaLabel is required
    render(IconButton, { icon: 'undo', onclick: () => {} });
    expect(warn).toHaveBeenCalledWith(expect.stringContaining('ariaLabel'));
  });
  it('pressed state, click activation and Tab order', async () => {
    const onclick = vi.fn();
    render(IconButton, { icon: 'pin', ariaLabel: 'Pin', onclick, pressed: true });
    const b = screen.getByRole('button', { name: 'Pin' });
    expect(b).toHaveAttribute('aria-pressed', 'true');
    expect(b.tabIndex).toBe(0);
    await fireEvent.click(b);
    expect(onclick).toHaveBeenCalledTimes(1);
  });
  it('disabled: no activation, reason in tooltip text', async () => {
    const onclick = vi.fn();
    render(IconButton, { icon: 'undo', ariaLabel: 'Undo', onclick, disabled: true, disabledReason: 'Nothing to undo' });
    const b = screen.getByRole('button', { name: 'Undo' });
    expect(describedText(b)).toContain('Nothing to undo');
    await fireEvent.click(b);
    expect(onclick).not.toHaveBeenCalled();
  });
});

describe('Input', () => {
  it('renders a labelled input and binds the value', async () => {
    render(Input, { value: 'a', label: 'Name' });
    const i = screen.getByLabelText('Name') as HTMLInputElement;
    expect(i.value).toBe('a');
    await fireEvent.input(i, { target: { value: 'b' } });
    expect(i.value).toBe('b');
  });
  it('error is announced and linked; hint is linked', () => {
    render(Input, { value: '', label: 'Pages', error: 'Enter a page number' });
    const i = screen.getByLabelText('Pages');
    expect(i).toHaveAttribute('aria-invalid', 'true');
    expect(describedText(i)).toBe('Enter a page number');
    cleanup();
    render(Input, { value: '', label: 'Pages', hint: 'For example 1-3' });
    expect(describedText(screen.getByLabelText('Pages'))).toBe('For example 1-3');
  });
  it('is in the Tab order; disabled exposes the reason and cannot take focus', () => {
    render(Input, { value: '', label: 'Pages' });
    expect((screen.getByLabelText('Pages') as HTMLElement).tabIndex).toBe(0);
    cleanup();
    render(Input, { value: '', label: 'Pages', disabled: true, disabledReason: 'Open a PDF first' });
    const i = screen.getByLabelText('Pages');
    expect(i).toBeDisabled();
    expect(describedText(i)).toBe('Open a PDF first');
  });
});

describe('Select', () => {
  const options = [{ value: 'a', label: 'Alpha' }, { value: 'b', label: 'Beta' }];
  it('renders options and binds the value', async () => {
    render(Select, { value: 'a', label: 'Mode', options });
    const s = screen.getByLabelText('Mode') as HTMLSelectElement;
    expect(s.options).toHaveLength(2);
    await fireEvent.change(s, { target: { value: 'b' } });
    expect(s.value).toBe('b');
    expect(s.tabIndex).toBe(0);
  });
  it('disabled exposes the reason', () => {
    render(Select, { value: 'a', label: 'Mode', options, disabled: true, disabledReason: 'One mode is available' });
    const s = screen.getByLabelText('Mode');
    expect(s).toBeDisabled();
    expect(describedText(s)).toBe('One mode is available');
  });
});

describe('Toggle', () => {
  it('is a switch, flips on click, in Tab order', async () => {
    render(Toggle, { checked: false, label: 'Show grid' });
    const t = screen.getByRole('switch', { name: 'Show grid' });
    expect(t.tabIndex).toBe(0);
    expect(t).toHaveAttribute('aria-checked', 'false');
    await fireEvent.click(t);
    expect(t).toHaveAttribute('aria-checked', 'true');
  });
  it('disabled: does not flip, reason is exposed', async () => {
    render(Toggle, { checked: false, label: 'Labs', disabled: true, disabledReason: 'Turn on Labs in Settings' });
    const t = screen.getByRole('switch', { name: 'Labs' });
    expect(describedText(t)).toBe('Turn on Labs in Settings');
    await fireEvent.click(t);
    expect(t).toHaveAttribute('aria-checked', 'false');
  });
});

describe('Dialog', () => {
  const mount = (over: Record<string, unknown> = {}) => {
    const onclose = vi.fn();
    const r = render(Dialog, { open: true, title: 'Delete 3 pages?', onclose, children: text('This removes them from the new file.'), actions: btn('Cancel'), ...over });
    return { onclose, ...r };
  };
  const press = async (key: string) => {
    window.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }));
    await tick();
  };
  it('renders role=dialog, aria-modal, labelled by its title', () => {
    mount();
    const d = screen.getByRole('dialog', { name: 'Delete 3 pages?' });
    expect(d).toHaveAttribute('aria-modal', 'true');
    expect(d).toHaveTextContent('This removes them from the new file.');
  });
  it('renders nothing when closed', () => {
    mount({ open: false });
    expect(screen.queryByRole('dialog')).toBeNull();
  });
  it('Escape (through commandEngine) closes it and calls onclose once', async () => {
    const { onclose } = mount();
    await press('Escape');
    expect(onclose).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole('dialog')).toBeNull();
  });
  it('registers no Escape handler while closed', async () => {
    const { onclose } = mount({ open: false });
    await press('Escape');
    expect(onclose).not.toHaveBeenCalled();
  });
  it('moves focus inside on open, honouring data-autofocus, and returns it on close', async () => {
    const opener = document.createElement('button');
    document.body.appendChild(opener);
    opener.focus();
    mount({ actions: createRawSnippet(() => ({ render: () => '<button data-autofocus>Cancel</button>' })) });
    await tick();
    await tick();
    expect(screen.getByRole('button', { name: 'Cancel' })).toHaveFocus();
    await press('Escape');
    await tick();
    expect(opener).toHaveFocus();
    opener.remove();
  });
  it('traps focus: Tab past the last control wraps to the first, and outside focus is pulled back', async () => {
    mount({ children: createRawSnippet(() => ({ render: () => '<div><button>One</button><button>Two</button></div>' })), actions: undefined });
    await tick();
    const outside = document.createElement('button');
    document.body.appendChild(outside);
    outside.focus();
    await tick();
    expect(screen.getByRole('dialog').contains(document.activeElement)).toBe(true);
    // the end sentinel (what Tab reaches after the last control) wraps to the first control
    const sentinels = document.querySelectorAll<HTMLElement>('.lp-sentinel');
    sentinels[1].focus();
    expect(screen.getByRole('button', { name: 'One' })).toHaveFocus();
    sentinels[0].focus();
    expect(screen.getByRole('button', { name: 'Two' })).toHaveFocus();
    outside.remove();
  });
  it('sheet variant renders as a dialog too', () => {
    mount({ variant: 'sheet' });
    expect(screen.getByRole('dialog')).toHaveClass('lp-dialog--sheet');
  });
});

describe('EmptyState / ErrorState / Progress / SidebarSection', () => {
  it('EmptyState shows title, body and a primary action that activates', async () => {
    const onclick = vi.fn();
    render(EmptyState, { title: 'No PDF open', body: 'Open a PDF to start.', action: { label: 'Open PDF', onclick } });
    expect(screen.getByText('Open a PDF to start.')).toBeInTheDocument();
    await fireEvent.click(screen.getByRole('button', { name: 'Open PDF' }));
    expect(onclick).toHaveBeenCalled();
  });
  it('EmptyState without an action has no button', () => {
    render(EmptyState, { title: 'Nothing here', body: 'Nothing yet.' });
    expect(screen.queryByRole('button')).toBeNull();
  });
  it('ErrorState is an alert with what and next; Retry is optional and works', async () => {
    const retry = vi.fn();
    render(ErrorState, { what: "Couldn't read page 4.", next: 'The file may be damaged; try another copy.', retry });
    expect(screen.getByRole('alert')).toHaveTextContent("Couldn't read page 4.");
    expect(screen.getByRole('alert')).toHaveTextContent('try another copy');
    await fireEvent.click(screen.getByRole('button', { name: 'Retry' }));
    expect(retry).toHaveBeenCalled();
    cleanup();
    render(ErrorState, { what: 'x', next: 'y' });
    expect(screen.queryByRole('button', { name: 'Retry' })).toBeNull();
  });
  it('Progress: determinate value, indeterminate, optional Cancel', async () => {
    const oncancel = vi.fn();
    render(Progress, { value: 42.4, label: 'Compressing', oncancel });
    const bar = screen.getByRole('progressbar', { name: 'Compressing' });
    expect(bar).toHaveAttribute('aria-valuenow', '42');
    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(oncancel).toHaveBeenCalled();
    cleanup();
    render(Progress, { value: null, label: 'Working' });
    expect(screen.getByRole('progressbar')).not.toHaveAttribute('aria-valuenow');
    expect(screen.queryByRole('button', { name: 'Cancel' })).toBeNull();
  });
  it('SidebarSection: titled group; collapsible toggles with aria-expanded', async () => {
    render(SidebarSection, { title: 'Tools', children: btn('Merge'), collapsible: true });
    const head = screen.getByRole('button', { name: 'Tools' });
    expect(head).toHaveAttribute('aria-expanded', 'true');
    expect(head.tabIndex).toBe(0);
    expect(screen.getByRole('button', { name: 'Merge' })).toBeInTheDocument();
    await fireEvent.click(head);
    expect(head).toHaveAttribute('aria-expanded', 'false');
    expect(screen.queryByRole('button', { name: 'Merge' })).toBeNull();
  });
});
