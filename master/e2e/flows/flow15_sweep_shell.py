"""Flow 15: UI sweep smoke, group 2 (app shell: Sidebar, StatusBar, MainLayout). NOT one of the 12 golden-path
flows: run with `--flows 15`.

Opens a PDF, then for light and dark at 1200x800 and 1600x1000 takes a screenshot of the shell, and checks:
every control in the sidebar and the help button is a ui/ component, the current view is marked
aria-current, collapse and expand work through the buttons, and there are no console errors.
Screenshots: <out>/runs/<run-id>/screens/sweep_shell_<theme>_<WxH>.png
"""
import os
import time

import common

NUM = 15
TITLE = "UI sweep smoke: app shell"
SIZES = [(1200, 800), (1600, 1000)]
THEMES = [("light", "Light"), ("dark", "Dark")]

RAW_JS = ("return [...document.querySelectorAll('aside.sidebar button, .faq-floating-trigger button, footer.status-bar button')]"
          ".filter(e => !e.closest('.lp-btnwrap, .lp-iconwrap')).length")
NAV_JS = "return document.querySelectorAll(\"nav[aria-label='Primary Navigation'] button\").length"
CURRENT_JS = "return [...document.querySelectorAll('nav button[aria-current=page]')].map(b => b.textContent.trim())"
COLLAPSED_JS = "return document.querySelector('.app-shell').classList.contains('sidebar-collapsed')"


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
        flow.check(f"{key}: 11 navigation items", s.js(NAV_JS) == 11, str(s.js(NAV_JS)))
        flow.check(f"{key}: sidebar, help and status controls are ui/ components", s.js(RAW_JS) == 0, str(s.js(RAW_JS)))
        flow.check(f"{key}: exactly one nav item marked current", len(s.js(CURRENT_JS) or []) == 1, str(s.js(CURRENT_JS)))
        s.wait("return document.querySelectorAll('.toast').length === 0", 15)
        for size in SIZES:
            s.set_window(*size)
            time.sleep(0.5)
            path = os.path.join(flow.ctx.screens, f"sweep_shell_{key}_{size[0]}x{size[1]}.png")
            s.screenshot(path)
            flow.screens.append(path)
            flow.info(f"shot[{key}_{size[0]}x{size[1]}]", os.path.relpath(path, common.MASTER))
    # collapse and expand through the real buttons
    flow.check("collapse button present", app.click_button("", "aside.sidebar .lp-iconwrap button"), "")
    flow.check("sidebar collapses", bool(s.wait(COLLAPSED_JS, 5)), "")
    flow.check("expand button present", app.click_button("", "[data-testid=sidebar-toggle-btn]"), "")
    flow.check("sidebar expands", bool(s.wait("return !" + COLLAPSED_JS[len("return "):], 5)), "")
    errs = app.errors()
    flow.check("no console errors on the shell", not errs, " | ".join(map(str, errs))[:400])
    app.quit()
