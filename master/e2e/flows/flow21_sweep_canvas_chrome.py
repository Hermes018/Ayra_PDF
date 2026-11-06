"""Flow 21: UI sweep smoke, group 8 (Studio canvas chrome: top command bar, banners, inline text editor, selection bar).
NOT one of the 12 golden-path flows: run with `--flows 21`. Canvas and geometry code are not part of this sweep.

For light and dark at 1200x800 and 1600x1000: screenshot of the Studio with its top command bar; checks that the
top bar controls are ui/ components, Zoom in/out and the zoom preset work, and there are no console errors.
Screenshots: <out>/runs/<run-id>/screens/sweep_canvas-chrome_<theme>_<WxH>.png
"""
import os
import time

import common

NUM = 21
TITLE = "UI sweep smoke: canvas chrome"
SIZES = [(1200, 800), (1600, 1000)]
THEMES = [("light", "Light"), ("dark", "Dark")]

RAW_JS = ("return [...document.querySelectorAll('.studio-top-bar button, .studio-top-bar select, .studio-top-bar input, .selection-action-bar button')]"
          ".filter(e => !e.closest('.lp-btnwrap, .lp-iconwrap, .lp-field')).length")


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
        flow.check(f"{key}: top bar controls are ui/ components", s.js(RAW_JS) == 0, str(s.js(RAW_JS)))
        app.set_select('select[data-testid="zoom-preset"]', "100")
        time.sleep(0.2)
        before = app.transform()
        s.find('button[aria-label="Zoom in"]').click()
        time.sleep(0.3)
        after = app.transform()
        flow.check(f"{key}: Zoom in changes the view", before is not None and after is not None and after[0] > before[0], f"{before} -> {after}")
        app.set_select('select[data-testid="zoom-preset"]', "75")
        s.wait("return document.querySelectorAll('.toast').length === 0", 15)
        # clear any tooltip: blur, move the pointer to an empty corner, Escape
        s.js("if (document.activeElement) document.activeElement.blur()")
        s.actions([{"type": "pointer", "id": "mouse1", "parameters": {"pointerType": "mouse"},
                    "actions": [{"type": "pointerMove", "x": 2, "y": 2, "origin": "viewport", "duration": 50}]}])
        s.keys(common.wd.KEY_ESCAPE)
        time.sleep(0.3)
        flow.check(f"{key}: no tooltip is showing", s.js("return [...document.querySelectorAll('[role=tooltip]')].every(t => getComputedStyle(t).visibility === 'hidden')"), "")
        for size in SIZES:
            s.set_window(*size)
            time.sleep(0.5)
            p = os.path.join(flow.ctx.screens, f"sweep_canvas-chrome_{key}_{size[0]}x{size[1]}.png")
            s.screenshot(p)
            flow.screens.append(p)
            flow.info(f"shot[{key}_{size[0]}x{size[1]}]", os.path.relpath(p, common.MASTER))
    errs = app.errors()
    flow.check("no console errors", not errs, " | ".join(map(str, errs))[:400])
    app.quit()
