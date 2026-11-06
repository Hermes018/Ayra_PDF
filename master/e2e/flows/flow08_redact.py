"""Flow 8: redact.

Mark a text region (the word's box from `pdftotext -bbox`) with the redaction tool and commit the
redaction. In the NEW output file `pdftotext` no longer contains the redacted string and
`qpdf --check` passes. The original file is unchanged (SHA-256).
"""
import os
import time

import common

NUM = 8
TITLE = "Redact"
TARGET = "Dummy"


def run(flow):
    path = flow.corpus_copy("standard-text.pdf")
    page_w = float(common.pdfinfo(path)["Page size"].split()[0])
    words = common.pdftotext_words(path, 1)
    hit = next((w for w in words if w[0] == TARGET), None)
    if not hit:
        raise common.Blocked(f"'{TARGET}' not found in {path} by pdftotext -bbox")
    before_text = common.pdftotext(path)
    flow.check("ground truth: original contains the target string", TARGET in (before_text or ""), "")
    app = flow.launch([path], expect_pages=1)
    if not flow.check("document open", app.page_count() == 1, f"{app.page_count()} pages"):
        return
    app.snap_off()
    _, x0, y0, x1, y1 = hit
    sx0, sy0 = app.pdf_to_screen(x0 - 2, y0 - 2, page_w)
    sx1, sy1 = app.pdf_to_screen(x1 + 2, y1 + 2, page_w)
    flow.check("select redaction tool", app.select_tool("review", "Cryptographic Redaction"), "")
    n0 = app.vector_nodes()
    app.s.mouse_path([(sx0, sy0), ((sx0 + sx1) / 2, (sy0 + sy1) / 2), (sx1, sy1)])
    time.sleep(0.3)
    flow.check("redaction region marked on canvas", app.vector_nodes() == n0 + 1,
               f"nodes {n0} -> {app.vector_nodes()}")
    has_commit = app.s.find_text(".studio-top-bar button", "Commit redactions") is not None
    if not flow.check("'Commit Redactions' action offered", has_commit, ""):
        app.quit()
        return
    out, msg = app.save(button="Commit redactions")
    flow.info("commit_message", msg)
    new_file = bool(out) and os.path.exists(out) and os.path.abspath(out) != os.path.abspath(path)
    flow.check("commit writes a NEW output file", new_file, msg)
    flow.check("original file unchanged (hash)", flow.original_unchanged(path), "")
    if new_file:
        txt = common.pdftotext(out)
        flow.info("output_text", (txt or "").strip()[:200])
        flow.check(f"pdftotext of the output no longer contains '{TARGET}'",
                   txt is not None and TARGET not in txt, f"output text: {(txt or '<unreadable>').strip()[:120]!r}")
        other = [w[0] for w in words if w[0] != TARGET]
        kept = [w for w in other if w in (txt or "")]
        flow.check("non-redacted words are kept (no over-redaction)", other and len(kept) == len(other),
                   f"kept {kept} of {other}")
        ok, detail = common.qpdf_check(out)
        flow.check("output passes qpdf --check", ok, detail)
    errs = app.errors()
    flow.warn("no console errors", not errs, " | ".join(errs)[:300])
    app.quit()
