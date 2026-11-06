"""Flow 9: edit PDF text in place.

Enter Edit Text mode, click the text block (a real pointer click at the block's hotspot), change
its text and commit. After saving, `pdftotext` of the NEW file contains the new text and not the
old text. The original is unchanged (SHA-256).
"""
import os
import re
import time

import common

NUM = 9
TITLE = "Edit PDF text in place"
OLD = "Dummy PDF file"
NEW = "Edited by E2E"


def run(flow):
    path = flow.corpus_copy("standard-text.pdf")
    flow.check("ground truth: original contains the old text", OLD in (common.pdftotext(path) or ""), "")
    app = flow.launch([path], expect_pages=1)
    if not flow.check("document open", app.page_count() == 1, f"{app.page_count()} pages"):
        return
    app.click_button("Edit text", ".studio-top-bar .mode-segmented-switch button")
    banner = app.s.wait("return !!document.querySelector('[data-testid=structural-edit-banner]')", 5)
    flow.check("Edit Text mode entered", banner, "")
    blocks = app.s.wait("""
        return [...document.querySelectorAll('.pdf-text-block-hotspot')].map(e => {
            const r = e.getBoundingClientRect();
            return {label: e.getAttribute('aria-label'), x: r.x + r.width / 2, y: r.y + r.height / 2, w: r.width};
        });""", 15) or []
    flow.info("text_blocks", blocks[:5])
    blk = next((b for b in blocks if OLD.split()[0] in (b["label"] or "")), None)
    if not flow.check("text block hotspots listed for the page", blk is not None,
                      f"{len(blocks)} hotspot(s): {[b['label'] for b in blocks][:4]}"):
        app.quit()
        return
    app.s.mouse_click(blk["x"], blk["y"])
    ed = app.s.wait("const t = document.querySelector('[data-testid=inline-pdf-text-editor] textarea');"
                    "return t ? {value: t.value} : null", 5)
    top = app.s.js("const e = document.elementFromPoint(arguments[0], arguments[1]);"
                   "return e ? (e.id || e.className.baseVal || e.className || e.tagName) : null",
                   blk["x"], blk["y"])
    flow.check("clicking the block opens the editor pre-filled with its text",
               ed and OLD in ed["value"], f"editor {ed}; element under pointer: {top} "
               "(the hotspot is below #wet-layer; the default Pan tool takes the click)")
    if not ed or OLD not in ed["value"]:
        # Not user-reachable: click the hotspot element directly so the writer is still exercised.
        if ed:
            app.click_button("Cancel", "[data-testid=inline-pdf-text-editor] button")
        app.s.js("const h = [...document.querySelectorAll('.pdf-text-block-hotspot')]"
                 ".find(e => (e.getAttribute('aria-label') || '').includes(arguments[0])); if (h) h.click();",
                 OLD.split()[0])
        ed = app.s.wait("const t = document.querySelector('[data-testid=inline-pdf-text-editor] textarea');"
                        "return t ? {value: t.value} : null", 5)
        flow.info("fallback", "hotspot clicked from JS (element.click) because a pointer click cannot reach it")
        flow.check("fallback (JS click on hotspot) opens the editor pre-filled", ed and OLD in ed["value"],
                   f"editor {ed}")
        if not ed:
            app.quit()
            return
    app.set_input("[data-testid=inline-pdf-text-editor] textarea", NEW)
    app.dismiss_status()
    app.click_button("Commit text", "[data-testid=inline-pdf-text-editor] button")
    msg = app.s.wait("""
        const b = document.querySelector('.studio-status-banner span'); const t = b ? b.textContent.trim() : '';
        return /Structural text/i.test(t) ? t : null;""", 30)
    flow.info("commit_message", msg)
    m = re.search(r" to (/.+\.pdf)\s*$", msg or "")
    out = m.group(1) if m else None
    new_file = bool(out) and os.path.exists(out) and os.path.abspath(out) != os.path.abspath(path)
    flow.check("commit writes a NEW file", new_file, msg or "no status message")
    flow.check("original unchanged (hash)", flow.original_unchanged(path), "")
    if new_file:
        txt = common.pdftotext(out) or ""
        flow.info("output_text", txt.strip()[:200])
        flow.check("output text contains the new text", NEW in txt, repr(txt.strip()[:120]))
        flow.check("output text no longer contains the old text", OLD not in txt, repr(txt.strip()[:120]))
        ok, detail = common.qpdf_check(out)
        flow.check("output passes qpdf --check", ok, detail)
    errs = app.errors()
    flow.warn("no console errors", not errs, " | ".join(errs)[:300])
    time.sleep(0.2)
    app.quit()
