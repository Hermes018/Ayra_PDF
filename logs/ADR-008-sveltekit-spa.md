# ADR-008: SvelteKit Static SPA Mode Architectural Exception

- **Date:** 2026-10-13
- **Status:** Approved Exception (Retroactive Architectural Reconciliation)
- **Context:**
  Protocol 02 DON'T-18 strictly specifies "no SvelteKit/SSR". However, the application frontend was constructed using `@sveltejs/kit` with `@sveltejs/adapter-static`. We must reconcile protocol specifications with current repository architecture.
- **Decision:**
  Using `@sveltejs/kit` combined with `@sveltejs/adapter-static` (configured with `fallback: "index.html"`) and `ssr = false` in `+layout.ts` is an approved architectural exception. This configuration completely eliminates Server-Side Rendering (SSR) and generates a pure, static client-side Single Page Application (SPA) compatible with Tauri v2 offline execution requirements.
- **Constraints & Invariants:**
  1. `export const ssr = false;` must remain active in root `+layout.ts`.
  2. No server routes (`+server.ts`), endpoints, or Node.js runtime APIs may be used.
  3. All data interactions must occur exclusively through local Tauri IPC commands and events.
- **Approved by:** Staff Software Engineer & Enterprise Security Auditor.
