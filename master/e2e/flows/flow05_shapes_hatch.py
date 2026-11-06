"""Flow 5: shapes and hatching.

Rectangle, ellipse, polygon and revision cloud are created. A hatched concave polygon, saved and
re-extracted (qpdf --json: the /Polygon /Vertices and the hatch /Ink /InkList written with it),
has every hatch segment inside the polygon (ERR-007). Every hatch pattern produces distinct output
(hatch geometry normalised to its shape's origin, compared across patterns).
"""
import os
import time

import common

NUM = 5
TITLE = "Shapes and hatching"
PATTERNS = ["diagonal-steel", "crosshatch", "concrete", "timber", "tiling", "masonry", "earth",
            "insulation"]
HATCH_SELECT = 'select[data-testid="hatch-pattern"]'
HATCH_AUTHOR = "LocalPDF Studio Hatch"


def _nums(arr):
    return [float(v) for v in arr if isinstance(v, (int, float))]


def _multi_click_shape(app, pts):
    """Click each vertex, then double-click the last one (the Studio commits on dblclick)."""
    for x, y in pts[:-1]:
        app.s.mouse_click(x, y)
        time.sleep(0.08)
    app.s.mouse_click(pts[-1][0], pts[-1][1], double=True)
    time.sleep(0.3)


def _hatch_after(annots, idx):
    """The hatch Ink annotation written right after shape annotation `idx` (same save)."""
    nxt = annots[idx + 1] if idx + 1 < len(annots) else None
    if nxt and nxt.get("/Subtype") == "/Ink" and common.pdf_string(nxt.get("/T")) == HATCH_AUTHOR:
        return nxt
    return None


def _segments(ink):
    segs = []
    for stroke in ink.get("/InkList", []):
        v = _nums(stroke)
        pts = list(zip(v[0::2], v[1::2]))
        segs += list(zip(pts, pts[1:]))
    return segs


