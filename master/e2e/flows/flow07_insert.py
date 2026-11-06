"""Flow 7: insert.

Text box, callout, sticky note, stamp and signature (draw mode) are each created and visible on
the page, and survive save -> reopen (a fresh app session on the saved file shows all five; the
saved PDF carries one annotation per insert, read back with qpdf --json).
"""
import os
import time

import common
import ink_match

NUM = 7
TITLE = "Insert"

NEW_NODE_JS = """
const g = document.querySelector('#vector-layer > g');
const kids = [...g.children].filter(e => !e.classList.contains('selection-gizmo'));
const e = kids[kids.length - 1]; if (!e) return null;
const r = e.getBoundingClientRect();
return {tag: e.tagName.toLowerCase(), w: r.width, h: r.height, x: r.x, y: r.y,
        text: e.textContent.trim().slice(0, 60), html: e.outerHTML.slice(0, 160),
        drawn: e.querySelectorAll('path, polyline, image').length + (['path', 'image'].includes(e.tagName.toLowerCase()) ? 1 : 0)};
"""


def _text_editor(app, text):
    ok = app.s.wait("return !!document.querySelector('[data-testid=inline-pdf-text-editor] textarea')", 5)
    if not ok:
        return False
    app.set_input("[data-testid=inline-pdf-text-editor] textarea", text)
    btn = app.s.find_text("[data-testid=inline-pdf-text-editor] button", "Commit text")
    if not btn:
        return False
    btn.click()
    time.sleep(0.3)
    return True


N_STROKE_POINTS = 20  # points the flow draws for the signature


def _saved_view(path):
    """(page /MediaBox, annotations with /AP/N and indirect /Length resolved) via qpdf, not the app."""
    objs, pages = common.qpdf_objects(path)
    box = [float(v) for v in objs[pages[0]].get("/MediaBox", [0, 0, 612, 792])]
    annots = common.page_annotations(path)[0]

    def res(v):
        return objs.get(v, v) if isinstance(v, str) and v.endswith(" R") else v
    out = []
    for a in annots:
        a = dict(a)
        ap = res(a.get("/AP"))
        if isinstance(ap, dict):
            n = res(ap.get("/N"))
            if isinstance(n, dict):
                n = dict(n); n["/Length"] = res(n.get("/Length"))
            a["/AP"] = {"/N": n}
        a["/InkList"] = [res(s) for s in (res(a.get("/InkList")) or [])]
        out.append(a)
    return box, out


