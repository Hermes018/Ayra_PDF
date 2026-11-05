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
  | '--accent' | '--danger' | '--focus-ring' | '--motion-duration';
type TokenTable = Readonly<Record<TokenName, string>>;

const TOKENS: Readonly<Record<ResolvedTheme, TokenTable>> = Object.freeze({
  light: Object.freeze({
    '--glass-bg': 'rgba(255,255,255,0.62)', '--glass-border': 'rgba(0,0,0,0.08)', '--glass-blur': '24px',
    '--glass-shadow': '0 8px 32px rgba(0,0,0,0.12)', '--glass-text': '#1d1d1f',
    '--canvas-bg': '#e9e9ee', '--canvas-page-shadow': 'rgba(0,0,0,0.18)', '--canvas-grid-major': 'rgba(0,0,0,0.14)',
    '--canvas-grid-minor': 'rgba(0,0,0,0.06)', '--canvas-selection': 'rgba(0,122,255,0.22)',
    '--snap-indicator-endpoint': '#ff3b30', '--snap-indicator-midpoint': '#34c759', '--snap-indicator-intersection': '#ff9500',
    '--snap-indicator-perpendicular': '#af52de', '--snap-indicator-grid': '#8e8e93', '--snap-indicator-size': '10px',
    '--laser-core': '#ffffff', '--laser-glow': '#ff2d55', '--measure-line': '#0a84ff',
    '--measure-label-bg': 'rgba(255,255,255,0.9)', '--measure-label-text': '#1d1d1f',
    '--accent': '#007aff', '--danger': '#ff3b30', '--focus-ring': '#0a84ff', '--motion-duration': '180ms',
  }),
  dark: Object.freeze({
    '--glass-bg': 'rgba(30,30,32,0.58)', '--glass-border': 'rgba(255,255,255,0.10)', '--glass-blur': '24px',
    '--glass-shadow': '0 8px 32px rgba(0,0,0,0.45)', '--glass-text': '#f5f5f7',
    '--canvas-bg': '#1c1c1e', '--canvas-page-shadow': 'rgba(0,0,0,0.6)', '--canvas-grid-major': 'rgba(255,255,255,0.16)',
    '--canvas-grid-minor': 'rgba(255,255,255,0.06)', '--canvas-selection': 'rgba(10,132,255,0.28)',
    '--snap-indicator-endpoint': '#ff453a', '--snap-indicator-midpoint': '#30d158', '--snap-indicator-intersection': '#ff9f0a',
    '--snap-indicator-perpendicular': '#bf5af2', '--snap-indicator-grid': '#98989d', '--snap-indicator-size': '10px',
    '--laser-core': '#ffffff', '--laser-glow': '#ff375f', '--measure-line': '#64d2ff',
    '--measure-label-bg': 'rgba(28,28,30,0.92)', '--measure-label-text': '#f5f5f7',
    '--accent': '#0a84ff', '--danger': '#ff453a', '--focus-ring': '#64d2ff', '--motion-duration': '180ms',
  }),
});

export class GlobalThemeContract {
  mode = $state<ThemeMode>('system');
  #systemDark = $state(false);
  #reducedMotion = $state(false);
  readonly resolved: ResolvedTheme = $derived(this.mode === 'system' ? (this.#systemDark ? 'dark' : 'light') : this.mode);
  readonly reducedMotion: boolean = $derived(this.#reducedMotion);
  readonly tokens: TokenTable = $derived(TOKENS[this.resolved]);

  /** Canvas-2D painters cannot read CSS vars cheaply; they read the same table here. */
  token(name: TokenName): string { return this.tokens[name]; }

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
