// Plain-words job errors. The backend refuses an output that is the original or an existing file with an
// OUTPUT_PATH_UNSAFE message (the TOCTOU backstop behind the free-name check); the UI never shows that raw text,
// but it keeps the file name so the user can tell which file it was.
export interface FriendlyJobError {
  what: string;
  next: string;
  /** True when a retry should use a new output name (the refusal was about the name). */
  newName: boolean;
}

/** The file name inside the "(path)" of a refusal message, if there is one. */
function fileNameIn(message: string): string | null {
  const m = /\(([^)]+)\)/.exec(message);
  if (!m) return null;
  const name = m[1].split(/[\\/]/).pop();
  return name || null;
}

/**
 * `canRetry`: true where the screen offers a Retry that picks the next free name; false for places with no retry
 * (a toast), where the text must not promise one.
 */
export function friendlyJobError(
  message: string,
  fallback: { what: string; next: string },
  canRetry = true
): FriendlyJobError {
  if (message.includes('OUTPUT_PATH_UNSAFE')) {
    const name = fileNameIn(message);
    const overwrite = message.includes('overwrite the original');
    const what = overwrite
      ? name
        ? `That would overwrite the original file, ${name}.`
        : 'That would overwrite the original file.'
      : name
        ? `A file named ${name} already exists.`
        : 'A file with that name already exists.';
    const next = canRetry
      ? overwrite
        ? 'Nothing was written. Try again to use a new name, or choose a different output name.'
        : 'Nothing was written. Try again and the next free name will be used.'
      : 'Nothing was written. Choose a different name or location and run it again.';
    return { what, next, newName: canRetry };
  }
  return { ...fallback, newName: false };
}
