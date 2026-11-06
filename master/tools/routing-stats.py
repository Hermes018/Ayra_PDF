#!/usr/bin/env python3
"""Escalation rate per task type from master/logs/ROUTING.md (CLAUDE.md "Model routing" self-tuning).
Rule: rate > 25 % -> move the type up one tier; a Sonnet-high type with 0 escalations over >= 10 tasks
-> try one tier down. Prints a table plus the suggested moves; changes are made by the lead and reported."""
import collections, pathlib, sys
p = pathlib.Path(__file__).resolve().parents[1] / "logs" / "ROUTING.md"
n, esc, model = collections.Counter(), collections.Counter(), {}
for line in (l for l in p.read_text(encoding="utf-8").splitlines() if l.lstrip().startswith("|")):
    c = [x.strip() for x in line.strip().strip("|").split("|")]
    if len(c) < 7 or c[0] in ("time", "---") or c[0].startswith("-"):
        continue
    t = c[1]; n[t] += 1; model[t] = c[3]
    if c[4] == "escalated" or "→" in c[5]:
        esc[t] += 1
print(f"{'task_type':24} {'tasks':>5} {'esc':>4} {'rate':>6}  model/effort  suggestion")
for t in sorted(n):
    r = esc[t] / n[t]
    s = "UP one tier" if r > 0.25 else ("try DOWN one tier" if model[t] == "sonnet/high" and esc[t] == 0 and n[t] >= 10 else "")
    print(f"{t:24} {n[t]:5} {esc[t]:4} {r:6.0%}  {model[t]:12}  {s}")
sys.exit(0)
