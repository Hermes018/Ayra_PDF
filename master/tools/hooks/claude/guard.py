#!/usr/bin/env python3
"""PreToolUse(Bash) guard — makes the sprint guardrails mandatory (PROMPT_LOG #18).
Blocks (exit 2, reason on stderr): `cargo clean`; `git push`; `git reset --hard`; `rm -rf` outside
target/ or logs/runs/ (or this session's /tmp/claude-<uid>/ scratch); cargo/pnpm/npm/gradle BUILDS
that are not run through master/tools/run.sh (or verify.sh). Only commands in *command position*
are checked: quoted strings, heredoc bodies and commit messages that merely mention them are fine.
TRANSITIONAL: direct master/tools/build-lock.sh calls are still allowed until all builders are
re-briefed to run.sh (see logs/PROGRESS_LOG.md); set ALLOW_BUILD_LOCK = False to enforce."""
import json, os, re, shlex, sys

ALLOW_BUILD_LOCK = True
CARGO_BUILD = {"build", "b", "test", "t", "check", "c", "clippy", "nextest", "run", "r", "xtask",
               "doc", "d", "bench", "install", "vendor", "rustc", "fix", "miri", "llvm-cov"}
PNPM_SAFE = {"--version", "-v", "-V", "store", "list", "ls", "why", "outdated", "config", "root", "bin", "help"}
WRAPPERS = {"sudo", "nohup", "setsid", "exec", "time", "command", "builtin", "stdbuf", "unbuffer"}
SEPS = {";", "&&", "||", "|", "&", "(", ")", "|&", ";;"}


def strip_heredocs(s: str) -> str:
    out, pending, lines = [], [], s.split("\n")
    for line in lines:
        if pending:
            if line.strip() == pending[0]:
                pending.pop(0)
            continue
        out.append(line)
        pending += [m.group(2) for m in re.finditer(r"<<-?\s*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\1", line)]
    return "\n".join(out)


def segments(cmd: str):
    s = strip_heredocs(cmd).replace("$(", " ; ").replace("`", " ; ").replace("\n", " ; ")
    lex = shlex.shlex(s, posix=True, punctuation_chars=True)
    lex.whitespace_split = True
    seg = []
    try:
        for tok in lex:
            if tok in SEPS:
                if seg:
                    yield seg
                seg = []
            else:
                seg.append(tok)
    except ValueError:  # unbalanced quotes: fall back to a plain split, never crash the hook
        for part in re.split(r"[;&|\n]+", cmd):
            if part.strip():
                yield part.split()
        return
    if seg:
        yield seg


def unwrap(tokens):
    """Drop env assignments and wrapper commands; recurse into `bash -c "<cmd>"`."""
    t = list(tokens)
    while t:
        head = os.path.basename(t[0])
        if re.match(r"^[A-Za-z_][A-Za-z0-9_]*=", t[0]):
            t = t[1:]
        elif head in WRAPPERS:
            t = t[1:]
        elif head == "env":
            t = t[1:]
            while t and (t[0].startswith("-") or "=" in t[0]):
                t = t[1:]
        elif head in ("nice", "ionice", "timeout", "chrt", "taskset"):
            t = t[1:]
            while t and (t[0].startswith("-") or re.match(r"^[\d.]+[smhd]?$", t[0])):
                t = t[1:]
        elif head == "flock":
            t = t[1:]
            while t and t[0].startswith("-"):
                t = t[1:]
            t = t[1:]  # lock file
        elif head in ("bash", "sh", "dash", "zsh") and "-c" in t:
            i = t.index("-c")
            return ("subshell", t[i + 1] if i + 1 < len(t) else "")
        else:
            break
    return ("cmd", t)


def first_sub(t, value_opts=()):
    i = 1
    while i < len(t):
        a = t[i]
        if a in value_opts:
            i += 2
            continue
        if a.startswith("-") or a.startswith("+"):
            i += 1
            continue
        return a, i
    return None, i


def in_allowed_rm_zone(path: str, cwd: str) -> bool:
    p = os.path.normpath(os.path.join(cwd, os.path.expanduser(path)))
    parts = p.split(os.sep)
    if "target" in parts:
        return True
    if "/logs/runs" in p:
        return True
    return p.startswith(f"/tmp/claude-{os.getuid()}/")


def check(cmd: str, cwd: str, depth=0):
    for seg in segments(cmd):
        kind, t = unwrap(seg)
        if kind == "subshell":
            if depth < 3:
                r = check(t, cwd, depth + 1)
                if r:
                    return r
            continue
        if not t:
            continue
        head = os.path.basename(t[0])
        if head in ("build-lock.sh",) and not ALLOW_BUILD_LOCK:
            return "run heavy builds via /home/rakib/Desktop/Ayra_PDF/master/tools/run.sh <name> <cmd>, not build-lock.sh directly"
        if head == "cargo":
            sub, _ = first_sub(t, ("--manifest-path", "-Z", "--config", "-C"))
            if sub == "clean":
                return "`cargo clean` is forbidden (shared target dir across worktrees; never clean)"
            if sub in CARGO_BUILD:
                return f"`cargo {sub}` must run through /home/rakib/Desktop/Ayra_PDF/master/tools/run.sh <name> cargo {sub} … (build lock + logs)"
        if head == "corepack" and len(t) > 1 and t[1] in ("pnpm", "npm", "yarn"):
            t, head = t[1:], t[1]
        if head in ("pnpm", "npm", "npx", "yarn"):
            sub, _ = first_sub(t, ("--dir", "-C", "--filter", "-F", "--prefix", "--workspace"))
            if head == "npx" or (sub and sub not in PNPM_SAFE) or (sub is None and len(t) > 1 and t[1] not in PNPM_SAFE):
                return f"`{head} {sub or ''}` must run through /home/rakib/Desktop/Ayra_PDF/master/tools/run.sh <name> {head} …"
        if head in ("gradle", "gradlew", "./gradlew"):
            return "gradle builds must run through /home/rakib/Desktop/Ayra_PDF/master/tools/run.sh"
        if head == "git":
            sub, i = first_sub(t, ("-C", "-c", "--git-dir", "--work-tree"))
            if sub == "push":
                return "`git push` is forbidden in this workspace (the owner pushes)"
            if sub == "reset" and "--hard" in t[i:]:
                return "`git reset --hard` is forbidden (use git revert / git stash)"
        if head == "rm":
            flags = "".join(a.lstrip("-") for a in t[1:] if a.startswith("-") and not a.startswith("--"))
            longf = [a for a in t[1:] if a.startswith("--")]
            recursive = "r" in flags or "R" in flags or "--recursive" in longf
            force = "f" in flags or "--force" in longf
            if recursive and force:
                targets = [a for a in t[1:] if not a.startswith("-")]
                bad = [a for a in targets if not in_allowed_rm_zone(a, cwd)]
                if bad or not targets:
                    return f"`rm -rf` outside target/ or logs/runs/ is forbidden: {' '.join(bad) or '(no path)'}"
    return None


def main():
    try:
        data = json.load(sys.stdin)
    except Exception:
        return 0
    cmd = (data.get("tool_input") or {}).get("command") or ""
    reason = check(cmd, data.get("cwd") or os.getcwd())
    if reason:
        print(f"BLOCKED by project guard: {reason}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
