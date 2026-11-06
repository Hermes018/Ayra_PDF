#!/usr/bin/env python3
"""BUG-031 test: the post-run check catches a leaked harness process; PrivateBus.stop and the
signal-safe run guard leave nothing behind.  python3 master/e2e/test_leak_check.py [--leak-only]"""
import os, shutil, signal, subprocess, sys, tempfile, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common  # noqa: E402

fails = []
def check(name, ok):
    print(("PASS " if ok else "FAIL ") + name); 
    if not ok: fails.append(name)

marker = f"test-{os.getpid()}"
os.environ[common.RUN_ENV] = marker

# 1. A deliberately leaked process (own session, marker env, like a stray dbus-daemon) is caught.
leak = subprocess.Popen(["sleep", "300"], start_new_session=True)
time.sleep(0.2)
check("leak detected", leak.pid in common.harness_procs(marker))
# 2. A leaked bus identified only by its /tmp/lpdf-bus-* path is caught (no env marker).
d = tempfile.mkdtemp(prefix="lpdf-bus-", dir="/tmp")
env = {k: v for k, v in os.environ.items() if k != common.RUN_ENV}
conf = os.path.join(d, "bus.conf")
open(conf, "w").write(common._BUS_CONF.format(dir=d))
bus = subprocess.Popen([shutil.which("dbus-daemon"), f"--config-file={conf}", "--nofork", "--print-address=1"],
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=env, start_new_session=True)
time.sleep(0.5)
check("path-only leak detected", bus.pid in common.harness_procs("none", d))
if "--leak-only" in sys.argv:
    print("leaked pids left running:", leak.pid, bus.pid); sys.exit(1)
killed, surv = common.kill_harness(marker, d)
leak.wait(); bus.wait()
check("sweep kills leaks, none survive", leak.pid in killed and bus.pid in killed and not surv)
shutil.rmtree(d, ignore_errors=True)

# 3. PrivateBus.stop leaves nothing.
b = common.PrivateBus("/tmp")
bd = b.dir
check("bus running", bool(common.harness_procs(marker, bd)))
b.stop()
check("bus stop leaves nothing", not common.harness_procs(marker, bd))

# 4. SIGTERM to a run (bus + a marker child in its own session) -> guard cleans up everything.
child = r'''
import os, signal, subprocess, sys, time
sys.path.insert(0, %r)
import common
os.environ[common.RUN_ENV] = %r
def h(s, f): raise SystemExit(1)
signal.signal(signal.SIGTERM, h)
ctx = common.Context.__new__(common.Context); ctx.bus = common.PrivateBus("/tmp")
print(ctx.bus.dir, flush=True)
subprocess.Popen(["sleep", "300"], start_new_session=True)
try:
    time.sleep(300)
finally:
    ctx.close(); ctx.post_run_check(%r)
''' % (os.path.dirname(os.path.abspath(__file__)), marker, marker)
p = subprocess.Popen([sys.executable, "-c", child], stdout=subprocess.PIPE, text=True)
bd = p.stdout.readline().strip(); time.sleep(0.5)
check("run alive with children", len(common.harness_procs(marker, bd)) >= 3)
p.send_signal(signal.SIGTERM); p.wait(timeout=20); time.sleep(0.3)
check("SIGTERM leaves no harness process", not common.harness_procs(marker, bd))
sys.exit(1 if fails else 0)
