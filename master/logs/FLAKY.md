# FLAKY TESTS — a test that passed only on rerun

Never loop on retries. Log the flake here, report it, and fix the cause (or quarantine the test with a reason).

```
## [date] test name (suite: vitest | nextest | e2e flow #)
- Symptom (first failure output, log path in logs/runs/):
- Passed on rerun? y/n — how many attempts:
- Suspected cause:
- Action: fixed (commit) | quarantined (reason) | open
```

## [2026-11-06 06:05] gate step `cargo deny check` (suite: verify.sh / xtask gate)
- Symptom: failed fetching the RustSec advisory DB ("no network") in wt/fixes; no dependency changes in that diff.
- Passed on rerun? y, 1 attempt (`tools/run.sh deny-recheck cargo deny check` → advisories ok, bans ok, licenses ok, sources ok). GitHub reachable at 06:10.
- Suspected cause: transient network blip during the advisory-db fetch (the gate needs the network for this one step).
- Action: open. Proposal: gate fetches the DB; if the fetch fails, run against the cached DB (`--disable-fetch`) and print a WARN with the DB age. Never silently pass with a DB older than 7 days.
- 2026-11-06 | e2e flow 2 | RemoteDisconnected on relaunch | passed on rerun: y, attempts: 1 | cause: the previous instance was still alive and single-instance swallowed the new launch | FIXED in harness (wait for exit before relaunch)
- 2026-11-06 | e2e flow 12 (soak) | RemoteDisconnected at iteration 38, app still alive | not retried, attempts: 1 | suspect tauri-driver | OPEN
