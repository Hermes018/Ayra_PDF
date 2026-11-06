"""Flow 1: install and cold launch.

(a) Packages (.rpm, AppImage, .deb) install -> BLOCKED until packaging lands (checks for bundles).
(b) Cold launch < 3 s: fresh XDG profile, process start (WebDriver new-session request) until
    `document.readyState == 'complete'` and the Studio root (.spatial-viewport) is visible.
(c) Zero console errors: console.error / uncaught errors / unhandled rejections captured by a
    hook installed right after session creation, plus app stdout/stderr (direct launch) and the
    app's own tracing log scanned for panic / ERROR lines.
"""
import glob
import os
import re
import signal
import subprocess
import time

import common

NUM = 1
TITLE = "Install and cold launch"
COLD_LIMIT_S = 3.0
BAD_LINE = re.compile(r"panicked at|\bpanic\b|\bERROR\b|thread '.*' panicked", re.I)


def _targets():
    """CARGO_TARGET_DIR, this checkout's target/, and the shared main-checkout target/ that
    tools/build-lock.sh uses for every worktree."""
    dirs = [os.environ.get("CARGO_TARGET_DIR"), os.path.join(common.MASTER, "target")]
    try:
        common_dir = subprocess.run(["git", "-C", common.MASTER, "rev-parse", "--path-format=absolute",
                                     "--git-common-dir"], capture_output=True, text=True,
                                    timeout=10).stdout.strip()
        if common_dir.endswith("/.git"):
            dirs.append(os.path.join(common_dir[:-5], "master", "target"))
    except (OSError, subprocess.SubprocessError):
        pass
    return [d for d in dict.fromkeys(dirs) if d]


def _bundles():
    found = {}
    for target in _targets():
        for kind, pat in (("rpm", "*.rpm"), ("appimage", "*.AppImage"), ("deb", "*.deb")):
            hits = glob.glob(os.path.join(target, "release", "bundle", kind, pat))
            if hits:
                found.setdefault(kind, []).extend(hits)
    return found


def run(flow):
    # (a) packages. Under e2e/pkg_smoke.py (LPDF_E2E_PACKAGE=<kind>:<file>:<pid>) the binary under test
    # comes from the package itself, launched with a fresh HOME.
    pkg = os.environ.get("LPDF_E2E_PACKAGE")
    if pkg:
        kind, _, rest = pkg.partition(":")
        pfile = rest.rsplit(":", 1)[0]  # value is <kind>:<file>:<smoke pid>
        if kind == "appimage":
            flow.check("AppImage installs for a plain user (copy + chmod +x, no root)",
                       os.access(pfile, os.X_OK), pfile)
        else:
            flow.blocked(f"system install of the .{kind}",
                         f"needs root (sudo {'dnf' if kind == 'rpm' else 'apt'} install); the package "
                         f"was extracted by pkg_smoke.py and its own binary is under test: {pfile}")
    else:
        found = _bundles()
        if not found.get("rpm") or not found.get("appimage"):
            flow.blocked("package install (.rpm + AppImage)",
                         f"no release bundles yet (found: {sorted(found) or 'none'})")
        else:
            flow.blocked("package install (.rpm + AppImage)",
                         f"bundles exist ({sorted(found)}); run e2e/pkg_smoke.py to test them "
                         "(system install needs root)")

    # (b) cold launch into the empty Studio: first attempt only, no retry (timing criterion)
    try:
        app = flow.launch([], retries=0)
    except Exception as e:  # noqa: BLE001 - a failed first launch is a FAIL, never retried
        flow.check("cold launch < 3 s (readyState complete + Studio root visible)", False,
                   f"first launch failed: {type(e).__name__}: {e}"[:400])
        return
    flow.info("hook_installed_at_readyState", app.hooked_at)
    ready = app.ready_s
    visible = app.s.js(common.VISIBLE_JS, ".spatial-viewport")
    flow.info("cold_launch_s", round(ready, 3) if ready else None)
    flow.check("cold launch < 3 s (readyState complete + Studio root visible)",
               ready is not None and ready < COLD_LIMIT_S and visible,
               f"{ready:.2f} s" if ready else "Studio root never became ready")
    empty = app.s.js(common.VISIBLE_JS, '[data-testid="studio-empty-state"]')
    flow.check("empty Studio shows the 'Open a PDF' state", empty, "")
    time.sleep(3)  # let late async work (prefs load, job listener, theme) report errors
    errs = app.errors()
    flow.check("zero console errors after cold launch", not errs, " | ".join(errs)[:500])
    app.quit()

    # warm relaunches for information
    warm = []
    for _ in range(2):
        a2 = flow.launch([])
        warm.append(round(a2.ready_s, 3) if a2.ready_s else None)
        a2.quit()
    flow.info("warm_launch_s", warm)

    # (c) stderr / tracing log: launch the binary directly with the same isolated profile
    drv = flow.driver()
    env = dict(drv.env)
    # Own process group + output to a file: an AppImage runtime does not pass SIGTERM on to the
    # app it starts, and a surviving child would keep a stdout pipe open forever.
    out_path = os.path.join(flow.dir, "direct-launch.out")
    with open(out_path, "wb") as out_f:
        p = subprocess.Popen([flow.ctx.binary], env=env, stdout=out_f, stderr=subprocess.STDOUT,
                             start_new_session=True)
        time.sleep(6)
        alive = p.poll() is None
        for sig in (signal.SIGTERM, signal.SIGKILL):
            try:
                os.killpg(p.pid, sig)
            except ProcessLookupError:
                break
            try:
                p.wait(timeout=5)
                break
            except subprocess.TimeoutExpired:
                continue
    with open(out_path, errors="replace") as f:
        out = f.read()
    flow.check("direct launch stays up for 6 s", alive, f"exit code {p.returncode}")
    bad = [line for line in out.splitlines() if BAD_LINE.search(line)]
    flow.check("app stdout/stderr has no panic/ERROR lines", not bad, " | ".join(bad)[:500])
    logs = glob.glob(os.path.join(drv.profile_dir, "state", "*", "localpdf.log*"))
    bad_log = []
    for lp in logs:
        with open(lp, errors="replace") as f:
            bad_log += [line.strip() for line in f if BAD_LINE.search(line)]
    flow.info("tracing_logs", [os.path.relpath(x, flow.dir) for x in logs])
    if not logs:
        # No log file means the scan proved nothing; never report that as a PASS.
        flow.warn("app tracing log found and free of panic/ERROR lines", False,
                  f"no localpdf.log* under {os.path.join(drv.profile_dir, 'state')}")
    else:
        flow.check("app tracing log has no panic/ERROR lines", not bad_log, " | ".join(bad_log)[:500])
