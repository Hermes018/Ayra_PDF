"""Shared E2E plumbing: paths, independent ground-truth tools (poppler / qpdf / hashes), the flow
result model, an isolated session bus, and the `App` wrapper that drives LocalPDF Studio.

Ground truth never comes from the app: page counts from `pdfinfo`, text from `pdftotext`,
structure from `qpdf --check` / `qpdf --json`, "original untouched" from SHA-256.
"""
import hashlib
import json
import os
import re
import select
import shutil
import signal
import subprocess
import tempfile
import time
import traceback

import wd

E2E_DIR = os.path.dirname(os.path.abspath(__file__))
MASTER = os.path.dirname(E2E_DIR)
CORPUS = os.path.join(MASTER, "corpus")
OUT = os.path.join(E2E_DIR, "out")
DEFAULT_BINARY = os.path.join(OUT, "bin", "localpdf-studio")
ENC_PASSWORD = "lpdf-test"
PT_TO_CSS = 96 / 72


# ---------------------------------------------------------------------------------------------
# Ground truth helpers
# ---------------------------------------------------------------------------------------------
def run(cmd, timeout=120):
    p = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    return p.returncode, p.stdout, p.stderr


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def pdfinfo(path, password=None, first=None, last=None):
    cmd = ["pdfinfo"]
    if password:
        cmd += ["-upw", password]
    if first:
        cmd += ["-f", str(first), "-l", str(last or first)]
    cmd.append(path)
    rc, out, err = run(cmd)
    info = {"_rc": rc, "_err": err.strip()}
    for line in out.splitlines():
        if ":" in line:
            k, v = line.split(":", 1)
            info[k.strip()] = v.strip()
    return info


def pdf_pages(path, password=None):
    v = pdfinfo(path, password).get("Pages")
    return int(v) if v and v.isdigit() else None


def page_rotations(path, n_pages):
    """Per-page /Rotate as reported by `pdfinfo -f 1 -l N` ("Page    1 rot:  90")."""
    rc, out, _ = run(["pdfinfo", "-f", "1", "-l", str(n_pages), path])
    rots = {}
    for m in re.finditer(r"^Page\s+(\d+)\s+rot:\s+(-?\d+)", out, re.M):
        rots[int(m.group(1))] = int(m.group(2))
    return [rots.get(i + 1) for i in range(n_pages)]


def pdftotext(path, first=None, last=None, password=None):
    cmd = ["pdftotext", "-q", "-layout"]
    if password:
        cmd += ["-upw", password]
    if first:
        cmd += ["-f", str(first), "-l", str(last or first)]
    cmd += [path, "-"]
    rc, out, _ = run(cmd)
    return out if rc == 0 else None


def pdftotext_words(path, page=1):
    """Words with bboxes in PDF points from the top-left: [(text, xMin, yMin, xMax, yMax)]."""
    rc, out, _ = run(["pdftotext", "-q", "-bbox", "-f", str(page), "-l", str(page), path, "-"])
    words = []
    for m in re.finditer(r'<word xMin="([\d.]+)" yMin="([\d.]+)" xMax="([\d.]+)" yMax="([\d.]+)">'
                         r'([^<]*)</word>', out):
        words.append((m.group(5), float(m.group(1)), float(m.group(2)), float(m.group(3)),
                      float(m.group(4))))
    return words


def qpdf_check(path, password=None):
    cmd = ["qpdf", "--check"]
    if password:
        cmd.append(f"--password={password}")
    cmd.append(path)
    rc, out, err = run(cmd)
    # qpdf exit codes: 0 ok, 3 warnings (still readable), 2 errors.
    return rc in (0, 3) and "error" not in err.lower(), f"rc={rc} {(out + err).strip()[-400:]}"


def qpdf_objects(path):
    rc, out, err = run(["qpdf", "--json", "--json-stream-data=none", path], timeout=300)
    if rc not in (0, 3):
        raise RuntimeError(f"qpdf --json failed: {err[-300:]}")
    data = json.loads(out)
    objs = {}
    for k, v in data["qpdf"][1].items():
        if k.startswith("obj:"):
            objs[k[4:]] = v.get("value", v.get("stream", {}).get("dict"))
    pages = [p["object"] for p in data.get("pages", [])]
    return objs, pages


