"""Flow 3: navigate.

Pan, zoom (buttons, presets, ctrl+wheel), pinch (two synthetic touch pointers) and Fit Width /
Fit Page change the view as expected, measured from the PDF layer's CSS transform and the page's
on-screen box. Thumbnails render for the open file. Two documents open as separate tabs (or split
panes) and switching keeps each one's view state.
"""
import os
import subprocess
import time

import common
import wd

NUM = 3
TITLE = "Navigate"
TOL = 0.02  # 2 % on zoom factors


def _close(a, b, tol=TOL):
    return abs(a - b) <= tol * max(abs(b), 1e-9)


def _viewport_rect(app):
    return app.s.js("const r = document.querySelector('.spatial-viewport').getBoundingClientRect();"
                    "return {x: r.x, y: r.y, w: r.width, h: r.height};")


def _ctrl_wheel(app, x, y, delta):
    """ctrl+wheel through W3C actions (wheel + key sources); falls back to a synthetic WheelEvent
    when the driver has no wheel source or drops the modifier (no zoom change). Returns the
    method that changed the zoom (or the last one tried)."""
    z0 = app.transform()[0]
    try:
        app.s.actions([
            {"type": "key", "id": "kbd1", "actions": [
                {"type": "keyDown", "value": wd.KEY_CONTROL}, {"type": "pause", "duration": 0},
                {"type": "pause", "duration": 0}, {"type": "keyUp", "value": wd.KEY_CONTROL}]},
            {"type": "wheel", "id": "wheel1", "actions": [
                {"type": "pause", "duration": 0},
                {"type": "scroll", "x": int(x), "y": int(y), "deltaX": 0, "deltaY": int(delta),
                 "origin": "viewport", "duration": 0},
                {"type": "pause", "duration": 0}, {"type": "pause", "duration": 0}]}])
        time.sleep(0.3)
        if app.transform()[0] != z0:
            return "webdriver-wheel"
        method = "synthetic-wheelevent (WebDriver wheel+ctrl had no effect)"
    except Exception:  # noqa: BLE001
        method = "synthetic-wheelevent (no WebDriver wheel source)"
    app.s.js("""
        const [x, y, d] = arguments;
        const t = document.elementFromPoint(x, y) || document.querySelector('.spatial-viewport');
        t.dispatchEvent(new WheelEvent('wheel', {clientX: x, clientY: y, deltaY: d, deltaMode: 0,
            ctrlKey: true, bubbles: true, cancelable: true}));
    """, x, y, delta)
    return method


def _pinch(app, cx, cy, d0, d1, steps=8):
    """Two synthetic touch pointers spreading from distance d0 to d1 around (cx, cy)."""
    app.s.js("""
        const [cx, cy, d0, d1, steps] = arguments;
        const t = document.getElementById('wet-layer');
        const ev = (type, id, x, y) => t.dispatchEvent(new PointerEvent(type, {pointerId: id,
            pointerType: 'touch', isPrimary: id === 11, clientX: x, clientY: y, buttons: type === 'pointerup' ? 0 : 1,
            bubbles: true, cancelable: true}));
        ev('pointerdown', 11, cx - d0 / 2, cy); ev('pointerdown', 12, cx + d0 / 2, cy);
        for (let i = 1; i <= steps; i++) {
            const d = d0 + (d1 - d0) * i / steps;
            ev('pointermove', 11, cx - d / 2, cy); ev('pointermove', 12, cx + d / 2, cy);
        }
        ev('pointerup', 11, cx - d1 / 2, cy); ev('pointerup', 12, cx + d1 / 2, cy);
    """, cx, cy, d0, d1, steps)


