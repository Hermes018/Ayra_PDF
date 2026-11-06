import { describe, expect, test } from 'vitest';
import pkg from '../../package.json';
import tauriConf from '../../src-tauri/tauri.conf.json';
import workspaceToml from '../../../Cargo.toml?raw';
import appToml from '../../src-tauri/Cargo.toml?raw';
import xtaskToml from '../../../xtask/Cargo.toml?raw';

// Packaging prep (v0.2.0-rc1): one version everywhere. The Cargo workspace version is the source of truth
// (the About screen shows CARGO_PKG_VERSION); package.json and tauri.conf.json must repeat it exactly, and every
// workspace crate must inherit it instead of carrying its own copy.
const crateManifests = import.meta.glob('../../../crates/*/Cargo.toml', {
    query: '?raw',
    import: 'default',
    eager: true,
}) as Record<string, string>;

function section(toml: string, name: string): string {
    const m = toml.match(new RegExp(`^\\[${name.replace('.', '\\.')}\\]\\s*$([\\s\\S]*?)(?=^\\[|(?![\\s\\S]))`, 'm'));
    return m ? m[1] : '';
}

function versionLine(body: string): string | null {
    const m = body.match(/^version\s*=\s*"([^"]+)"/m);
    return m ? m[1] : null;
}

describe('version consistency (v0.2.0-rc1 packaging)', () => {
    const workspaceVersion = versionLine(section(workspaceToml, 'workspace.package'));

    test('the Cargo workspace declares a semver version', () => {
        expect(workspaceVersion).toMatch(/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/);
    });

    test('package.json and tauri.conf.json carry the workspace version', () => {
        expect(pkg.version).toBe(workspaceVersion);
        expect((tauriConf as { version: string }).version).toBe(workspaceVersion);
    });

    test('the app crate and xtask inherit the workspace version', () => {
        expect(section(appToml, 'package')).toMatch(/^version\.workspace\s*=\s*true/m);
        expect(section(xtaskToml, 'package')).toMatch(/^version\.workspace\s*=\s*true/m);
    });

    test('no workspace crate carries its own version', () => {
        const names = Object.keys(crateManifests);
        expect(names.length).toBeGreaterThan(5);
        for (const [file, toml] of Object.entries(crateManifests)) {
            expect(section(toml, 'package'), file).toMatch(/^version\.workspace\s*=\s*true/m);
        }
    });
});
