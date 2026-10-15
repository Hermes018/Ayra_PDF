import type { ThemePreference } from './bindings/ThemePreference';

class AppState {
    activeFiles = $state<string[]>([]);
    themePreference = $state<ThemePreference>('System');
    effectiveTheme = $state<'light' | 'dark'>('dark');
    private mediaQueryList: MediaQueryList | null = null;
    private mediaListener: ((e: MediaQueryListEvent) => void) | null = null;
    
    get currentFile(): string | null {
        return this.activeFiles.length > 0 ? this.activeFiles[0] : null;
    }

    constructor() {
        if (typeof window !== 'undefined') {
            this.initTheme();
        }
    }

    addFiles(paths: string[]) {
        for (const path of paths) {
            if (!this.activeFiles.includes(path)) {
                this.activeFiles.push(path);
            }
        }
    }

    clear() {
        this.activeFiles = [];
    }

    initTheme() {
        if (typeof window === 'undefined') return;

        try {
            const saved = localStorage.getItem('localpdf_theme') as ThemePreference | null;
            if (saved === 'Light' || saved === 'Dark' || saved === 'System') {
                this.themePreference = saved;
            }
        } catch {
            // ignore localStorage quota or security errors
        }

        if (window.matchMedia) {
            this.mediaQueryList = window.matchMedia('(prefers-color-scheme: dark)');
            this.mediaListener = () => {
                if (this.themePreference === 'System') {
                    this.applyTheme();
                }
            };
            if (this.mediaQueryList.addEventListener) {
                this.mediaQueryList.addEventListener('change', this.mediaListener);
            } else if ((this.mediaQueryList as any).addListener) {
                (this.mediaQueryList as any).addListener(this.mediaListener);
            }
        }

        this.applyTheme();
    }

    setTheme(pref: ThemePreference) {
        this.themePreference = pref;
        try {
            localStorage.setItem('localpdf_theme', pref);
        } catch {
            // ignore localStorage errors
        }
        this.applyTheme();
    }

    private applyTheme() {
        if (typeof document === 'undefined') return;

        let resolved: 'light' | 'dark' = 'dark';
        if (this.themePreference === 'Light') {
            resolved = 'light';
        } else if (this.themePreference === 'Dark') {
            resolved = 'dark';
        } else {
            // System preference
            const prefersDark = this.mediaQueryList ? this.mediaQueryList.matches : true;
            resolved = prefersDark ? 'dark' : 'light';
        }

        this.effectiveTheme = resolved;

        const docEl = document.documentElement;
        docEl.setAttribute('data-theme', resolved);
        if (resolved === 'dark') {
            docEl.classList.add('dark');
            docEl.classList.remove('light');
        } else {
            docEl.classList.add('light');
            docEl.classList.remove('dark');
        }
    }
}

export const appState = new AppState();

