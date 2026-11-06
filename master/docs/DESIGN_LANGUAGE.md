# LocalPDF Studio: Design Language

Status: draft for the UI consistency sweep (feature freeze in force). Scope: the whole app (Studio plus the 12 screens in `lib/views/`). This spec governs consistency only. It adds no features and changes no behaviour.

## 1. Principles

1. **Clarity first.** Content (the PDF, the file list, the result) is the largest thing on screen. Chrome recedes.
2. **Every control has a purpose.** If removing a control loses nothing, remove it. No decorative badges, gradients, glows or icons without a function.
3. **No decorative chrome.** Glass (`--glass-*`) is for surfaces that float over content: toolbars, sheets, toasts. Static page sections do not use it.
4. **Progressive disclosure.** Show the common action. Put advanced options behind a disclosure, a secondary sheet or the Labs toggle (`components/studio/labs.svelte.ts`).
5. **Disabled controls explain why.** Every `disabled` control carries a `disabledReason`, shown as a tooltip and, where space allows, inline text ("Open a PDF to enable Save").
6. **Honest labels.** A label says what the control does, no more (CLAUDE.md, "Honest UI").

## 2. Tokens

There are two token sources today: `core/themeContract.svelte.ts` (`TokenName`, set on the root by `theme.attach()`) and `src/app.css` (`:root` / `[data-theme]`). Components use `var(--token)` only. No hex or rgb literals in `.svelte` files. **NEW** marks a proposed token; add it to `themeContract` `TokenName` and to both the light and dark tables. Known conflict: `--glass-blur` is `24px` in themeContract but `blur(24px) saturate(180%)` in app.css. Use `backdrop-filter: var(--glass-blur)` only as app.css defines it, and resolve the conflict in themeContract before the sweep starts.

### Colour (semantic roles)

| Role | Token | Source | Light / dark |
|---|---|---|---|
| Accent (primary action, selection) | `--accent` (also `--accent-primary`, `--accent-primary-hover`) | themeContract / app.css | `#007aff` / `#0a84ff` |
| Text on accent | `--text-on-accent` | app.css | white in both themes |
| Destructive | `--danger` (also `--semantic-danger`) | themeContract / app.css | `#ff3b30` / `#ff453a` |
| Success / warning | `--semantic-success`, `--semantic-warning` | app.css | same in both themes |
| Focus ring | `--focus-ring` (outline), `--accent-ring` (halo) | both | per theme |
| Text: primary / secondary / tertiary (also disabled) | `--text-primary`, `--text-secondary`, `--text-tertiary` | app.css | per theme |
| Window background | `--glass-bg-canvas` (`--bg-color`) | app.css | per theme |
| Surface / elevated / input | `--glass-bg-surface`, `--glass-bg-elevated`, `--glass-bg-input` | app.css | per theme |
| Hover / pressed-selected | `--glass-bg-hover`, `--glass-bg-active` | app.css | per theme |
| Hairline border | `--glass-border-subtle`, `--glass-border` | both | per theme |
| Select options | `--option-bg`, `--option-fg` | app.css | per theme |
| Studio canvas only | `--canvas-*`, `--snap-indicator-*`, `--measure-*`, `--laser-*` | themeContract | per theme, Studio only |

### Type, spacing, shape, depth, motion

| Group | Tokens |
|---|---|
| Family | `--font-sans`, `--font-mono` (system fonts only) |
| Type scale | **NEW** `--font-size-caption` 11px, `--font-size-footnote` 12px, `--font-size-body` 13px (root default today), `--font-size-title3` 15px, `--font-size-title2` 17px, `--font-size-title1` 22px. These replace roughly 12 ad-hoc sizes now in use (0.85rem, 0.9rem, 11px and others). Weights: 400 for body, 600 for titles and buttons. |
| Spacing | `--space-1` 4px · `--space-2` 8 · `--space-3` 12 · `--space-4` 16 · `--space-5` 20 · `--space-6` 24 · `--space-8` 32 |
| Control height | **NEW** `--control-height-sm` 24px, `--control-height-md` 28px, `--control-height-lg` 36px |
| Radii | `--radius-xs` 6 · `--radius-sm` 8 · `--radius-md` 10 (controls) · `--radius-lg` 14 (cards) · `--radius-xl` 20 (sheets, toasts) · `--radius-pill` |
| Elevation / glass | `--glass-shadow-sm` (raised controls), `--glass-shadow-lg` / `--glass-shadow` (sheets, popovers); `--glass-blur`, `--glass-blur-toast` |
| Layering | **NEW** `--z-dialog` (above the RadialMenu backdrop at 199), **NEW** `--z-toast` (above dialogs) |
| Motion | `--dur-fast` 120ms (hover, press), `--motion-duration` 180ms (state changes; forced to 0ms under reduced motion by `theme.attach()`), `--dur-med` 220ms (sheets, toasts); easing `--ease-out` by default, `--ease-spring` only for direct-manipulation feedback |

Total NEW: 11 tokens.

## 3. Components (location: `lib/components/ui/`)

Svelte 5 `$props()`, typed. Every component takes `class?` and `data-testid?`. All glyphs come from `components/studio/StudioIcon.svelte` (`name`, `size`).

