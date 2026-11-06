"""Flow 22: UI sweep smoke, group 9 (PropertyPill). NOT one of the 12 golden-path flows: run with `--flows 22`.

For light and dark at 1200x800 and 1600x1000: with the Pen tool the pill shows colour swatches, stroke widths and
opacity presets; with the Rectangle tool also the hatch controls; with a measure tool the measure controls.
Checks: every control in the pill is a ui/ component, a swatch click sets aria-pressed on that colour only, the
hatch select works, and there are no console errors.
Screenshots: <out>/screens/sweep_pill_<theme>_<WxH>.png (shapes tool: hatch and line controls visible)
"""
import os
import time

import common

NUM = 22
TITLE = "UI sweep smoke: property pill"
SIZES = [(1200, 800), (1600, 1000)]
THEMES = [("light", "Light"), ("dark", "Dark")]

RAW_JS = ("return [...document.querySelectorAll('.property-pill button, .property-pill select, .property-pill input')]"
          ".filter(e => !e.closest('.lp-btnwrap, .lp-iconwrap, .lp-field, .lp-swatchwrap')).length")
PRESSED_SWATCHES = "return [...document.querySelectorAll('.property-pill .lp-swatch[aria-pressed=true]')].map(b => b.getAttribute('aria-label'))"


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
        flow.check(f"{key}: Pen tool selected", app.select_tool("ink", "Pen"), "")
        flow.check(f"{key}: pill controls are ui/ components (pen)", s.js(RAW_JS) == 0, str(s.js(RAW_JS)))
        red = s.find('.property-pill .lp-swatch[aria-label="Red color"]')
        if flow.check(f"{key}: Red swatch present", bool(red), ""):
            red.click()
            time.sleep(0.2)
            pressed = s.js(PRESSED_SWATCHES)
            flow.check(f"{key}: clicking Red marks only Red as pressed", pressed == ["Red color"], str(pressed))
        flow.check(f"{key}: Rectangle tool selected", app.select_tool("shapes", "Rectangle"), "")
        has_hatch = s.wait("return !!document.querySelector('select[data-testid=hatch-pattern]')", 5)
        flow.check(f"{key}: hatch select is shown for shapes", bool(has_hatch), "")
        flow.check(f"{key}: hatch select can be set", app.set_select('select[data-testid="hatch-pattern"]', "crosshatch") is True, "")
        flow.check(f"{key}: pill controls are ui/ components (shapes)", s.js(RAW_JS) == 0, str(s.js(RAW_JS)))
        flow.check(f"{key}: Measure tool selected", app.select_tool("measure", "Linear Length"), "")
        flow.check(f"{key}: measure controls are ui/ components", s.js(RAW_JS) == 0, str(s.js(RAW_JS)))
        app.select_tool("shapes", "Rectangle")
        s.wait("return document.querySelectorAll('.toast').length === 0", 15)
        s.js("if (document.activeElement) document.activeElement.blur()")
        for size in SIZES:
            s.set_window(*size)
            time.sleep(0.5)
            p = os.path.join(flow.ctx.screens, f"sweep_pill_{key}_{size[0]}x{size[1]}.png")
            s.screenshot(p)
            flow.screens.append(p)
            flow.info(f"shot[{key}_{size[0]}x{size[1]}]", os.path.relpath(p, common.MASTER))
    errs = app.errors()
    flow.check("no console errors", not errs, " | ".join(map(str, errs))[:400])
    app.quit()
