#!/usr/bin/env python3
"""Regenerates /tree.md and every <folder>/<folder>.md from descriptions.tsv + headers/.

Usage (from anywhere):  python3 master/tools/tree/render_tree.py          # rewrite the maps
                        python3 master/tools/tree/render_tree.py --check  # fail if maps are stale
Fails if any tracked file has no line in descriptions.tsv, or a line names a path that no
longer exists. To map a new file: add "<repo-relative path><TAB><one-line description>" to
descriptions.tsv (directories end with "/"), then rerun. Dev tooling only; never shipped."""
import os, subprocess, sys

SCRATCH = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(SCRATCH, "..", "..", ".."))
CHECK = "--check" in sys.argv[1:]
outputs = {}
FOLDERS = ["master", "ubuntu", "fedora", "windows", "macos", "ios", "android"]

desc = {}
for line in open(os.path.join(SCRATCH, "descriptions.tsv"), encoding="utf-8"):
    line = line.rstrip("\n")
    if line:
        p, d = line.split("\t", 1)
        desc[p] = d

tracked = subprocess.run(["git", "-C", ROOT, "ls-files"], capture_output=True, text=True, check=True).stdout.split()
# generated maps are tracked once written; make sure they are always present in the listing
tracked = set(tracked) | {"tree.md"} | {f"{f}/{f}.md" for f in FOLDERS}
# Third-party trees mapped as ONE entry (their contents are upstream code, described in that entry).
COLLAPSED = ["master/vendor/pdfcraft/", "master/vendor/patches/winprint/src/",
             "master/vendor/craft/cadcraft/", "master/vendor/patches/cadcraft-render/src/"]
tracked = {t for t in tracked if not any(t.startswith(c) for c in COLLAPSED)}

files = set(tracked)
dirs = set()
for f in files:
    parts = f.split("/")
    for i in range(1, len(parts)):
        dirs.add("/".join(parts[:i]) + "/")
for c in COLLAPSED:
    if c.rstrip("/") + "/" != "master/vendor/pdfcraft/" and os.path.exists(os.path.join(ROOT, c)):
        dirs.add(c)
        parts = c.rstrip("/").split("/")
        for i in range(1, len(parts)):
            dirs.add("/".join(parts[:i]) + "/")
dirs.discard("master/vendor/pdfcraft/")
dirs = {d for d in dirs if not any(d.startswith(c) and d != c for c in COLLAPSED)}

missing = sorted(p for p in files | dirs if p not in desc)
stale = sorted(p for p in desc if p not in files | dirs)
if missing or stale:
    for p in missing: print("NO DESCRIPTION:", p)
    for p in stale: print("STALE ENTRY:  ", p)
    sys.exit(1)

def children(prefix):
    out = set()
    for p in files | dirs:
        if p.startswith(prefix) and p != prefix:
            rest = p[len(prefix):]
            head = rest.split("/")[0]
            out.add(prefix + head + ("/" if "/" in rest else ""))
    # directories first, then files; dotfiles first within each group
    return sorted(out, key=lambda q: (not q.endswith("/"), q.lower()))

def render(prefix):
    rows = []
    def walk(pre, indent):
        kids = children(pre)
        for i, k in enumerate(kids):
            last = i == len(kids) - 1
            name = k[len(pre):]
            rows.append((indent + ("└── " if last else "├── ") + name, desc[k]))
            if k.endswith("/"):
                walk(k, indent + ("    " if last else "│   "))
    walk(prefix, "")
    width = min(max(len(r[0]) for r in rows), 60) + 2
    return "\n".join(f"{a.ljust(width)}# {b}" if len(a) < width else f"{a}  # {b}" for a, b in rows)

def header(name):
    return open(os.path.join(SCRATCH, "headers", name + ".md"), encoding="utf-8").read().rstrip() + "\n"

NOTE = ("Every entry has a one-line description. `master/vendor/pdfcraft` is an upstream submodule and is "
        "listed as one entry. Build output (`target/`, `node_modules/`, `build/`) and downloaded corpus PDFs "
        "are not tracked and not listed.\n")

for f in FOLDERS:
    body = header(f) + "\n## Map of `" + f + "/`\n\n" + NOTE + "\n```text\n" + f + "/\n" + render(f + "/") + "\n```\n"
    outputs[os.path.join(f, f + ".md")] = body

root_rows = [p for p in children("") if p.rstrip("/") not in FOLDERS]
top = []
for p in root_rows:
    top.append(p)
tree = header("tree") + "\n## Repository root\n\n" + NOTE + "\n```text\nAyra_PDF/\n"
rows = []
order = [p for p in children("") if p.rstrip("/") not in FOLDERS] + [f + "/" for f in FOLDERS]
for p in order:
    rows.append((p, desc[p]))
w = max(len(a) for a, _ in rows) + 6
tree += "\n".join((("└── " if i == len(rows) - 1 else "├── ") + a).ljust(w) + "# " + b
                  for i, (a, b) in enumerate(rows)) + "\n```\n"
for p in [p for p in children("") if p.endswith("/") and p.rstrip("/") not in FOLDERS]:
    tree += "\n## `" + p + "`\n\n```text\n" + p + "\n" + render(p) + "\n```\n"
for f in FOLDERS:
    tree += "\n## `" + f + "/` — see also [`" + f + "/" + f + ".md`](" + f + "/" + f + ".md)\n\n```text\n" + f + "/\n" + render(f + "/") + "\n```\n"
outputs["tree.md"] = tree
stale_maps = []
for rel, text in outputs.items():
    path = os.path.join(ROOT, rel)
    current = open(path, encoding="utf-8").read() if os.path.exists(path) else None
    if current != text:
        stale_maps.append(rel)
        if not CHECK:
            open(path, "w", encoding="utf-8").write(text)
if CHECK and stale_maps:
    print("STALE MAPS (run python3 master/tools/tree/render_tree.py):", ", ".join(sorted(stale_maps)))
    sys.exit(1)
print("ok:", len(files), "files,", len(dirs), "directories mapped;",
      ("checked" if CHECK else "updated " + str(len(stale_maps)) + " map(s)"))
