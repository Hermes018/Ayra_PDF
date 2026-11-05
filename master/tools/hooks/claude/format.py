#!/usr/bin/env python3
"""PostToolUse(Edit|Write) — format only the touched file. Never fails the tool call.
.rs → rustfmt (edition 2021). .ts/.js/.svelte/.css/.json → prettier from that worktree's
app-tauri/node_modules, but only for new files or files that were already prettier-clean at HEAD,
so editing one line never reformats a whole legacy file (mass reformat would conflict across
worktrees)."""
import json, os, subprocess, sys

ROOTS = ("/home/rakib/Desktop/Ayra_PDF/", "/home/rakib/Desktop/Ayra_PDF-wt/")
PRETTIER_EXT = (".ts", ".js", ".svelte", ".css", ".json")


def run(args, cwd=None, inp=None):
    return subprocess.run(args, cwd=cwd, input=inp, capture_output=True, timeout=60)


def main():
    try:
        data = json.load(sys.stdin)
    except Exception:
        return 0
    f = (data.get("tool_input") or {}).get("file_path") or (data.get("tool_response") or {}).get("filePath") or ""
    if not f or not os.path.isfile(f) or not f.startswith(ROOTS) or "/node_modules/" in f or "/target/" in f:
        return 0
    if f.endswith(".rs"):
        rf = os.path.expanduser("~/.cargo/bin/rustfmt")
        if os.path.exists(rf):
            run([rf, "--edition", "2021", f])
        return 0
    if f.endswith(PRETTIER_EXT):
        top = run(["git", "rev-parse", "--show-toplevel"], cwd=os.path.dirname(f)).stdout.decode().strip()
        pz = os.path.join(top, "master/app-tauri/node_modules/.bin/prettier")
        if not top or not os.path.exists(pz):
            return 0
        rel = os.path.relpath(f, top)
        tracked = run(["git", "cat-file", "-e", f"HEAD:{rel}"], cwd=top).returncode == 0
        if tracked:
            head = run(["git", "show", f"HEAD:{rel}"], cwd=top).stdout
            if run([pz, "--check", "--stdin-filepath", f], cwd=os.path.dirname(pz), inp=head).returncode != 0:
                return 0  # legacy file not prettier-formatted at HEAD: leave it alone
        run([pz, "--write", f], cwd=os.path.join(top, "master/app-tauri"))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception:
        sys.exit(0)
