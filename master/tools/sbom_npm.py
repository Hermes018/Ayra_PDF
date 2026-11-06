#!/usr/bin/env python3
"""sbom_npm.py <in.cdx.json> <out.cdx.json> — offline, stdlib-only SBOM completion step.

`cargo xtask sbom` lists the Cargo.lock crates but only the DIRECT package.json npm entries and a
hard-coded app version. This step replaces the npm part with every package in
app-tauri/pnpm-lock.yaml (the full resolved tree, with its integrity hash and, when the package is
installed in node_modules/.pnpm, its declared licence) and stamps the app version from
app-tauri/package.json. No network: it only reads files in this repo.
"""
import json, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
APP = os.path.normpath(os.path.join(HERE, "..", "app-tauri"))
KEY = re.compile(r"^  '?(@?[^'@\s][^'\s]*?)@([^'\s(]+)(?:\([^']*\))?'?:\s*$")


def lock_packages():
    out, cur, in_pk = {}, None, False
    for line in open(os.path.join(APP, "pnpm-lock.yaml"), encoding="utf-8"):
        line = line.rstrip("\n")
        if line.startswith("packages:"):
            in_pk = True
            continue
        if line.startswith("snapshots:"):
            break
        if not in_pk:
            continue
        m = KEY.match(line)
        if m:
            cur = (m.group(1), m.group(2))
            out[cur] = None
        elif cur and line.strip().startswith("resolution:"):
            im = re.search(r"integrity:\s*([A-Za-z0-9+/=-]+)", line)
            out[cur] = im.group(1) if im else None
    return out


def licence(name, ver):
    d = os.path.join(APP, "node_modules", ".pnpm", f"{name.replace('/', '+')}@{ver}", "node_modules", name, "package.json")
    try:
        lic = json.load(open(d, encoding="utf-8")).get("license")
        return lic if isinstance(lic, str) else (lic or {}).get("type")
    except Exception:
        return None


def main(src, dst):
    bom = json.load(open(src, encoding="utf-8"))
    ver = json.load(open(os.path.join(APP, "package.json"), encoding="utf-8"))["version"]
    keep = [c for c in bom["components"]
            if not any(p.get("name") == "ecosystem" and p.get("value") == "npm" for p in c.get("properties", []))]
    npm = []
    for (name, v), integ in sorted(lock_packages().items()):
        props = [{"name": "ecosystem", "value": "npm"}]
        lic = licence(name, v)
        c = {"type": "library", "name": name, "version": v, "purl": f"pkg:npm/{name}@{v}"}
        if lic:
            c["licenses"] = [{"expression": lic}]
        if integ:
            c["properties"] = props + [{"name": "integrity", "value": integ}]
        else:
            c["properties"] = props
        npm.append(c)
    bom["components"] = sorted(keep + npm, key=lambda c: (c["name"], c["version"]))
    bom["metadata"]["component"]["version"] = ver
    if dst:
        json.dump(bom, open(dst, "w", encoding="utf-8"), indent=2)
        n_lic = sum(1 for c in npm if "licenses" in c)
        print(f"rust={len(keep)} npm={len(npm)} (licence known: {n_lic}) version={ver} -> {dst}")
    else:
        json.dump(bom, sys.stdout, indent=2)


if __name__ == "__main__":
    src = sys.argv[1] if len(sys.argv) > 1 else os.path.join(HERE, "..", "logs", "sbom.cdx.json")
    dst = sys.argv[2] if len(sys.argv) > 2 else (None if not sys.stdout.isatty() else os.path.join(HERE, "..", "docs", "SBOM-v0.2.0.json"))
    main(src, dst)
