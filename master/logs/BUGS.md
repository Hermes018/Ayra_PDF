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
