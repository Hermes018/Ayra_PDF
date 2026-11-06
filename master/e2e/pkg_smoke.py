#!/usr/bin/env python3
"""Clean-user install smoke test for the Linux packages (docs/PROTOTYPE_SCOPE.md flow 1 + 2).

    python3 master/e2e/pkg_smoke.py [--bundle-dir DIR] [--kinds rpm,deb,appimage]

For each package found under <target>/release/bundle/{rpm,deb,appimage}:
  1. "install" it for a plain user. The AppImage is copied and made executable (its real user-level
     install). The .rpm and .deb are unpacked into a private prefix, because a system install needs
     root, which this test never uses; flow 1 reports that step as BLOCKED.
  2. Check the contents: the GUI binary, the `localpdf` CLI and the `lpdf-worker`, a .desktop entry
     for application/pdf, an icon, and that `localpdf --help` runs from the package.
  3. Run E2E flows 1 and 2 against the package's own binary with a fresh, empty HOME and XDG
     profile. Output: e2e/out/pkg/<kind>/ (report.json per package) and e2e/out/pkg/summary.json.
Standard library only; unpacking uses dpkg-deb (deb), 7z or the stdlib (rpm) and the AppImage
runtime's own --appimage-extract.
"""
import argparse
import glob
import gzip
import json
import lzma
import os
import shutil
import signal
import stat
import struct
import subprocess
import sys
import time

E2E = os.path.dirname(os.path.abspath(__file__))
MASTER = os.path.dirname(E2E)
OUT = os.path.join(E2E, "out", "pkg")
GUI = "localpdf-studio"


def sh(cmd, **kw):
    p = subprocess.run(cmd, capture_output=True, text=True, timeout=kw.pop("timeout", 300), **kw)
    return p.returncode, (p.stdout + p.stderr)


def rpm_payload(path, dest):
    """Unpack an .rpm without rpm tools: skip lead + signature + header, decompress the cpio
    payload (gzip/xz/zstd) and unpack it with cpio."""
    data = open(path, "rb").read()
    off = 96  # lead

    def header(o):
        if data[o:o + 3] != b"\x8e\xad\xe8":
            raise ValueError(f"bad rpm header magic at {o}")
        n, size = struct.unpack(">II", data[o + 8:o + 16])
        return o + 16 + n * 16 + size

    off = header(off)
    off += (8 - off % 8) % 8  # signature header is padded to 8 bytes
    off = header(off)
    payload = data[off:]
    if payload[:2] == b"\x1f\x8b":
        raw = gzip.decompress(payload)
    elif payload[:6] == b"\xfd7zXZ\x00":
        raw = lzma.decompress(payload)
    elif payload[:4] == b"\x28\xb5\x2f\xfd":
        if not shutil.which("zstd"):
            raise RuntimeError("zstd-compressed rpm payload and no zstd tool installed")
        raw = subprocess.run(["zstd", "-dc"], input=payload, capture_output=True, check=True).stdout
    else:
        raw = payload
    os.makedirs(dest, exist_ok=True)
    p = subprocess.run(["cpio", "-idm", "--quiet", "--no-absolute-filenames"], input=raw, cwd=dest,
                       capture_output=True)
    if p.returncode != 0:
        raise RuntimeError(f"cpio failed: {p.stderr[-300:]!r}")


def unpack(kind, pkg, work):
    """Returns (root of the unpacked tree, binary to test, notes)."""
    root = os.path.join(work, "root")
    notes = {}
    if kind == "deb":
        rc, out = sh(["dpkg-deb", "-x", pkg, root])
        if rc:
            raise RuntimeError(out)
        notes["depends"] = sh(["dpkg-deb", "-f", pkg, "Depends"])[1].strip()
        notes["recommends"] = sh(["dpkg-deb", "-f", pkg, "Recommends"])[1].strip()
        return root, os.path.join(root, "usr", "bin", GUI), notes
    if kind == "rpm":
        rpm_payload(pkg, root)
        return root, os.path.join(root, "usr", "bin", GUI), notes
    # appimage: the user-level install is "copy it somewhere and make it executable"
    inst = os.path.join(work, "Applications")
    os.makedirs(inst, exist_ok=True)
    dst = os.path.join(inst, os.path.basename(pkg))
    shutil.copyfile(pkg, dst)
    os.chmod(dst, os.stat(dst).st_mode | stat.S_IXUSR)
    rc, out = sh([dst, "--appimage-extract"], cwd=work, env={**os.environ, "APPIMAGE_EXTRACT_AND_RUN": "1"})
    if rc:
        raise RuntimeError(f"--appimage-extract failed: {out[-300:]}")
    return os.path.join(work, "squashfs-root"), dst, notes