def run(flow):
    path = flow.corpus_copy("standard-text.pdf")
    page_w = float(common.pdfinfo(path)["Page size"].split()[0])
    app = flow.launch([path], expect_pages=1)
    if not flow.check("document open", app.page_count() == 1, f"{app.page_count()} pages"):
        return
    app.snap_off()
    app.set_select('select[data-testid="zoom-preset"]', "75")
    time.sleep(0.3)
    pr = app.page_rect(0)
    X, Y = pr["x"], pr["y"]

    def created(label, tag, fn):
        n0 = app.vector_nodes()
        fn()
        shapes = app.vector_shapes()
        new = shapes[n0:] if len(shapes) > n0 else []
        flow.check(f"{label} created", len(shapes) == n0 + 1 and new and new[0][0] == tag,
                   f"nodes {n0} -> {len(shapes)}, new {[s[0] for s in new]}")
        return new[0][1] if new else {}

    # --- the four shape kinds ----------------------------------------------------------------
    app.select_tool("shapes", "Rectangle")
    created("rectangle", "rect", lambda: app.s.mouse_path([(X + 40, Y + 40), (X + 110, Y + 90),
                                                           (X + 140, Y + 110)]))
    app.select_tool("shapes", "Ellipse / Circle")
    created("ellipse", "ellipse", lambda: app.s.mouse_path([(X + 170, Y + 40), (X + 220, Y + 80),
                                                            (X + 260, Y + 110)]))
    app.select_tool("shapes", "Polygon")
    app.set_select(HATCH_SELECT, "none")
    created("polygon", "path", lambda: _multi_click_shape(
        app, [(X + 300, Y + 40), (X + 380, Y + 50), (X + 360, Y + 110), (X + 310, Y + 100)]))
    app.select_tool("shapes", "Revision Cloud")
    created("revision cloud", "path", lambda: _multi_click_shape(
        app, [(X + 420, Y + 40), (X + 520, Y + 40), (X + 520, Y + 110), (X + 420, Y + 110)]))

    # --- hatched concave polygon (L shape) ---------------------------------------------------
    app.select_tool("shapes", "Polygon")
    sel = app.set_select(HATCH_SELECT, "diagonal-steel")
    flow.check("hatch pattern selectable for the polygon tool", sel is True, f"select -> {sel}")
    bx, by = X + 60, Y + 170
    L = [(bx, by), (bx + 220, by), (bx + 220, by + 60), (bx + 80, by + 60), (bx + 80, by + 220),
         (bx, by + 220)]
    attrs = created("hatched concave polygon", "path", lambda: _multi_click_shape(app, L))
    flow.check("hatched polygon is filled with the hatch pattern on canvas",
               "hatch-diagonal-steel" in attrs.get("fill", ""), f"fill={attrs.get('fill')}")

    # --- one rectangle per hatch pattern ------------------------------------------------------
    app.select_tool("shapes", "Rectangle")
    ui_fills = {}
    for i, pat in enumerate(PATTERNS):
        app.set_select(HATCH_SELECT, pat)
        cx, cy = X + 40 + (i % 4) * 120, Y + 420 + (i // 4) * 110
        n0 = app.vector_nodes()
        app.s.mouse_path([(cx, cy), (cx + 50, cy + 40), (cx + 90, cy + 80)])
        time.sleep(0.2)
        sh = app.vector_shapes()
        ui_fills[pat] = sh[-1][1].get("fill") if len(sh) == n0 + 1 else None
    flow.check("every hatch pattern has its own canvas fill", len(set(ui_fills.values())) == len(PATTERNS)
               and None not in ui_fills.values(), str(ui_fills))

    # --- save and re-extract -------------------------------------------------------------------
    out, msg = app.save()
    flow.info("save_message", msg)
    if not flow.check("Save PDF writes a new file", out and os.path.exists(out) and out != path, msg):
        app.quit()
        return
    flow.check("original unchanged", flow.original_unchanged(path), "")
    ok, detail = common.qpdf_check(out)
    flow.check("saved file passes qpdf --check", ok, detail)
    annots = common.page_annotations(out)[0]
    subtypes = [a.get("/Subtype") for a in annots]
    flow.info("saved_annotation_subtypes", subtypes)

    polys = [i for i, a in enumerate(annots) if a.get("/Subtype") == "/Polygon"]
    hatched = [(i, _hatch_after(annots, i)) for i in polys]
    hatched = [(i, h) for i, h in hatched if h]
    if not flow.check("hatched polygon saved with hatch geometry", hatched,
                      f"subtypes {subtypes}"):
        app.quit()
        return
    i, ink = hatched[0]
    v = _nums(annots[i].get("/Vertices", []))
    poly = list(zip(v[0::2], v[1::2]))
    segs = _segments(ink)
    outside = []
    for a, b in segs:
        for k in range(11):
            t = k / 10
            px, py = a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t
            if not common.point_in_polygon(px, py, poly) and not common.near_polygon_edge(px, py, poly, 1.0):
                outside.append((round(px, 1), round(py, 1)))
                break
    flow.info("concave_polygon", {"vertices": len(poly), "hatch_segments": len(segs),
                                  "segments_leaving_polygon": len(outside)})
    flow.check("every hatch segment lies inside the concave polygon (ERR-007)",
               segs and not outside,
               f"{len(outside)}/{len(segs)} segments leave the polygon, e.g. {outside[:3]}")

    # distinct output per pattern: hatch Ink after each Square, normalised to the square's origin
    squares = [i for i, a in enumerate(annots) if a.get("/Subtype") == "/Square"]
    sigs = {}
    for i in squares[-len(PATTERNS):]:
        h = _hatch_after(annots, i)
        r = _nums(annots[i].get("/Rect", []))
        if not h or len(r) < 4:
            continue
        sig = tuple(sorted((round(a[0] - r[0], 1), round(a[1] - r[1], 1), round(b[0] - r[0], 1),
                            round(b[1] - r[1], 1)) for a, b in _segments(h)))
        sigs[i] = sig
    distinct = len(set(sigs.values()))
    flow.info("pattern_signatures", {"hatched_squares": len(sigs), "distinct": distinct})
    flow.check("every hatch pattern produces distinct saved output",
               len(sigs) == len(PATTERNS) and distinct == len(PATTERNS),
               f"{len(sigs)} hatched rectangles saved for {len(PATTERNS)} patterns, {distinct} distinct")
    _ = page_w
    errs = app.errors()
    flow.warn("no console errors", not errs, " | ".join(errs)[:300])
    app.quit()
