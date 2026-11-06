"""Minimal W3C WebDriver client for tauri-driver (Python standard library only).

`Driver` owns one tauri-driver process (and through it WebKitWebDriver) on a free port, with an
isolated XDG profile so every run is a cold, clean launch that cannot touch the user's settings.
`Session` is one app process: `Session(driver, binary, args)` launches the app; `.quit()` ends it.
"""
import base64
import json
import os
import pwd
import shutil
import signal
import socket
import subprocess
import time
import urllib.error
import urllib.request

ELEMENT_KEY = "element-6066-11e4-a52e-4f735466cecf"


class WebDriverError(RuntimeError):
    def __init__(self, status, error, message):
        super().__init__(f"{status} {error}: {message}")
        self.status, self.error, self.message = status, error, message


def _free_port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def find_tauri_driver():
    found = shutil.which("tauri-driver")
    if found:
        return found
    # The real home, not $HOME: the package smoke test runs with a fresh, empty HOME.
    cand = os.path.join(pwd.getpwuid(os.getuid()).pw_dir, ".cargo", "bin", "tauri-driver")
    return cand if os.path.exists(cand) else None


class Driver:
    """A tauri-driver process. Env vars in `profile_dir` isolate config/data/state/cache."""

    def __init__(self, profile_dir, log_path, extra_env=None):
        exe = find_tauri_driver()
        if not exe:
            raise RuntimeError("tauri-driver not found (cargo install tauri-driver)")
        self.port = _free_port()
        self.native_port = _free_port()
        self.profile_dir = profile_dir
        self.log_path = log_path
        env = dict(os.environ)
        for sub, var in (("config", "XDG_CONFIG_HOME"), ("data", "XDG_DATA_HOME"),
                         ("state", "XDG_STATE_HOME"), ("cache", "XDG_CACHE_HOME")):
            d = os.path.join(profile_dir, sub)
            os.makedirs(d, exist_ok=True)
            env[var] = d
        env.setdefault("RUST_BACKTRACE", "1")
        if extra_env:
            env.update(extra_env)
        self.env = env
        self._log = open(log_path, "ab")
        cmd = [exe, "--port", str(self.port), "--native-port", str(self.native_port)]
        if os.path.exists("/usr/bin/WebKitWebDriver"):
            cmd += ["--native-driver", "/usr/bin/WebKitWebDriver"]
        self.proc = subprocess.Popen(cmd, stdout=self._log, stderr=subprocess.STDOUT, env=env,
                                     start_new_session=True)
        self.url = f"http://127.0.0.1:{self.port}"
        deadline = time.time() + 15
        while time.time() < deadline:
            try:
                with socket.create_connection(("127.0.0.1", self.port), timeout=0.5):
                    return
            except OSError:
                if self.proc.poll() is not None:
                    raise RuntimeError(f"tauri-driver exited early, see {log_path}")
                time.sleep(0.1)
        raise RuntimeError("tauri-driver did not open its port in 15 s")

    def log_offset(self):
        self._log.flush()
        return os.path.getsize(self.log_path)

    def log_since(self, offset):
        self._log.flush()
        with open(self.log_path, "rb") as f:
            f.seek(offset)
            return f.read().decode("utf-8", "replace")

    def stop(self):
        stop_group(self.proc)
        self._log.close()



