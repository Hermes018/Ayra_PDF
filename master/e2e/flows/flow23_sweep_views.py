"""Flow 23: UI sweep smoke for the tool views outside the main flows (grows with each sweep group).
NOT one of the 12 golden-path flows: run with `--flows 23`.

For each view in VIEWS, for light and dark at 1200x800 and 1600x1000: open it from the sidebar, check that its
controls are ui/ components and screenshot it. No console errors.
Screenshots: <out>/screens/sweep_view-<name>_<theme>_<WxH>.png
"""
import os
import time

import common

NUM = 23
TITLE = "UI sweep smoke: tool views"
SIZES = [(1200, 800), (1600, 1000)]
THEMES = [("light", "Light"), ("dark", "Dark")]
# (sidebar label, short name, css scope for the raw-control check)
VIEWS = [("Convert", "convert", ".convert-container .tab-bar"),
         ("Optimize", "optimize", ".view-container"),
         ("Edit", "edit", ".view-container")]

RAW_JS = ("return [...document.querySelectorAll(arguments[0] + ' button, ' + arguments[0] + ' input, ' + arguments[0] + ' select')]"
          ".filter(e => !e.closest('.lp-btnwrap, .lp-iconwrap, .lp-field, .lp-swatchwrap')).length")


def run(flow):
    path = flow.corpus_copy("standard-text.pdf")
    app = flow.launch([path], expect_pages=1)
    s = app.s
    for key, pref in THEMES:
        s.js("window.__preReload = true; localStorage.setItem('localpdf_theme', arguments[0]); setTimeout(() => location.reload(), 50)", pref)
        time.sleep(1.0)
        s.wait("return !window.__preReload && document.readyState === 'complete' && !!document.querySelector(\"nav[aria-label='Primary Navigation']\")", 30)
        s.js(common.HOOK_JS)
        for label, name, scope in VIEWS:
            btn = s.find_text("nav[aria-label='Primary Navigation'] button", label)
            if not flow.check(f"{key}: {label} reachable", bool(btn), ""):
                continue
            btn.click()
            time.sleep(0.8)
            flow.check(f"{key}: {label} controls are ui/ components", s.js(RAW_JS, scope) == 0, str(s.js(RAW_JS, scope)))
            if name in ("edit", "optimize"):
                clipped = s.js("const c = document.querySelector('.view-container .sidebar'); if (!c) return null;"
                               " return [...c.querySelectorAll('input, select, button')].filter(e => e.getBoundingClientRect().right > c.getBoundingClientRect().right + 1).length")
                flow.check(f"{key}: {label} tools column has no clipped controls", clipped == 0, str(clipped))
            s.wait("return document.querySelectorAll('.toast').length === 0", 15)
            for size in SIZES:
                s.set_window(*size)
                time.sleep(0.5)
                p = os.path.join(flow.ctx.screens, f"sweep_view-{name}_{key}_{size[0]}x{size[1]}.png")
                s.screenshot(p)
                flow.screens.append(p)
                flow.info(f"shot[{name}_{key}_{size[0]}x{size[1]}]", os.path.relpath(p, common.MASTER))
    errs = app.errors()
    flow.check("no console errors", not errs, " | ".join(map(str, errs))[:400])
    app.quit()
