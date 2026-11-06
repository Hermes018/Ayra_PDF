#!/usr/bin/env python3
"""BUG-062 test: the soak's counted driver retry, against a fake driver.
python3 master/e2e/test_soak_retry.py"""
import http.client, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import soak_retry as sr  # noqa: E402

fails = []
def check(name, ok):
    print(("PASS " if ok else "FAIL ") + name)
    if not ok:
        fails.append(name)

class FakeDriver:
    """body() raises RemoteDisconnected on the iterations/calls listed in `drop` (call number)."""
    def __init__(self, drop=(), app_alive=True):
        self.drop, self.calls, self.app_alive, self.done = set(drop), 0, app_alive, 0
    def body(self):
        self.calls += 1
        if self.calls in self.drop:
            raise http.client.RemoteDisconnected("Remote end closed connection without response")
        self.done += 1
    def alive(self):
        return self.app_alive

def soak(drv, n):
    t = sr.Hiccups()
    err = None
    for it in range(1, n + 1):
        try:
            sr.run_iteration(it, drv.body, drv.alive, t)
        except sr.SoakFailure as e:
            err = str(e)
            break
    return t, err, sr.verdict(t, n, err)

# 1. one disconnect, app alive -> WARN, passes, entry recorded with iteration and count
d = FakeDriver(drop=[5])
t, err, v = soak(d, 200)
check("one disconnect, app alive: run passes", v["ok"] and err is None)
check("one disconnect: iteration retried once and completed", d.done == 200 and d.calls == 201)
check("one disconnect: WARN carries iteration and count", v["warn"] and "iteration 5" in v["detail"] and "1 driver hiccup" in v["detail"])
check("one disconnect: FLAKY entry present", len(v["flaky"]) == 1 and v["flaky"][0]["test"] == "e2e flow 12")

# 2. disconnect + dead app -> FAIL
d = FakeDriver(drop=[5], app_alive=False)
t, err, v = soak(d, 200)
check("disconnect with dead app: FAIL", not v["ok"] and "app" in (err or ""))

# 3. two failures in a row -> FAIL
d = FakeDriver(drop=[5, 6])
t, err, v = soak(d, 200)
check("retry also fails: FAIL", not v["ok"] and err is not None)

# 4. more than 1 % of iterations -> FAIL (3 of 200 = 1.5 %); exactly at 1 % passes
d = FakeDriver(drop=[5, 40, 90])
t, err, v = soak(d, 200)
check("hiccups above 1 %: FAIL", not v["ok"] and err is None and "1.5" in v["detail"])
d = FakeDriver(drop=[5, 40])
t, err, v = soak(d, 200)
check("hiccups at 1 %: passes with WARN", v["ok"] and v["warn"])

# 5. non-driver errors are not swallowed
def boom():
    raise ValueError("app bug")
try:
    sr.run_iteration(1, boom, lambda: True, sr.Hiccups())
    check("other exceptions propagate", False)
except ValueError:
    check("other exceptions propagate", True)

# 6. no hiccups -> clean pass, no warn
t, err, v = soak(FakeDriver(), 50)
check("no hiccups: clean", v["ok"] and not v["warn"])
sys.exit(1 if fails else 0)