def stop_group(proc, term_wait=10.0):
    """SIGTERM the process group, then wait until EVERY member is gone (the leader can exit before its
    WebKitWebDriver child), and SIGKILL the group if some member outlives `term_wait`."""
    pgid = proc.pid
    try:
        os.killpg(pgid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    try:
        proc.wait(timeout=term_wait)
    except subprocess.TimeoutExpired:
        pass
    deadline = time.time() + term_wait
    while time.time() < deadline:
        try:
            os.killpg(pgid, 0)
        except ProcessLookupError:
            return
        time.sleep(0.05)
    try:
        os.killpg(pgid, signal.SIGKILL)
    except ProcessLookupError:
        return
    deadline = time.time() + 3
    while time.time() < deadline:
        try:
            os.killpg(pgid, 0)
        except ProcessLookupError:
            return
        time.sleep(0.05)

class Element:
    def __init__(self, session, ref):
        self.s, self.ref = session, ref

    def as_arg(self):
        return {ELEMENT_KEY: self.ref}

    def click(self):
        self.s.call("POST", f"/element/{self.ref}/click", {})

    def text(self):
        return self.s.call("GET", f"/element/{self.ref}/text")

    def rect(self):
        return self.s.call("GET", f"/element/{self.ref}/rect")

    def send_keys(self, text):
        self.s.call("POST", f"/element/{self.ref}/value", {"text": text})


class Session:
    def __init__(self, driver, binary, args=(), timeout=120):
        self.driver = driver
        self.base = None
        self.timeout = timeout
        caps = {"capabilities": {"alwaysMatch": {
            "tauri:options": {"application": binary, "args": list(args)}}}}
        self.t_request = time.monotonic()
        value = self._raw("POST", driver.url + "/session", caps)
        self.t_session = time.monotonic()
        self.id = value["sessionId"]
        self.base = f"{driver.url}/session/{self.id}"

    # ---- transport ------------------------------------------------------------------------
    def _raw(self, method, url, body=None):
        data = json.dumps(body).encode() if body is not None else None
        req = urllib.request.Request(url, data=data, method=method,
                                     headers={"Content-Type": "application/json"})
        try:
            with urllib.request.urlopen(req, timeout=self.timeout) as r:
                payload = r.read()
        except urllib.error.HTTPError as e:
            payload = e.read()
            try:
                v = json.loads(payload).get("value", {})
            except ValueError:
                v = {"error": "http", "message": payload[:300].decode("utf-8", "replace")}
            raise WebDriverError(e.code, v.get("error"), v.get("message")) from None
        return json.loads(payload or b"{}").get("value")

    def call(self, method, path, body=None):
        return self._raw(method, self.base + path, body)

    # ---- scripting ----------------------------------------------------------------------
    def js(self, script, *args):
        """Run a synchronous script body (use `return`)."""
        return self.call("POST", "/execute/sync", {"script": script, "args": list(args)})

    def js_async(self, script, *args):
        """Run an async script body; the last argument (`arguments[arguments.length-1]`) is the
        callback. Convenience: `done(x)` is defined for you."""
        body = "const done = arguments[arguments.length - 1];\n" + script
        return self.call("POST", "/execute/async", {"script": body, "args": list(args)})

    def wait(self, predicate_js, timeout=20, interval=0.2, args=()):
        """Poll a JS expression body (returning truthy) until it holds; returns its value."""
        deadline = time.time() + timeout
        last = None
        while time.time() < deadline:
            try:
                last = self.js(predicate_js, *args)
            except WebDriverError:
                last = None
            if last:
                return last
            time.sleep(interval)
        return last

    # ---- elements -----------------------------------------------------------------------
    def find(self, css):
        try:
            v = self.call("POST", "/element", {"using": "css selector", "value": css})
        except WebDriverError as e:
            if e.error == "no such element":
                return None
            raise
        return Element(self, v[ELEMENT_KEY])

    def find_all(self, css):
        v = self.call("POST", "/elements", {"using": "css selector", "value": css})
        return [Element(self, x[ELEMENT_KEY]) for x in v]

    def find_text(self, css, text):
        """First element matching css whose trimmed textContent contains `text`."""
        ref = self.js("""
            const [css, t] = arguments;
            return [...document.querySelectorAll(css)].find(e => e.textContent.trim().includes(t)) || null;
        """, css, text)
        return Element(self, ref[ELEMENT_KEY]) if ref else None

    # ---- input --------------------------------------------------------------------------
    def actions(self, sources):
        self.call("POST", "/actions", {"actions": sources})
        try:
            self.call("DELETE", "/actions")
        except WebDriverError:
            pass

    def mouse_path(self, points, button=0, step_ms=16):
        """Press at points[0], move through the rest, release (viewport CSS px)."""
        acts = [{"type": "pointerMove", "x": int(points[0][0]), "y": int(points[0][1]),
                 "origin": "viewport", "duration": 0},
                {"type": "pointerDown", "button": button}]
        for x, y in points[1:]:
            acts.append({"type": "pointerMove", "x": int(x), "y": int(y), "origin": "viewport",
                         "duration": step_ms})
        acts.append({"type": "pointerUp", "button": button})
        self.actions([{"type": "pointer", "id": "mouse1", "parameters": {"pointerType": "mouse"},
                       "actions": acts}])

    def mouse_click(self, x, y, button=0, double=False):
        acts = [{"type": "pointerMove", "x": int(x), "y": int(y), "origin": "viewport",
                 "duration": 0},
                {"type": "pointerDown", "button": button}, {"type": "pointerUp", "button": button}]
        if double:
            acts += [{"type": "pointerDown", "button": button},
                     {"type": "pointerUp", "button": button}]
        self.actions([{"type": "pointer", "id": "mouse1", "parameters": {"pointerType": "mouse"},
                       "actions": acts}])

    def keys(self, *keys):
        """Press and release keys in order, e.g. keys('\\ue007') for Enter."""
        acts = []
        for k in keys:
            acts += [{"type": "keyDown", "value": k}, {"type": "keyUp", "value": k}]
        self.actions([{"type": "key", "id": "kbd1", "actions": acts}])

    def chord(self, modifier, key):
        acts = [{"type": "keyDown", "value": modifier}, {"type": "keyDown", "value": key},
                {"type": "keyUp", "value": key}, {"type": "keyUp", "value": modifier}]
        self.actions([{"type": "key", "id": "kbd1", "actions": acts}])

    # ---- window / output ----------------------------------------------------------------
    def set_window(self, width, height):
        try:
            self.call("POST", "/window/rect", {"width": width, "height": height})
        except WebDriverError:
            pass

    def screenshot(self, path):
        png = base64.b64decode(self.call("GET", "/screenshot"))
        with open(path, "wb") as f:
            f.write(png)
        return path

    def alert_text(self):
        try:
            return self.call("GET", "/alert/text")
        except WebDriverError:
            return None

    def accept_alert(self):
        try:
            self.call("POST", "/alert/accept", {})
            return True
        except WebDriverError:
            return False

    def quit(self):
        if self.base:
            try:
                self.call("DELETE", "")
            except (WebDriverError, OSError):
                pass
            self.base = None


# Key codes (W3C WebDriver "normalised key values").
KEY_ENTER = "\ue007"
KEY_ESCAPE = "\ue00c"
KEY_CONTROL = "\ue009"
KEY_SHIFT = "\ue008"
KEY_DELETE = "\ue017"
