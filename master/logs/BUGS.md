# BUGS (sprint v0.2.0-prototype) — triage owned by the lead

Sources: the E2E baseline and runs, builder reports, the adversarial tester and reviews. Triage rules are in `docs/PROTOTYPE_SCOPE.md`: **P0** breaks a main flow or loses/corrupts data; **P1** visible, with a workaround; **P2** after the release.

Entry format:
```
## BUG-NNN [P0|P1|P2] short title — status: open|fixing|fixed|deferred (KNOWN_ISSUES)
- Flow: # (PROTOTYPE_SCOPE)   Found by: e2e | builder | tester | owner   Date:
- Repro:
- Expected / actual:
- Owner (worktree): engines | fixes | release
- Fix commit / test:
```

## BUG-001 [P2] Math-text note omits the "∑ " prefix — status: open
- Flow: — (Labs only)   Found by: reviewer   Date: 2026-11-05
- Repro: Labs on → insert Math text: the text always starts with "∑ " (insertEngine.svelte.ts:612); the note in labs.svelte.ts:28 doesn't say so.
- Owner: fixes

## BUG-002 [P2] Undo label "Handwriting Ink-to-Text" overclaims — status: open
- Flow: — (Labs only)   Found by: reviewer   Date: 2026-11-05
- Repro: Labs on → Shape-to-character guess → undo tooltip (SpatialCanvas.svelte:1528) shows "Handwriting Ink-to-Text" (insertEngine.svelte.ts:1106).
- Owner: engines (core/)

## BUG-003 [P2] "Reset to defaults" leaves parked Labs shortcut bindings — status: open
- Found by: reviewer   Date: 2026-11-05   Repro: Labs off → Shortcut settings → Reset; turn Labs on → the old Labs chord is back (ShortcutSettings.svelte:59, labs.svelte.ts:135).
- Owner: fixes

## BUG-004 [P2] Ink-to-text fallback can replace a stamp/signature, copy says "ink strokes" — status: open
- Found by: reviewer   Date: 2026-11-05   Where: insertEngine.svelte.ts:1068 (last stroke of any kind).
- Owner: engines (core/) for behaviour, fixes for copy

## BUG-005 [P2] Labs honest note only in a title attribute (a11y) — status: open
- Found by: reviewer   Date: 2026-11-05   Where: DockableToolbar.svelte:377 — screen readers may not announce title.
- Owner: fixes

## BUG-006 [P2] Code comments reference `./labs.ts`, file is `labs.svelte.ts`; empty-stroke selection yields "Ink Note" without colon — status: open
- Found by: verifier   Date: 2026-11-05
- Owner: fixes

## BUG-007 [P2] Labs tools not refused at the core level (toolEngine.selectTool / commandEngine) — status: open (reviewer: sufficient for release)
- Found by: fixes builder + reviewer   Date: 2026-11-05   All UI call sites are guarded and a root effect resets to Pan; a core-level refusal belongs in core/ (engines).
- Owner: engines

## BUG-008 [P2] Studio prefs never keep the toolbar dock position — status: open
- Found by: fixes builder   Where: SpatialCanvas.svelte persistStudioPrefs always saves dockEdge 'left'; loadStudioPrefs ignores it.
- Owner: fixes

## BUG-009 [P2] PropertyPill insert picker hidden for note/stamp/signature/image tools — status: open
- Found by: fixes builder   Where: PropertyPill.svelte:450 inside the stroke-properties branch (line 169).
- Owner: fixes

## BUG-010 [P1] toolEngine label for insert.latex still "LaTeX Math Block" — status: open
- Found by: fixes builder   Where: core/toolEngine.svelte.ts:131 (overridden at display sites; fix at source).
- Owner: engines
