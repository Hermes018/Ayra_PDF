import '@testing-library/jest-dom/vitest';

if (typeof window !== 'undefined') {
    (window as any).__TAURI_INTERNALS__ = {
        transformCallback: (fn: any) => fn,
        invoke: () => Promise.resolve(1),
    };
    (window as any).__TAURI_EVENT_PLUGIN_INTERNALS__ = {
        unregisterListener: () => {},
    };
}
