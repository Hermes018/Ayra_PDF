"""Flow 20: UI sweep smoke, group 7 (AnnotationLedger drawer and its dialogs). NOT one of the 12 golden-path flows:
run with `--flows 20`.

For light and dark at 1200x800 and 1600x1000: the Ledger drawer is opened and screenshotted; the Paste-to-pages
dialog is opened (focus inside, Escape closes) and screenshotted. Checks that every control in the drawer and the
dialog is a ui/ component and that there are no console errors.
Screenshots: <out>/runs/<run-id>/screens/sweep_ledger_<theme>_<WxH>.png and sweep_ledger-dialog_<theme>_<WxH>.png
"""
import os
import time

import common

NUM = 20
TITLE = "UI sweep smoke: Ledger drawer"
SIZES = [(1200, 800), (1600, 1000)]
THEMES = [("light", "Light"), ("dark", "Dark")]

DRAWER = '[aria-label="Annotation History and Quantity Takeoff Drawer"]'
DIALOG = '[aria-label="Multi-page advanced paste"]'
RAW_JS = ("return [...document.querySelectorAll(arguments[0] + ' button, ' + arguments[0] + ' input, ' + arguments[0] + ' select')]"
          ".filter(e => !e.closest('.lp-btnwrap, .lp-iconwrap, .lp-field')).length")
FOCUS_JS = "const d = document.querySelector(arguments[0]); return d ? d.contains(document.activeElement) : null"


def _shot(flow, s, name, key, size):
    p = os.path.join(flow.ctx.screens, f"sweep_{name}_{key}_{size[0]}x{size[1]}.png")
    s.screenshot(p)
    flow.screens.append(p)
    flow.info(f"shot[{name}_{key}_{size[0]}x{size[1]}]", os.path.relpath(p, common.MASTER))


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
        btn = s.find('button[aria-label^="Ledger"]')
        if not flow.check(f"{key}: Ledger button present", bool(btn), ""):
            break
        btn.click()
        flow.check(f"{key}: drawer opens", bool(s.wait(f"return !!document.querySelector('{DRAWER}')", 5)), "")
        time.sleep(0.4)
        flow.check(f"{key}: drawer controls are ui/ components", s.js(RAW_JS, DRAWER) == 0, str(s.js(RAW_JS, DRAWER)))
        for size in SIZES:
            s.set_window(*size)
            time.sleep(0.5)
            _shot(flow, s, "ledger", key, size)
        flow.check(f"{key}: Paste to pages clicked", app.click_button("Paste to pages", f"{DRAWER} button"), "")
        flow.check(f"{key}: dialog opens with focus inside", bool(s.wait(f"return !!document.querySelector('{DIALOG}')", 5)) and s.js(FOCUS_JS, DIALOG) is True, "")
        flow.check(f"{key}: dialog controls are ui/ components", s.js(RAW_JS, DIALOG) == 0, str(s.js(RAW_JS, DIALOG)))
        time.sleep(0.4)
        for size in SIZES:
            s.set_window(*size)
            time.sleep(0.5)
            _shot(flow, s, "ledger-dialog", key, size)
        s.keys(common.wd.KEY_ESCAPE)
        flow.check(f"{key}: Escape closes the dialog", bool(s.wait(f"return !document.querySelector('{DIALOG}')", 5)), "")
        app.click_button("", f"{DRAWER} .lp-iconwrap button")  # close the drawer
    errs = app.errors()
    flow.check("no console errors", not errs, " | ".join(map(str, errs))[:400])
    app.quit()
