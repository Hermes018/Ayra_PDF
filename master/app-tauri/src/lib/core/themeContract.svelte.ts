// app-tauri/src/lib/core/themeContract.svelte.ts
// The ONLY file permitted to contain colour literals (protocol-lint rule `no-hardcoded-colour`).
export type ThemeMode = 'system' | 'light' | 'dark';
export type ResolvedTheme = 'light' | 'dark';
export type TokenName =
  | '--glass-bg' | '--glass-border' | '--glass-blur' | '--glass-shadow' | '--glass-text'
  | '--canvas-bg' | '--canvas-page-shadow' | '--canvas-grid-major' | '--canvas-grid-minor' | '--canvas-selection'
  | '--snap-indicator-endpoint' | '--snap-indicator-midpoint' | '--snap-indicator-intersection'
  | '--snap-indicator-perpendicular' | '--snap-indicator-grid' | '--snap-indicator-size'
  | '--laser-core' | '--laser-glow' | '--measure-line' | '--measure-label-bg' | '--measure-label-text'
  | '--accent' | '--danger' | '--focus-ring' | '--motion-duration'
  | '--glass-bg-canvas' | '--glass-canvas-gradient' | '--glass-bg-sidebar' | '--glass-bg-surface' | '--glass-bg-elevated'
  | '--glass-bg-input' | '--glass-bg-hover' | '--glass-bg-active' | '--glass-border-subtle' | '--glass-border-specular'
  | '--glass-shadow-sm' | '--glass-shadow-lg' | '--text-primary' | '--text-secondary' | '--text-tertiary'
  | '--accent-primary' | '--accent-primary-hover' | '--accent-ring' | '--text-on-accent'
  | '--semantic-success' | '--semantic-warning' | '--semantic-danger' | '--option-bg' | '--option-fg' | '--surface-dialog' | '--scrim';
export type StaticTokenName =
  | '--font-sans' | '--font-mono' | '--radius-xs' | '--radius-sm' | '--radius-md' | '--radius-lg' | '--radius-xl' | '--radius-pill'
  | '--space-1' | '--space-2' | '--space-3' | '--space-4' | '--space-5' | '--space-6' | '--space-8'
  | '--glass-blur-toast' | '--ease-spring' | '--ease-out' | '--dur-fast' | '--dur-med'
  | '--font-size-caption' | '--font-size-footnote' | '--font-size-body' | '--font-size-title3' | '--font-size-title2' | '--font-size-title1'
  | '--control-height-sm' | '--control-height-md' | '--control-height-lg' | '--z-dialog' | '--z-toast';
type TokenTable = Readonly<Record<TokenName, string>>;

