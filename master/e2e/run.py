#!/usr/bin/env python3
"""LocalPDF Studio golden-path E2E runner (docs/PROTOTYPE_SCOPE.md, flows 1-12).

    python3 master/e2e/run.py [--flows 1,2] [--binary PATH] [--soak-minutes 60]

Drives the real app through tauri-driver + WebKitWebDriver (W3C WebDriver). The binary must embed
the frontend (built with the `custom-protocol` feature), e.g.
    master/tools/build-lock.sh pnpm --dir app-tauri tauri build --debug --no-bundle
then copy target/debug/localpdf-studio to master/e2e/out/bin/ (the default --binary) so other
builds cannot replace it mid-run. Writes master/e2e/out/report.json and out/runs/<run-id>/screens/*.png (out/screens = latest symlink).
Python standard library only; ground truth from poppler-utils and qpdf.
"""
import argparse
import datetime
import glob
import importlib
import json
import os
import shutil
import signal
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common  # noqa: E402
import wd  # noqa: E402


ALL_FLOWS = list(range(1, 13))  # docs/PROTOTYPE_SCOPE.md: 12 golden-path flows


def discover():
    mods = {}
    for path in sorted(glob.glob(os.path.join(common.E2E_DIR, "flows", "flow[0-9][0-9]_*.py"))):
        name = os.path.splitext(os.path.basename(path))[0]
        mods[int(name[4:6])] = "flows." + name
    return mods


