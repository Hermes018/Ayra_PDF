import type { ThemePreference } from './bindings/ThemePreference';
import { IpcClient } from './ipc';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface ToastItem {
    id: number;
    kind: ToastType;
    message: string;
    detail?: string;
    timeoutMs: number;
}

class AppState {
    activeFiles = $state<string[]>([]);
    themePreference = $state<ThemePreference>('System');
    effectiveTheme = $state<'light' | 'dark'>('dark');
    toasts = $state<ToastItem[]>([]);

    private mediaQueryList: MediaQueryList | null = null;
    private mediaListener: ((e: MediaQueryListEvent) => void) | null = null;
    private nextToastId = 1;
    private toastTimers = new Map<number, ReturnType<typeof setTimeout>>();
    private jobListenerInitialized = false;

    get currentFile(): string | null {
        return this.activeFiles.length > 0 ? this.activeFiles[0] : null;
    }

    constructor() {
        if (typeof window !== 'undefined') {
            this.initTheme();
            this.initJobListener();
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

    /* ---------- Toast Management ---------- */
    addToast(
        message: string,
        type: ToastType = 'info',
        detail?: string,
        timeoutMs = 6000
    ): number {
        const id = this.nextToastId++;
        const actualTimeout = type === 'error' ? 10000 : timeoutMs;
        const toast: ToastItem = {
            id,
            kind: type,
            message,
            detail,
            timeoutMs: actualTimeout
        };

        this.toasts = [toast, ...this.toasts].slice(0, 5);

        if (actualTimeout > 0 && typeof setTimeout !== 'undefined') {
            const timer = setTimeout(() => {
                this.dismissToast(id);
            }, actualTimeout);
            this.toastTimers.set(id, timer);
        }

        return id;
    }

    dismissToast(id: number) {
        const timer = this.toastTimers.get(id);
        if (timer) {
            clearTimeout(timer);
            this.toastTimers.delete(id);
        }
        this.toasts = this.toasts.filter(t => t.id !== id);
    }

    pauseToast(id: number) {
        const timer = this.toastTimers.get(id);
        if (timer) {
            clearTimeout(timer);
            this.toastTimers.delete(id);
        }
    }

    resumeToast(id: number) {
        const toast = this.toasts.find(t => t.id === id);
        if (toast && toast.timeoutMs > 0 && typeof setTimeout !== 'undefined') {
            const timer = setTimeout(() => {
                this.dismissToast(id);
            }, toast.timeoutMs);
            this.toastTimers.set(id, timer);
        }
    }

    clearToasts() {
        for (const timer of this.toastTimers.values()) {
            clearTimeout(timer);
        }
        this.toastTimers.clear();
        this.toasts = [];
    }

    /* ---------- Job Event Listener ---------- */
    async initJobListener() {
        if (typeof window === 'undefined' || this.jobListenerInitialized) return;
        this.jobListenerInitialized = true;

        try {
            await IpcClient.onJobEvent((event) => {
                if (event.kind === 'Completed') {
                    const out = (event as any)?.data?.output_path;
                    this.addToast(
                        'Operation Completed',
                        'success',
                        out ? `Output: ${out}` : undefined
                    );
                } else if (event.kind === 'Failed') {
                    const err =
                        (event as any)?.data?.error_message ||
                        (event as any)?.data?.error_code ||
                        'Operation failed';
                    this.addToast('Operation Failed', 'error', err);
                } else if (event.kind === 'Cancelled') {
                    this.addToast('Operation Cancelled', 'warning');
                }
            });
        } catch (e) {
            console.error('Failed to register global job event listener:', e);
        }
    }

    /* ---------- Theme Management ---------- */
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

export function addToast(
    message: string,
    type: ToastType = 'info',
    detail?: string,
    timeoutMs?: number
): number {
    return appState.addToast(message, type, detail, timeoutMs);
}
