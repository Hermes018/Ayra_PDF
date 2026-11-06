"""Flow 16: UI sweep smoke, group 3 (global overlays: ToastContainer, FaqModal). NOT one of the 12 golden-path
flows: run with `--flows 16`.

For light and dark at 1200x800 and 1600x1000: Organize -> "Rotate pages" (page 2) raises a toast (completed or
failed, both are shown); screenshot it; "Clear" dismisses it. Then Help and FAQ opens the dialog (focus inside,
first topic expands), screenshot, Escape closes it. No console errors.
Screenshots: <out>/screens/sweep_toast_<theme>_<WxH>.png and sweep_faq_<theme>_<WxH>.png
"""
import os
import time

import common

NUM = 16
TITLE = "UI sweep smoke: toast and FAQ overlays"
SIZES = [(1200, 800), (1600, 1000)]
THEMES = [("light", "Light"), ("dark", "Dark")]

TOASTS_JS = "document.querySelectorAll('.toast').length"
RAW_TOAST_JS = "return [...document.querySelectorAll('.toast button')].filter(b => !b.closest('.lp-btnwrap')).length"
DIALOG_FOCUS_JS = "const d = document.querySelector('[role=dialog]'); return d ? d.contains(document.activeElement) : null"


def _shot(flow, s, name, key, size):
    path = os.path.join(flow.ctx.screens, f"sweep_{name}_{key}_{size[0]}x{size[1]}.png")
    s.screenshot(path)
    flow.screens.append(path)
    flow.info(f"shot[{name}_{key}_{size[0]}x{size[1]}]", os.path.relpath(path, common.MASTER))


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
        btn = s.find_text("nav[aria-label='Primary Navigation'] button", "Organize")
        if not flow.check(f"{key}: Organize reachable", bool(btn), ""):
            break
        btn.click()
        s.wait("return document.querySelectorAll('.grid-layout .page-card').length > 0", 30)
        for size in SIZES:
            s.set_window(*size)
            time.sleep(0.4)
            s.wait(f"return {TOASTS_JS} === 0", 15)
            app.set_input("[data-testid=organize-rotate-pages-input]", "2")
            flow.check(f"{key} {size[0]}x{size[1]}: Rotate pages clicked", app.click_button("Rotate pages", "[data-testid=organize-rotate-pages-btn]"), "")
            shown = s.wait(f"return {TOASTS_JS} > 0", 20)
            flow.check(f"{key} {size[0]}x{size[1]}: a toast appears", bool(shown), "")
            flow.check(f"{key} {size[0]}x{size[1]}: toast buttons are ui/ components", s.js(RAW_TOAST_JS) == 0, str(s.js(RAW_TOAST_JS)))
            time.sleep(0.6)  # entry animation
            _shot(flow, s, "toast", key, size)
            app.click_button("Clear", ".toast button")
            flow.check(f"{key} {size[0]}x{size[1]}: Clear dismisses the toast", bool(s.wait(f"return {TOASTS_JS} === 0", 5)), "")
            flow.check(f"{key} {size[0]}x{size[1]}: Help and FAQ opens", app.click_button("Help and FAQ", "aside.sidebar button"), "")
            flow.check(f"{key} {size[0]}x{size[1]}: dialog opens with focus inside", s.wait(DIALOG_FOCUS_JS, 5) is True, "")
            app.click_button("", ".faq-item button")
            time.sleep(0.6)
            _shot(flow, s, "faq", key, size)
            s.keys(common.wd.KEY_ESCAPE)
            closed = s.wait("return !document.querySelector('[role=dialog]')", 5)
            flow.check(f"{key} {size[0]}x{size[1]}: Escape closes the dialog", bool(closed), "")
    errs = app.errors()
    flow.check("no console errors", not errs, " | ".join(map(str, errs))[:400])
    app.quit()
