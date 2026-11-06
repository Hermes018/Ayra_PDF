"""Flow 12: soak.

A scripted loop for --soak-minutes (default 60; the baseline uses 5) in ONE app process:
navigate (zoom presets, buttons, pan, fit) -> clear previous markups (marquee + Delete) ->
annotate (pen stroke + rectangle) -> Save (the app writes a new file and reloads it, i.e. closes
the old pdf.js document and opens the new one). PASS when: no crash (process alive, webview
answers), every save succeeds, RSS (app + WebKit children) stays within 1.5x its value after the
warm-up (5 min, or 1/5 of shorter runs), and neither temp files (app TMPDIR) nor open file
descriptors keep growing. Each Save opens the saved copy as a NEW tab (BUG-061), so after the
iteration's checks the loop CLOSES every tab it created with the tab close button
([data-testid=doc-tab-close]) and verifies only the baseline tab is left (--no-close keeps the old
behaviour for comparison). A dropped WebDriver connection is retried once and counted
(soak_retry.py, BUG-062). Optional --idle-minutes N: after the loop leave the app idle and sample
RSS every 30 s. After the loop a fresh process reopens the last saved file. Runs under 60 min
report SHORT-PASS.
"""
import os
import time

import common
import soak_retry

NUM = 12
TITLE = "Soak"
RSS_FACTOR = 1.5
FD_SLACK = 16


def _temp_files(d):
    n = 0
    for _, _, files in os.walk(d):
        n += len(files)
    return n


def _clear_markups(app, pr):
    if app.vector_nodes() <= 0:
        return True
    app.select_tool("nav", "Marquee Select")
    app.s.mouse_path([(pr["x"] + 5, pr["y"] + 5), (pr["x"] + 300, pr["y"] + 300),
                      (min(pr["x"] + pr["w"] - 5, 1150), min(pr["y"] + pr["h"] - 5, 760))])
    time.sleep(0.2)
    btn = app.s.find_text(".selection-action-bar button", "Delete")
    if btn:
        btn.click()
        time.sleep(0.2)
    return app.vector_nodes() == 0


TABS_JS = ("return [...document.querySelectorAll('[data-testid=doc-tab]')]"
           ".map(t => t.getAttribute('data-path'))")


def _tabs(app):
    return app.s.js(TABS_JS) or []


def _close_new_tabs(app, keep):
    """Close every tab not in `keep` with its close button. If the unsaved-changes dialog shows
    (the markups are already saved into the copy), choose Discard. Returns (closed, left_over)."""
    closed = 0
    for _ in range(10):
        tabs = _tabs(app)
        extra = [i for i, t in enumerate(tabs) if t not in keep]
        if not extra:
            break
        btns = app.s.find_all('[data-testid="doc-tab-close"]')
        if extra[-1] >= len(btns):
            break
        btns[extra[-1]].click()
        time.sleep(0.2)
        dlg = app.s.find('[data-testid="close-tab-discard"]')
        if dlg:
            dlg.click()
            time.sleep(0.2)
        closed += 1
    time.sleep(0.2)
    return closed, [t for t in _tabs(app) if t not in keep]


