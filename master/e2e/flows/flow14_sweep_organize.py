"""Flow 14: UI sweep smoke, group 1 (Organize). NOT part of the 12 golden-path flows: run with `--flows 14`.

Opens a PDF, goes to Organize, and for light and dark at 1200x800 and 1600x1000 takes a screenshot and
checks the page grid, the Tools column controls and that there are no console errors. Light/dark is
switched with the app's own persisted preference (localStorage) and a reload.
Screenshots: <out>/screens/sweep_organize_<theme>_<WxH>.png
"""
import os
import time

import common

NUM = 14
TITLE = "UI sweep smoke: Organize"
SIZES = [(1200, 800), (1600, 1000)]
THEMES = [("light", "Light"), ("dark", "Dark")]

GRID_JS = "return document.querySelectorAll('.grid-layout .page-card').length"
RAW_JS = ("return [...document.querySelectorAll('.view-container button, .view-container input, .view-container select')]"
          ".filter(e => !e.closest('.lp-btnwrap, .lp-iconwrap, .lp-field')).length")


def _goto(app, label):
    btn = app.s.find_text("nav[aria-label='Primary Navigation'] button", label)
    if btn:
        btn.click()
        time.sleep(0.4)
    return bool(btn)


def run(flow):
    src = flow.corpus_copy("standard-form.pdf")
    n = common.pdf_pages(src)
    app = flow.launch([src], expect_pages=n)
    s = app.s
    for key, pref in THEMES:
        s.js("window.__preReload = true; localStorage.setItem('localpdf_theme', arguments[0]); setTimeout(() => location.reload(), 50)", pref)
        time.sleep(1.0)
        s.wait("return !window.__preReload && document.readyState === 'complete' && !!document.querySelector(\"nav[aria-label='Primary Navigation']\")", 30)
        s.js(common.HOOK_JS)
        if not flow.check(f"{key}: Organize reachable", _goto(app, "Organize"), ""):
            break
        cards = s.wait(GRID_JS, 30)
        flow.check(f"{key}: page grid shows pages", bool(cards) and cards == n, f"{cards} cards for {n} pages")
        flow.check(f"{key}: controls are ui/ components (no raw button/input/select)", s.js(RAW_JS) == 0, str(s.js(RAW_JS)))
        # let the "Operation ..." toasts from the open/probe jobs expire (6 s) so the shot shows the screen
        s.wait("return document.querySelectorAll('.toast').length === 0", 15)
        for size in SIZES:
            s.set_window(*size)
            time.sleep(0.5)
            path = os.path.join(flow.ctx.screens, f"sweep_organize_{key}_{size[0]}x{size[1]}.png")
            s.screenshot(path)
            flow.screens.append(path)
            flow.info(f"shot[{key}_{size[0]}x{size[1]}]", os.path.relpath(path, common.MASTER))
    errs = app.errors()
    flow.check("no console errors on Organize", not errs, " | ".join(map(str, errs))[:400])
    app.quit()