export const TOKENS: Readonly<Record<ResolvedTheme, TokenTable>> = Object.freeze({
  light: Object.freeze({
    '--glass-bg': 'rgba(255,255,255,0.62)', '--glass-border': 'rgba(0,0,0,0.08)', '--glass-blur': 'blur(24px) saturate(180%)',
    '--glass-shadow': '0 8px 32px rgba(0,0,0,0.12)', '--glass-text': '#1d1d1f',
    '--canvas-bg': '#e9e9ee', '--canvas-page-shadow': 'rgba(0,0,0,0.18)', '--canvas-grid-major': 'rgba(0,0,0,0.14)',
    '--canvas-grid-minor': 'rgba(0,0,0,0.06)', '--canvas-selection': 'rgba(0,122,255,0.22)',
    '--snap-indicator-endpoint': '#ff3b30', '--snap-indicator-midpoint': '#34c759', '--snap-indicator-intersection': '#ff9500',
    '--snap-indicator-perpendicular': '#af52de', '--snap-indicator-grid': '#8e8e93', '--snap-indicator-size': '10px',
    '--laser-core': '#ffffff', '--laser-glow': '#ff2d55', '--measure-line': '#0a84ff',
    '--measure-label-bg': 'rgba(255,255,255,0.9)', '--measure-label-text': '#1d1d1f',
    '--accent': '#007aff', '--danger': '#ff3b30', '--focus-ring': '#0a84ff', '--motion-duration': '180ms',
    '--glass-bg-canvas': 'rgba(245, 245, 247, 0.8)',
    '--glass-canvas-gradient': 'radial-gradient(1200px 800px at 0% 0%, rgba(0, 122, 255, 0.08), transparent 60%), radial-gradient(900px 700px at 100% 100%, rgba(175, 82, 222, 0.06), transparent 60%)',
    '--glass-bg-sidebar': 'rgba(246, 246, 250, 0.68)', '--glass-bg-surface': 'rgba(255, 255, 255, 0.6)',
    '--glass-bg-elevated': 'rgba(255, 255, 255, 0.82)', '--glass-bg-input': 'rgba(118, 118, 128, 0.12)',
    '--glass-bg-hover': 'rgba(0, 0, 0, 0.04)', '--glass-bg-active': 'rgba(0, 122, 255, 0.14)',
    '--glass-border-subtle': 'rgba(0, 0, 0, 0.08)', '--glass-border-specular': 'rgba(255, 255, 255, 0.90)',
    '--glass-shadow-sm': '0 1px 2px rgba(0, 0, 0, 0.06), 0 2px 8px rgba(0, 0, 0, 0.06)',
    '--glass-shadow-lg': '0 12px 40px rgba(0, 0, 0, 0.14), 0 2px 6px rgba(0, 0, 0, 0.08)',
    '--text-primary': 'rgba(0, 0, 0, 0.88)', '--text-secondary': 'rgba(60, 60, 67, 0.66)', '--text-tertiary': 'rgba(60, 60, 67, 0.36)',
    '--accent-primary': '#007AFF', '--accent-primary-hover': '#0066D6', '--accent-ring': 'rgba(0, 122, 255, 0.35)', '--text-on-accent': '#FFFFFF',
    '--semantic-success': '#34C759', '--semantic-warning': '#FF9F0A', '--semantic-danger': '#FF3B30',
    '--option-bg': '#FFFFFF', '--option-fg': '#1C1C1E',
    '--surface-dialog': '#FFFFFF', '--scrim': 'rgba(0, 0, 0, 0.30)',
  }),
  dark: Object.freeze({
    '--glass-bg': 'rgba(30,30,32,0.58)', '--glass-border': 'rgba(255,255,255,0.10)', '--glass-blur': 'blur(24px) saturate(180%)',
    '--glass-shadow': '0 8px 32px rgba(0,0,0,0.45)', '--glass-text': '#f5f5f7',
    '--canvas-bg': '#1c1c1e', '--canvas-page-shadow': 'rgba(0,0,0,0.6)', '--canvas-grid-major': 'rgba(255,255,255,0.16)',
    '--canvas-grid-minor': 'rgba(255,255,255,0.06)', '--canvas-selection': 'rgba(10,132,255,0.28)',
    '--snap-indicator-endpoint': '#ff453a', '--snap-indicator-midpoint': '#30d158', '--snap-indicator-intersection': '#ff9f0a',
    '--snap-indicator-perpendicular': '#bf5af2', '--snap-indicator-grid': '#98989d', '--snap-indicator-size': '10px',
    '--laser-core': '#ffffff', '--laser-glow': '#ff375f', '--measure-line': '#64d2ff',
    '--measure-label-bg': 'rgba(28,28,30,0.92)', '--measure-label-text': '#f5f5f7',
    '--accent': '#0a84ff', '--danger': '#ff453a', '--focus-ring': '#64d2ff', '--motion-duration': '180ms',
    '--glass-bg-canvas': 'rgba(18, 18, 22, 1)',
    '--glass-canvas-gradient': 'radial-gradient(1200px 800px at 0% 0%, rgba(10, 132, 255, 0.10), transparent 60%), radial-gradient(900px 700px at 100% 100%, rgba(191, 90, 242, 0.08), transparent 60%)',
    '--glass-bg-sidebar': 'rgba(30, 30, 36, 0.62)', '--glass-bg-surface': 'rgba(44, 44, 52, 0.55)',
    '--glass-bg-elevated': 'rgba(58, 58, 68, 0.72)', '--glass-bg-input': 'rgba(118, 118, 128, 0.24)',
    '--glass-bg-hover': 'rgba(255, 255, 255, 0.06)', '--glass-bg-active': 'rgba(10, 132, 255, 0.22)',
    '--glass-border-subtle': 'rgba(255, 255, 255, 0.08)', '--glass-border-specular': 'rgba(255, 255, 255, 0.14)',
    '--glass-shadow-sm': '0 1px 2px rgba(0, 0, 0, 0.40), 0 2px 8px rgba(0, 0, 0, 0.25)',
    '--glass-shadow-lg': '0 12px 40px rgba(0, 0, 0, 0.55), 0 2px 6px rgba(0, 0, 0, 0.35)',
    '--text-primary': 'rgba(255, 255, 255, 0.92)', '--text-secondary': 'rgba(235, 235, 245, 0.62)', '--text-tertiary': 'rgba(235, 235, 245, 0.36)',
    '--accent-primary': '#0A84FF', '--accent-primary-hover': '#409CFF', '--accent-ring': 'rgba(10, 132, 255, 0.45)', '--text-on-accent': '#FFFFFF',
    '--semantic-success': '#34C759', '--semantic-warning': '#FF9F0A', '--semantic-danger': '#FF3B30',
    '--option-bg': '#2C2C34', '--option-fg': '#F2F2F7',
    '--surface-dialog': '#2C2C34', '--scrim': 'rgba(0, 0, 0, 0.55)',
  }),
});

