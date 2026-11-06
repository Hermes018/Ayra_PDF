# RAM BUDGET — agent jobs vs. model training (2026-11-06)

Model training on this machine has **absolute priority**. It must never be pushed into swap or OOM-killed (PROMPT_LOG #20). The rules are enforced by `tools/sched.sh` (admission), `tools/protect.sh` (cgroup cap, OOM preference, nice/ionice) and `tools/run.sh` (the only entry point for heavy commands).

## Machine and training (measured)

| Item | Value |
|---|---|
| MemTotal | 31.1 GiB |
| Swap | 2 GiB, **full** at measurement time (36 kB free), consumed by desktop apps, not training |
| MemAvailable at measurement | ~13.6–15.5 GiB (VS Code + rust-analyzer ~4.5 GiB, Firefox, etc.) |
| Training process | Jupyter ipykernel `swin_v2_training_v5` (PID 2262027) |
| Training RSS / peak | RSS 2.43 GiB, **peak VmHWM 3.16 GiB**; no dataloader worker children seen yet; GPU RTX 4060 Ti 9.1/16 GiB VRAM |
| Training sampler | `tools/sample-training.sh` logs total RSS incl. any workers every 30 s to `logs/runs/training-mem.log` |

## Budget
- `AGENT_BUDGET = MemTotal − training peak − 6 GiB reserve = 31.1 − 3.16 − 6 ≈ 21.9 GiB` (theoretical).
- **Live rule actually enforced:** every admission recomputes from live MemAvailable, because the desktop apps also hold ~10–14 GiB. A HEAVY job is admitted only if MemAvailable ≥ HEAVY peak + 6 GiB; each job's cgroup is capped at MemAvailable − 6 GiB (`protect.sh`).

## Measured peak per job class (`tools/measure-ram.sh`, warm target, CARGO_BUILD_JOBS=2)
Two numbers per job:
- **Whole-cgroup peak** (`memory.peak`): all processes of the job, including page cache charged to the job, so it errs on the high side.
- **Largest single process** (GNU `/usr/bin/time -v`).

| Job | Class | Whole-job peak | Largest process | Time |
|---|---|---|---|---|
| `cargo check -p lpdf-hw` | LIGHT | 299 MiB | 326 MiB | 4.7 s |
| `cargo clippy -p app-tauri --all-targets` | LIGHT | 680 MiB | 579 MiB | 2.6 s |
| `svelte-check` | LIGHT | 670 MiB | 662 MiB | 4.3 s |
| vitest, 1 file | LIGHT | 449 MiB | 301 MiB | 1.3 s |
| vitest, full (--maxWorkers=2) | HEAVY | 925 MiB | 510 MiB | 7.7 s |
| `cargo build -p app-tauri` (cascade from lpdf-core-types) | HEAVY | 2,986 MiB | 1,244 MiB | 11.4 s |
| `cargo xtask gate` (fmt, clippy, nextest, deny, audit, lint) | HEAVY | 3,042 MiB | 1,094 MiB | 26.5 s |
| `cargo nextest run --test-threads 2` | HEAVY | **3,972 MiB** | 1,071 MiB | 17.2 s |
| One E2E run (app + WebKitGTK + tauri-driver) | HEAVY | *pending: measured on the next E2E run in wt/release* | | |

**Caveat:** these are warm, incremental builds. A cold build of a fresh worktree target compiles many crates, but CARGO_BUILD_JOBS=2 still caps it at two rustc processes.

## Scheduler settings (`tools/ram-budget.env`)
- **PEAK_HEAVY_MIB = 5120:** the 3.9 GiB maximum × 1.25 margin. A HEAVY job therefore needs MemAvailable ≥ 11 GiB.
- **Second HEAVY slot:** only if MemAvailable ≥ 2 × 5 + 6 = 16 GiB. At today's ~14–15 GiB available, that means **1 HEAVY slot** in practice; the scheduler decides per admission.
- **LIGHT jobs:** run in parallel freely while MemAvailable > 8 GiB (reserve + 2 GiB).
- **Tight budget:** if MemAvailable < peak + reserve + 2 GiB at admission, the job builds with CARGO_BUILD_JOBS=1.
- **Queue order:** verifier > p0 > e2e > other (`LPDF_PRIO`), FIFO within a priority. Every ADMIT/WAIT/TIGHT/KILLED/DONE decision goes to `logs/runs/scheduler.log` with MemAvailable.

## Protection applied to every agent job (`protect.sh`)
- `systemd-run --user --scope -p MemoryHigh=<cap−2G> -p MemoryMax=<cap> -p MemorySwapMax=0`, with cap = live MemAvailable − 6 GiB (minimum 2 GiB, else refused).
- `choom -n 800`: the kernel's OOM killer picks agent jobs, never training. Training's oom_score_adj stays 0.
- `nice -n 10` and `ionice -c3`.
- NEXTEST_TEST_THREADS=2 and NODE_OPTIONS=--max-old-space-size=3072.
- A job killed by memory (exit 137) is re-queued **once** with both slots (exclusive). If it is killed again, the kill is logged and reported, with no retry loop.

## Footprint settings
- **In place:** dev profile `debug = "line-tables-only"` with deps `debug = 0`; no LTO in dev; mold; sccache (10 GB disk cap, server kept running); `cargo check -p` / `clippy -p` while iterating; vitest `--maxWorkers=2`; svelte-check once per change set.
- **Deferred until after the Fri 08:00 milestone:** `split-debuginfo = "unpacked"`. Changing the profile forces a cold rebuild in all four worktrees mid-sprint, and the measured link peaks are already modest (≤ 3 GiB whole-job).
- **E2E:** one app instance at a time; kill the app, WebKitWebDriver and tauri-driver after every flow (release builder).

## Incident fixed while setting this up
The sccache server daemon inherited the flock file descriptor and held the single build lock indefinitely, blocking all builds for about 8 minutes. Fixed in two ways:
- `flock -o` closes the lock fd in the child (the scheduler likewise closes slot fds for the job's child);
- `run.sh` starts sccache outside any lock.

## Recompute 2026-11-06 08:05 (owner: switch to 2 HEAVY slots if 2× peak fits)
- Budget = MemTotal 31,859 MiB − training peak (kernel hwm) 3,233 MiB − reserve 6,144 MiB = **22,482 MiB**.
- Measured HEAVY peak: nextest-full 3,972 MiB (others: gate 3,042, cascade build 2,986, vitest-full 925). 2 × 3,972 = 7,944 MiB ≤ 22,482 MiB → fits.
- **Decision: 2 HEAVY slots.** PEAK_HEAVY_MIB 5,120 → 4,096 (the measured max rounded up; the 1.25 margin is dropped). The live rule is unchanged: slot 2 opens only when MemAvailable ≥ 2 × 4,096 + 6,144 = 14,336 MiB (it was 16,384, which never opened at ~16.0 GiB available). The per-job cgroup cap is unchanged.
- E2E peak is still unmeasured, so `LPDF_PRIO=e2e` jobs take both slots (sched.sh) until it is measured.
- Live test 08:04: two concurrent HEAVY jobs admitted on separate slots at 16,033 MiB available.
