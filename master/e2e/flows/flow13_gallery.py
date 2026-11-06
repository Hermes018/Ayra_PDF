"""Flow 13: component gallery smoke (C0). NOT part of the 12 golden-path flows: run it with `--flows 13`.

Needs a binary built with VITE_LPDF_GALLERY=1 (the gallery is compiled out of release builds). Opens
/gallery in one app instance, then for light and dark at 1200x800 and 1600x1000: screenshot, a real
keyboard check (Tab order, Enter and Space on a Button), the disabled-reason tooltip on hover, and the
Dialog (opens, focus inside, Escape closes it through commandEngine). Fails on any console error.
Screenshots: <out>/runs/<run-id>/screens/gallery_<theme>_<WxH>_<state>.png
"""
import os
import time

import common
import wd

NUM = 13
TITLE = "Component gallery smoke"
SIZES = [(1200, 800), (1600, 1000)]
THEMES = ["light", "dark"]
KEY_TAB = ""
KEY_SPACE = " "

READY_JS = "return document.documentElement.dataset.galleryReady === '1' && !!document.querySelector('[data-gallery]')"
ACTIVE_JS = "const e = document.activeElement; return e ? (e.dataset.testid || e.getAttribute('aria-label') || e.tagName) : null"
RECT_JS = ("const e = document.querySelector(arguments[0]); if (!e) return null; e.scrollIntoView({block: 'center'});"
           " const r = e.getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]")
TOOLTIP_JS = ("const e = document.querySelector(arguments[0]); if (!e) return null;"
              " const t = document.getElementById(e.getAttribute('aria-describedby').split(' ')[0]);"
              " return t ? [t.textContent, getComputedStyle(t).opacity, getComputedStyle(t).visibility] : null")
CLICKS_JS = "return document.querySelector('[data-testid=g-clicks]').textContent"
DIALOG_JS = "const d = document.querySelector('[role=dialog]'); return d ? d.contains(document.activeElement) : null"


def _shot(flow, s, theme, size, state):
    path = os.path.join(flow.ctx.screens, f"gallery_{theme}_{size[0]}x{size[1]}_{state}.png")
    s.screenshot(path)
    flow.screens.append(path)
    flow.info(f"shot[{theme}_{size[0]}x{size[1]}_{state}]", os.path.relpath(path, common.MASTER))


def _hover(s, selector):
    xy = s.js(RECT_JS, selector)
    s.actions([{"type": "pointer", "id": "mouse1", "parameters": {"pointerType": "mouse"},
                "actions": [{"type": "pointerMove", "x": int(xy[0]), "y": int(xy[1]), "origin": "viewport", "duration": 100}]}])


def run(flow):
    # The Studio root never appears on /gallery, so do not wait for it long at launch.
    app = flow.launch([], timeout=6, retries=1)
    s = app.s
    origin = s.js("return location.origin")
    for theme in THEMES:
        s.call("POST", "/url", {"url": f"{origin}/gallery?theme={theme}"})
        ok = s.wait(READY_JS, 20)
        s.js(common.HOOK_JS)  # a page load drops the hook; reinstall (the gallery also records early errors itself)
        if not flow.check(f"{theme}: gallery route renders", ok, f"url={s.js('return location.href')}"):
            break
        for size in SIZES:
            s.set_window(*size)
            s.js("window.scrollTo(0, 0); document.querySelector('.gallery').scrollTop = 0; document.activeElement.blur()")
            _shot(flow, s, theme, size, "default")

            # real keyboard: Tab to the first Button, then Enter and Space each activate it once
            before = int(s.js(CLICKS_JS).split(":")[1])
            for _ in range(6):
                s.keys(KEY_TAB)
                if s.js(ACTIVE_JS) == "g-btn-primary":
                    break
            flow.check(f"{theme} {size[0]}x{size[1]}: Tab reaches the primary Button", s.js(ACTIVE_JS) == "g-btn-primary", str(s.js(ACTIVE_JS)))
            _shot(flow, s, theme, size, "focus")
            s.keys(wd.KEY_ENTER)
            mid = int(s.js(CLICKS_JS).split(":")[1])
            flow.check(f"{theme} {size[0]}x{size[1]}: Enter activates the Button", mid - before == 1, f"{before} -> {mid}")
            s.keys(KEY_SPACE)
            after = int(s.js(CLICKS_JS).split(":")[1])
            flow.check(f"{theme} {size[0]}x{size[1]}: Space activates the Button (BUG-043)", after - mid == 1, f"{mid} -> {after}")

            # disabled control explains itself on hover
            _hover(s, "[data-testid=g-btn-disabled]")
            s.wait(
                "const e = document.querySelector('[data-testid=g-btn-disabled]');"
                " return getComputedStyle(document.getElementById(e.getAttribute('aria-describedby'))).opacity === '1'", 3)
            tip = s.js(TOOLTIP_JS, "[data-testid=g-btn-disabled]")
            tr = s.js("const t = document.querySelector('[data-testid=g-btn-disabled]').getAttribute('aria-describedby');"
                      " const r = document.getElementById(t).getBoundingClientRect(); return [r.left, r.right, r.top, r.bottom, innerWidth, innerHeight]")
            flow.check(f"{theme} {size[0]}x{size[1]}: tooltip stays inside the viewport", tr[0] >= 0 and tr[1] <= tr[4] and tr[2] >= 0 and tr[3] <= tr[5], str(tr))
            flow.check(f"{theme} {size[0]}x{size[1]}: disabled Button shows its reason on hover",
                       bool(tip) and tip[0] == "Open a PDF to enable Save" and tip[1] == "1" and tip[2] == "visible", str(tip))
            _shot(flow, s, theme, size, "hover-disabled")

            # Dialog: opens with focus inside; Escape (commandEngine) closes it
            s.js("document.querySelector('[data-testid=g-open-dialog]').click()")
            flow.check(f"{theme} {size[0]}x{size[1]}: Dialog opens with focus inside", s.wait(DIALOG_JS, 5) is True, "")
            time.sleep(0.5)  # let the entry animation finish before the screenshot
            _shot(flow, s, theme, size, "dialog")
            s.keys(wd.KEY_ESCAPE)
            closed = s.wait("return !document.querySelector('[role=dialog]')", 5)
            flow.check(f"{theme} {size[0]}x{size[1]}: Escape closes the Dialog", closed, "")

    errs = app.errors() + (s.js("return window.__galleryErrors || []") or [])
    flow.check("no console errors on the gallery", not errs, " | ".join(map(str, errs))[:400])
    app.quit()
