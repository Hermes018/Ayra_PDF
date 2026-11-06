import type { ThemePreference } from './bindings/ThemePreference';
import { IpcClient } from './ipc';
import { theme, type ThemeMode } from './core/themeContract.svelte';
import { friendlyJobError } from './core/jobError';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface ToastItem {
    id: number;
    kind: ToastType;
    message: string;
    detail?: string;
    timeoutMs: number;
}

export const LABS_STORAGE_KEY = 'localpdf_labs_enabled';

/** Reads a persisted boolean; anything other than a stored `true` (missing, corrupt, wrong type) is `false`. */
function readStoredBoolean(key: string): boolean {
    try {
        const saved = localStorage.getItem(key);
        return saved !== null && JSON.parse(saved) === true;
    } catch {
        return false;
    }
}

class AppState {
    activeFiles = $state<string[]>([]);
    /** BUG-016: index into activeFiles of the document shown (the selected tab). */
    activeIndex = $state<number>(0);
    themePreference = $state<ThemePreference>('System');
    effectiveTheme = $state<'light' | 'dark'>('dark');
    toasts = $state<ToastItem[]>([]);
    preferNativeText = $state<boolean>(false);
    /**
     * Labs (experimental) features: OFF by default. This is the single switch the UI reads;
     * see components/studio/labs.svelte.ts for the list of features it gates.
     */
    labsEnabled = $state<boolean>(false);
    sidebarCollapsed = $state<boolean>(false);

    toggleSidebar(): void {
        this.sidebarCollapsed = !this.sidebarCollapsed;
    }

    private mediaQueryList: MediaQueryList | null = null;
    private mediaListener: ((e: MediaQueryListEvent) => void) | null = null;
    private nextToastId = 1;
    private toastTimers = new Map<number, ReturnType<typeof setTimeout>>();
    private jobListenerInitialized = false;

    get currentFile(): string | null {
        if (this.activeFiles.length === 0) return null;
        return this.activeFiles[Math.min(Math.max(0, this.activeIndex), this.activeFiles.length - 1)];
    }

    constructor() {
        if (typeof window !== 'undefined') {
            this.initTheme();
            this.initJobListener();
            this.initPreferences();
        }
    }

    /** Adds files; the last one given becomes the shown document (opening a file shows it). */
    addFiles(paths: string[]) {
        for (const path of paths) {
            if (!this.activeFiles.includes(path)) {
                this.activeFiles.push(path);
            }
        }
        const last = paths.at(-1);
        if (last !== undefined) this.setActiveFile(last);
    }

    setActiveFile(path: string) {
        const idx = this.activeFiles.indexOf(path);
        if (idx >= 0) this.activeIndex = idx;
    }

    closeFile(path: string) {
        const idx = this.activeFiles.indexOf(path);
        if (idx < 0) return;
        const shown = this.currentFile;
        this.activeFiles = this.activeFiles.filter((p) => p !== path);
        if (shown && shown !== path) {
            this.activeIndex = Math.max(0, this.activeFiles.indexOf(shown));
        } else {
            this.activeIndex = Math.min(idx, Math.max(0, this.activeFiles.length - 1));
        }
    }

    clear() {
        this.activeFiles = [];
        this.activeIndex = 0;
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
            await IpcClient.onJobEvent(async (event) => {
                // Internal jobs (e.g. the open-time GetInfo probe) never toast. Their event can
                // arrive before submit_job has returned the id, so wait for silent submits first.
                await IpcClient.silentSubmitsSettled();
                if (IpcClient.isSilentJob(event.data.job_id)) return;
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
                    // The backend's refusal to overwrite a file is shown in plain words (no raw code or path).
                    const f = friendlyJobError(String(err), { what: 'Operation Failed', next: String(err) }, false);
                    if (String(err).includes('OUTPUT_PATH_UNSAFE')) this.addToast(f.what, 'error', f.next);
                    else this.addToast('Operation Failed', 'error', err);
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
            } else {
                this.themePreference = 'System';
            }
        } catch {
            this.themePreference = 'System';
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
            // System preference failsafe
            const prefersDark = this.mediaQueryList
                ? this.mediaQueryList.matches
                : (typeof window !== 'undefined' && window.matchMedia
                    ? window.matchMedia('(prefers-color-scheme: dark)').matches
                    : true);
            resolved = prefersDark ? 'dark' : 'light';
        }

        this.effectiveTheme = resolved;
        theme.mode = this.themePreference.toLowerCase() as ThemeMode;

        const docEl = document.documentElement;
        const table = theme.tokens;
        for (const key of Object.keys(table) as Array<keyof typeof table>) {
            docEl.style.setProperty(key, table[key]);
        }
        docEl.setAttribute('data-theme', resolved);
        docEl.style.colorScheme = resolved;
        if (resolved === 'dark') {
            docEl.classList.add('dark');
            docEl.classList.remove('light');
        } else {
            docEl.classList.add('light');
            docEl.classList.remove('dark');
        }
        theme.syncWindowVibrancy(resolved);
    }

    /* ---------- User Preferences Persistence ---------- */
    initPreferences() {
        if (typeof window === 'undefined') return;
        try {
            const saved = localStorage.getItem('localpdf_prefer_native_text');
            if (saved !== null) {
                const parsed = JSON.parse(saved);
                this.preferNativeText = typeof parsed === 'boolean' ? parsed : false;
            } else {
                this.preferNativeText = false;
            }
        } catch {
            this.preferNativeText = false;
        }
        this.labsEnabled = readStoredBoolean(LABS_STORAGE_KEY);
    }

    setLabsEnabled(val: boolean) {
        this.labsEnabled = val === true;
        try {
            localStorage.setItem(LABS_STORAGE_KEY, JSON.stringify(this.labsEnabled));
        } catch {
            // Failsafe: ignore localStorage write errors (e.g. private mode/quota)
        }
    }

    setPreferNativeText(val: boolean) {
        this.preferNativeText = val;
        try {
            localStorage.setItem('localpdf_prefer_native_text', JSON.stringify(val));
        } catch {
            // Failsafe: ignore localStorage write errors (e.g. private mode/quota)
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
