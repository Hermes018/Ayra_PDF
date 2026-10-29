# APPLE HIG — Frosted Liquid Glass Design System (LocalPDF Studio `v0.1.0-rc5`)

Status: SPEC (design-only, no code changed yet) • Target: `app-tauri/src` (Svelte 5 runes, SvelteKit SPA per ADR-008) • Implementer: Gemini 3.8 Flash workhorse

> [!IMPORTANT]
> **Protocol constraints still apply (02_RULES / 03_FAILSAFES / 04 drift detectors):**
> - No remote assets: **no web fonts from CDNs**, no `url(http…)`. The font stack is system-only.
> - No `{@html}`, no `fetch(`, no `XMLHttpRequest`, no `WebSocket`, no `eval(` in `ui/src` / `app-tauri/src`.
> - No `localStorage` holding document data (the theme preference is allowed — it is a setting, not doc data; prefer the existing settings command if one exists).
> - **Verify every Tauri command / store name from source before wiring.** Names in this spec marked `⟨verify⟩` are placeholders that MUST be checked against `state.svelte.ts` and `src-tauri` before use. Do not invent commands.
> - Gate green → log → commit → tag `v0.1.0-rc5`.

---

## 0. Rollout Order (do it in this order; one commit per step, gate after each)

| Step | Files | Pillar |
|---|---|---|
| 1 | `app-tauri/src/app.css` (new), `+layout.svelte` (import) | 1, 2 |
| 2 | `lib/theme.svelte.ts` (new), `SettingsView.svelte` (segmented control) | 1, 5 |
| 3 | `MainLayout.svelte`, `Sidebar.svelte`, `StatusBar.svelte` | 3 |
| 4 | `lib/toast.svelte.ts` (new), `ToastContainer.svelte` (new/replace), mount in `MainLayout` | 4 |
| 5 | Global `job-event` → toast bridge in `state.svelte.ts` / `+page.svelte` | 4 |
| 6 | Every view: replace hardcoded colours + apply layout primitives + CTA footer | 1, 3, 5 |
| 7 | Visual QA at 800 / 960 / 1080 / 1440 px widths, both themes | all |

---

## 1. Pillar 1 — Frosted Liquid Glass Material System

### 1.1 Token table

| Token | Dark | Light | Purpose |
|---|---|---|---|
| `--glass-bg-canvas` | `rgba(18, 18, 22, 1)` + gradient | `rgba(242, 242, 247, 1)` + gradient | Window backdrop (opaque base under glass) |
| `--glass-bg-sidebar` | `rgba(30, 30, 36, 0.62)` | `rgba(246, 246, 250, 0.68)` | Sidebar vibrancy |
| `--glass-bg-surface` | `rgba(44, 44, 52, 0.55)` | `rgba(255, 255, 255, 0.62)` | Cards / panels |
| `--glass-bg-elevated` | `rgba(58, 58, 68, 0.72)` | `rgba(255, 255, 255, 0.82)` | Popovers, toasts, modals |
| `--glass-bg-input` | `rgba(118, 118, 128, 0.24)` | `rgba(118, 118, 128, 0.12)` | Fields, segmented track |
| `--glass-bg-hover` | `rgba(255, 255, 255, 0.06)` | `rgba(0, 0, 0, 0.04)` | Row/item hover |
| `--glass-bg-active` | `rgba(10, 132, 255, 0.22)` | `rgba(0, 122, 255, 0.14)` | Sidebar active pill |
| `--glass-border-subtle` | `rgba(255, 255, 255, 0.08)` | `rgba(0, 0, 0, 0.08)` | Hairline borders |
| `--glass-border-specular` | `rgba(255, 255, 255, 0.14)` | `rgba(255, 255, 255, 0.9)` | 1px inner top highlight |
| `--glass-shadow-sm` | `0 1px 2px rgba(0,0,0,.4), 0 2px 8px rgba(0,0,0,.25)` | `0 1px 2px rgba(0,0,0,.06), 0 2px 8px rgba(0,0,0,.06)` | Cards |
| `--glass-shadow-lg` | `0 12px 40px rgba(0,0,0,.55), 0 2px 6px rgba(0,0,0,.35)` | `0 12px 40px rgba(0,0,0,.14), 0 2px 6px rgba(0,0,0,.08)` | Toasts, popovers |
| `--text-primary` | `rgba(255, 255, 255, 0.92)` | `rgba(0, 0, 0, 0.88)` | Body, titles |
| `--text-secondary` | `rgba(235, 235, 245, 0.62)` | `rgba(60, 60, 67, 0.66)` | Captions, labels |
| `--text-tertiary` | `rgba(235, 235, 245, 0.36)` | `rgba(60, 60, 67, 0.36)` | Placeholders, disabled |
| `--accent-primary` | `#0A84FF` | `#007AFF` | CTA, focus, links |
| `--accent-primary-hover` | `#409CFF` | `#0066D6` | CTA hover |
| `--text-on-accent` | `#FFFFFF` | `#FFFFFF` | Text on CTA |
| `--semantic-success` | `#34C759` (dark: `#30D158` allowed) | `#34C759` | Completed |
| `--semantic-warning` | `#FF9F0A` | `#FF9F0A` | Warning |
| `--semantic-danger` | `#FF3B30` (dark: `#FF453A` allowed) | `#FF3B30` | Failed / destructive |
| `--option-bg` | `#2C2C34` (OPAQUE) | `#FFFFFF` (OPAQUE) | Native `<option>` popup |
| `--option-fg` | `#F2F2F7` | `#1C1C1E` | Native `<option>` text |

### 1.2 `app-tauri/src/app.css` (copy-paste; create file)

