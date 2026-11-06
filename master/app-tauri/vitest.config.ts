import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
    plugins: [svelte({ compilerOptions: { dev: true } })],
    resolve: {
        alias: {
            $lib: '/src/lib'
        },
        conditions: ['browser']
    },
    // The version test reads the Cargo manifests one level above the app (read-only).
    server: { fs: { allow: ['..'] } },
    test: {
        environment: 'jsdom',
        setupFiles: ['./vitest-setup.ts'],
        include: ['src/**/*.{test,spec}.{js,ts}']
    },
});
