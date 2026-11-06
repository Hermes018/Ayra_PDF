import '@testing-library/jest-dom/vitest';

if (typeof window !== 'undefined') {
    (window as any).__TAURI_INTERNALS__ = {
        transformCallback: (fn: any) => fn,
        // Output-name checks (outputNames.pathExistsOnDisk) see an empty disk unless a test says otherwise.
        invoke: (cmd: string) =>
            cmd === 'path_exists'
                ? Promise.resolve(false)
                : cmd === 'read_file_bytes'
                  ? Promise.reject(new Error('No such file or directory (os error 2)'))
                  : Promise.resolve(1),
    };
    (window as any).__TAURI_EVENT_PLUGIN_INTERNALS__ = {
        unregisterListener: () => {},
    };
}