def preflight(binary):
    problems = []
    if not os.path.isfile(binary) or not os.access(binary, os.X_OK):
        problems.append(f"binary not found or not executable: {binary}")
    for tool in ("pdfinfo", "pdftotext", "pdftoppm", "qpdf"):
        if not shutil.which(tool):
            problems.append(f"missing tool: {tool}")
    if not wd.find_tauri_driver():
        problems.append("missing tauri-driver (cargo install tauri-driver --locked)")
    if not os.path.exists("/usr/bin/WebKitWebDriver"):
        problems.append("missing /usr/bin/WebKitWebDriver (apt install webkit2gtk-driver)")
    if not os.environ.get("DISPLAY") and not os.environ.get("WAYLAND_DISPLAY"):
        problems.append("no DISPLAY/WAYLAND_DISPLAY (run under a desktop session or xvfb-run)")
    return problems


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--flows", default="", help="comma list, e.g. 1,2,5 (default: flows 1-12; 13 = optional gallery smoke, needs a VITE_LPDF_GALLERY=1 binary)")
    ap.add_argument("--binary", default=common.DEFAULT_BINARY)
    ap.add_argument("--soak-minutes", type=float, default=60.0, help="flow 12 duration (default 60)")
    ap.add_argument("--no-close", action="store_true", help="flow 12: do not close the tabs each save opens (BUG-061 comparison)")
    ap.add_argument("--idle-minutes", type=float, default=0.0, help="flow 12: after the soak loop idle this long, sampling RSS every 30 s")
    ap.add_argument("--out", default=common.OUT)
    ap.add_argument("--offline", action="store_true",
                    help="run inside a new network namespace with only loopback (unshare -rn)")
    a = ap.parse_args()
    if a.offline and not os.environ.get("LPDF_E2E_NETNS"):
        # Re-exec in a private network namespace: no route off the machine, loopback up for the
        # tauri-driver <-> WebKitWebDriver link. Needs unprivileged user namespaces.
        argv = [x for x in sys.argv[1:] if x != "--offline"] + ["--offline"]
        os.environ["LPDF_E2E_NETNS"] = "1"
        os.execvp("unshare", ["unshare", "-rn", "--", "sh", "-c", 'ip link set lo up && exec "$0" "$@"',
                              sys.executable, os.path.abspath(__file__), *argv])
    network = None
    if a.offline:
        with open("/proc/self/net/dev") as f:  # per-netns view (sysfs shows the mounter's netns)
            ifaces = sorted(ln.split(":")[0].strip() for ln in f.readlines()[2:])
        network = {"isolated": True, "interfaces": ifaces}
        if ifaces != ["lo"]:
            print(f"--offline: expected only 'lo' in the namespace, found {ifaces}", file=sys.stderr)
            return 2

    binary = os.path.abspath(a.binary)
    a.out = os.path.abspath(a.out)  # the app gets absolute file paths, whatever the cwd
    mods = discover()
    try:
        wanted = sorted({int(x) for x in a.flows.split(",") if x.strip()}) if a.flows else ALL_FLOWS
    except ValueError:
        print(f"--flows must be a comma list of flow numbers (1-12 are the required golden paths, 13 is the optional gallery smoke), got {a.flows!r}", file=sys.stderr)
        return 2
    problems = preflight(binary)
    if problems:
        for p in problems:
            print("preflight:", p, file=sys.stderr)
        return 2

    os.makedirs(a.out, exist_ok=True)
    # BUG-031: own session for the whole run, marker env inherited by every child, and cleanup on
    # signals as well as on normal exit/exceptions.
    marker = f"{os.getpid()}-{int(time.time())}"
    os.environ[common.RUN_ENV] = marker
    if not sys.stdin.isatty():  # interactive runs keep the terminal's Ctrl-C; the marker sweep covers both
        try:
            os.setsid()
        except OSError:
            pass  # already a group leader

    def _on_signal(signum, _frame):
        raise SystemExit(128 + signum)  # unwinds through the finally below

    for sg in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(sg, _on_signal)
    started = datetime.datetime.now().astimezone()
    mode = "offline" if a.offline else "online"
    run_id = started.strftime("%Y%m%dT%H%M%S") + f"-{mode}-" + ("all" if not a.flows else "f" + "_".join(map(str, wanted)))
    run_dir = os.path.join(a.out, "runs", run_id)
    os.makedirs(run_dir, exist_ok=True)
    screens_dir = common.prepare_run_screens(a.out, run_id)  # BUG-048: per-run; out/screens = latest symlink
    ctx = common.Context(binary, a.out, a.soak_minutes, screens=screens_dir)
    ctx.no_close, ctx.idle_minutes = a.no_close, a.idle_minutes

    def write(rep_):
        # Each run keeps its own report under out/runs/<run_id>/; <out>/report.json is the latest
        # run for this --out, stamped with run_id, binary, mode and time so it is never ambiguous.
        for path in (os.path.join(run_dir, "report.json"), os.path.join(a.out, "report.json")):
            with open(path, "w") as f:
                json.dump(rep_, f, indent=2)

    report = {"run_id": run_id, "mode": mode, "flows_requested": wanted, "out": a.out,
              "report_path": os.path.join(run_dir, "report.json"),
              "started": started.isoformat(timespec="seconds"),
              "binary": binary, "binary_mtime": datetime.datetime.fromtimestamp(
                  os.path.getmtime(binary)).isoformat(timespec="seconds"),
              "soak_minutes": a.soak_minutes, "network": network or {"isolated": False}, "flows": []}
    t0 = time.monotonic()
    try:
        for n in wanted:
            if n not in mods:
                # An unknown or missing flow is a FAIL, never a silent skip.
                why = (f"no module e2e/flows/flow{n:02d}_*.py" if n in ALL_FLOWS
                       else f"unknown flow {n} (valid: 1-12 required, 13 optional gallery smoke)")
                print(f"== flow {n:02d} FAIL -- {why}", flush=True)
                report["flows"].append({"flow": n, "title": "(missing)", "status": "FAIL", "reason": why,
                                        "duration_s": 0, "warnings": [], "steps": [], "metrics": {},
                                        "screenshots": []})
                continue
            mod = importlib.import_module(mods[n])
            print(f"== flow {n:02d} {mod.TITLE}", flush=True)
            res = common.run_flow(mod, ctx)
            print(f"== flow {n:02d} {res['status']} ({res['duration_s']} s)"
                  + (f" -- {res['reason']}" if res["reason"] else ""), flush=True)
            report["flows"].append(res)
            write(report)
    finally:
        ctx.close()
        leaked, survivors = ctx.post_run_check(marker)
    leak_msg = None
    if leaked or survivors:
        leak_msg = (f"harness processes remained after the run: killed {leaked}, "
                    f"still alive {survivors}")
        print("LEAK:", leak_msg, file=sys.stderr)
        report["leak"] = {"killed": leaked, "survivors": survivors}
    report["duration_s"] = round(time.monotonic() - t0, 1)
    counts = {}
    for r in report["flows"]:
        counts[r["status"]] = counts.get(r["status"], 0) + 1
    report["summary"] = counts
    report["finished"] = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    # Machine-readable flaky list (launch retries): the lead copies these into logs/FLAKY.md.
    flaky = [dict(x, flow=r["flow"]) for r in report["flows"] for x in r.get("metrics", {}).get("flaky", [])]
    report["flaky"] = flaky
    write(report)
    print(f"\nRUN {run_id}  binary={binary}  mode={mode}")
    print("SUMMARY", json.dumps(counts), "->", report["report_path"])
    print("SCREENS", screens_dir, f"(latest view: {os.path.join(a.out, 'screens')})")
    for r in report["flows"]:
        print(f"  {r['flow']:>2} {r['status']:<8} {r['title']}")
    ran = {r["flow"] for r in report["flows"]}
    complete = a.flows or set(ALL_FLOWS) <= ran
    if not complete:
        print(f"INCOMPLETE: a default run must cover flows 1-12; ran {sorted(ran)}", file=sys.stderr)
    report["complete"] = bool(complete)
    write(report)
    if flaky:
        print(f"\nFLAKY ({len(flaky)}): a launch needed a retry; this run cannot count as green.")
        for x in flaky:
            print(f"## [{started.date()}] {x['test']} (suite: e2e flow {x['flow']})\n"
                  f"- Symptom: {x['symptom']}\n- Passed on rerun? {x['passed_on_rerun']} — attempts: {x['attempts']}\n"
                  f"- Suspected cause: {x['suspected_cause']}\n- Action: {x['action']}")
    # Exit 0 only when every requested flow is a full PASS (SHORT-PASS / BLOCKED / FAIL -> 1) and no
    # launch had to be retried.
    all_pass = report["flows"] and all(r["status"] == "PASS" for r in report["flows"])
    return 0 if all_pass and complete and not flaky and not leak_msg else 1


if __name__ == "__main__":
    sys.exit(main())