def page_annotations(path):
    """List (per page) of resolved annotation dicts, using qpdf JSON (independent of the app)."""
    objs, pages = qpdf_objects(path)

    def res(v):
        return objs.get(v, v) if isinstance(v, str) and v.endswith(" R") else v

    out = []
    for pref in pages:
        page = objs.get(pref) or {}
        annots = res(page.get("/Annots")) or []
        out.append([res(a) for a in annots if isinstance(res(a), dict)])
    return out


def pdf_string(v):
    if isinstance(v, str) and v.startswith("u:"):
        return v[2:]
    if isinstance(v, str) and v.startswith("b:"):
        try:
            return bytes.fromhex(v[2:]).decode("latin-1")
        except ValueError:
            return v
    return v


def point_in_polygon(x, y, poly):
    inside = False
    n = len(poly)
    for i in range(n):
        x1, y1 = poly[i]
        x2, y2 = poly[(i + 1) % n]
        if (y1 > y) != (y2 > y):
            xin = x1 + (y - y1) * (x2 - x1) / (y2 - y1)
            if x < xin:
                inside = not inside
    return inside


def dist_point_segment(px, py, a, b):
    ax, ay = a
    bx, by = b
    dx, dy = bx - ax, by - ay
    L2 = dx * dx + dy * dy
    t = 0.0 if L2 == 0 else max(0.0, min(1.0, ((px - ax) * dx + (py - ay) * dy) / L2))
    return ((px - ax - t * dx) ** 2 + (py - ay - t * dy) ** 2) ** 0.5


def near_polygon_edge(x, y, poly, tol):
    return any(dist_point_segment(x, y, poly[i], poly[(i + 1) % len(poly)]) <= tol
               for i in range(len(poly)))


# ---------------------------------------------------------------------------------------------
# Process inspection (RSS / file descriptors) for the soak flow
# ---------------------------------------------------------------------------------------------
_NOT_APP = {"tauri-driver", "WebKitWebDriver", "WebKitWebProcess", "WebKitNetworkProcess",
            "WebKitGPUProcess", "dbus-daemon", "bwrap", "xdg-dbus-proxy", "python3"}


def find_app_pids(binary, marker):
    """App PIDs: processes whose exe is `binary`, or (for an AppImage, which runs from an extracted
    temp path) any non-driver, non-WebKit process carrying `marker` (the per-flow XDG_CONFIG_HOME)
    in its environment; plus all their descendants (WebKit web/network processes)."""
    real = os.path.realpath(binary)
    roots, parents = [], {}
    for d in os.listdir("/proc"):
        if not d.isdigit():
            continue
        pid = int(d)
        try:
            with open(f"/proc/{pid}/stat") as f:
                parents[pid] = int(f.read().rsplit(")", 1)[1].split()[1])
            exe = os.path.realpath(f"/proc/{pid}/exe")
            if exe == real or os.path.basename(exe) not in _NOT_APP:
                with open(f"/proc/{pid}/environ", "rb") as f:
                    if marker.encode() in f.read():
                        roots.append(pid)
        except (OSError, IndexError, ValueError):
            continue
    found = set(roots)
    changed = True
    while changed:
        changed = False
        for pid, ppid in parents.items():
            if ppid in found and pid not in found:
                found.add(pid)
                changed = True
    return roots, sorted(found)


def _is_zombie(pid):
    try:
        with open(f"/proc/{pid}/stat") as f:
            return f.read().rsplit(")", 1)[1].split()[0] == "Z"
    except (OSError, IndexError):
        return True


def rss_kb(pids):
    total = 0
    for pid in pids:
        try:
            with open(f"/proc/{pid}/status") as f:
                for line in f:
                    if line.startswith("VmRSS:"):
                        total += int(line.split()[1])
        except OSError:
            pass
    return total


def fd_count(pids):
    total = 0
    for pid in pids:
        try:
            total += len(os.listdir(f"/proc/{pid}/fd"))
        except OSError:
            pass
    return total


