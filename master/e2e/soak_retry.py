"""BUG-062: counted, visible retry of a soak iteration when tauri-driver drops a WebDriver
connection (RemoteDisconnected / connection reset). Policy (owner-approved): confirm the app is
alive, retry the iteration ONCE, record a "driver hiccup" with the iteration number; fail if the
app is dead, the retry also fails, or hiccups exceed 1 % of iterations. Never hidden: every hiccup
is a WARN plus a FLAKY.md-ready entry in the run output."""
import http.client
import urllib.error

MAX_FRACTION = 0.01


class SoakFailure(Exception):
    pass


def is_driver_drop(e):
    if isinstance(e, urllib.error.URLError) and not isinstance(e, urllib.error.HTTPError):
        e = e.reason if isinstance(e.reason, BaseException) else e
    return isinstance(e, (http.client.RemoteDisconnected, http.client.IncompleteRead,
                          ConnectionResetError, ConnectionAbortedError, BrokenPipeError))


class Hiccups:
    def __init__(self):
        self.items = []  # (iteration, "Type: message")


def run_iteration(it, body, alive, tracker):
    """Run body(); on a dropped driver connection retry it once (app must be alive)."""
    try:
        return body()
    except Exception as e:  # noqa: BLE001
        if not is_driver_drop(e):
            raise
        first = f"{type(e).__name__}: {e}"
    if not alive():
        raise SoakFailure(f"iteration {it}: driver connection dropped ({first}) and the app is dead")
    tracker.items.append((it, first))
    try:
        return body()
    except Exception as e:  # noqa: BLE001
        raise SoakFailure(f"iteration {it}: retry after driver hiccup also failed "
                          f"({type(e).__name__}: {e}); first: {first}") from e


def verdict(tracker, iterations, failure=None):
    """-> {ok, warn, detail, flaky[]}. `failure` is a SoakFailure message, if any."""
    n = len(tracker.items)
    pct = 100.0 * n / iterations if iterations else 0.0
    its = ", ".join(str(i) for i, _ in tracker.items[:10])
    detail = (f"{n} driver hiccup(s) in {iterations} iterations ({pct:.2f} %)"
              + (f" at iteration {its}" if n else "") + (f"; {failure}" if failure else ""))
    ok = failure is None and pct <= MAX_FRACTION * 100 + 1e-9
    flaky = [{"test": "e2e flow 12", "symptom": f"iteration {i}: {m}"[:200], "passed_on_rerun": "y (retry)",
              "attempts": 2, "suspected_cause": "tauri-driver dropped the WebDriver connection, app alive (BUG-062)",
              "action": "open"} for i, m in tracker.items]
    return {"ok": ok, "warn": n > 0, "detail": detail, "flaky": flaky}