```css
/* ==========================================================================
   LocalPDF Studio — Apple HIG Frosted Liquid Glass tokens (rc5)
   System fonts only. No remote assets.
   ========================================================================== */

:root {
  --font-sans: -apple-system, BlinkMacSystemFont, "SF Pro Text", "SF Pro Display",
    "Inter", "Inter Variable", "Cantarell", "Noto Sans", "Segoe UI", system-ui, sans-serif;
  --font-mono: "SF Mono", ui-monospace, "JetBrains Mono", "Fira Code", "DejaVu Sans Mono", monospace;

  --radius-xs: 6px;
  --radius-sm: 8px;
  --radius-md: 10px;   /* buttons, sidebar pills, inputs */
  --radius-lg: 14px;   /* cards */
  --radius-xl: 20px;   /* toasts, modals */
  --radius-pill: 999px;

  --space-1: 0.25rem; --space-2: 0.5rem; --space-3: 0.75rem;
  --space-4: 1rem;    --space-5: 1.25rem; --space-6: 1.5rem; --space-8: 2rem;

  --sidebar-width: 210px;
  --statusbar-height: 28px;

  --glass-blur: blur(24px) saturate(180%);
  --glass-blur-toast: blur(20px) saturate(180%);

  --ease-spring: cubic-bezier(0.34, 1.56, 0.64, 1);
  --ease-out: cubic-bezier(0.22, 1, 0.36, 1);
  --dur-fast: 120ms;
  --dur-med: 220ms;

  font-family: var(--font-sans);
  font-size: 13px;               /* macOS body size */
  line-height: 1.45;
  -webkit-font-smoothing: antialiased;
  text-rendering: optimizeLegibility;
}

/* ---------- DARK ---------- */
:root[data-theme="dark"] {
  color-scheme: dark;
  --glass-bg-canvas: rgba(18, 18, 22, 1);
  --glass-canvas-gradient:
    radial-gradient(1200px 800px at 0% 0%, rgba(10, 132, 255, 0.10), transparent 60%),
    radial-gradient(900px 700px at 100% 100%, rgba(191, 90, 242, 0.08), transparent 60%);
  --glass-bg-sidebar: rgba(30, 30, 36, 0.62);
  --glass-bg-surface: rgba(44, 44, 52, 0.55);
  --glass-bg-elevated: rgba(58, 58, 68, 0.72);
  --glass-bg-input: rgba(118, 118, 128, 0.24);
  --glass-bg-hover: rgba(255, 255, 255, 0.06);
  --glass-bg-active: rgba(10, 132, 255, 0.22);

  --glass-border-subtle: rgba(255, 255, 255, 0.08);
  --glass-border-specular: rgba(255, 255, 255, 0.14);
  --glass-shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.40), 0 2px 8px rgba(0, 0, 0, 0.25);
  --glass-shadow-lg: 0 12px 40px rgba(0, 0, 0, 0.55), 0 2px 6px rgba(0, 0, 0, 0.35);

  --text-primary: rgba(255, 255, 255, 0.92);
  --text-secondary: rgba(235, 235, 245, 0.62);
  --text-tertiary: rgba(235, 235, 245, 0.36);

  --accent-primary: #0A84FF;
  --accent-primary-hover: #409CFF;
  --accent-ring: rgba(10, 132, 255, 0.45);
  --text-on-accent: #FFFFFF;

  --semantic-success: #34C759;
  --semantic-warning: #FF9F0A;
  --semantic-danger: #FF3B30;

  --option-bg: #2C2C34;
  --option-fg: #F2F2F7;
}

/* ---------- LIGHT ---------- */
:root[data-theme="light"] {
  color-scheme: light;
  --glass-bg-canvas: rgba(242, 242, 247, 1);
  --glass-canvas-gradient:
    radial-gradient(1200px 800px at 0% 0%, rgba(0, 122, 255, 0.08), transparent 60%),
    radial-gradient(900px 700px at 100% 100%, rgba(175, 82, 222, 0.06), transparent 60%);
  --glass-bg-sidebar: rgba(246, 246, 250, 0.68);
  --glass-bg-surface: rgba(255, 255, 255, 0.62);
  --glass-bg-elevated: rgba(255, 255, 255, 0.82);
  --glass-bg-input: rgba(118, 118, 128, 0.12);
  --glass-bg-hover: rgba(0, 0, 0, 0.04);
  --glass-bg-active: rgba(0, 122, 255, 0.14);

  --glass-border-subtle: rgba(0, 0, 0, 0.08);
  --glass-border-specular: rgba(255, 255, 255, 0.90);
  --glass-shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.06), 0 2px 8px rgba(0, 0, 0, 0.06);
  --glass-shadow-lg: 0 12px 40px rgba(0, 0, 0, 0.14), 0 2px 6px rgba(0, 0, 0, 0.08);

  --text-primary: rgba(0, 0, 0, 0.88);
  --text-secondary: rgba(60, 60, 67, 0.66);
  --text-tertiary: rgba(60, 60, 67, 0.36);

  --accent-primary: #007AFF;
  --accent-primary-hover: #0066D6;
  --accent-ring: rgba(0, 122, 255, 0.35);
  --text-on-accent: #FFFFFF;

  --semantic-success: #34C759;
  --semantic-warning: #FF9F0A;
  --semantic-danger: #FF3B30;

  --option-bg: #FFFFFF;
  --option-fg: #1C1C1E;
}

/* ---------- Base ---------- */
*, *::before, *::after { box-sizing: border-box; }

html, body {
  margin: 0;
  height: 100%;
  overflow: hidden;              /* only .main-view scrolls */
  background: var(--glass-canvas-gradient), var(--glass-bg-canvas);
  color: var(--text-primary);
  transition: background-color var(--dur-med) var(--ease-out), color var(--dur-med) var(--ease-out);
}

/* ---------- Material utility classes ---------- */
.glass {
  background: var(--glass-bg-surface);
  -webkit-backdrop-filter: var(--glass-blur);
  backdrop-filter: var(--glass-blur);
  border: 1px solid var(--glass-border-subtle);
  box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-sm);
  border-radius: var(--radius-lg);
}
.glass--sidebar  { background: var(--glass-bg-sidebar);  border-radius: 0; box-shadow: none; }
.glass--elevated { background: var(--glass-bg-elevated); box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-lg); }

.card {
  composes: glass;               /* (documentation only — apply both classes or copy rules) */
  padding: var(--space-5);
  min-width: 0;
}

.text-secondary { color: var(--text-secondary); }
.text-tertiary  { color: var(--text-tertiary); }
.mono { font-family: var(--font-mono); font-size: 12px; }

/* ---------- Fallbacks (WebKitGTK perf / no backdrop-filter / a11y) ---------- */
@supports not ((backdrop-filter: blur(1px)) or (-webkit-backdrop-filter: blur(1px))) {
  :root[data-theme="dark"]  { --glass-bg-sidebar: rgb(30,30,36); --glass-bg-surface: rgb(40,40,47); --glass-bg-elevated: rgb(52,52,60); }
  :root[data-theme="light"] { --glass-bg-sidebar: rgb(244,244,248); --glass-bg-surface: rgb(255,255,255); --glass-bg-elevated: rgb(255,255,255); }
}
/* Low hardware tier or user preference → opaque, no blur (set from theme.svelte.ts) */
:root[data-reduce-transparency="true"] .glass,
:root[data-reduce-transparency="true"] .glass--sidebar,
:root[data-reduce-transparency="true"] .glass--elevated {
  -webkit-backdrop-filter: none;
  backdrop-filter: none;
}
@media (prefers-reduced-transparency: reduce) {
  .glass, .glass--sidebar, .glass--elevated { -webkit-backdrop-filter: none; backdrop-filter: none; }
}
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after { transition-duration: 0ms !important; animation-duration: 0ms !important; }
}

/* Focus ring — every interactive element */
:focus-visible {
  outline: none;
  box-shadow: 0 0 0 3px var(--accent-ring);
  border-radius: var(--radius-sm);
}
```

