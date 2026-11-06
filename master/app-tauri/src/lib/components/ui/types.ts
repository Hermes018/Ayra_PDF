// Shared prop types for lib/components/ui (DESIGN_LANGUAGE section 3).
// A disabled control must say why: `disabled: true` without `disabledReason` is a type error.
export type DisabledProps =
  | { disabled?: false; disabledReason?: string }
  | { disabled: true; disabledReason: string };

export interface CommonProps {
  class?: string;
  'data-testid'?: string;
}

/** Dev-only warning for a blank reason / label (types alone cannot catch an empty string). */
export function warnIfBlank(component: string, prop: string, value: string | undefined, active = true): void {
  if (import.meta.env?.DEV && active && (value === undefined || value.trim() === '')) {
    console.warn(`[ui/${component}] "${prop}" is required and must not be empty (accessibility / honest-UI rule).`);
  }
}
