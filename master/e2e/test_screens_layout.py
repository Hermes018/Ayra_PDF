#!/usr/bin/env python3
"""BUG-048 test: each run's screenshots live in out/runs/<run-id>/screens; out/screens is a 'latest'
symlink; a later run never deletes an earlier run's screenshots.  python3 master/e2e/test_screens_layout.py"""
import os, sys, tempfile
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common  # noqa: E402

fails = []
def check(name, ok):
    print(("PASS " if ok else "FAIL ") + name)
    if not ok: fails.append(name)

out = tempfile.mkdtemp(prefix="lpdf-screens-test-")
legacy = os.path.join(out, "screens"); os.makedirs(legacy)          # old layout: a real shared folder
open(os.path.join(legacy, "old.png"), "w").write("old")
try:
    s1 = common.prepare_run_screens(out, "run-1"); shot1 = os.path.join(s1, "a.png"); open(shot1, "w").write("1")
    s2 = common.prepare_run_screens(out, "run-2"); shot2 = os.path.join(s2, "b.png"); open(shot2, "w").write("2")
    check("run 1 screenshot survives run 2", os.path.isfile(shot1))
    check("run 2 screenshot exists", os.path.isfile(shot2))
    check("per-run layout", s1 == os.path.join(out, "runs", "run-1", "screens") and s2 != s1)
    check("latest is a symlink to run 2", os.path.islink(legacy) and os.path.realpath(legacy) == os.path.realpath(s2))
    check("latest view shows run 2 shot", os.path.isfile(os.path.join(legacy, "b.png")))
    check("old shared folder content preserved", os.path.isfile(os.path.join(out, "runs", "legacy-screens", "old.png")))
    ctx = common.Context("/bin/true", out, 0, screens=s2)
    check("Context uses per-run screens", ctx.screens == s2); ctx.close()
finally:
    import shutil; shutil.rmtree(out, ignore_errors=True)
print("FAILED: %s" % fails if fails else "ALL PASS"); sys.exit(1 if fails else 0)