> [!NOTE]
> `composes:` is not real CSS — it is a note. Implementers apply `class="glass card"` together, or copy the `.glass` rules into `.card`.

Import once in `app-tauri/src/routes/+layout.svelte`:

```svelte
<script lang="ts">
  import '../app.css';
  let { children } = $props();
</script>

{@render children()}
```

### 1.3 Theme engine — `app-tauri/src/lib/theme.svelte.ts` (new)

```ts
export type ThemePref = 'system' | 'light' | 'dark';

const KEY = 'lpdf.theme'; // UI preference only, never document data
const mq = typeof window !== 'undefined' ? window.matchMedia('(prefers-color-scheme: dark)') : null;

function readInitial(): ThemePref {
  const v = typeof localStorage !== 'undefined' ? localStorage.getItem(KEY) : null;
  return v === 'light' || v === 'dark' || v === 'system' ? v : 'system';
}

class ThemeStore {
  pref = $state<ThemePref>(readInitial());
  systemDark = $state<boolean>(mq?.matches ?? true);
  resolved = $derived<'light' | 'dark'>(
    this.pref === 'system' ? (this.systemDark ? 'dark' : 'light') : this.pref
  );

  constructor() {
    mq?.addEventListener('change', (e) => { this.systemDark = e.matches; });
  }

  set(p: ThemePref) {
    this.pref = p;
    localStorage.setItem(KEY, p);
    // If settings are persisted Rust-side, ALSO call the existing save command here ⟨verify⟩.
  }

  setReduceTransparency(on: boolean) {
    document.documentElement.dataset.reduceTransparency = on ? 'true' : 'false';
  }
}

export const theme = new ThemeStore();

/** Call once from MainLayout: applies data-theme whenever `resolved` changes. */
export function applyThemeEffect() {
  $effect(() => {
    document.documentElement.dataset.theme = theme.resolved;
  });
}
```

To avoid a flash of the wrong theme, add to `app.html` `<head>` (inline, no network):

```html
<script>
  (function () {
    var p = localStorage.getItem('lpdf.theme') || 'system';
    var d = p === 'dark' || (p === 'system' && matchMedia('(prefers-color-scheme: dark)').matches);
    document.documentElement.dataset.theme = d ? 'dark' : 'light';
  })();
</script>
```

### 1.4 Rule — Theme Switching (MANDATORY for every `.svelte` file)

Every view's `<style>` block must contain **zero literal colours** for backgrounds, borders, or text. Replace as follows:

| Hardcoded (find) | Replace with |
|---|---|
| `#0f172a`, `#111827` (page/app bg) | `var(--glass-bg-canvas)` (only on the root; views are usually transparent) |
| `#1e293b`, `#1f2937`, `#334155` (card/panel bg) | `var(--glass-bg-surface)` + `.glass` |
| Sidebar bg (any) | `var(--glass-bg-sidebar)` |
| Modal/popover/dropdown bg | `var(--glass-bg-elevated)` |
| `<input>`/`<select>` bg | `var(--glass-bg-input)` |
| `#ffffff`, `#f8fafc`, `#e2e8f0`, `#f1f5f9` (text) | `var(--text-primary)` |
| `#94a3b8`, `#9ca3af`, `#64748b` (muted text) | `var(--text-secondary)` |
| `#475569`, `#6b7280` (placeholder/disabled) | `var(--text-tertiary)` |
| `#ffffff`, `#f8fafc` (light-mode bg if any) | `var(--glass-bg-surface)` |
| Blue accents (`#3b82f6`, `#2563eb`, `#60a5fa`) | `var(--accent-primary)` |
| Green (`#22c55e`, `#10b981`) | `var(--semantic-success)` |
| Amber (`#f59e0b`, `#eab308`) | `var(--semantic-warning)` |
| Red (`#ef4444`, `#dc2626`) | `var(--semantic-danger)` |
| Border greys (`#334155`, `#374151`, `#e5e7eb`) | `var(--glass-border-subtle)` |

