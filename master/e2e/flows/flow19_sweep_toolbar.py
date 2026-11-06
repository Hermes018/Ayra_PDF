"""Flow 19: UI sweep smoke, group 6 (Studio toolbar: DockableToolbar; RadialMenu and MeasureOverlay were already
clean). NOT one of the 12 golden-path flows: run with `--flows 19`.

For light and dark at 1200x800 and 1600x1000: the ink category drawer is opened and screenshotted; checks that
the toolbar controls are ui/ components and the drawer lists tools as ui/ Buttons; selecting a tool from the drawer
works. Also: in the shortcuts dialog, Record then Escape cancels the capture and leaves the dialog open.
Screenshots: <out>/runs/<run-id>/screens/sweep_toolbar_<theme>_<WxH>.png
"""
import os
import time

import common

NUM = 19
TITLE = "UI sweep smoke: Studio toolbar"
SIZES = [(1200, 800), (1600, 1000)]
THEMES = [("light", "Light"), ("dark", "Dark")]

RAW_JS = ("return [...document.querySelectorAll('.dockable-toolbar button')]"
          ".filter(e => !e.closest('.lp-btnwrap, .lp-iconwrap')).length")
POPOVER = "[data-testid=toolbar-category-popover]"


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
        btn = s.find('[data-testid="category-btn-ink"]')
        if not flow.check(f"{key}: ink category button present", bool(btn), ""):
            break
        btn.click()
        flow.check(f"{key}: drawer opens", bool(s.wait(f"return !!document.querySelector('{POPOVER}')", 5)), "")
        time.sleep(0.4)
        flow.check(f"{key}: toolbar and drawer controls are ui/ components", s.js(RAW_JS) == 0, str(s.js(RAW_JS)))
        for size in SIZES:
            s.set_window(*size)
            time.sleep(0.5)
            p = os.path.join(flow.ctx.screens, f"sweep_toolbar_{key}_{size[0]}x{size[1]}.png")
            s.screenshot(p)
            flow.screens.append(p)
            flow.info(f"shot[{key}_{size[0]}x{size[1]}]", os.path.relpath(p, common.MASTER))
        flow.check(f"{key}: a tool can be picked from the drawer", app.select_tool("ink", "Pen"), "")
    # Escape while recording a shortcut cancels the capture only
    opened = app.click_button("", 'button[aria-label="Shortcuts settings"]')
    flow.check("shortcuts dialog opens", opened and bool(s.wait("return !!document.querySelector('.shortcuts-dialog')", 5)), "")
    flow.check("Record button clicked", app.click_button("Record", ".shortcuts-dialog td button"), "")
    flow.check("capture is armed", bool(s.wait("return [...document.querySelectorAll('.shortcuts-dialog td button')].some(b => b.textContent.includes('Press a key chord'))", 5)), "")
    s.keys(common.wd.KEY_ESCAPE)
    time.sleep(0.4)
    still = s.js("return !!document.querySelector('.shortcuts-dialog')")
    armed = s.js("return [...document.querySelectorAll('.shortcuts-dialog td button')].some(b => b.textContent.includes('Press a key chord'))")
    flow.check("Escape cancels the capture and keeps the dialog open", still and not armed, f"dialog {still}, still armed {armed}")
    errs = app.errors()
    flow.check("no console errors", not errs, " | ".join(map(str, errs))[:400])
    app.quit()