# ---------------------------------------------------------------------------------------------
# Isolated session bus (so tauri-plugin-single-instance never forwards our launch to a copy of
# the app the user already has open, and so the "open in running instance" path is testable).
# ---------------------------------------------------------------------------------------------
_BUS_CONF = """<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <type>session</type>
  <listen>unix:dir={dir}</listen>
  <auth>EXTERNAL</auth>
  <policy context="default">
    <allow send_destination="*" eavesdrop="true"/>
    <allow eavesdrop="true"/>
    <allow own="*"/>
  </policy>
</busconfig>
"""


class PrivateBus:
    def __init__(self, workdir):
        self.proc = None
        self.address = None
        exe = shutil.which("dbus-daemon")
        if not exe:
            # No silent fallback to the user's bus: single-instance would forward our launches
            # to any copy of the app the user has open.
            raise RuntimeError("dbus-daemon not found: the E2E run needs a private session bus "
                               "(apt install dbus-daemon)")
        # Short path in /tmp: a unix socket path must stay under ~108 bytes, and --out may be deep.
        d = self.dir = tempfile.mkdtemp(prefix="lpdf-bus-", dir="/tmp")
        conf = os.path.join(d, "bus.conf")
        with open(conf, "w") as f:
            f.write(_BUS_CONF.format(dir=d))
        # Own session/process group (BUG-031): stop() kills the whole group, never just the leader.
        self.proc = subprocess.Popen([exe, f"--config-file={conf}", "--nofork",
                                      "--print-address=1"], stdout=subprocess.PIPE,
                                     stderr=subprocess.DEVNULL, text=True, start_new_session=True)
        ready, _, _ = select.select([self.proc.stdout], [], [], 10)
        self.address = self.proc.stdout.readline().strip() if ready else None
        if not self.address:
            self.stop()
            raise RuntimeError("private dbus-daemon did not print its address within 10 s")

    def stop(self):
        if self.proc:
            for sig in (signal.SIGTERM, signal.SIGKILL):
                try:
                    os.killpg(self.proc.pid, sig)  # the group leader is the daemon (setsid)
                except OSError:
                    pass
                try:
                    self.proc.wait(timeout=3)
                    break
                except subprocess.TimeoutExpired:
                    continue
        if getattr(self, "dir", None):
            shutil.rmtree(self.dir, ignore_errors=True)


# ---------------------------------------------------------------------------------------------
# Harness-process sweep (BUG-031): every process of a run carries LPDF_E2E_RUN=<id> in its
# environment (inherited from run.py) and the private bus is identified by its /tmp/lpdf-bus-*
# config path. After a run, anything still alive fails the run.
# ---------------------------------------------------------------------------------------------
RUN_ENV = "LPDF_E2E_RUN"


def harness_procs(marker, bus_dir=None):
    """PIDs (excluding this process, zombies) whose environ holds RUN_ENV=<marker> or whose
    command line names `bus_dir`."""
    me, found = os.getpid(), []
    needle = f"{RUN_ENV}={marker}".encode()
    for d in os.listdir("/proc"):
        if not d.isdigit() or int(d) == me:
            continue
        pid = int(d)
        try:
            if _is_zombie(pid):
                continue
            with open(f"/proc/{pid}/environ", "rb") as f:
                hit = needle in f.read().split(b"\0")
            if not hit and bus_dir:
                with open(f"/proc/{pid}/cmdline", "rb") as f:
                    hit = bus_dir.encode() in f.read()
            if hit:
                found.append(pid)
        except OSError:
            continue
    return found


def kill_harness(marker, bus_dir=None, wait=5):
    """SIGKILL every harness process; returns (killed, survivors)."""
    killed = []
    deadline = time.time() + wait
    while True:
        pids = harness_procs(marker, bus_dir)
        if not pids or time.time() > deadline:
            return killed, pids
        for p in pids:
            try:
                os.kill(p, signal.SIGKILL)
                killed.append(p)
            except OSError:
                pass
        time.sleep(0.2)


# ---------------------------------------------------------------------------------------------
# Flow result model
# ---------------------------------------------------------------------------------------------
class Blocked(Exception):
    pass


class Context:
    def __init__(self, binary, out_dir, soak_minutes):
        self.binary = binary
        self.out = out_dir
        self.soak_minutes = soak_minutes
        self.screens = os.path.join(out_dir, "screens")
        self.work = os.path.join(out_dir, "work")
        os.makedirs(self.screens, exist_ok=True)
        os.makedirs(self.work, exist_ok=True)
        self.bus = PrivateBus(self.work)

    def close(self):
        self.bus.stop()

    def post_run_check(self, marker):
        """Fail-the-run check: returns (leaked_pids, survivors) after killing any leftovers."""
        return kill_harness(marker, getattr(self.bus, "dir", None))