Verification command (must return **no matches** in `<style>` blocks before commit):

```bash
grep -rnE '#(0f172a|1e293b|111827|1f2937|334155|ffffff|fff|f8fafc|e2e8f0|94a3b8|64748b)\b' app-tauri/src --include='*.svelte'
```

Allowed exceptions: SVG brand icons and PDF canvas/page rendering (pages are white paper in both themes).

---

## 2. Pillar 2 — Global High-Contrast Form Controls (Zero White-on-White)

Append to `app.css` (global file, so no `:global()` wrapper needed). If placed inside a component `<style>`, wrap each selector in `:global(...)` exactly as shown in the second block.

```css
/* ---------- Form controls (app.css) ---------- */
input:not([type="checkbox"]):not([type="radio"]):not([type="range"]),
select,
textarea {
  font: inherit;
  color: var(--text-primary);
  background-color: var(--glass-bg-input);
  border: 1px solid var(--glass-border-subtle);
  border-radius: var(--radius-md);
  padding: 0.45rem 0.7rem;
  min-height: 30px;
  min-width: 0;
  max-width: 100%;
  transition: border-color var(--dur-fast), box-shadow var(--dur-fast), background-color var(--dur-med);
}
input::placeholder, textarea::placeholder { color: var(--text-tertiary); opacity: 1; }
input:focus, select:focus, textarea:focus {
  outline: none;
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 3px var(--accent-ring);
}
input:disabled, select:disabled, textarea:disabled { opacity: 0.5; cursor: not-allowed; }

:root[data-theme="dark"]  select, :root[data-theme="dark"]  input, :root[data-theme="dark"]  textarea { color-scheme: dark; }
:root[data-theme="light"] select, :root[data-theme="light"] input, :root[data-theme="light"] textarea { color-scheme: light; }

/* Custom chevron so the native control never paints its own (possibly white) chrome */
select {
  -webkit-appearance: none;
  appearance: none;
  padding-right: 2rem;
  background-image:
    linear-gradient(45deg, transparent 50%, var(--text-secondary) 50%),
    linear-gradient(135deg, var(--text-secondary) 50%, transparent 50%);
  background-position: calc(100% - 15px) 52%, calc(100% - 10px) 52%;
  background-size: 5px 5px, 5px 5px;
  background-repeat: no-repeat;
  cursor: pointer;
}

/* WebKitGTK renders the <option> popup with GTK — MUST be OPAQUE + explicit colour */
select option,
select optgroup {
  background-color: var(--option-bg);
  color: var(--option-fg);
}
select option:checked { background-color: var(--accent-primary); color: var(--text-on-accent); }
select option:disabled { color: var(--text-tertiary); }

input[type="checkbox"], input[type="radio"], input[type="range"], progress {
  accent-color: var(--accent-primary);
}
```

Component-scoped equivalent (only if a component must override locally):

```css
:global(select),
:global(input),
:global(textarea) {
  color: var(--text-primary);
  background-color: var(--glass-bg-input);
}
:global(:root[data-theme="dark"] select)  { color-scheme: dark; }
:global(:root[data-theme="light"] select) { color-scheme: light; }
:global(select option),
:global(select optgroup) {
  background-color: var(--option-bg);   /* opaque, never rgba */
  color: var(--option-fg);
}
```

> [!WARNING]
> Never use an `rgba(...)` value for `option` backgrounds — GTK composites popups over white, which recreates the white-on-white bug. `--option-bg` is deliberately opaque.

---

## 3. Pillar 3 — Windowed-Mode Fluid Spatial Layout (800–1080 px, zero right-edge clipping)

### 3.1 Root rules

1. Every flex child that contains text or grids gets `min-width: 0`.
2. No fixed `width:` in px on panels > 320px. Use `max-width` + `width: 100%`.
3. Long paths: `overflow-wrap: anywhere;` or `.truncate` (ellipsis) + `title` attribute.
4. Only `.main-view` scrolls vertically. Nothing scrolls horizontally except explicit thumbnail strips.

Append to `app.css`:

