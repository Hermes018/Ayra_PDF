# 02 — HARDCODED DOs & DON'Ts (exclusive authority; never edit without user approval)

## DO
1. Boot per AGENTS.md; re-anchor per file 04.
2. Verify every external API from source/docs; record in `ENGINE_NOTES.md`.
3. Core crates are pure: `(inputs, options, ctx) -> Result<Output, Error>`. No Tauri/UI imports in core.
4. Write tests with the feature: unit + property (proptest) + golden corpus + hostile-input case.
5. Write outputs atomically to a NEW file by default. Originals are read-only unless the user explicitly picks overwrite (then backup first).
6. Stream / bound memory; obey the hardware-tier memory budget; support cancel at every long step.
7. Use typed errors (thiserror) with stable codes; map to plain-language UI messages; log with `tracing` (local only).
8. Run heavy/untrusted work in jobs on the worker process. Command handlers stay <10 ms.
9. Keep commits small, conventional, gate-green. Tag `phase-X.Y-green`.
10. Update PROGRESS_LOG, PLAN_LOG, PROMPT_LOG, ERROR_LOG, CHECKPOINT as specified. Log BEFORE moving on.
11. Vet every new dependency (file 08) and record it. Prefer own small code over a new dep.
12. Treat `ux-spec/` as the UI source of truth; test with screenshot diffs + e2e.
13. Keep the UI usable by keyboard; honour reduced-motion; support dark/light.
14. Ask the user when blocked (3 attempts) or for irreversible/locked-decision changes.
15. Zeroize passwords/keys in memory after use.
16. Build UI with Svelte 5 per ADR-003 (runes, text-node rendering, pointer-event actions).
17. Keep AI features behind the `ai-local` flag, default OFF; every AI feature has an offline non-AI state (greyed tile or Basic mode).
18. Report approximate conversions honestly: PDF→Office/MD results carry the ADR-006 quality report.
19. Always start a session on a new device with `git pull --recurse-submodules` and end by pushing commits and tags (`git push && git push --tags`).
20. Allow the bundling of heavy external binaries (OpenOffice/ONLYOFFICE/Pandoc) strictly via the Tauri Sidecar API (`tauri::api::process::Command`).
21. Enforce strict shell-injection prevention. All file paths passed to sidecars must be strictly validated, absolute local paths.
22. Structure sidecar execution to capture both `stdout` and `stderr` asynchronously, mapping CLI exit codes to typed Rust errors.
23. Prioritize native `pdfcraft` modules (`pdfcraft_ocr`, `pdfcraft_redact`, `pdfcraft_crypt`, `pdfcraft_sign`) over sidecars.
24. Enforce strict memory bounds and thread safety when invoking local ML inference (`ocrs`/`rten`) for text recognition.

## DON'T
1. No network at runtime: no HTTP clients, telemetry, analytics, update pings, CDN scripts/fonts/images, remote fonts. CSP `connect-src` limited to IPC only.
2. No FastAPI/Flask/any localhost server. No Python/Node in the shipped runtime. No Electron.
3. No `unsafe` outside allow-listed modules (`#![forbid(unsafe_code)]` default; exceptions need an ADR).
4. No `unwrap()/expect()/panic!/todo!/unimplemented!` in non-test code. No panics across IPC/process boundaries.
5. No placeholder/fake implementation reported as done. No "tests pass" without running them. No silently swallowed errors.
6. No base64 for large payloads. No blocking the UI thread/event loop.
7. No page/size/file-count caps. Resource guards may degrade gracefully (slower, chunked) and must tell the user — never silently truncate.
8. No password cracking/brute-forcing/bypass. Unlock = user supplies the password, or removes restrictions on files that need no open-password.
9. No network at runtime. No telemetry. GPL/AGPL permitted ONLY as isolated Tauri sidecar binaries. Max bundle size: 1GB.
10. Never pass raw, unsanitized user input or filenames directly to a sidecar CLI command.
11. No wildcard filesystem scopes (`$HOME/**`, `**`). No broad `shell`/`http` plugin permissions. Access only user-chosen paths.
11. No storing decrypted content, passwords or document text outside RAM; temp files for encrypted docs stay encrypted.
12. No copying iLovePDF code/CSS/assets/branding/copy.
13. No scope creep: anything not in `07_FEATURE_MAP.md` needs PLAN_LOG + ADR + approval.
14. No `git reset --hard` / force-push / history rewrites without approval. No editing protocol files except append-only logs (and CHECKPOINT overwrite).
15. No bumping the pinned pdfcraft commit without: changelog read, full gate, corpus diff, ADR.
16. No ignoring a red gate. Red gate = stop feature work.
17. No v1-era Tauri APIs (`emit_all`, allowlist config). Use v2 capabilities.
18. No `{@html}` of file-derived strings; no SvelteKit/SSR; no frontend `fetch(`/XMLHttpRequest/WebSocket/EventSource.
19. No model weights bundled in installers and none downloaded at runtime; models are user-imported files only.
44. No URL fetching in any tool (HTML to PDF accepts local files/pasted HTML only).
45. No "request signature from others" or timestamp-authority calls in Sign PDF.
46. Never commit, log, or store GitHub Personal Access Tokens (PATs) or credentials.
47. Never use pseudo-redaction (drawing black boxes over text). All redaction must be semantic and destructive at the byte level.
48. Do not leak cryptographic stack traces from `aws-lc-rs` to the frontend.
