"""Flow 7 matcher for content-less annotations (a drawn signature has no /Contents).

Pure functions over qpdf-JSON-style annotation dicts (independent of the app).  Each annotation is
a dict with /Subtype, /Rect, optionally /InkList (list of coordinate lists) and /AP ({"/N":
{"/Length": n}} once the flow has resolved the normal-appearance stream).
"""

SUBTYPES = {"/Ink", "/Stamp"}
MIN_POINT_FRACTION = 0.25   # saved InkList points >= this fraction of the points drawn
PAGE_TOL = 6.0              # pt of slack when testing "inside the page"
COVER_TOL = 24.0            # pt of slack around the expected stroke box
MIN_COVERAGE = 0.5          # share of the expected stroke box the saved /Rect must cover


def _nums(v):
    return [x for x in (v or []) if isinstance(x, (int, float)) and not isinstance(x, bool)]


def ink_points(annot):
    """Number of (x, y) points over all strokes in /InkList."""
    il = annot.get("/InkList")
    if not isinstance(il, list):
        return 0
    return sum(len(_nums(s)) // 2 for s in il if isinstance(s, list))


def ap_nonempty(annot):
    n = (annot.get("/AP") or {}).get("/N") if isinstance(annot.get("/AP"), dict) else None
    if isinstance(n, dict):
        ln = n.get("/Length")
        return isinstance(ln, (int, float)) and ln > 0
    return False


def rect_problem(rect, page_box, expect_box):
    """None if /Rect is inside the page and covers the expected stroke box, else a reason.
    Boxes are [x0, y0, x1, y1] in PDF points."""
    r = _nums(rect)
    if len(r) != 4:
        return f"/Rect malformed: {rect}"
    x0, x1 = sorted((r[0], r[2])); y0, y1 = sorted((r[1], r[3]))
    px0, py0, px1, py1 = page_box
    if x1 - x0 <= 0 or y1 - y0 <= 0:
        return f"/Rect empty: {rect}"
    if x0 < px0 - PAGE_TOL or y0 < py0 - PAGE_TOL or x1 > px1 + PAGE_TOL or y1 > py1 + PAGE_TOL:
        return f"/Rect {r} outside page {page_box}"
    ex0, ey0, ex1, ey1 = expect_box
    ow = max(0.0, min(x1 + COVER_TOL, ex1) - max(x0 - COVER_TOL, ex0))
    oh = max(0.0, min(y1 + COVER_TOL, ey1) - max(y0 - COVER_TOL, ey0))
    area = (ex1 - ex0) * (ey1 - ey0)
    if area > 0 and ow * oh < MIN_COVERAGE * area:
        return f"/Rect {r} covers only {ow * oh / area:.0%} of expected stroke box {expect_box}"
    return None


def match_drawn(annots, page_box, expect_box, points_drawn):
    """(ok, detail): exactly one /Ink or /Stamp annotation with non-empty /InkList or /AP, a
    sensible /Rect, and (when it has an /InkList) >= MIN_POINT_FRACTION of the drawn points."""
    cands = [a for a in annots if a.get("/Subtype") in SUBTYPES]
    if len(cands) != 1:
        return False, f"{len(cands)} /Ink|/Stamp candidates (need exactly 1) among " \
                      f"{[a.get('/Subtype') for a in annots]}"
    a = cands[0]
    pts, ap = ink_points(a), ap_nonempty(a)
    if pts == 0 and not ap:
        return False, f"{a.get('/Subtype')} has empty /InkList and no /AP stream: the drawing is lost"
    if pts and pts < MIN_POINT_FRACTION * points_drawn:
        return False, f"/InkList has {pts} points, fewer than {MIN_POINT_FRACTION:.0%} of the {points_drawn} drawn"
    bad = rect_problem(a.get("/Rect"), page_box, expect_box)
    if bad:
        return False, bad
    return True, f"{a.get('/Subtype')}: {pts} ink points (drawn {points_drawn}), AP stream {ap}, rect {_nums(a.get('/Rect'))}"