```css
/* ---------- Layout primitives ---------- */
.app-shell {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}

.sidebar {
  flex: 0 0 var(--sidebar-width);
  width: var(--sidebar-width);
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 0.75rem 0.5rem;
  background: var(--glass-bg-sidebar);
  -webkit-backdrop-filter: var(--glass-blur);
  backdrop-filter: var(--glass-blur);
  border-right: 1px solid var(--glass-border-subtle);
  overflow-y: auto;
}
.sidebar__section-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-tertiary);
  padding: 0.75rem 0.625rem 0.25rem;
}
.sidebar__item {
  display: flex;
  align-items: center;
  gap: 0.55rem;
  width: 100%;
  padding: 0.4rem 0.625rem;
  border: none;
  border-radius: var(--radius-md);      /* pill-ish active state */
  background: transparent;
  color: var(--text-primary);
  font: inherit;
  text-align: left;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: background-color var(--dur-fast);
}
.sidebar__item:hover { background: var(--glass-bg-hover); }
.sidebar__item[aria-current="page"] {
  background: var(--glass-bg-active);
  color: var(--accent-primary);
  font-weight: 600;
}
.sidebar__item svg { flex: 0 0 16px; width: 16px; height: 16px; }

.content-column {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.main-view {
  flex: 1;
  min-width: 0;
  width: 100%;
  max-width: 100%;
  overflow-x: hidden;
  overflow-y: auto;
  padding: clamp(1rem, 2.5vw, 2rem);
  box-sizing: border-box;
}
.main-view > .view-inner { max-width: 1200px; margin: 0 auto; }

.statusbar {
  flex: 0 0 var(--statusbar-height);
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0 0.875rem;
  font-size: 11px;
  color: var(--text-secondary);
  background: var(--glass-bg-sidebar);
  -webkit-backdrop-filter: var(--glass-blur);
  backdrop-filter: var(--glass-blur);
  border-top: 1px solid var(--glass-border-subtle);
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
}

/* Two-column split views: ConvertFromPdfView, ConvertToPdfView, OrganizeView, MarkdownView
   (also OptimizeView, SecurityView, FormsView, AiView where a summary panel exists) */
.split-view {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 320px), 1fr));
  gap: 1.25rem;
  min-width: 0;
  align-items: start;
}
.split-view > * { min-width: 0; }

/* Toolbars / header rows: BatchQueueView, SettingsView, WorkflowsView, EditView */
.toolbar,
.view-header {
  display: flex;
  flex-wrap: wrap;
  gap: 0.625rem;
  align-items: center;
  justify-content: space-between;
  min-width: 0;
}
.view-header { margin-bottom: 1.25rem; }
.view-header h1 { font-size: 22px; font-weight: 700; letter-spacing: -0.01em; margin: 0; }
.view-header p  { margin: 0.15rem 0 0; color: var(--text-secondary); }
.toolbar__group { display: flex; flex-wrap: wrap; gap: 0.5rem; align-items: center; min-width: 0; }

.truncate { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; min-width: 0; }
.path     { font-family: var(--font-mono); font-size: 12px; overflow-wrap: anywhere; color: var(--text-secondary); }

/* Narrow windows: tighten sidebar (never hide it entirely below 800px requirement) */
@media (max-width: 860px) {
  :root { --sidebar-width: 184px; }
}
```

### 3.2 `MainLayout.svelte` skeleton

```svelte
<script lang="ts">
  import Sidebar from './Sidebar.svelte';
  import StatusBar from './StatusBar.svelte';
  import ToastContainer from './ToastContainer.svelte';
  import { applyThemeEffect } from '$lib/theme.svelte';
  let { children } = $props();
  applyThemeEffect();
</script>

<div class="app-shell">
  <Sidebar />
  <div class="content-column">
    <main class="main-view">
      <div class="view-inner">
        {@render children()}
      </div>
    </main>
    <StatusBar />
  </div>
</div>
<ToastContainer />
```

### 3.3 Per-view mapping

| View | Root wrapper | Notes |
|---|---|---|
| `ConvertView` → `ConvertFromPdfView` / `ConvertToPdfView` | `.view-header` + `.split-view` | Left: options card; Right: summary/preview card. Stacks below 1050px. |
| `OrganizeView` | `.view-header` + `.toolbar` + `.split-view` | Thumbnail grid: `grid-template-columns: repeat(auto-fill, minmax(120px, 1fr))`. |
| `MarkdownView` | `.split-view` | Editor + preview; preview `overflow-wrap: anywhere`. |
| `OptimizeView`, `SecurityView`, `FormsView`, `AiView` | `.view-header` + `.split-view` | |
| `EditView` | `.toolbar` (sticky top) + canvas area | Canvas area `min-width: 0; overflow: auto` (only allowed horizontal scroll). |
| `BatchQueueView`, `WorkflowsView` | `.view-header` + `.toolbar` + list of `.glass` rows | Row: `display:grid; grid-template-columns: auto 1fr auto; gap:.75rem; min-width:0`. |
| `SettingsView` | `.view-header` + `.settings-group` stack | See Pillar 5. |
| `Dropzone` | `.glass` dashed variant | See §5.3. |

---

## 4. Pillar 4 — Floating Glass Toast & Live Job Notification Center

### 4.1 Store — `app-tauri/src/lib/toast.svelte.ts` (new)

```ts
export type ToastKind = 'success' | 'error' | 'warning' | 'info';

export interface ToastAction {
  label: string;           // "Open", "Show in Folder", "Retry"
  run: () => void | Promise<void>;
}

export interface Toast {
  id: number;
  kind: ToastKind;
  title: string;           // "Converted to DOCX"
  detail?: string;         // output path or error diagnostic
  action?: ToastAction;
  timeoutMs: number;       // 0 = sticky
}

const DEFAULT_TIMEOUT = 6000;
const MAX_VISIBLE = 5;

class ToastStore {
  items = $state<Toast[]>([]);
  #seq = 0;
  #timers = new Map<number, ReturnType<typeof setTimeout>>();

  push(t: Omit<Toast, 'id' | 'timeoutMs'> & { timeoutMs?: number }): number {
    const id = ++this.#seq;
    const toast: Toast = { timeoutMs: t.kind === 'error' ? 10000 : DEFAULT_TIMEOUT, ...t, id };
    this.items = [toast, ...this.items].slice(0, MAX_VISIBLE);
    this.#arm(toast);
    return id;
  }

  success(title: string, detail?: string, action?: ToastAction) { return this.push({ kind: 'success', title, detail, action }); }
  error(title: string, detail?: string, action?: ToastAction)   { return this.push({ kind: 'error', title, detail, action }); }
  warning(title: string, detail?: string)                       { return this.push({ kind: 'warning', title, detail }); }
  info(title: string, detail?: string)                          { return this.push({ kind: 'info', title, detail }); }

  dismiss(id: number) {
    const h = this.#timers.get(id);
    if (h) clearTimeout(h);
    this.#timers.delete(id);
    this.items = this.items.filter((t) => t.id !== id);
  }

  clearAll() { for (const t of this.items) this.dismiss(t.id); }

  /** Hover pauses auto-dismiss (HIG: don't remove content the user is reading). */
  pause(id: number) {
    const h = this.#timers.get(id);
    if (h) clearTimeout(h);
    this.#timers.delete(id);
  }
  resume(id: number) {
    const t = this.items.find((x) => x.id === id);
    if (t) this.#arm(t);
  }

  #arm(t: Toast) {
    if (t.timeoutMs <= 0) return;
    this.#timers.set(t.id, setTimeout(() => this.dismiss(t.id), t.timeoutMs));
  }
}

export const toasts = new ToastStore();
```

