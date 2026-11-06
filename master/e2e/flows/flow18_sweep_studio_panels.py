"""Flow 18: UI sweep smoke, group 5 (Studio panels: CalibrationModal, ShortcutSettings). NOT one of the 12
golden-path flows: run with `--flows 18`. (PropertyPill is deferred, see docs/UI_BACKLOG.md.)

For light and dark at 1200x800 and 1600x1000: the calibration dialog (two-point drag with the calibrate tool)
and the shortcuts dialog are opened, checked (ui/ controls, focus inside, Escape closes) and screenshotted.
Screenshots: <out>/screens/sweep_calibration_<theme>_<WxH>.png and sweep_shortcuts_<theme>_<WxH>.png
"""
import os
import time

import common

NUM = 18
TITLE = "UI sweep smoke: Studio panels"
SIZES = [(1200, 800), (1600, 1000)]
THEMES = [("light", "Light"), ("dark", "Dark")]

CAL = '[data-testid="calibration-dialog"]'
RAW_JS = "return [...document.querySelectorAll(arguments[0] + ' button, ' + arguments[0] + ' input, ' + arguments[0] + ' select')].filter(e => !e.closest('.lp-btnwrap, .lp-iconwrap, .lp-field')).length"
FOCUS_JS = "const d = document.querySelector(arguments[0]); return d ? d.contains(document.activeElement) : null"


def _shot(flow, s, name, key, size):
    path = os.path.join(flow.ctx.screens, f"sweep_{name}_{key}_{size[0]}x{size[1]}.png")
    s.screenshot(path)
    flow.screens.append(path)
    flow.info(f"shot[{name}_{key}_{size[0]}x{size[1]}]", os.path.relpath(path, common.MASTER))


def run(flow):
    path = flow.corpus_copy("standard-text.pdf")
    app = flow.launch([path], expect_pages=1)
    s = app.s
    for key, pref in THEMES:
        s.js("window.__preReload = true; localStorage.setItem('localpdf_theme', arguments[0]); setTimeout(() => location.reload(), 50)", pref)
        time.sleep(1.0)
        s.wait("return !window.__preReload && document.readyState === 'complete' && document.querySelectorAll('.spatial-page').length > 0", 30)
        s.js(common.HOOK_JS)
        app.wait_doc(1)
        app.snap_off()
        app.set_select('select[aria-label="Zoom Level Preset"]', "75")
        time.sleep(0.3)
        pr = app.page_rect(0)
        X, Y = int(pr["x"]) + 60, int(pr["y"]) + 60
        flow.check(f"{key}: select calibrate tool", app.select_tool("measure", "Calibrate Page Scale"), "")
        s.mouse_path([(X, Y), (X + 150, Y), (X + 300, Y)])
        opened = s.wait(f"return !!document.querySelector('{CAL}')", 5)
        if flow.check(f"{key}: calibration dialog opens", opened, ""):
            time.sleep(0.5)
            flow.check(f"{key}: calibration controls are ui/ components", s.js(RAW_JS, CAL) == 0, str(s.js(RAW_JS, CAL)))
            flow.check(f"{key}: focus is inside the calibration dialog", s.js(FOCUS_JS, CAL) is True, "")
            for size in SIZES:
                s.set_window(*size)
                time.sleep(0.5)
                _shot(flow, s, "calibration", key, size)
            s.keys(common.wd.KEY_ESCAPE)
            flow.check(f"{key}: Escape closes the calibration dialog", bool(s.wait(f"return !document.querySelector('{CAL}')", 5)), "")
        # shortcuts dialog
        opened = app.click_button("", 'button[aria-label="Shortcuts Settings"]')
        flow.check(f"{key}: shortcuts dialog opens", opened and bool(s.wait("return !!document.querySelector('.shortcuts-dialog')", 5)), "")
        time.sleep(0.5)
        flow.check(f"{key}: shortcuts controls are ui/ components", s.js(RAW_JS, ".shortcuts-dialog") == 0, str(s.js(RAW_JS, ".shortcuts-dialog")))
        for size in SIZES:
            s.set_window(*size)
            time.sleep(0.5)
            _shot(flow, s, "shortcuts", key, size)
        s.keys(common.wd.KEY_ESCAPE)
        flow.check(f"{key}: Escape closes the shortcuts dialog", bool(s.wait("return !document.querySelector('.shortcuts-dialog')", 5)), "")
    errs = app.errors()
    flow.check("no console errors", not errs, " | ".join(map(str, errs))[:400])
    app.quit()
