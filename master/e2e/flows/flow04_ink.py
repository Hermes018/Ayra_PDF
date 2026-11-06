"""Flow 4: ink.

Pen and highlighter strokes are created (vector-layer node count +1 each). The whole-stroke eraser
removes a stroke. The partial eraser splits or trims a stroke (remaining geometry checked with
SVG isPointInFill: gone at the erase point, present on both sides). Undo and redo restore the
exact previous state (vector layer markup compared byte for byte).
"""
import time

import common

NUM = 4
TITLE = "Ink"

FILL_AT_JS = """
const [x, y, which] = arguments;
const g = document.querySelector('#vector-layer > g');
const paths = [...g.querySelectorAll(':scope > path')];
const pick = which === 'all' ? paths : paths.filter(p => (p.getAttribute('style') || '').includes('multiply') === (which === 'hl'));
return pick.some(p => { try { return p.isPointInFill(new DOMPoint(x, y)); } catch (e) { return false; } });
"""


def _draw(app, pts):
    app.s.mouse_path(pts, step_ms=10)
    time.sleep(0.3)


def _undo(app):
    app.s.find('button[aria-label="Undo"]').click()
    time.sleep(0.3)


def _redo(app):
    app.s.find('button[aria-label="Redo"]').click()
    time.sleep(0.3)


def _undo_redo(flow, app, label, before, after):
    _undo(app)
    got = app.vector_html()
    flow.check(f"undo {label} restores the exact previous state", got == before,
               f"{app.vector_nodes()} node(s) after undo; markup {'equal' if got == before else 'differs'}")
    _redo(app)
    got = app.vector_html()
    flow.check(f"redo {label} restores the exact state", got == after,
               f"{app.vector_nodes()} node(s) after redo; markup {'equal' if got == after else 'differs'}")


def run(flow):
    path = flow.corpus_copy("standard-text.pdf")
    app = flow.launch([path], expect_pages=1)
    if not flow.check("document open", app.page_count() == 1, f"{app.page_count()} pages"):
        return
    app.snap_off()
    pr = app.page_rect(0)
    x0, y0 = pr["x"] + 120, pr["y"] + 380

    # pen
    s0 = app.vector_html()
    n0 = app.vector_nodes()
    flow.check("select pen tool", app.select_tool("ink", "Technical Pen A"), "")
    _draw(app, [(x0 + i * 12, y0 + (i % 4) * 6) for i in range(20)])
    s1, n1 = app.vector_html(), app.vector_nodes()
    flow.check("pen stroke adds one vector node", n1 == n0 + 1, f"{n0} -> {n1}")

    # highlighter
    flow.check("select highlighter tool", app.select_tool("ink", "Highlighter"), "")
    _draw(app, [(x0 + i * 12, y0 + 80) for i in range(20)])
    s2, n2 = app.vector_html(), app.vector_nodes()
    flow.check("highlighter stroke adds one vector node", n2 == n1 + 1, f"{n1} -> {n2}")
    hl = app.s.js("return [...document.querySelectorAll('#vector-layer > g > path')]"
                  ".filter(p => (p.getAttribute('style') || '').includes('multiply')).length")
    flow.check("highlighter stroke renders as a highlighter (multiply blend)", hl == 1, f"{hl} found")
    _undo_redo(flow, app, "highlighter stroke", s1, s2)

    # whole-stroke eraser on the pen stroke
    flow.check("select whole-stroke eraser", app.select_tool("erase", "Whole Stroke Eraser"), "")
    app.s.mouse_click(x0 + 6 * 12, y0 + (6 % 4) * 6)
    time.sleep(0.3)
    s3, n3 = app.vector_html(), app.vector_nodes()
    pen_left = app.s.js("return [...document.querySelectorAll('#vector-layer > g > path')]"
                        ".filter(p => !(p.getAttribute('style') || '').includes('multiply')).length")
    flow.check("whole-stroke eraser removes the pen stroke (highlighter kept)",
               n3 == n2 - 1 and pen_left == 0, f"{n2} -> {n3} nodes, pen paths left {pen_left}")
    if n3 != n2:
        _undo_redo(flow, app, "whole-stroke erase", s2, s3)

    # partial eraser across the middle of a straight pen stroke
    app.select_tool("ink", "Technical Pen A")
    n3 = app.vector_nodes()  # re-baseline: a failed undo/redo above may have changed the layer
    ya = y0 + 160
    _draw(app, [(x0 + i * 10, ya) for i in range(31)])
    s4, n4 = app.vector_html(), app.vector_nodes()
    xm = x0 + 150
    wl, wy = app.screen_to_world(xm - 60, ya)
    wm, _ = app.screen_to_world(xm, ya)
    wr, _ = app.screen_to_world(xm + 60, ya)
    pen_at = lambda wx: app.s.js(FILL_AT_JS, wx, wy, "pen")  # noqa: E731
    flow.check("straight pen stroke drawn (covers its midpoint)", n4 == n3 + 1 and pen_at(wm),
               f"nodes {n3} -> {n4}")
    flow.check("select partial eraser", app.select_tool("erase", "Boolean Partial Eraser"), "")
    app.s.mouse_path([(xm, ya - 2), (xm, ya + 2)])
    time.sleep(0.3)
    s5, n5 = app.vector_html(), app.vector_nodes()
    gone, left, right = pen_at(wm), pen_at(wl), pen_at(wr)
    flow.check("partial eraser removes geometry at the erase point", s5 != s4 and not gone,
               f"nodes {n4} -> {n5}; ink at erase point: {gone}")
    flow.check("partial eraser keeps geometry on both sides (split/trim)", left and right,
               f"ink 60px left: {left}, right: {right}; nodes {n4} -> {n5}")
    if s5 != s4:
        _undo_redo(flow, app, "partial erase", s4, s5)

    errs = app.errors()
    flow.warn("no console errors during inking", not errs, " | ".join(errs)[:300])
    app.quit()