def run(flow):
    path = flow.corpus_copy("standard-text.pdf")
    app = flow.launch([path], expect_pages=1)
    if not flow.check("document open", app.page_count() == 1, f"{app.page_count()} pages"):
        return
    app.snap_off()
    app.set_select('select[data-testid="zoom-preset"]', "75")
    time.sleep(0.3)
    pr = app.page_rect(0)
    X, Y = pr["x"] + 60, pr["y"] + 80
    made = {}

    def place(label, cat, tool_label, x, y, after=None):
        n0 = app.vector_nodes()
        if not app.select_tool(cat, tool_label):
            flow.check(f"{label}: tool selectable", False, f"'{tool_label}' not in drawer '{cat}'")
            return
        app.s.mouse_click(x, y)
        time.sleep(0.3)
        extra = after() if after else True
        n1 = app.vector_nodes()
        node = app.s.js(NEW_NODE_JS) if n1 > n0 else None
        inside = bool(node) and node["w"] > 2 and node["h"] > 2 and \
            pr["x"] <= node["x"] <= pr["x"] + pr["w"] and pr["y"] <= node["y"] <= pr["y"] + pr["h"]
        made[label] = node
        flow.check(f"{label} created and visible on the page", extra and n1 == n0 + 1 and inside,
                   f"nodes {n0} -> {n1}; node {node}")

    place("text box", "review", "Rich Text Box", X, Y,
          after=lambda: _text_editor(app, "E2E text box"))
    place("callout", "insert", "Single Callout", X + 250, Y)
    place("sticky note", "insert", "Sticky Note", X, Y + 150,
          after=lambda: _text_editor(app, "E2E sticky note"))
    place("stamp", "insert", "Engineering Stamp", X + 250, Y + 150)

    def sign():
        if not app.s.wait("return !!document.querySelector('[aria-label=\"Signature capture modal\"]')", 5):
            return False
        app.click_button("Draw", "[aria-label='Signature capture modal'] button")
        pad = app.s.find('[aria-label="Freehand signature pad"]')
        if not pad:
            return False
        r = pad.rect()
        sx, sy = r["x"] + 20, r["y"] + r["height"] / 2
        app.s.mouse_path([(sx + i * 10, sy + (12 if i % 2 else -12)) for i in range(N_STROKE_POINTS)], step_ms=8)
        time.sleep(0.2)
        app.click_button("Stamp signature")
        time.sleep(0.4)
        return True
    place("signature (draw)", "insert", "Signature Suite", X, Y + 300, after=sign)
    sig = made.get("signature (draw)") or {}
    flow.check("drawn signature shows the drawn strokes (not only a typed label)",
               sig.get("drawn", 0) > 0, f"signature node renders {sig.get('drawn', 0)} path/image "
               f"element(s), text '{sig.get('text')}'")
    flow.info("inserted_nodes", made)
    n_before = app.vector_nodes()
    errs = app.errors()
    flow.warn("no console errors while inserting", not errs, " | ".join(errs)[:300])

    # --- save -> reopen ---------------------------------------------------------------------
    out, msg = app.save()
    flow.info("save_message", msg)
    if not flow.check("Save PDF writes a new file", out and os.path.exists(out) and out != path, msg):
        app.quit()
        return
    flow.check("original unchanged", flow.original_unchanged(path), "")
    annots = common.page_annotations(out)[0]
    flow.info("saved_annotations", [(a.get("/Subtype"), common.pdf_string(a.get("/Contents"))) for a in annots])
    saved = [(a.get("/Subtype"), (common.pdf_string(a.get("/Contents")) or "").strip()) for a in annots]
    flow.check("saved PDF holds exactly five annotations (one per insert)", len(annots) == 5,
               f"{len(annots)} annotations: {saved}")
    native = {"text box": {"/FreeText"}, "callout": {"/FreeText"}, "sticky note": {"/Text"},
              "stamp": {"/Stamp"}, "signature (draw)": {"/Ink", "/Stamp"}}
    for label, node in made.items():
        want = (node or {}).get("text") or ""
        if label == "signature (draw)" and not want:
            # content-less (no /Contents): match by subtype + ink/AP content + geometry, from the saved file
            box, resolved = _saved_view(out)
            sc = pr["w"] / (box[2] - box[0])
            nx, ny, nw, nh = node["x"], node["y"], node["w"], node["h"]
            expect = [box[0] + (nx - pr["x"]) / sc, box[3] - (ny + nh - pr["y"]) / sc,
                      box[0] + (nx + nw - pr["x"]) / sc, box[3] - (ny - pr["y"]) / sc]
            ok, detail = ink_match.match_drawn(resolved, box, expect, N_STROKE_POINTS)
            flow.check(f"{label}: exactly one saved annotation carries it (ink/AP + rect)", ok,
                       f"{detail}; expected box {[round(v) for v in expect]}; saved {saved}")
            continue
        hits = [st for st, c in saved if want and c == want]
        flow.check(f"{label}: exactly one saved annotation carries it", len(hits) == 1,
                   f"content {want!r} matched {len(hits)} annotation(s); saved {saved}")
        if hits:
            flow.warn(f"{label}: saved as its native PDF annotation type", hits[0] in native[label],
                      f"saved as {hits[0]}, expected one of {sorted(native[label])}")
    app.quit()

    app2 = flow.launch([out], expect_pages=1)
    time.sleep(1.0)
    n_after = app2.vector_nodes()
    texts = app2.s.js("return document.querySelector('#vector-layer > g').textContent")
    flow.check("reopened file shows all five inserts", n_after == n_before,
               f"{n_after} vector nodes after reopen vs {n_before} before save")
    flow.check("reopened file keeps the typed texts", "E2E text box" in (texts or "") and
               "E2E sticky note" in (texts or ""), f"vector text '{(texts or '')[:120]}'")
    app2.quit()