def run(flow):
    path_a = flow.corpus_copy("standard-form.pdf")
    path_b = flow.corpus_copy("standard-text.pdf")
    pages_a = common.pdf_pages(path_a)
    app = flow.launch([path_a], expect_pages=pages_a)
    if not flow.check("document A open", app.page_count() == pages_a, f"{app.page_count()} pages"):
        return
    vp = _viewport_rect(app)
    cx, cy = vp["x"] + vp["w"] * 0.45, vp["y"] + vp["h"] * 0.55

    # --- pan (Pan tool drag) --------------------------------------------------------------
    app.select_tool("nav", "Pan Canvas")
    t0 = app.transform()
    app.s.mouse_path([(cx, cy), (cx - 60, cy - 80), (cx - 120, cy - 160)])
    t1 = app.transform()
    flow.check("pan: drag moves the view by the drag delta",
               abs((t1[4] - t0[4]) - (-120)) <= 3 and abs((t1[5] - t0[5]) - (-160)) <= 3,
               f"pan delta ({t1[4] - t0[4]:.1f}, {t1[5] - t0[5]:.1f}) for drag (-120, -160)")

    # --- zoom buttons ---------------------------------------------------------------------
    z0 = app.transform()[0]
    app.s.find('button[aria-label="Zoom In"]').click()
    time.sleep(0.2)
    z1 = app.transform()[0]
    flow.check("zoom in button: x1.25", _close(z1 / z0, 1.25), f"{z0:.3f} -> {z1:.3f}")
    app.s.find('button[aria-label="Zoom Out"]').click()
    time.sleep(0.2)
    z2 = app.transform()[0]
    flow.check("zoom out button: x0.8", _close(z2 / z1, 0.8), f"{z1:.3f} -> {z2:.3f}")

    # --- presets --------------------------------------------------------------------------
    for val, want in (("150", 1.5), ("50", 0.5), ("100", 1.0)):
        app.set_select('select[aria-label="Zoom Level Preset"]', val)
        time.sleep(0.2)
        z = app.transform()[0]
        flow.check(f"zoom preset {val}%", _close(z, want), f"zoom {z:.3f}")

    # --- ctrl + wheel ---------------------------------------------------------------------
    z0 = app.transform()[0]
    method = _ctrl_wheel(app, cx, cy, -100)
    time.sleep(0.3)
    z1 = app.transform()[0]
    flow.info("ctrl_wheel_method", method)
    flow.check("ctrl+wheel up zooms in", z1 > z0 * 1.05, f"{z0:.3f} -> {z1:.3f} ({method})")

    # --- pinch ----------------------------------------------------------------------------
    app.set_select('select[aria-label="Zoom Level Preset"]', "100")
    time.sleep(0.2)
    z0 = app.transform()[0]
    _pinch(app, cx, cy, 100, 200)
    time.sleep(0.3)
    z1 = app.transform()[0]
    flow.check("pinch spread x2 zooms x2", _close(z1 / z0, 2.0, 0.05), f"{z0:.3f} -> {z1:.3f}")
    nodes = app.vector_nodes()
    flow.check("pinch leaves no stray markup", nodes == 0, f"{nodes} vector node(s)")

    # --- fit width / fit page (page 1 must fit the canvas) ---------------------------------
    app.set_select('select[aria-label="Zoom Level Preset"]', "fit-width")
    time.sleep(0.3)
    pr, vp = app.page_rect(0), _viewport_rect(app)
    left, right = pr["x"] - vp["x"], pr["x"] + pr["w"] - vp["x"]
    flow.check("fit width: page 1 spans the canvas width and is fully visible",
               left >= -1 and right <= vp["w"] + 1 and pr["w"] >= 0.9 * vp["w"],
               f"page x {left:.0f}..{right:.0f} in canvas width {vp['w']:.0f}")
    app.set_select('select[aria-label="Zoom Level Preset"]', "fit-page")
    time.sleep(0.3)
    pr, vp = app.page_rect(0), _viewport_rect(app)
    inside = (pr["x"] >= vp["x"] - 1 and pr["x"] + pr["w"] <= vp["x"] + vp["w"] + 1 and
              pr["y"] >= vp["y"] - 1 and pr["y"] + pr["h"] <= vp["y"] + vp["h"] + 1)
    # The Studio keeps ~80 px vertical / 64 px horizontal padding around a fitted page.
    fills = pr["w"] >= 0.9 * (vp["w"] - 64) or pr["h"] >= 0.9 * (vp["h"] - 80)
    flow.check("fit page: whole page 1 visible and filling one dimension", inside and fills,
               f"page {pr['w']:.0f}x{pr['h']:.0f} at ({pr['x'] - vp['x']:.0f},{pr['y'] - vp['y']:.0f}) "
               f"in canvas {vp['w']:.0f}x{vp['h']:.0f}")
    errs = app.errors()
    flow.warn("no console errors while navigating", not errs, " | ".join(errs)[:300])

    # --- two documents: open B in the running instance (OS "open with" path) ------------
    app.set_select('select[aria-label="Zoom Level Preset"]', "150")
    time.sleep(0.2)
    view_a = app.transform()
    drv = flow.driver()
    r = subprocess.run([flow.ctx.binary, path_b], env=dict(drv.env), capture_output=True, timeout=30)
    time.sleep(2.5)
    flow.info("second_launch_exit", r.returncode)
    tabs = app.s.js("""
        const vis = (e) => { const r = e.getBoundingClientRect(); return r.width > 0 && r.height > 0; };
        return [...document.querySelectorAll('[role=tab], .tab, .doc-tab, .document-tab, [data-testid*=tab]')]
            .filter(vis).map(e => e.textContent.trim()).filter(t => t);
    """)
    name_b = os.path.basename(path_b)
    has_tab_b = any(name_b in t or "standard-text" in t for t in tabs)
    split = app.s.js("return document.querySelectorAll('.spatial-viewport').length")
    flow.check("second document opens as a tab or split pane", has_tab_b or split > 1,
               f"tabs={tabs[:6]}, canvases={split}, studio still shows {app.page_count()} page(s) "
               f"(B has 1); view A transform {'kept' if app.transform() == view_a else 'changed'}")
    # Switch A -> B, give B its own view, then B -> A -> B: each must come back exactly.
    tab_css = "[role=tab], .tab, .doc-tab, .document-tab, [data-testid*=tab]"
    name_a = os.path.basename(path_a)

    def show(name, pages):
        el = app.s.find_text(tab_css, name) or app.s.find_text(tab_css, os.path.splitext(name)[0])
        if not el:
            return False
        el.click()
        return app.wait_doc(pages, 15)

    same = lambda t, u: t is not None and u is not None and all(abs(x - y) <= 0.5 for x, y in zip(t, u))  # noqa: E731
    if not has_tab_b:
        flow.check("switching documents keeps each one's view state", False,
                   "no document tabs to switch between")
    else:
        on_b = show(name_b, 1)
        if on_b:
            app.set_select('select[aria-label="Zoom Level Preset"]', "50")
            time.sleep(0.2)
        view_b = app.transform() if on_b else None
        back_a = show(name_a, pages_a)
        got_a = app.transform() if back_a else None
        flow.check("switching back to document A restores A's view state",
                   back_a and same(got_a, view_a), f"A view {view_a} -> after B and back {got_a}")
        again_b = show(name_b, 1)
        got_b = app.transform() if again_b else None
        flow.check("switching to document B again restores B's own view state",
                   on_b and again_b and same(got_b, view_b) and not same(view_b, view_a),
                   f"B view {view_b} -> after A and back {got_b}")

    # --- thumbnails (Studio strip, else the Organize page grid) ---------------------------
    studio_thumbs = app.s.js("""
        return [...document.querySelectorAll('[class*=thumb] img, [class*=thumb] canvas, [aria-label*=humbnail] img, [aria-label*=humbnail] canvas')]
            .filter(e => e.getBoundingClientRect().width > 0).length;
    """)
    where = "studio"
    count = studio_thumbs
    if not studio_thumbs:
        btn = app.s.find_text("nav[aria-label='Primary Navigation'] button", "Organize")
        if btn:
            btn.click()
            app.s.wait("return document.querySelectorAll('.grid-layout .page-card img').length", 30)
            time.sleep(1.0)
            count = app.s.js("return [...document.querySelectorAll('.grid-layout .page-card img')]"
                             ".filter(i => i.complete && i.naturalWidth > 0).length")
            imgs = app.s.js("return [...document.querySelectorAll('.grid-layout .page-card img')]"
                            ".map(i => [i.src.slice(0, 22), i.naturalWidth])")
            where = f"organize grid, img (src, naturalWidth) = {imgs[:2]}, errors={app.errors()[-2:]}"
    flow.info("thumbnails", {"studio": studio_thumbs, where: count})
    flow.check("thumbnails render for the open file", count == pages_a,
               f"{count} thumbnail(s) for {pages_a} pages ({where}); Studio has "
               f"{studio_thumbs} thumbnail(s)")
    app.quit()