class Flow:
    """One flow run. Use `check()` for every PASS criterion; any False check makes the flow FAIL,
    `blocked()` marks a criterion that cannot be exercised yet (BLOCKED unless something FAILed)."""

    def __init__(self, ctx, num, title):
        self.ctx, self.num, self.title = ctx, num, title
        self.steps = []
        self.metrics = {}
        self.screens = []
        self.app = None
        self.apps = []
        self.dir = os.path.join(ctx.work, f"flow{num:02d}")
        shutil.rmtree(self.dir, ignore_errors=True)
        os.makedirs(self.dir)
        self.originals = {}  # workdir copy -> sha256 at copy time
        self._driver = None
        self._napp = 0
        self.short = None  # set to a reason when a PASS must be reported as SHORT-PASS

    # ---- recording ----------------------------------------------------------------------
    def check(self, name, ok, detail=""):
        ok = bool(ok)
        step = {"step": name, "result": "PASS" if ok else "FAIL", "detail": str(detail)[:600]}
        if not ok:
            shot = self.screenshot(name)
            if shot:
                step["screenshot"] = shot
        self.steps.append(step)
        print(f"    [{step['result']}] {name}{': ' + step['detail'] if step['detail'] else ''}",
              flush=True)
        return ok

    def warn(self, name, ok, detail=""):
        """A non-criterion observation (e.g. console errors outside flow 1): listed as WARN in the
        report but does not change the flow's status."""
        if ok:
            return self.check(name, True, detail)
        self.steps.append({"step": name, "result": "WARN", "detail": str(detail)[:600]})
        print(f"    [WARN] {name}: {str(detail)[:600]}", flush=True)
        return False

    def blocked(self, name, reason):
        self.steps.append({"step": name, "result": "BLOCKED", "detail": reason})
        print(f"    [BLOCKED] {name}: {reason}", flush=True)

    def info(self, key, value):
        self.metrics[key] = value

    def screenshot(self, name):
        app = self.app
        if not app or not app.alive():
            return None
        safe = re.sub(r"[^A-Za-z0-9_.-]+", "_", name)[:60]
        path = os.path.join(self.ctx.screens, f"flow{self.num:02d}_{len(self.screens):02d}_{safe}.png")
        try:
            app.s.screenshot(path)
        except Exception:  # noqa: BLE001 - a screenshot must never mask the real failure
            return None
        self.screens.append(path)
        return os.path.relpath(path, self.ctx.out)

    # ---- files --------------------------------------------------------------------------
    def corpus_copy(self, name, as_name=None):
        src = os.path.join(CORPUS, name)
        if not os.path.exists(src):
            raise Blocked(f"corpus file missing: {src} (run sh corpus/fetch.sh)")
        dst = os.path.join(self.dir, as_name or name)
        shutil.copyfile(src, dst)
        self.originals[dst] = sha256(dst)
        return dst

    def original_unchanged(self, path):
        return sha256(path) == self.originals[path]

    # ---- app ----------------------------------------------------------------------------
    def driver(self, env=None):
        """The flow's tauri-driver (started on first use). `env` adds variables for the app; it
        only applies if the driver is not running yet."""
        if self._driver is None:
            extra = {"NO_AT_BRIDGE": "1", **(env or {})}
            if self.ctx.bus.address:
                extra["DBUS_SESSION_BUS_ADDRESS"] = self.ctx.bus.address
            self._driver = wd.Driver(os.path.join(self.dir, "profile"),
                                     os.path.join(self.dir, "tauri-driver.log"), extra)
        return self._driver

    def launch(self, files=(), expect_pages=None, timeout=60, retries=1):
        """Start the app. `retries` > 0 allows ONE relaunch after a WebDriver session failure;
        each retry is recorded as a WARN step (and a FLAKY note in the report), and the orphan
        instance is killed first. Timing-critical launches (flow 1) pass retries=0."""
        self._napp += 1
        app = App(self, list(files), expect_pages=expect_pages, timeout=timeout, retries=retries)
        self.app = app
        self.apps.append(app)
        return app

    def close(self):
        for a in self.apps:
            a.quit()
        if self._driver:
            self._driver.stop()
            self._driver = None

    def leftovers(self):
        """PIDs of any process (app, WebKit children, WebKitWebDriver, tauri-driver) still carrying
        this flow's private profile in its environment."""
        marker = os.path.join(self.dir, "profile", "config").encode()
        found = []
        for d in os.listdir("/proc"):
            if not d.isdigit() or int(d) == os.getpid():
                continue
            try:
                with open(f"/proc/{d}/environ", "rb") as f:
                    if marker in f.read():
                        found.append(int(d))
            except OSError:
                continue
        return found

    def reap(self, wait=10):
        """Kill everything left from this flow and verify nothing survives (RAM rule: one app
        instance at a time). Returns the PIDs that had to be killed."""
        killed = self.leftovers()
        for p in killed:
            try:
                os.kill(p, signal.SIGKILL)
            except OSError:
                pass
        deadline = time.time() + wait
        while time.time() < deadline:
            alive = [p for p in self.leftovers() if not _is_zombie(p)]
            if not alive:
                return killed
            time.sleep(0.2)
        raise RuntimeError(f"flow {self.num}: processes survive SIGKILL: {self.leftovers()}")

    def result(self, duration):
        res = [s["result"] for s in self.steps]
        if "FAIL" in res:
            status = "FAIL"
        elif "BLOCKED" in res:
            status = "BLOCKED"
        elif res:
            status = "PASS"
        else:
            status = "FAIL"
        if status == "PASS" and self.short:
            status = "SHORT-PASS"
        failing = [s for s in self.steps if s["result"] == status and status != "PASS"]
        warnings = [f"{s['step']}: {s['detail']}"[:300] for s in self.steps if s["result"] == "WARN"]
        reason = "; ".join(f"{s['step']}: {s['detail']}" for s in failing)[:900]
        if status == "SHORT-PASS":
            reason = self.short
        return {"flow": self.num, "title": self.title, "status": status, "reason": reason,
                "duration_s": round(duration, 1), "warnings": warnings, "steps": self.steps, "metrics": self.metrics,
                "screenshots": [os.path.relpath(p, self.ctx.out) for p in self.screens]}