### 4.2 Component — `app-tauri/src/lib/components/ToastContainer.svelte`

```svelte
<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import { backOut } from 'svelte/easing';
  import { flip } from 'svelte/animate';
  import { toasts, type ToastKind } from '$lib/toast.svelte';

  const ICON: Record<ToastKind, string> = { success: '✓', error: '!', warning: '⚠', info: 'i' };
</script>

<section class="toast-stack" aria-live="polite" aria-label="Notifications">
  {#each toasts.items as t (t.id)}
    <div
      class="toast toast--{t.kind}"
      role={t.kind === 'error' ? 'alert' : 'status'}
      animate:flip={{ duration: 220 }}
      in:fly={{ x: 40, y: -8, duration: 420, easing: backOut }}
      out:fade={{ duration: 160 }}
      onmouseenter={() => toasts.pause(t.id)}
      onmouseleave={() => toasts.resume(t.id)}
    >
      <span class="toast__badge" aria-hidden="true">{ICON[t.kind]}</span>
      <div class="toast__body">
        <div class="toast__title">{t.title}</div>
        {#if t.detail}<div class="toast__detail" title={t.detail}>{t.detail}</div>{/if}
      </div>
      <div class="toast__actions">
        {#if t.action}
          <button class="toast__btn toast__btn--primary"
            onclick={async () => { await t.action?.run(); toasts.dismiss(t.id); }}>
            {t.action.label}
          </button>
        {/if}
        <button class="toast__btn" aria-label="Clear notification" onclick={() => toasts.dismiss(t.id)}>Clear</button>
      </div>
    </div>
  {/each}
</section>

<style>
  .toast-stack {
    position: fixed;
    top: 16px;
    right: 16px;
    z-index: 9999;
    display: flex;
    flex-direction: column;
    gap: 10px;
    width: min(380px, calc(100vw - 32px));
    pointer-events: none;
  }
  .toast {
    pointer-events: auto;
    display: grid;
    grid-template-columns: auto 1fr;
    grid-template-areas: "badge body" "badge actions";
    column-gap: 12px;
    row-gap: 8px;
    padding: 12px 14px;
    border-radius: var(--radius-xl);
    background: var(--glass-bg-elevated);
    -webkit-backdrop-filter: var(--glass-blur-toast);
    backdrop-filter: var(--glass-blur-toast);
    border: 1px solid var(--glass-border-subtle);
    box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-lg);
    color: var(--text-primary);
    min-width: 0;
  }
  .toast__badge {
    grid-area: badge;
    width: 28px; height: 28px;
    border-radius: 50%;
    display: grid; place-items: center;
    font-weight: 700; font-size: 14px;
    color: #fff;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.35);
  }
  .toast--success .toast__badge { background: var(--semantic-success); }
  .toast--error   .toast__badge { background: var(--semantic-danger); }
  .toast--warning .toast__badge { background: var(--semantic-warning); }
  .toast--info    .toast__badge { background: var(--accent-primary); }

  .toast__body   { grid-area: body; min-width: 0; }
  .toast__title  { font-weight: 600; font-size: 13px; }
  .toast__detail {
    margin-top: 2px;
    font-family: var(--font-mono);
    font-size: 11.5px;
    color: var(--text-secondary);
    overflow-wrap: anywhere;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .toast__actions { grid-area: actions; display: flex; gap: 6px; justify-content: flex-end; }
  .toast__btn {
    font: inherit; font-size: 12px; font-weight: 500;
    padding: 4px 12px;
    border-radius: var(--radius-pill);
    border: 1px solid var(--glass-border-subtle);
    background: var(--glass-bg-input);
    color: var(--text-primary);
    cursor: pointer;
  }
  .toast__btn:hover { background: var(--glass-bg-hover); }
  .toast__btn--primary { background: var(--accent-primary); color: var(--text-on-accent); border-color: transparent; }
  .toast__btn--primary:hover { background: var(--accent-primary-hover); }
</style>
```

> [!NOTE]
> Svelte 5 event syntax is `onclick` / `onmouseenter` (not `on:click`). Text is rendered via `{…}` interpolation only — never `{@html}` (drift detector).

### 4.3 Global wiring — job events → toasts

Mount the listener **once**, at app root (`+page.svelte` `onMount`, or inside `state.svelte.ts` init). Do NOT add per-view listeners for toasts — that guarantees every tool gets one toast and none are duplicated.

```ts
// in state.svelte.ts (or +page.svelte) — run once at startup
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { toasts } from '$lib/toast.svelte';

// ⟨verify⟩ payload shape against the Rust JobEvent enum in src-tauri before use.
type JobEventPayload =
  | { kind: 'Completed'; job_id: string; tool: string; output_path?: string }
  | { kind: 'Failed';    job_id: string; tool: string; error: string }
  | { kind: 'Cancelled'; job_id: string; tool: string }
  | { kind: 'Progress';  job_id: string; tool: string; percent: number };

const TOOL_LABEL: Record<string, string> = {
  // ⟨verify⟩ keys against the tool registry (07_FEATURE_MAP)
  compress: 'Compressed PDF', protect: 'Protected PDF', convert_docx: 'Converted to Word',
  pdf_to_images: 'Exported Images', merge: 'Merged PDFs', split: 'Split PDF', ocr: 'OCR Complete',
};

let unlisten: UnlistenFn | null = null;

export async function initJobToasts(): Promise<void> {
  if (unlisten) return; // idempotent
  unlisten = await listen<JobEventPayload>('job-event', ({ payload }) => {
    const label = TOOL_LABEL[payload.tool] ?? payload.tool;
    switch (payload.kind) {
      case 'Completed':
        toasts.success(label, payload.output_path, payload.output_path
          ? { label: 'Open', run: () => openPath(payload.output_path as string) }
          : undefined);
        break;
      case 'Failed':
        toasts.error(`${label} failed`, payload.error);
        break;
      case 'Cancelled':
        toasts.warning(`${label} cancelled`);
        break;
      default:
        break; // Progress → StatusBar / BatchQueueView, never a toast
    }
  });
}

/** ⟨verify⟩ Use the EXISTING open/reveal command or opener plugin already in the repo.
 *  If none exists, omit the action (do not add a new capability without an ADR). */
async function openPath(_path: string): Promise<void> { /* wire to existing command */ }
```

