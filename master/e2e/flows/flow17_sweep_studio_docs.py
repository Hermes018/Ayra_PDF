"""Flow 17: UI sweep smoke, group 4 (Studio documents: DocumentTabs, FavoritesBar, SplitLayout). NOT one of the
12 golden-path flows: run with `--flows 17`.

Opens two PDFs (two tabs), then for light and dark at 1200x800 and 1600x1000 takes a screenshot of the Studio
with its tab strip and pinned-tools bar. Checks: both tabs are ui/ Buttons with role=tab and exactly one
selected; the tab close buttons and the pin bar controls are ui/ components; switching tabs by click works;
no console errors. (The unsaved-changes close dialog is covered by StudioTabs.test.ts.)
Screenshots: <out>/runs/<run-id>/screens/sweep_studio-docs_<theme>_<WxH>.png
"""
import os
import time

import common

NUM = 17
TITLE = "UI sweep smoke: Studio documents"
SIZES = [(1200, 800), (1600, 1000)]
THEMES = [("light", "Light"), ("dark", "Dark")]

RAW_JS = ("return [...document.querySelectorAll('.doc-tabs button, .favorites-bar button, .split-toggle button')]"
          ".filter(e => !e.closest('.lp-btnwrap, .lp-iconwrap')).length")
TABS_JS = "return [...document.querySelectorAll('[data-testid=doc-tab]')].map(t => [t.getAttribute('role'), t.getAttribute('aria-selected'), t.tagName])"


def run(flow):
    a = flow.corpus_copy("standard-form.pdf")
    b = flow.corpus_copy("standard-text.pdf")
    n = common.pdf_pages(a)
    app = flow.launch([a, b], expect_pages=n)
    s = app.s
    for key, pref in THEMES:
        s.js("window.__preReload = true; localStorage.setItem('localpdf_theme', arguments[0]); setTimeout(() => location.reload(), 50)", pref)
        time.sleep(1.0)
        s.wait("return !window.__preReload && document.readyState === 'complete' && document.querySelectorAll('[data-testid=doc-tab]').length > 0", 30)
        s.js(common.HOOK_JS)
        tabs = s.js(TABS_JS) or []
        flow.check(f"{key}: two tabs, role=tab on a native button", len(tabs) == 2 and all(t[0] == "tab" and t[2] == "BUTTON" for t in tabs), str(tabs))
        flow.check(f"{key}: exactly one tab selected", sum(1 for t in tabs if t[1] == "true") == 1, str(tabs))
        flow.check(f"{key}: tab, close and pin controls are ui/ components", s.js(RAW_JS) == 0, str(s.js(RAW_JS)))
        second = s.find_text("[data-testid=doc-tab]", "standard-text")
        if second:
            second.click()
            time.sleep(0.8)
        sel = s.js("return [...document.querySelectorAll('[data-testid=doc-tab][aria-selected=true]')].map(t => t.textContent.trim())")
        flow.check(f"{key}: clicking a tab selects it", bool(sel) and "standard-text" in sel[0], str(sel))
        s.wait("return document.querySelectorAll('.toast').length === 0", 15)
        for size in SIZES:
            s.set_window(*size)
            time.sleep(0.5)
            path = os.path.join(flow.ctx.screens, f"sweep_studio-docs_{key}_{size[0]}x{size[1]}.png")
            s.screenshot(path)
            flow.screens.append(path)
            flow.info(f"shot[{key}_{size[0]}x{size[1]}]", os.path.relpath(path, common.MASTER))
    errs = app.errors()
    flow.check("no console errors", not errs, " | ".join(map(str, errs))[:400])
    app.quit()
