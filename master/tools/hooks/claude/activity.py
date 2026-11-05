#!/usr/bin/env python3
"""Stop / SubagentStop — append one status line to master/logs/AGENT_ACTIVITY.md. Never blocks."""
import datetime, json, os, sys

try:
    data = json.load(sys.stdin)
except Exception:
    data = {}
root = os.environ.get("CLAUDE_PROJECT_DIR", "/home/rakib/Desktop/Ayra_PDF")
path = os.path.join(root, "master/logs/AGENT_ACTIVITY.md")
ev = data.get("hook_event_name", "?")
who = data.get("agent_type") or ("lead" if ev == "Stop" else "subagent")
aid = (data.get("agent_id") or "")[:8]
sid = (data.get("session_id") or "")[:8]
cwd = data.get("cwd") or ""
wt = os.path.basename(cwd.rstrip("/")) if cwd else "-"
line = f"- {datetime.datetime.now().astimezone().isoformat(timespec='seconds')} {ev} {who}{('#' + aid) if aid else ''} session={sid} cwd={wt}\n"
try:
    new = not os.path.exists(path)
    with open(path, "a", encoding="utf-8") as fh:
        if new:
            fh.write("# AGENT ACTIVITY (append-only, written by the Stop/SubagentStop hooks)\n\n")
        fh.write(line)
except Exception:
    pass
sys.exit(0)
