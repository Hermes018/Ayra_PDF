import { describe, expect, test } from 'vitest';
import tauriConf from '../../../../src-tauri/tauri.conf.json';

// BUG-012: Organize thumbnails, snapshot stamps and webcam signature previews are data:/blob:
// images; with only `default-src 'self'` the webview blocks them. connect-src must stay IPC-only
// (DON'T-1: no network at runtime).
function directives(csp: string): Map<string, string[]> {
    const out = new Map<string, string[]>();
    for (const part of csp.split(';')) {
        const [name, ...sources] = part.trim().split(/\s+/);
        if (name) out.set(name, sources);
    }
    return out;
}

describe('tauri.conf.json CSP (BUG-012)', () => {
    const csp = directives((tauriConf as any).app.security.csp as string);

    test("img-src allows exactly 'self', data: and blob:", () => {
        expect(csp.get('img-src')).toEqual(["'self'", 'data:', 'blob:']);
    });

    test('connect-src stays IPC-only and default-src stays self', () => {
        expect(csp.get('connect-src')).toEqual(['ipc:', 'http://ipc.localhost']);
        expect(csp.get('default-src')).toEqual(["'self'"]);
    });

    test('no directive allows a remote origin', () => {
        for (const [name, sources] of csp) {
            for (const s of sources) {
                if (name === 'connect-src' && s === 'http://ipc.localhost') continue;
                expect(s, `${name} ${s}`).not.toMatch(/^(https?:|wss?:|\*)/);
            }
        }
    });
});
