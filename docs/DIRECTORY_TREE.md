# Workspace Directory Tree — LocalPDF Studio

This document indexes the complete directory and file layout of LocalPDF Studio, highlighting core engines, spatial studio components, architecture decisions, and audit logs.

```text
localpdf-studio-protocol/
├── AGENTS.md                                   # Boot pointer & agent operational contract
├── ANTIGRAVITY_PROMPTS.md                      # Agent prompt orchestration index
├── ANTIGRAVITY_RUNBOOK.md                      # Single-shot execution runbook
├── Cargo.lock                                  # Pinned Rust dependencies
├── Cargo.toml                                  # Workspace root manifest
├── deny.toml                                   # Supply chain & license policies (MIT / Apache-2.0 only)
├── rust-toolchain.toml                         # Pinned Rust toolchain (1.85.0)
│
├── app-tauri/                                  # Tauri v2 Desktop & Svelte 5 Frontend
│   ├── Cargo.toml                              # Tauri backend crate manifest
│   ├── package.json                            # Frontend npm scripts & dependencies
│   ├── pnpm-lock.yaml                          # Pinned pnpm dependencies
│   ├── svelte.config.js                        # SvelteKit SPA configuration
│   ├── tsconfig.json                           # TypeScript configuration
│   ├── vite.config.js                          # Vite build configuration
│   ├── vitest.config.ts                        # Vitest unit testing configuration
│   ├── vitest-setup.ts                         # Vitest DOM & canvas mocks
│   │
│   ├── src/                                    # Frontend source
│   │   ├── app.css                             # Global CSS styles & base tokens
│   │   ├── app.html                            # HTML template shell
│   │   ├── routes/                             # SvelteKit file routes
│   │   │   ├── +layout.svelte                  # Base application layout & theme provider
│   │   │   └── +page.svelte                    # Main studio canvas route
│   │   │
│   │   └── lib/                                # Core modules & reusable components
│   │       ├── bindings/                       # Type generation bindings (TS definitions)
│   │       │   ├── ControlMessage.d.ts         # Bidirectional control messages
│   │       │   ├── HatchPatternId.d.ts         # Procedural CAD hatching identifiers
│   │       │   ├── SceneNode.d.ts              # Spatial scene graph node definitions
│   │       │   └── ViewTransform.d.ts          # Matrix coordinate transforms
│   │       │
│   │       ├── core/                           # Drawboard Studio Global Svelte 5 Controllers
│   │       │   ├── commandEngine.svelte.ts     # Transactional Undo/Redo & command bus
│   │       │   ├── core.test.ts                # Unit tests for core controllers
│   │       │   ├── defaultPresets.json         # Out-of-the-box pen/highlighter presets
│   │       │   ├── defaultShortcuts.json       # Canonical keyboard shortcut bindings
│   │       │   ├── inkEngine.svelte.ts         # Variable-width spline ink, laser decay & eraser
│   │       │   ├── radialGeometry.ts           # Math for 8-wedge radial hit testing
│   │       │   ├── radialGeometry.test.ts      # Unit tests for radial arc/wedge math
│   │       │   ├── radialLayout.json           # 8-wedge layout & flyout registry
│   │       │   ├── selectionEngine.svelte.ts   # Marquee/lasso selection, group, align, flip, z-order & snapshot
│   │       │   ├── shapeAndHatchEngine.svelte.ts # Geometric shapes, polylines, clouds, hatch & text review
│   │       │   ├── shapeAndSelection.test.ts   # Unit tests for Phase D4 shapes, selection, align & flip
│   │       │   ├── studioReactivity.test.ts    # Unit tests for reactivity, ink, RDP & dock math
│   │       │   ├── themeContract.svelte.ts     # CSS variable cascade (single colour source)
│   │       │   ├── toolEngine.svelte.ts        # Reactive active tool & preset engine
│   │       │   └── viewportEngine.svelte.ts    # Hardware-accelerated DOMMatrix transforms
│   │       │
│   │       ├── components/studio/              # Drawboard-Class Spatial Studio UI
│   │       │   ├── DockableToolbar.svelte      # Frosted glass toolbar (snappable/floating)
│   │       │   ├── FavoritesBar.svelte         # Pinned favorite tool presets pill
│   │       │   ├── HatchDefs.svelte            # Reusable theme-aware CAD hatch SVG <pattern> defs
│   │       │   ├── PropertyPill.svelte         # Floating reactive tool property pill
│   │       │   ├── RadialMenu.svelte           # 8-wedge marking menu with nested flyouts
│   │       │   ├── ShortcutSettings.svelte     # Custom keybinding configuration modal
│   │       │   └── StudioIcon.svelte           # Deterministic vector SVG icon registry
│   │       │
│   │       └── views/studio/                   # Studio Viewports
│   │           ├── SpatialCanvas.svelte        # Hardware-accelerated infinite canvas
│   │           └── SplitLayout.svelte          # Multi-document comparison split view
│   │
│   └── src-tauri/                              # Tauri Rust Backend
│       ├── capabilities/                       # Tauri v2 security capabilities
│       │   ├── default.json                    # Scoped local-only filesystem capabilities
│       │   └── studio-split.json               # Multi-window / split-view local capabilities
│       ├── src/
│       │   ├── commands/                       # Tauri IPC modular command handlers
│       │   │   ├── mod.rs                      # Commands module index
│       │   │   └── studio.rs                   # Atomic prefs_save/prefs_load & control_validate
│       │   ├── lib.rs                          # Tauri application setup, state & IPC commands
│       │   └── main.rs                         # Application entrypoint
│       └── tauri.conf.json                     # Tauri bundle & window configuration
│
├── crates/                                     # Pure Rust Workspace Crates
│   ├── lpdf-ai-worker/                         # Sandboxed local ML inference worker
│   ├── lpdf-cli/                               # Headless command-line utility
│   ├── lpdf-control-bus/                       # Deterministic ControlMessage & replay engine
│   ├── lpdf-core-types/                        # Canonical PDF domain models & errors
│   ├── lpdf-engine-fallback/                   # lopdf fallback parser & repair engine
│   ├── lpdf-engine-pdfcraft/                   # Primary pdfcraft engine adapter
│   ├── lpdf-engine-port/                       # Port traits for engine decoupling
│   ├── lpdf-engine-switcher/                   # Resilient dual-engine runtime router
│   ├── lpdf-fs-safety/                         # Path traversal & atomic write guards
│   ├── lpdf-hw/                                # Hardware tier & memory profiling
│   ├── lpdf-jobs/                              # Resilient crash-isolated worker jobs
│   ├── lpdf-tools-convert/                     # Pure conversion pipelines
│   ├── lpdf-tools-edit/                        # Annotation & structural edit operations
│   ├── lpdf-tools-markdown/                    # Markdown import/export composers
│   ├── lpdf-tools-optimize/                    # Lossless & perceptual stream compression
│   ├── lpdf-tools-organize/                    # Page manipulation & layout tools
│   ├── lpdf-tools-security/                    # AES-256 encryption & redaction verification
│   └── lpdf-worker/                            # Worker process supervisor
│
├── docs/                                       # Architecture & Specifications
│   ├── adr/                                    # Architectural Decision Records
│   │   ├── ADR-011-windows-platform-architecture.md
│   │   └── ADR-012-drawboard-artcraft-spatial-studio.md
│   ├── APPLE_HIG_GLASS_SPEC.md                 # Frosted glass & depth design system
│   ├── ARCHITECTURE_AND_COMPLIANCE_AUDIT.md    # Codebase vs. Opus 5.5 & placeholder audit
│   ├── DIRECTORY_TREE.md                       # Comprehensive directory index (this file)
│   ├── DRAWBOARD_FEATURE_CHECKLIST.md          # 109-feature engineering checklist & tracker
│   ├── DRAWBOARD_STUDIO_MASTER_PLAN.md         # Drawboard-class spatial editor roadmap
│   ├── FEDORA_INSTALL.md                       # Linux RPM installation guide
│   ├── QA_VETTING_CHECKLIST.md                 # Release testing checklist
│   ├── RC1_INSTALL_REPORT.md                   # Installation verification report
│   ├── RELEASE_NOTES_v0.1.0.md                 # v0.1.0 release notes
│   └── WINDOWS_MASTER_PLAN.md                  # Windows native integration roadmap
│
├── logs/                                       # Immutable Protocols & Live Progress Logs
│   ├── CHECKPOINT.md                           # Single source of active phase status
│   ├── DECISIONS_ADR.md                        # ADR status registry
│   ├── ERROR_LOG.md                            # Triaged runtime & build failures
│   ├── PLAN_LOG.md                             # Architectural planning changelog
│   ├── PROGRESS_LOG.md                         # Chronological phase completion log
│   ├── PROMPT_LOG.md                           # Record of user prompts & requirements
│   ├── sbom.cdx.json                           # CycloneDX Software Bill of Materials
│   └── UI_AUDIT_LOG.md                         # UAT defects, root causes & verifications
│
├── protocol/                                   # Non-Negotiable Operational Protocols
│   ├── 01_MASTER_CONTEXT.md                    # Core project context & vision
│   ├── 02_RULES_DOS_DONTS.md                   # Hardcoded rules & constraints
│   ├── 03_FAILSAFES.md                         # System failsafes & test criteria
│   ├── 04_INSTRUCTION_FADE_PREVENTION.md       # Memory retention anchors
│   ├── 05_PHASES.md                            # Development lifecycle phases
│   ├── 06_ARCHITECTURE.md                      # System architecture overview
│   ├── 07_FEATURE_MAP.md                       # Tool feature registry & status
│   ├── 08_DEPENDENCY_VETTING.md                # Crate & package vetting standards
│   ├── 09_DIRECTORY_TREE.md                    # Protocol directory tree specification
│   ├── A11Y_AUDIT.md                           # Accessibility audit rules
│   ├── ENGINE_NOTES.md                         # Engine behavior & API notes
│   ├── ENV.md                                  # Environment & dependency versions
│   └── THREAT_MODEL.md                         # Security threat modeling
│
├── tools/                                      # Developer & Build Tools
│   ├── sb.sh                                   # Sandboxed execution wrapper
│   └── hooks/                                  # Git commit hooks
│
├── vendor/                                     # Pinned Vendored Dependencies
│   └── pdfcraft/                               # Storytold pdfcraft core engine
│
└── xtask/                                      # Cargo Xtask Automation Suite
    └── src/main.rs                             # Automated gates, lints, and checks
```