# ---------------------------------------------------------------------------------------------
# The app under test
# ---------------------------------------------------------------------------------------------
HOOK_JS = r"""
if (!window.__e2e) {
  const E = window.__e2e = { errors: [], warns: [], hookedAt: document.readyState };
  const oe = console.error.bind(console), ow = console.warn.bind(console);
  const fmt = (a) => a.map((x) => { try {
      if (x && typeof x === 'object' && 'message' in x) return (x.name || 'Error') + ': ' + x.message;
      return typeof x === 'object' ? JSON.stringify(x) : String(x); } catch (e) { return String(x); } }).join(' ');
  console.error = (...a) => { E.errors.push(fmt(a)); oe(...a); };
  console.warn = (...a) => { E.warns.push(fmt(a)); ow(...a); };
  window.addEventListener('error', (e) => E.errors.push('uncaught: ' + (e.message || e)));
  window.addEventListener('unhandledrejection', (e) => E.errors.push('unhandledrejection: ' + String(e.reason)));
  // window.alert would block the page modally; record the message (it is user-visible text).
  E.alerts = [];
  window.alert = (m) => { E.alerts.push(String(m)); };
  // CSP violations are logged by WebKit as console errors but never reach console.error.
  document.addEventListener('securitypolicyviolation', (e) => E.errors.push('CSP blocked ' +
      e.violatedDirective + ': ' + String(e.blockedURI).slice(0, 60)));
}
return window.__e2e.hookedAt;
"""

# Visible in the current view: element exists, has a box, and is not display:none.
VISIBLE_JS = """
const e = document.querySelector(arguments[0]); if (!e) return false;
const r = e.getBoundingClientRect(); return r.width > 0 && r.height > 0;
"""


