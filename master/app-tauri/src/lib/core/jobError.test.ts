import { describe, it, expect } from 'vitest';
import { friendlyJobError } from './jobError';

const FALLBACK = { what: "Couldn't finish the job.", next: 'Try again.' };

describe('friendlyJobError (BUG-052)', () => {
  it('an existing output file: what happened + what to do, with a new-name retry', () => {
    const r = friendlyJobError('OUTPUT_PATH_UNSAFE: a file already exists at the output path (/x.pdf). Nothing was written.', FALLBACK);
    expect(r.what).toBe('A file named x.pdf already exists.');
    expect(r.next).toMatch(/Nothing was written/);
    expect(r.next).toMatch(/next free name/);
    expect(r.newName).toBe(true);
  });
  it('without a path in the message the wording stays generic', () => {
    expect(friendlyJobError('OUTPUT_PATH_UNSAFE: a file already exists at the output path', FALLBACK).what).toBe('A file with that name already exists.');
  });
  it('where there is no retry, the next step does not promise one', () => {
    const r = friendlyJobError('OUTPUT_PATH_UNSAFE: a file already exists at the output path (/x.pdf). Nothing was written.', FALLBACK, false);
    expect(r.next).not.toMatch(/Try again|next free name/i);
    expect(r.next).toMatch(/Nothing was written/);
    expect(r.next).toMatch(/different name or location/);
  });
  it('an output that is the original: plain words, no raw code or path', () => {
    const r = friendlyJobError('OUTPUT_PATH_UNSAFE: the output file would overwrite the original (/docs/a.pdf). Nothing was written.', FALLBACK);
    expect(r.what).toBe('That would overwrite the original file, a.pdf.');
    expect(`${r.what} ${r.next}`).not.toMatch(/OUTPUT_PATH_UNSAFE|\/docs/);
    expect(r.newName).toBe(true);
  });
  it('anything else uses the fallback and never shows the raw text', () => {
    const r = friendlyJobError('engine exploded: code 7', FALLBACK);
    expect(r).toMatchObject({ what: FALLBACK.what, next: FALLBACK.next, newName: false });
  });
  it('accepts non-Error values (backend strings)', () => {
    expect(friendlyJobError(new Error('OUTPUT_PATH_UNSAFE: a file already exists at x').message, FALLBACK).newName).toBe(true);
  });
});