| Component | Status | Required props (optional props in parentheses) |
|---|---|---|
| Button | CREATE | `variant: 'primary'\|'secondary'\|'destructive'\|'plain'`, `size: 'sm'\|'md'\|'lg'`, `onclick`, `children` (`disabled`, `disabledReason`, `busy`) |
| IconButton | CREATE (glyph from StudioIcon) | `icon: StudioIconName`, `ariaLabel` (required, also used as the tooltip), `onclick` (`pressed`, `disabled`, `disabledReason`, `shortcutCmd`, which shows `commands.shortcutFor()`) |
| Input | CREATE | `value` (bindable), `label` (`placeholder`, `error`, `hint`, `type`) |
| Select | CREATE | `value` (bindable), `label`, `options: {value,label}[]` (`disabledReason`) |
| Toggle | CREATE | `checked` (bindable), `label` (`disabledReason`) |
| Dialog / Sheet | CREATE (absorbs `FaqModal.svelte` and `CalibrationModal.svelte` markup) | `open` (bindable), `title`, `onclose`, `children` (`actions` snippet). Uses `role="dialog"` and `aria-modal`; Escape is routed through commandEngine. |
| Toast | REUSE `lib/components/ToastContainer.svelte` | via `appState.addToast(message, type, detail?, timeoutMs?)` |
| EmptyState | CREATE (pattern from `views/studio/SpatialCanvas.svelte` `studio-empty-state`) | `title`, `body`, `action?: {label, onclick}` |
| ErrorState | CREATE | `what`, `next`, `retry?` |
| Progress | CREATE | `value: number\|null` (null = indeterminate), `label`, `oncancel` |
| Toolbar | CREATE (Studio keeps `components/studio/DockableToolbar.svelte`, rebuilt on IconButton) | `ariaLabel`, `children`; `role="toolbar"` |
| SidebarSection | REUSE: extract from `lib/components/Sidebar.svelte` | `title`, `children` (`collapsible`) |

Totals: 10 CREATE, 2 REUSE.

## 4. Patterns

- **Open / save.** Open with `lib/components/Dropzone.svelte` or the native dialog. Originals are never modified: every operation writes a new file through `lpdf_fs_safety::atomic_write`. The success toast names the output file: `addToast('Saved report-annotated.pdf', 'success', fullPath)`.
- **Destructive confirm.** Use a Dialog with a specific title ("Delete 3 pages?"), a body that states the consequence, a destructive Button labelled with the verb ("Delete pages"), and Cancel as the default focus. Actions that only touch the new output file and can be undone do not need a confirm.
- **Long jobs.** Show Progress inline in the screen, with a determinate value when one is known. Cancel is always present and always works. Afterwards, send one toast with the result or the error.
- **Errors.** Say what happened, then what to do next. Use plain words; put file paths in the toast `detail`. No stack traces, Rust error enums or IPC codes in the UI; those go to the log. Use ErrorState for whole-screen failures and a toast for action failures.
- **Empty states.** One sentence on what the screen does, plus one primary action (usually "Open PDF"). No illustrations.
- **Keyboard.** Shortcuts go only through `core/commandEngine.svelte.ts`: `commands.register(id, handler)`, defaults in `core/defaultShortcuts.json`, rebinding in `components/studio/ShortcutSettings.svelte`. No local `onkeydown` or `addEventListener('keydown')`. Labels show the chord from `commands.shortcutFor(id)`.

## 5. Wording guide

Button labels are verbs. Use sentence case everywhere. No exclamation marks. No overclaiming ("instantly", "perfect", "lossless", "AI-powered", "secure"). The one exception is the About screen's "100% offline" statement, which is true by design (ADR-005).

| Do | Don't |
|---|---|
| Save copy | OK |
| Compress PDF | Optimize instantly |
| Remove password | Unlock |
| Couldn't read page 4. The file may be damaged; try another copy. | Error: render failed (code 3) |
| Saved invoice-compressed.pdf | Success! |
| Open a PDF to enable Merge (tooltip on a disabled button) | (a disabled button with no reason) |
| Summarize (Labs) | Perfect AI summary |

## 6. Platform note (ADR-020)

Components in `ui/` have no platform branches and no layout assumptions: no fixed widths, no `--sidebar-width`, no hover-only affordances (every hover hint has a focus or press equivalent). Layout lives in a separate shell (`MainLayout.svelte`, `Sidebar.svelte`, `StatusBar.svelte`) that owns window chrome, the sidebar and vibrancy (`theme.syncWindowVibrancy`). Android can later supply its own shell and reuse `ui/` unchanged. Any capability difference goes through the single frontend platform-capability module that ADR-020 calls for (not yet present), never through checks inside components.

## 7. Sweep rules: when a screen counts as "done"

A screen (one of the 12 views, or a Studio surface) is done only when all of these hold:

1. `cargo xtask studio-audit --strict` passes. (Its keydown check covers only `views/studio/`, `components/studio/` and `lib/core/` today. Until it is widened, run a grep for `onkeydown` and `addEventListener('keydown'` on the swept file.)
2. It uses only `ui/` components for the controls listed in section 3. There are no raw `<button>`, `<input>` or `<select>` elements outside `ui/`.
3. Styling uses tokens only: no literals, and no `font-size`, spacing or radius values outside the token scale.
4. Light and dark screenshots exist at 1200×800 and 1600×1000 (four per screen), attached to the sweep log entry.
5. There is no behaviour change. The same IPC calls, outputs and E2E results as before; any E2E flow that covers the screen (today only Organize, flows 03 and 10) still passes.
6. Every disabled control has a `disabledReason`, and every label passes section 5.