Synchronous (non-job) actions — call directly at the action site:

| Event | Call |
|---|---|
| Settings saved | `toasts.success('Settings saved')` |
| Theme changed | none (the change itself is the feedback) |
| File loaded / dropped | `toasts.info('Opened document', fileName)` |
| File picker cancelled | none |
| Validation error (e.g. no file selected) | `toasts.warning('Select a PDF first')` |
| Invoke rejected (typed error from Rust) | `toasts.error('<Operation> failed', String(err))` |
| Copied to clipboard | `toasts.info('Copied')` |

Pattern for every `invoke` call site:

```ts
try {
  await invoke('⟨verified_command⟩', { /* args */ });
  // job-based tools: no toast here — the job-event listener handles it
  // synchronous tools: toasts.success('…', outputPath)
} catch (err) {
  toasts.error('Operation failed', String(err));
}
```

---

## 5. Pillar 5 — Human-Centered Buttons & Settings Placement (Apple HIG)

### 5.1 Buttons — append to `app.css`

```css
.btn {
  display: inline-flex; align-items: center; justify-content: center; gap: 0.4rem;
  font: inherit; font-size: 13px; font-weight: 500;
  min-height: 30px;
  padding: 0.4rem 1rem;
  border-radius: var(--radius-md);
  border: 1px solid var(--glass-border-subtle);
  background: var(--glass-bg-input);
  color: var(--text-primary);
  box-shadow: inset 0 1px 0 0 var(--glass-border-specular);
  cursor: pointer;
  white-space: nowrap;
  transition: background-color var(--dur-fast), transform var(--dur-fast), box-shadow var(--dur-fast);
}
.btn:hover   { background: var(--glass-bg-hover); }
.btn:active  { transform: scale(0.98); }
.btn:disabled { opacity: 0.45; cursor: not-allowed; transform: none; }

.btn--primary {
  background: linear-gradient(180deg, color-mix(in srgb, var(--accent-primary) 92%, white) 0%, var(--accent-primary) 100%);
  color: var(--text-on-accent);
  border-color: transparent;
  box-shadow: inset 0 1px 0 0 rgba(255, 255, 255, 0.28), 0 1px 3px rgba(0, 0, 0, 0.2);
  font-weight: 600;
}
.btn--primary:hover { background: var(--accent-primary-hover); }

.btn--destructive { background: var(--semantic-danger); color: #fff; border-color: transparent; }
.btn--plain { background: transparent; border-color: transparent; box-shadow: none; color: var(--accent-primary); }
.btn--lg { min-height: 36px; padding: 0.55rem 1.25rem; font-size: 14px; }

/* CTA footer: primary action ALWAYS bottom-right of its card */
.card-footer {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  align-items: center;
  gap: 0.5rem;
  margin-top: 1.25rem;
  padding-top: 1rem;
  border-top: 1px solid var(--glass-border-subtle);
}
.card-footer__hint { margin-right: auto; color: var(--text-secondary); font-size: 12px; min-width: 0; }
```

> [!NOTE]
> If `color-mix()` is unsupported in the shipped WebKitGTK, the gradient line is ignored and `.btn--primary:hover`'s solid colour still applies; add `background: var(--accent-primary);` on the line above as a fallback.

### 5.2 CTA placement rules

1. One primary (`.btn--primary`) per card, **rightmost** in `.card-footer`. Secondary (`Cancel`, `Reset`) to its left. Hint/estimate text far left (`.card-footer__hint`).
2. Verb labels: **Convert**, **Compress**, **Protect**, **Apply**, **Merge**, **Split**, **Export**, **Run OCR**. Never "Submit" / "OK".
3. Disabled until valid input; while running, label becomes "Converting…" with a spinner and the button stays disabled (prevents double-submit).
4. Destructive actions (Redact, Remove pages, Clear queue) use `.btn--destructive` and a confirm step.
5. `Enter` on a focused card triggers its primary action; `Esc` cancels/clears.

Card template (every tool view):

```svelte
<div class="split-view">
  <section class="glass card">
    <h2 class="card-title">Options</h2>
    <!-- fields -->
    <footer class="card-footer">
      <span class="card-footer__hint">{estimate}</span>
      <button class="btn" onclick={reset}>Reset</button>
      <button class="btn btn--primary" disabled={!canRun || running} onclick={run}>
        {running ? 'Converting…' : 'Convert'}
      </button>
    </footer>
  </section>
  <aside class="glass card"><!-- summary / preview --></aside>
</div>
```

### 5.3 Dropzone

