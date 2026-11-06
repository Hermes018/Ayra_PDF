"""Flow 10: organize.

In the Organize view: reorder (page-card drag and drop), rotate and delete pages, then Apply; and
merge two files. Output page count and order match (`pdfinfo` + per-page text fingerprints),
rotation shows in `pdfinfo -l`, and the originals are unchanged (SHA-256).

HTML5 drag and drop is not driven by WebDriver pointer actions in WebKit, so the reorder uses
synthetic dragstart/dragover/drop events on the real page cards (the app's own handlers run).
"""
import glob
import os
import time

import common

NUM = 10
TITLE = "Organize"


def _fingerprints(path, n):
    return [" ".join((common.pdftotext(path, i, i) or "").split())[:60] for i in range(1, n + 1)]


def _wait_file(pattern, timeout=40):
    deadline = time.time() + timeout
    while time.time() < deadline:
        hits = [h for h in glob.glob(pattern) if os.path.getsize(h) > 0]
        if hits:
            time.sleep(0.5)  # let an atomic rename / second job settle
            return sorted(hits, key=os.path.getmtime)[-1]
        time.sleep(0.25)
    return None


def _toasts(app):
    return app.s.js("return [...document.querySelectorAll('.toast')].map(t => t.textContent.replace(/\\s+/g, ' ').trim())") or []


def _goto(app, label):
    btn = app.s.find_text("nav[aria-label='Primary Navigation'] button", label)
    if btn:
        btn.click()
        time.sleep(0.4)
    return bool(btn)


def _card_click(app, original_page, testid):
    return app.s.js("""
        const [n, testid] = arguments;
        const card = [...document.querySelectorAll('.grid-layout .page-card')]
            .find(c => (c.querySelector('img') || {}).alt === 'Page ' + n);
        const b = card && card.querySelector(`button[data-testid="${testid}"]`);
        if (!b) return false; b.click(); return true;
    """, original_page, testid)


def run(flow):
    src = flow.corpus_copy("standard-form.pdf")
    other = flow.corpus_copy("standard-text.pdf")
    n = common.pdf_pages(src)
    fp = _fingerprints(src, n)
    flow.check("ground truth: per-page text fingerprints are distinct", len(set(fp)) == n, str(fp))

    # --- reorder + rotate + delete ---------------------------------------------------------
    app = flow.launch([src], expect_pages=n)
    flow.check("Organize view reachable", _goto(app, "Organize"), "")
    cards = app.s.wait("return document.querySelectorAll('.grid-layout .page-card').length", 30)
    if not flow.check("page grid shows every page", cards == n, f"{cards} cards for {n} pages"):
        app.quit()
        return
    app.s.js("""
        const cards = document.querySelectorAll('.grid-layout .page-card');
        const dt = new DataTransfer();
        const fire = (el, type) => el.dispatchEvent(new DragEvent(type, {dataTransfer: dt, bubbles: true, cancelable: true}));
        fire(cards[0], 'dragstart'); fire(cards[2], 'dragenter'); fire(cards[2], 'dragover');
        fire(cards[2], 'drop'); fire(cards[0], 'dragend');
    """)
    time.sleep(0.3)
    order = app.s.js("return [...document.querySelectorAll('.grid-layout .page-card img')].map(i => i.alt)")
    flow.check("drag page 1 onto position 3 reorders the grid", order[:3] == ["Page 2", "Page 3", "Page 1"],
               f"grid order {order}")
    rot = _card_click(app, 3, "organize-rotate-btn")
    dele = _card_click(app, 4, "organize-delete-btn")
    time.sleep(0.2)
    state = app.s.js("return [...document.querySelectorAll('.grid-layout .page-card')].map(c => "
                     "[c.querySelector('img').alt, c.querySelector('img').style.transform, c.classList.contains('deleted')])")
    flow.check("rotate and delete controls on the page cards", rot and dele,
               f"rotate {rot}, delete {dele}; cards {state}")
    base = os.path.splitext(src)[0]
    app.click_button("Apply changes")
    out = _wait_file(base + ".organized-*.pdf")
    flow.info("organize_alerts", app.alerts())
    flow.info("organize_toasts", _toasts(app))
    flow.info("organize_outputs", sorted(os.path.basename(p) for p in glob.glob(base + "_*.pdf")))
    expected_order = [2, 3, 1, 5]
    if flow.check("Apply writes a new organized file", out is not None,
                  os.path.basename(out) if out else f"no {os.path.basename(base)}.organized-*.pdf; alerts {app.alerts()}"):
        pages = common.pdf_pages(out)
        flow.check("organized page count (5 - 1 deleted = 4)", pages == 4, f"{pages} pages in {os.path.basename(out)}")
        got = _fingerprints(out, pages or 0)
        flow.check("organized page order is 2,3,1,5 (per-page text)", got == [fp[i - 1] for i in expected_order],
                   f"got {[fp.index(g) + 1 if g in fp else '?' for g in got]}")
        rots = common.page_rotations(out, pages or 0)
        flow.check("rotation of the moved page 3 shows in pdfinfo -l (page 2 rot 90)",
                   len(rots) == 4 and rots[1] == 90 and rots[0] == 0 and rots[2] == 0,
                   f"rotations {rots} ({os.path.basename(out)})")
    # the sidebar "Rotate Pages" tool on its own (isolates the rotate job from the grid chaining)
    app.set_input("[data-testid=organize-rotate-pages-input]", "2")
    app.click_button("Rotate pages", "[data-testid=organize-rotate-pages-btn]")
    rotated = _wait_file(base + "_rotated.pdf", 30)
    rots = common.page_rotations(rotated, n) if rotated else []
    flow.check("Rotate Pages tool: page 2 rotated 90 in pdfinfo -l", rotated and rots[1] == 90 and
               rots.count(0) == n - 1, f"{os.path.basename(rotated) if rotated else 'no _rotated.pdf'} "
               f"rotations {rots}; toasts {_toasts(app)[:3]}")
    flow.check("original unchanged after organize", flow.original_unchanged(src), "")
    app.quit()

    # --- merge two files -------------------------------------------------------------------
    app = flow.launch([src, other], expect_pages=n)
    _goto(app, "Organize")
    app.s.wait("return document.querySelectorAll('.file-list li').length === 2", 10)
    files = app.s.js("return [...document.querySelectorAll('.file-list li')].map(l => l.textContent.trim())")
    flow.check("both files listed for merge", len(files) == 2, str(files))
    app.click_button("Merge all")
    merged = _wait_file(base + "_merged.pdf")
    n2 = common.pdf_pages(other)
    if flow.check("Merge writes a new file", merged is not None,
                  os.path.basename(merged) if merged else f"no {os.path.basename(base)}_merged.pdf"):
        pages = common.pdf_pages(merged)
        flow.check(f"merged page count {n}+{n2}", pages == n + n2, f"{pages} pages")
        got = _fingerprints(merged, pages or 0)
        want = fp + _fingerprints(other, n2)
        flow.check("merged page order is file A then file B (per-page text)", got == want,
                   f"first/last page text {got[:1]} ... {got[-1:]}")
        ok, detail = common.qpdf_check(merged)
        flow.check("merged file passes qpdf --check", ok, detail)
    flow.check("originals unchanged after merge", flow.original_unchanged(src) and flow.original_unchanged(other), "")
    errs = app.errors()
    flow.warn("no console errors", not errs, " | ".join(errs)[:300])
    app.quit()