def run(flow):
    minutes = float(flow.ctx.soak_minutes)
    warmup_s = min(300.0, minutes * 60 / 5)
    tmpdir = os.path.join(flow.dir, "app-tmp")
    os.makedirs(tmpdir, exist_ok=True)
    flow.driver({"TMPDIR": tmpdir})  # the app inherits TMPDIR from tauri-driver
    path = flow.corpus_copy("standard-form.pdf")
    n = common.pdf_pages(path)
    app = flow.launch([path], expect_pages=n)
    if not flow.check("document open", app.page_count() == n, f"{app.page_count()} pages"):
        return
    app.snap_off()
    roots, pids = app.pids()
    flow.check("app process found for RSS/FD sampling", bool(roots), f"pids {pids}")

    samples = []
    saves_ok = saves_failed = 0
    first_save_error = None
    crash = None
    outputs = []
    names = set()
    clear_failed = reopen_bad = 0
    first_reopen_error = None
    no_close = bool(getattr(flow.ctx, "no_close", False))
    idle_minutes = float(getattr(flow.ctx, "idle_minutes", 0) or 0)
    base_tabs = set(_tabs(app))
    hic = soak_retry.Hiccups()
    soak_failure = None
    tabs_closed = 0
    tab_leftover = bad_file = 0
    first_file_error = None
    max_tabs = len(base_tabs)
    t_start = time.monotonic()
    it = 0
    baseline = None

    def body():
        nonlocal saves_ok, saves_failed, first_save_error, clear_failed, reopen_bad, first_reopen_error
        nonlocal tabs_closed, tab_leftover, bad_file, first_file_error, max_tabs
        # navigation
        for val in ("150", "75", "fit-width", "100"):
            app.set_select('select[data-testid="zoom-preset"]', val)
        app.s.find('button[aria-label="Zoom in"]').click()
        app.s.find('button[aria-label="Zoom out"]').click()
        app.set_select('select[data-testid="zoom-preset"]', "75")
        time.sleep(0.2)
        pr = app.page_rect(0)
        # clear the previous markups, annotate, save (save opens the output as a new tab)
        if not _clear_markups(app, pr):
            clear_failed += 1
        app.select_tool("ink", "Technical Pen A")
        x, y = pr["x"] + 80, pr["y"] + 120
        app.s.mouse_path([(x + i * 10, y + (i % 3) * 6) for i in range(12)], step_ms=8)
        app.select_tool("shapes", "Rectangle")
        app.s.mouse_path([(x, y + 60), (x + 60, y + 100), (x + 120, y + 140)], step_ms=8)
        out, msg = app.save(timeout=60)
        if out and os.path.exists(out):
            saves_ok += 1
            names.add(os.path.basename(out))
            ok_q, qmsg = common.qpdf_check(out)
            n_annots = sum(len(a) for a in common.page_annotations(out)) if ok_q else 0
            if not ok_q or common.pdf_pages(out) != n:
                bad_file += 1
                first_file_error = first_file_error or f"iteration {it}: {qmsg[:120]}"
            if out in outputs:
                outputs.remove(out)
            outputs.append(out)
            # Save opened the saved output: it must come back with its pages and exactly the two
            # markups just saved.
            back = app.wait_doc(n, 30)
            time.sleep(0.3)
            if not back or app.vector_nodes() != 2:
                reopen_bad += 1
                first_reopen_error = first_reopen_error or (
                    f"iteration {it}: pages ok={back}, {app.vector_nodes()} markups after reopen; "
                    f"saved file annots={n_annots}")
            max_tabs = max(max_tabs, len(_tabs(app)))
            if not no_close:
                closed, left = _close_new_tabs(app, base_tabs)
                tabs_closed += closed
                if left or len(_tabs(app)) != len(base_tabs):
                    tab_leftover += 1
                    first_file_error = first_file_error or f"iteration {it}: tabs left open {left}"
                # the saved copy is closed; delete older outputs to keep disk bounded
                while len(outputs) > 2:
                    old = outputs.pop(0)
                    for f in (old, old + ".studio.json", old + ".scales.studio.json"):
                        if os.path.exists(f) and f != path:
                            os.remove(f)
            else:
                while len(outputs) > 2:
                    old = outputs.pop(0)
                    for f in (old, old + ".studio.json", old + ".scales.studio.json"):
                        if os.path.exists(f) and f != path:
                            os.remove(f)
        else:
            saves_failed += 1
            first_save_error = first_save_error or f"iteration {it}: {msg}"
        app.select_tool("nav", "Pan Canvas")
        app.s.mouse_path([(600, 400), (560, 360), (520, 320)], step_ms=8)

    while time.monotonic() - t_start < minutes * 60:
        it += 1
        try:
            soak_retry.run_iteration(it, body, app.alive, hic)
        except soak_retry.SoakFailure as e:
            soak_failure = str(e)
            crash = soak_failure if not app.alive() else None
            if crash is None:
                saves_failed += 1
                first_save_error = first_save_error or soak_failure
            break
        except Exception as e:  # noqa: BLE001
            if not app.alive():
                crash = f"iteration {it}: webview gone ({type(e).__name__}: {e})"
                break
            first_save_error = first_save_error or f"iteration {it}: {type(e).__name__}: {e}"
            saves_failed += 1
        roots_now, pids_now = app.pids()
        if not roots_now:
            crash = f"iteration {it}: app process exited"
            break
        el = time.monotonic() - t_start
        s = {"t": round(el, 1), "it": it, "rss_kb": common.rss_kb(pids_now), "fds": common.fd_count(pids_now),
             "tmp_files": _temp_files(tmpdir), "procs": len(pids_now), "tabs": len(_tabs(app)) if app.alive() else -1}
        samples.append(s)
        if baseline is None and el >= warmup_s:
            baseline = s
        if it % 10 == 0:
            print(f"      soak t={s['t']}s it={it} rss={s['rss_kb'] // 1024} MB fds={s['fds']} "
                  f"tmp={s['tmp_files']} tabs={s['tabs']} saves ok/failed={saves_ok}/{saves_failed} "
                  f"hiccups={len(hic.items)}", flush=True)

    # optional idle phase: no input, RSS sampled every 30 s
    idle = []
    if idle_minutes > 0 and crash is None and soak_failure is None:
        t_idle = time.monotonic()
        while time.monotonic() - t_idle < idle_minutes * 60:
            time.sleep(30)
            _, pids_now = app.pids()
            idle.append({"t": round(time.monotonic() - t_idle, 1), "rss_kb": common.rss_kb(pids_now)})
            print(f"      idle t={idle[-1]['t']}s rss={idle[-1]['rss_kb'] // 1024} MB", flush=True)

    last = samples[-1] if samples else None
    baseline = baseline or (samples[len(samples) // 5] if samples else None)
    peak_after = max((x["rss_kb"] for x in samples if baseline and x["t"] >= baseline["t"]), default=0)
    flow.info("soak", {"minutes": minutes, "warmup_s": warmup_s, "iterations": it, "saves_ok": saves_ok,
                       "saves_failed": saves_failed, "first_save_error": first_save_error,
                       "distinct_output_names": sorted(names)[:5], "clear_failed": clear_failed,
                       "reopen_bad": reopen_bad, "no_close": no_close, "tabs_closed": tabs_closed,
                       "max_tabs_open": max_tabs, "tab_leftover_iterations": tab_leftover,
                       "bad_saved_files": bad_file, "idle_samples": idle,
                       "baseline": baseline, "last": last, "peak_rss_kb_after_warmup": peak_after,
                       "samples": samples[:: max(1, len(samples) // 40)]})
    flow.check("no crash during the soak", crash is None and app.alive(), crash or "")
    flow.check("every iteration completed (annotate + save; no app or WebDriver error)",
               saves_failed == 0 and saves_ok > 0,
               f"{saves_ok} ok, {saves_failed} failed; first error: {first_save_error}")
    flow.check("every close/reopen (save -> reload) shows the document and its 2 markups",
               reopen_bad == 0 and saves_ok > 0, f"{reopen_bad} bad reopen(s); first: {first_reopen_error}")
    hv = soak_retry.verdict(hic, it, soak_failure)
    flow.check("driver connection drops: app alive, retried once, within 1 % of iterations",
               hv["ok"], hv["detail"])
    flow.warn("no WebDriver driver hiccups (BUG-062; copy to FLAKY.md if any)", not hv["warn"], hv["detail"])
    if hv["flaky"]:
        flow.metrics.setdefault("flaky", []).extend(hv["flaky"])
    flow.check("every saved file passes qpdf --check and has all pages", bad_file == 0 and saves_ok > 0,
               f"{bad_file} bad; first: {first_file_error}")
    if not no_close:
        flow.check("the tabs each iteration created are closed (only the baseline tab is left)",
                   tab_leftover == 0 and tabs_closed >= saves_ok,
                   f"{tabs_closed} tab(s) closed, {tab_leftover} iteration(s) left tabs; max open {max_tabs}")
    flow.check("clearing the previous markups (marquee + Delete) worked every iteration",
               clear_failed == 0, f"{clear_failed} of {it} iteration(s) left markups behind")
    if baseline and last:
        flow.check(f"RSS stays within {RSS_FACTOR}x of the post-warm-up value",
                   peak_after <= RSS_FACTOR * baseline["rss_kb"],
                   f"baseline {baseline['rss_kb'] // 1024} MB at {baseline['t']} s, peak after "
                   f"{peak_after // 1024} MB, last {last['rss_kb'] // 1024} MB")
        flow.check("no file-descriptor leak", last["fds"] <= baseline["fds"] + FD_SLACK,
                   f"fds {baseline['fds']} -> {last['fds']}")
        flow.check("no growing temp files", last["tmp_files"] <= baseline["tmp_files"] + 2,
                   f"app TMPDIR files {baseline['tmp_files']} -> {last['tmp_files']}")
    else:
        flow.check("enough samples for RSS/FD/temp trends", False, f"{len(samples)} samples")
    flow.check("original unchanged", flow.original_unchanged(path), "")
    errs = app.errors()
    flow.warn("no console errors", not errs, f"{len(errs)} error(s): " + " | ".join(sorted(set(errs)))[:300])
    app.quit()
    # process-level close and reopen of the last saved document
    if outputs and os.path.exists(outputs[-1]):
        app2 = flow.launch([outputs[-1]], expect_pages=n)
        time.sleep(1.0)
        flow.check("after the soak, a fresh process reopens the last saved file with its markups",
                   app2.page_count() == n and app2.vector_nodes() == 2,
                   f"{app2.page_count()} pages, {app2.vector_nodes()} markups")
        app2.quit()
    if minutes < 60:
        flow.short = f"SHORT-PASS: soak ran {minutes:g} min; the PASS criterion needs 60 min (--soak-minutes 60)"