/** Theme-independent scale tokens (type, spacing, shape, motion, layering). Spacing stays in rem: the root font-size is 13px. */
export const STATIC_TOKENS: Readonly<Record<StaticTokenName, string>> = Object.freeze({
  '--font-sans': '-apple-system, BlinkMacSystemFont, "SF Pro Text", "SF Pro Display", "Inter", "Inter Variable", "Cantarell", "Noto Sans", "Segoe UI", system-ui, sans-serif',
  '--font-mono': '"SF Mono", ui-monospace, "JetBrains Mono", "Fira Code", "DejaVu Sans Mono", monospace',
  '--radius-xs': '6px', '--radius-sm': '8px', '--radius-md': '10px', '--radius-lg': '14px', '--radius-xl': '20px', '--radius-pill': '999px',
  '--space-1': '0.25rem', '--space-2': '0.5rem', '--space-3': '0.75rem', '--space-4': '1rem',
  '--space-5': '1.25rem', '--space-6': '1.5rem', '--space-8': '2rem',
  '--glass-blur-toast': 'blur(20px) saturate(180%)',
  '--ease-spring': 'cubic-bezier(0.34, 1.56, 0.64, 1)', '--ease-out': 'cubic-bezier(0.22, 1, 0.36, 1)',
  '--dur-fast': '120ms', '--dur-med': '220ms',
  // NEW (DESIGN_LANGUAGE section 2)
  '--font-size-caption': '11px', '--font-size-footnote': '12px', '--font-size-body': '13px',
  '--font-size-title3': '15px', '--font-size-title2': '17px', '--font-size-title1': '22px',
  '--control-height-sm': '24px', '--control-height-md': '28px', '--control-height-lg': '36px',
  '--z-dialog': '300', '--z-toast': '400',
});
export const NEW_TOKEN_NAMES: readonly StaticTokenName[] = [
  '--font-size-caption', '--font-size-footnote', '--font-size-body', '--font-size-title3', '--font-size-title2', '--font-size-title1',
  '--control-height-sm', '--control-height-md', '--control-height-lg', '--z-dialog', '--z-toast',
];

export class GlobalThemeContract {
  mode = $state<ThemeMode>('system');
  #systemDark = $state(false);
  #reducedMotion = $state(false);
  readonly resolved: ResolvedTheme = $derived(this.mode === 'system' ? (this.#systemDark ? 'dark' : 'light') : this.mode);
  readonly reducedMotion: boolean = $derived(this.#reducedMotion);
  readonly tokens: TokenTable = $derived(TOKENS[this.resolved]);

  /** Canvas-2D painters cannot read CSS vars cheaply; they read the same table here. */
  token(name: TokenName): string { return this.tokens[name]; }

  /**
   * Write every token onto `root` synchronously (call before first paint, e.g. from the root layout).
   * app.css defines no token values of its own; this is the only writer.
   */
  applyToRoot(root: HTMLElement): void {
    if (typeof window !== 'undefined' && window.matchMedia && this.mode === 'system') {
      this.#systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    }
    for (const [k, v] of Object.entries(STATIC_TOKENS)) root.style.setProperty(k, v);
    const table = this.tokens;
    for (const key of Object.keys(table) as TokenName[]) root.style.setProperty(key, table[key]);
    root.dataset.theme = this.resolved;
  }

  /** Mount once from root view/component. Returns a disposer. */
  attach(root: HTMLElement): () => void {
    if (typeof window === 'undefined') return () => {};
    const dark = window.matchMedia('(prefers-color-scheme: dark)');
    const motion = window.matchMedia('(prefers-reduced-motion: reduce)');
    this.#systemDark = dark.matches;
    this.#reducedMotion = motion.matches;
    const onDark = (e: MediaQueryListEvent): void => { this.#systemDark = e.matches; };
    const onMotion = (e: MediaQueryListEvent): void => { this.#reducedMotion = e.matches; };
    dark.addEventListener('change', onDark);
    motion.addEventListener('change', onMotion);
    for (const [k, v] of Object.entries(STATIC_TOKENS)) root.style.setProperty(k, v);
    const stop = $effect.root(() => {
      $effect(() => {
        const table = this.tokens;
        for (const key of Object.keys(table) as TokenName[]) root.style.setProperty(key, table[key]);
        root.style.setProperty('--motion-duration', this.reducedMotion ? '0ms' : table['--motion-duration']);
        root.dataset.theme = this.resolved;
        this.syncWindowVibrancy(this.resolved);
      });
    });
    return () => { dark.removeEventListener('change', onDark); motion.removeEventListener('change', onMotion); stop(); };
  }

  /** Phase W2: Synchronize Windows 11/10 DWM Mica/Acrylic window vibrancy with active theme. */
  syncWindowVibrancy(themeName?: string): void {
    if (typeof window === 'undefined') return;
    const targetTheme = themeName ?? this.resolved;
    import('@tauri-apps/api/core')
      .then(({ invoke }) => invoke('set_window_vibrancy_theme', { theme: targetTheme }))
      .catch(() => {});
  }
}
export const theme = new GlobalThemeContract();
