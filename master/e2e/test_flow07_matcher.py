#!/usr/bin/env python3
"""BUG-046 self-test: the flow 7 drawn-signature matcher can fail.  python3 master/e2e/test_flow07_matcher.py"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ink_match as m  # noqa: E402

fails = []
def check(name, ok):
    print(("PASS " if ok else "FAIL ") + name)
    if not ok: fails.append(name)

PAGE = [0, 0, 612, 792]; EXPECT = [100, 600, 300, 640]
pts = [[110 + i * 9, 620 + (10 if i % 2 else -10)] for i in range(20)]
flat = [c for p in pts for c in p]
good = {"/Subtype": "/Ink", "/Rect": [105, 605, 295, 635], "/InkList": [flat], "/AP": {"/N": {"/Length": 80}}}
text = {"/Subtype": "/FreeText", "/Rect": [0, 0, 10, 10], "/Contents": "u:x"}

def run(annots, **kw):
    return m.match_drawn(annots, kw.get("page", PAGE), kw.get("exp", EXPECT), 20)[0]

check("Ink with points passes", run([good, text]) is True)
check("Ink with only an AP stream passes", run([{**good, "/InkList": []}]) is True)
check("Ink with empty InkList and no AP FAILS", run([{"/Subtype": "/Ink", "/Rect": good["/Rect"], "/InkList": [[]]}]) is False)
check("Ink with no InkList key and zero-length AP FAILS",
      run([{"/Subtype": "/Ink", "/Rect": good["/Rect"], "/AP": {"/N": {"/Length": 0}}}]) is False)
check("two candidate Ink annotations FAIL", run([good, dict(good)]) is False)
check("no candidate FAILS", run([text]) is False)
check("too few points FAILS", run([{**good, "/InkList": [flat[:6]], "/AP": {}}]) is False)
check("Rect off the page FAILS", run([{**good, "/Rect": [700, 900, 800, 950]}]) is False)
check("Rect far from the stroke FAILS", run([{**good, "/Rect": [400, 100, 500, 140]}]) is False)
if fails:
    print(f"{len(fails)} FAILED"); sys.exit(1)
print("all PASS")