class App:
    def __init__(self, flow, files, expect_pages=None, timeout=60, retries=1):
        self.flow = flow
        self.files = files
        self.marker = os.path.join(flow.dir, "profile", "config")
        drv = flow.driver()
        for attempt in range(retries + 1):
            self.t0 = time.monotonic()
            try:
                self.s = wd.Session(drv, flow.ctx.binary, files, timeout=max(120, timeout))
                break
            except (wd.WebDriverError, OSError) as e:
                if attempt == retries:
                    raise
                # One relaunch at most; kill the orphan instance first so a lingering app
                # cannot swallow the relaunch (single-instance), and make the retry visible.
                orphans = [p for p in flow.leftovers()
                           if os.path.realpath(f"/proc/{p}/exe") == os.path.realpath(flow.ctx.binary)]
                for p in orphans:
                    try:
                        os.kill(p, signal.SIGKILL)
                    except OSError:
                        pass
                note = (f"launch {os.path.basename(files[0]) if files else '(no file)'}: "
                        f"{type(e).__name__}: {e}"[:200])
                flow.warn("app launch needed a retry (flaky)", False,
                          f"{note}; killed orphan pids {orphans}")
                flow.metrics.setdefault("flaky", []).append(
                    {"test": f"e2e flow {flow.num}", "symptom": note, "passed_on_rerun": "pending",
                     "attempts": attempt + 2, "suspected_cause": "tauri-driver dropped the new-session "
                     "request right after the previous session ended", "action": "open"})
                time.sleep(2)
        if flow.metrics.get("flaky") and attempt > 0:
            flow.metrics["flaky"][-1]["passed_on_rerun"] = "y"
        self.hooked_at = self.s.js(HOOK_JS)
        self.s.set_window(1200, 800)
        self.ready_s = None
        ok = self.s.wait("return document.readyState === 'complete' && "
                         "!!document.querySelector('.spatial-viewport')", timeout, 0.02)
        if ok:
            self.ready_s = time.monotonic() - self.t0
        self.doc_s = None
        if expect_pages:
            self.wait_doc(expect_pages, timeout)

    # ---- lifecycle ----------------------------------------------------------------------
    def alive(self):
        if not self.s.base:
            return False
        try:
            self.s.js("return 1")
            return True
        except Exception:  # noqa: BLE001
            return False

    def quit(self, wait=15):
        """End the session and wait until the app process is really gone: with
        tauri-plugin-single-instance a lingering instance would swallow the next launch."""
        try:
            roots, _ = self.pids()
        except OSError:
            roots = []
        self.s.quit()
        deadline = time.time() + wait
        while roots and time.time() < deadline:
            roots = [p for p in roots if os.path.exists(f"/proc/{p}") and not _is_zombie(p)]
            if roots:
                time.sleep(0.1)
        for p in roots:
            try:
                os.kill(p, 9)
            except OSError:
                pass
        if roots:
            time.sleep(0.5)

    def alerts(self):
        return self.s.js("return window.__e2e ? window.__e2e.alerts.slice() : []") or []

    def pids(self):
        return find_app_pids(self.flow.ctx.binary, self.marker)

    def errors(self):
        try:
            return self.s.js("return window.__e2e ? window.__e2e.errors.slice() : null") or []
        except Exception:  # noqa: BLE001
            return ["<could not read error log: webview gone>"]

    # ---- document -----------------------------------------------------------------------
    def wait_doc(self, pages, timeout=60):
        """Wait until the Studio laid out `pages` pages and the first page canvas has pixels."""
        ok = self.s.wait("""
            const n = document.querySelectorAll('.spatial-page').length;
            if (n !== arguments[0]) return false;
            const c = document.querySelector('.spatial-page canvas'); return !!c && c.width > 0;
        """, timeout, 0.1, args=(pages,))
        if ok and self.doc_s is None:
            self.doc_s = time.monotonic() - self.t0
        return bool(ok)

    def page_count(self):
        return self.s.js("return document.querySelectorAll('.spatial-page').length")

    def badge(self):
        return self.s.js("const b = document.querySelector('.page-indicator-badge');"
                         "return b ? b.textContent.replace(/\\s+/g, ' ').trim() : null")

    def page_rect(self, index=0):
        """Screen rect (viewport CSS px) of page `index` as laid out by the Studio."""
        return self.s.js("""
            const p = document.querySelectorAll('.spatial-page')[arguments[0]]; if (!p) return null;
            const r = p.getBoundingClientRect(); return {x: r.x, y: r.y, w: r.width, h: r.height};
        """, index)

    def canvas_inked(self, index=0, timeout=15):
        """Fraction of non-white pixels in page `index`'s pdf.js canvas, polled until it is
        non-zero or `timeout` passes (pdf.js paints asynchronously). 0 = blank, -1 = no canvas."""
        deadline = time.time() + timeout
        v = self._inked(index)
        while (v is None or v <= 0) and time.time() < deadline:
            time.sleep(0.25)
            v = self._inked(index)
        return v

    def _inked(self, index):
        return self.s.js("""
            const c = document.querySelectorAll('.spatial-page canvas')[arguments[0]];
            if (!c || !c.width) return -1;
            const ctx = c.getContext('2d'); const d = ctx.getImageData(0, 0, c.width, c.height).data;
            let ink = 0, n = 0;
            for (let i = 0; i < d.length; i += 4 * 97) { n++; if (d[i+3] > 0 && (d[i] < 200 || d[i+1] < 200 || d[i+2] < 200)) ink++; }
            return ink / Math.max(1, n);
        """, index)

    def pdf_to_screen(self, x_pt, y_pt_from_top, page_w_pt, index=0):
        r = self.page_rect(index)
        k = r["w"] / page_w_pt
        return r["x"] + x_pt * k, r["y"] + y_pt_from_top * k

    # ---- view state ---------------------------------------------------------------------
    def transform(self):
        """The world transform of the PDF layer as [a, b, c, d, e, f] (zoom = a, pan = e, f)."""
        return self.s.js("""
            const t = getComputedStyle(document.getElementById('pdf-layer')).transform;
            if (!t || t === 'none') return [1, 0, 0, 1, 0, 0];
            const m = new DOMMatrix(t); return [m.a, m.b, m.c, m.d, m.e, m.f];
        """)

    def zoom_select_value(self):
        return self.s.js("const s = document.querySelector('select[aria-label=\"Zoom Level Preset\"]');"
                         "return s ? s.value : null")

    def set_select(self, css, value):
        """Set a <select> value and fire input+change (what a user's choice does)."""
        return self.s.js("""
            const s = document.querySelector(arguments[0]); if (!s) return false;
            if (![...s.options].some(o => o.value === arguments[1])) return 'no-option';
            s.value = arguments[1];
            s.dispatchEvent(new Event('input', {bubbles: true}));
            s.dispatchEvent(new Event('change', {bubbles: true}));
            return true;
        """, css, value)

    def set_input(self, css, value, index=0):
        return self.s.js("""
            const el = document.querySelectorAll(arguments[0])[arguments[2]]; if (!el) return false;
            const proto = el.tagName === 'TEXTAREA' ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
            Object.getOwnPropertyDescriptor(proto, 'value').set.call(el, String(arguments[1]));
            el.dispatchEvent(new Event('input', {bubbles: true}));
            el.dispatchEvent(new Event('change', {bubbles: true}));
            return true;
        """, css, value, index)

    # ---- tools & canvas -----------------------------------------------------------------
    def select_tool(self, category, label):
        """Pick a tool through the dockable toolbar: open the category drawer, click the item."""
        btn = self.s.find(f'[data-testid="category-btn-{category}"]')
        if not btn:
            return False
        for _ in range(2):
            if self.s.js("return !!document.querySelector('[data-testid=toolbar-category-popover]')"):
                break
            btn.click()
            time.sleep(0.15)
        item = self.s.find_text('[data-testid=toolbar-category-popover] .item-left', label)
        if not item:
            return False
        item.click()
        time.sleep(0.15)
        return not self.s.js("return !!document.querySelector('[data-testid=toolbar-category-popover]')")

    def vector_nodes(self):
        """Rendered markup nodes in the vector layer (excludes the selection gizmo)."""
        return self.s.js("""
            const g = document.querySelector('#vector-layer > g'); if (!g) return -1;
            return [...g.children].filter(e => !e.classList.contains('selection-gizmo')).length;
        """)

    def vector_html(self):
        return self.s.js("const g = document.querySelector('#vector-layer > g');"
                         "return g ? g.innerHTML.replace(/<!---->/g, '') : null")

    def vector_shapes(self):
        """[(tagName, attrs)] for each vector node; geometry is in world (CSS px) units."""
        return self.s.js("""
            const g = document.querySelector('#vector-layer > g'); if (!g) return [];
            return [...g.children].filter(e => !e.classList.contains('selection-gizmo')).map(e => {
                const a = {}; for (const at of e.attributes) a[at.name] = at.value;
                return [e.tagName.toLowerCase(), a];
            });
        """)

    def world_to_screen(self, x, y):
        return self.s.js("""
            const g = document.querySelector('#vector-layer > g');
            const m = new DOMMatrix(getComputedStyle(g).transform === 'none' ? undefined : getComputedStyle(g).transform);
            const svg = document.getElementById('vector-layer').getBoundingClientRect();
            const p = m.transformPoint(new DOMPoint(arguments[0], arguments[1]));
            return [svg.x + p.x, svg.y + p.y];
        """, x, y)

    def screen_to_world(self, x, y):
        return self.s.js("""
            const g = document.querySelector('#vector-layer > g');
            const m = new DOMMatrix(getComputedStyle(g).transform === 'none' ? undefined : getComputedStyle(g).transform).inverse();
            const svg = document.getElementById('vector-layer').getBoundingClientRect();
            const p = m.transformPoint(new DOMPoint(arguments[0] - svg.x, arguments[1] - svg.y));
            return [p.x, p.y];
        """, x, y)

    def status(self):
        return self.s.js("const b = document.querySelector('.studio-status-banner span');"
                         "return b ? b.textContent.trim() : null")

    def dismiss_status(self):
        self.s.js("const b = document.querySelector('.studio-status-banner .banner-close button'); if (b) b.click();")

    def click_button(self, text, css="button"):
        el = self.s.find_text(css, text)
        if not el:
            return False
        el.click()
        return True

    def button_state(self, aria_label):
        return self.s.js("""
            const b = document.querySelector(`button[aria-label="${arguments[0]}"]`);
            return b ? {disabled: b.disabled, pressed: b.getAttribute('aria-pressed')} : null;
        """, aria_label)

    def snap_off(self):
        st = self.button_state("Snap to drawing geometry")
        if st and st["pressed"] == "true":
            self.s.find('button[aria-label="Snap to drawing geometry"]').click()
            time.sleep(0.1)

    def save(self, flatten=False, timeout=60, button=None):
        """Click Save PDF / Flatten PDF / a named button and wait for the status banner to report
        the outcome. Returns (output_path or None, banner text)."""
        self.dismiss_status()
        label = button or ("Flatten" if flatten else "Save")
        el = self.s.find_text(".studio-top-bar .lp-btnwrap button", label)
        if not el:
            return None, f"button '{label}' not found"
        el.click()
        msg = self.s.wait("""
            const b = document.querySelector('.studio-status-banner span');
            const t = b ? b.textContent.trim() : '';
            return /Saved |Flattened |failed|error/i.test(t) ? t : null;
        """, timeout, 0.2)
        if not msg:
            return None, "no status message after save"
        m = re.search(r" to (/.+\.pdf)\s*$", msg)
        return (m.group(1) if m else None), msg


def run_flow(module, ctx):
    """Run one flow module's `run(flow)` and always return a result dict."""
    flow = Flow(ctx, module.NUM, module.TITLE)
    t0 = time.monotonic()
    try:
        module.run(flow)
    except Blocked as e:
        flow.blocked("precondition", str(e))
    except Exception as e:  # noqa: BLE001 - a crashing test is a FAIL with its traceback
        tb = traceback.format_exc()
        with open(os.path.join(flow.dir, "traceback.txt"), "w") as f:
            f.write(tb)
        here = [ln.strip() for ln in tb.splitlines() if "/e2e/" in ln and "File" in ln]
        flow.check("test raised", False, f"{type(e).__name__}: {e} | at {here[-1] if here else '?'}"
                   f" | after step '{flow.steps[-1]['step'] if flow.steps else '-'}'")
    finally:
        try:
            flow.close()
        except Exception:  # noqa: BLE001
            pass
        killed = flow.reap()  # raises (aborting the run) if anything survives SIGKILL
        if killed:
            flow.warn("no leftover processes after the flow", False,
                      f"had to kill {len(killed)} process(es): {killed}")
    return flow.result(time.monotonic() - t0)