def reap(marker):
    """Kill every process whose environment carries `marker` (LPDF_E2E_PACKAGE=..., inherited by
    run.py, tauri-driver, WebKitWebDriver, the app and its WebKit children: tauri-driver runs in its
    own session, so a process-group kill alone misses it). Returns the PIDs killed; raises if any
    survive SIGKILL."""
    me, killed = os.getpid(), []
    for _ in range(3):
        found = []
        for d in os.listdir("/proc"):
            if not d.isdigit() or int(d) == me:
                continue
            try:
                with open(f"/proc/{d}/environ", "rb") as f:
                    if marker in f.read().split(b"\0"):
                        found.append(int(d))
            except OSError:
                continue
        if not found:
            return killed
        for pid in found:
            try:
                os.kill(pid, signal.SIGKILL)
                killed.append(pid)
            except OSError:
                pass
        time.sleep(1)
    raise RuntimeError(f"processes survive SIGKILL: {found}")


def judge(rc, timed_out, report):
    """Package verdict from run.py's exit code AND its per-flow results. Both flows 1 and 2 must be
    present in a complete report; a missing flow, a timeout or an abnormal exit is a FAIL."""
    if timed_out:
        return "FAIL", "run.py timed out"
    if rc not in (0, 1):
        return "FAIL", f"run.py exited {rc} (crash/abort, not a test verdict)"
    if not report:
        return "FAIL", "no report.json"
    try:
        flows = {r["flow"]: r["status"] for r in report.get("flows", [])}
    except (AttributeError, KeyError, TypeError):
        return "FAIL", "malformed report.json"
    missing = [n for n in (1, 2) if n not in flows]
    if missing:
        return "FAIL", f"flows missing from report: {missing}"
    if rc == 0 and not all(v == "PASS" for v in flows.values()):
        return "FAIL", "exit 0 but non-PASS flows (inconsistent report)"
    if rc == 1 and all(v == "PASS" for v in flows.values()):
        return "FAIL", "flaky retry (run.py exit 1, every flow PASS): not a clean PASS"
    statuses = list(flows.values())
    if "FAIL" in statuses or any(v not in ("PASS", "BLOCKED") for v in statuses):
        return "FAIL", f"flows {flows}"
    return ("PASS" if all(v == "PASS" for v in statuses) else "BLOCKED"), f"flows {flows}"


