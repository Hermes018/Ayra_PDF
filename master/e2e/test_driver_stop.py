#!/usr/bin/env python3
"""Flow 10/23 leftover-process test: stopping the driver must leave NO process of its group behind, even when the
leader exits on SIGTERM before a child (WebKitWebDriver) has.  python3 master/e2e/test_driver_stop.py"""
import os, signal, subprocess, sys, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import wd  # noqa: E402

# Leader exits at once on SIGTERM; its child ignores SIGTERM and would linger.
leader = subprocess.Popen(["sh", "-c", "(trap '' TERM; sleep 30) & wait"], start_new_session=True)
time.sleep(0.3)
wd.stop_group(leader, term_wait=1.0)


def group_alive(pgid):
    try:
        os.killpg(pgid, 0)
        return True
    except ProcessLookupError:
        return False


# a reaped leader leaves the group alive only while a member survives
time.sleep(0.2)
ok = not group_alive(leader.pid)
print(("PASS" if ok else "FAIL") + " driver group fully gone right after stop")
sys.exit(0 if ok else 1)