```css
.dropzone {
  display: grid; place-items: center; gap: 0.5rem;
  padding: 2.5rem 1.5rem;
  border: 1.5px dashed var(--glass-border-subtle);
  border-radius: var(--radius-lg);
  background: var(--glass-bg-surface);
  -webkit-backdrop-filter: var(--glass-blur);
  backdrop-filter: var(--glass-blur);
  color: var(--text-secondary);
  text-align: center;
  transition: border-color var(--dur-fast), background-color var(--dur-fast);
}
.dropzone[data-dragover="true"] {
  border-color: var(--accent-primary);
  background: var(--glass-bg-active);
  color: var(--text-primary);
}
```
"Browse Files…" inside the dropzone is a `.btn` (not primary — the tool's CTA is the primary).

### 5.4 Settings — macOS System Settings grouped inset style

Section order (fixed): **Appearance → Performance & Hardware Tier → Output & Safety → Offline AI Models → Diagnostics**.

```css
.settings-group { margin-bottom: 1.5rem; }
.settings-group__title {
  font-size: 13px; font-weight: 600; color: var(--text-secondary);
  margin: 0 0 0.5rem 0.75rem;
}
.settings-group__list {
  background: var(--glass-bg-surface);
  -webkit-backdrop-filter: var(--glass-blur);
  backdrop-filter: var(--glass-blur);
  border: 1px solid var(--glass-border-subtle);
  border-radius: var(--radius-lg);
  box-shadow: inset 0 1px 0 0 var(--glass-border-specular);
  overflow: hidden;
}
.settings-row {
  display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between;
  gap: 0.75rem;
  padding: 0.7rem 1rem;
  min-width: 0;
}
.settings-row + .settings-row { border-top: 1px solid var(--glass-border-subtle); }
.settings-row__label { min-width: 0; }
.settings-row__label small { display: block; color: var(--text-secondary); font-size: 11.5px; }

/* Segmented control: System | Light | Dark */
.segmented {
  display: inline-flex;
  padding: 2px;
  border-radius: var(--radius-sm);
  background: var(--glass-bg-input);
  border: 1px solid var(--glass-border-subtle);
}
.segmented button {
  font: inherit; font-size: 12px; font-weight: 500;
  min-width: 64px;
  padding: 0.25rem 0.75rem;
  border: none; border-radius: 6px;
  background: transparent;
  color: var(--text-primary);
  cursor: pointer;
  transition: background-color var(--dur-fast), box-shadow var(--dur-fast);
}
.segmented button[aria-checked="true"] {
  background: var(--glass-bg-elevated);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.18), inset 0 1px 0 0 var(--glass-border-specular);
  font-weight: 600;
}
```

```svelte
<script lang="ts">
  import { theme, type ThemePref } from '$lib/theme.svelte';
  const OPTIONS: { value: ThemePref; label: string }[] = [
    { value: 'system', label: 'System' }, { value: 'light', label: 'Light' }, { value: 'dark', label: 'Dark' },
  ];
</script>

<header class="view-header"><div><h1>Settings</h1></div></header>

<section class="settings-group">
  <h2 class="settings-group__title">Appearance</h2>
  <div class="settings-group__list">
    <div class="settings-row">
      <div class="settings-row__label">Theme<small>System follows your desktop setting.</small></div>
      <div class="segmented" role="radiogroup" aria-label="Theme">
        {#each OPTIONS as o (o.value)}
          <button role="radio" aria-checked={theme.pref === o.value} onclick={() => theme.set(o.value)}>{o.label}</button>
        {/each}
      </div>
    </div>
    <div class="settings-row">
      <div class="settings-row__label">Reduce transparency<small>Opaque surfaces; faster on low-tier GPUs.</small></div>
      <!-- toggle → theme.setReduceTransparency(v) -->
    </div>
  </div>
</section>

<!-- Repeat .settings-group for: -->
<!-- Performance & Hardware Tier : tier selector (auto/low/mid/high), worker threads, memory cap  -->
<!-- Output & Safety             : default output folder, "never overwrite originals" (locked ON), verify-after-write (locked ON) -->
<!-- Offline AI Models           : model list, size on disk, import model file, remove -->
<!-- Diagnostics                 : app version, open log folder, copy diagnostics, offline status (always "No network") -->
```

Settings rules: changes apply **instantly** (no global "Save" button, macOS style). Persisted settings fire a single `toasts.success('Settings saved')`, debounced 800 ms. Safety invariants (no overwrite, verify-after-write) are shown as locked-on rows, never toggleable.

### 5.5 Sidebar grouping (HIG source list)

| Group label | Items |
|---|---|
| (none) | Home / Open |
| PDF Tools | Organize, Edit, Forms, Optimize, Security |
| Convert | Convert, Markdown |
| Intelligence | AI (Offline) |
| Automation | Batch Queue, Workflows |
| (bottom, pinned) | Settings |

Active item: `aria-current="page"` → `.sidebar__item[aria-current="page"]` pill style.

---

## 6. Acceptance Criteria (rc5 gate — all must be YES)

| # | Check | How |
|---|---|---|
| A1 | System/Light/Dark switches every surface instantly, no reload | Manual, all 15 views |
| A2 | No hardcoded colour hits in `.svelte` styles | grep in §1.4 returns empty |
| A3 | Dropdown popups readable in both themes on WebKitGTK | Manual on Fedora/KDE |
| A4 | No horizontal scrollbar / clipping at 800, 960, 1080 px | Resize window, every view |
| A5 | Every job (Completed/Failed/Cancelled) shows exactly one toast | Run 3 tools + cancel 1 |
| A6 | Toast auto-dismisses at 6 s (errors 10 s), pauses on hover | Manual |
| A7 | Primary CTA bottom-right in every tool card | Visual audit |
| A8 | Settings in 5 grouped sections, segmented theme control | Visual |
| A9 | Reduce-transparency / low tier disables blur | Toggle + inspect |
| A10 | `cargo xtask gate` + `protocol-lint` green; no `{@html}`, `fetch(`, remote URLs | Gate |

## 7. Out of Scope

- No Rust/back-end changes except (if needed) exposing an already-existing open/reveal command — requires verification, and an ADR if a new capability is added.
- No new npm dependencies (Svelte built-in `transition`/`animate`/`easing` only).
- No custom window chrome / titlebar changes in rc5.