def contents(root, kind):
    checks = {}
    gui = os.path.join(root, "usr", "bin", GUI)
    checks["gui binary"] = os.access(gui, os.X_OK)
    for tool in ("localpdf", "lpdf-worker"):
        checks[f"{tool} shipped"] = os.access(os.path.join(root, "usr", "bin", tool), os.X_OK)
    desk = glob.glob(os.path.join(root, "usr", "share", "applications", "*.desktop")) + \
        glob.glob(os.path.join(root, "*.desktop"))
    text = open(desk[0]).read() if desk else ""
    checks[".desktop entry"] = bool(desk)
    checks[".desktop opens PDFs (MimeType application/pdf)"] = "application/pdf" in text
    checks["icon"] = bool(glob.glob(os.path.join(root, "usr", "share", "icons", "**", "*.png"), recursive=True))
    cli = os.path.join(root, "usr", "bin", "localpdf")
    if os.access(cli, os.X_OK):
        rc, out = sh([cli, "--help"], timeout=30)
        checks["localpdf --help runs"] = rc == 0 and "USAGE" in out
    return checks


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--bundle-dir", default=os.path.join(MASTER, "target", "release", "bundle"))
    ap.add_argument("--kinds", default="rpm,deb,appimage")
    a = ap.parse_args()
    pats = {"rpm": "rpm/*.rpm", "deb": "deb/*.deb", "appimage": "appimage/*.AppImage"}
    summary = {"bundle_dir": a.bundle_dir, "packages": []}
    os.makedirs(OUT, exist_ok=True)
    kinds = [k.strip() for k in a.kinds.split(",") if k.strip()]
    bad = [k for k in kinds if k not in pats]
    if bad or not kinds:
        reason = f"unknown --kinds {bad}; valid: {sorted(pats)}" if bad else "no --kinds given"
        summary["packages"].append({"kind": ",".join(bad) or "-", "status": "FAIL", "why": reason})
        print("FAIL:", reason)
        kinds = []
    for kind in kinds:
        hits = sorted(glob.glob(os.path.join(a.bundle_dir, pats[kind])), key=os.path.getmtime)
        entry = {"kind": kind, "package": hits[-1] if hits else None}
        summary["packages"].append(entry)
        if not hits:
            entry["status"] = "MISSING"
            print(f"== {kind}: no package in {a.bundle_dir}/{pats[kind]}")
            continue
        pkg = hits[-1]
        work = os.path.join(OUT, kind)
        shutil.rmtree(work, ignore_errors=True)
        os.makedirs(work)
        entry["size_mb"] = round(os.path.getsize(pkg) / 1e6, 1)
        print(f"== {kind}: {pkg} ({entry['size_mb']} MB)", flush=True)
        try:
            root, binary, notes = unpack(kind, pkg, work)
        except Exception as e:  # noqa: BLE001
            entry["status"] = "FAIL"
            entry["error"] = f"unpack: {e}"[:400]
            print("   unpack FAILED:", entry["error"])
            continue
        entry.update(notes)
        entry["contents"] = contents(root, kind)
        for k, v in entry["contents"].items():
            print(f"   [{'PASS' if v else 'FAIL'}] {k}")
        home = os.path.join(work, "home")  # a brand-new user: empty HOME, no config, no cache
        os.makedirs(home)
        marker_val = f"{kind}:{binary}:{os.getpid()}"
        env = {**os.environ, "HOME": home, "LPDF_E2E_PACKAGE": marker_val}
        for v in ("XDG_CONFIG_HOME", "XDG_DATA_HOME", "XDG_STATE_HOME", "XDG_CACHE_HOME"):
            env.pop(v, None)
        if kind == "appimage":
            env["APPIMAGE_EXTRACT_AND_RUN"] = "1"  # no FUSE needed
        proc = subprocess.Popen([sys.executable, os.path.join(E2E, "run.py"), "--flows", "1,2", "--binary",
                                 binary, "--out", os.path.join(work, "e2e")], env=env, text=True,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        timed_out = False
        try:
            stdout, stderr = proc.communicate(timeout=1800)
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(proc.pid, signal.SIGKILL)
            stdout, stderr = proc.communicate()
        # Always reap by the env marker: tauri-driver lives in its own session.
        reap_err = None
        try:
            entry["reaped_pids"] = reap(f"LPDF_E2E_PACKAGE={marker_val}".encode())
        except RuntimeError as e:
            reap_err = str(e)
            entry["reaped_pids"] = []
        print("\n".join("   " + ln for ln in stdout.splitlines() if ln.startswith(("==", "    ["))))
        try:
            with open(os.path.join(work, "e2e", "report.json")) as f:
                rep = json.load(f)
            if not isinstance(rep, dict):
                rep = {"flows": 0}
        except (OSError, ValueError):
            rep = None
        entry["run_rc"] = proc.returncode
        try:
            entry["flows"] = {r["flow"]: {"status": r["status"], "reason": str(r.get("reason", ""))[:300]}
                              for r in (rep or {}).get("flows", [])}
        except (AttributeError, KeyError, TypeError):
            entry["flows"] = {}
            rep = {"flows": 0}  # forces judge() to report a malformed report
        verdict, why = judge(proc.returncode, timed_out, rep)
        if reap_err:
            verdict, why = "FAIL", f"reap failed: {reap_err}"
        if not all(entry["contents"].values()):
            verdict, why = "FAIL", "package contents check failed"
        entry["status"], entry["why"] = verdict, why
        if verdict == "FAIL" and stderr.strip():
            entry["stderr_tail"] = stderr[-400:]
    with open(os.path.join(OUT, "summary.json"), "w") as f:
        json.dump(summary, f, indent=2)
    print("\nSUMMARY ->", os.path.join(OUT, "summary.json"))
    for e in summary["packages"]:
        print(f"  {e['kind']:<9} {e.get('status')}  {e.get('why', '')[:120]}")
    return 0 if all(e.get("status") == "PASS" for e in summary["packages"]) else 1


if __name__ == "__main__":
    sys.exit(main())
