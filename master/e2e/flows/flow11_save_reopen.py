"""Flow 11: save and reopen.

Save creates a NEW file (the original's hash is unchanged). Reopening it in our app shows the
markups. The file passes `qpdf --check`. Poppler (`pdfinfo` / `pdftotext`) reads it, and pdf.js
(our viewer, run inside the webview on the saved bytes) lists and renders the annotations
(page 1 rendered with annotations differs from the original's render).
"""
import os
import shutil
import time

import common

NUM = 11
TITLE = "Save and reopen"

PDFJS_ANNOTS_JS = """
const [path, origPath] = arguments;
(async () => {
  const lib = window.pdfjsLib || window['pdfjs-dist/build/pdf'];
  if (!lib) return done({error: 'pdf.js not loaded'});
  lib.GlobalWorkerOptions.workerSrc = '/pdf.worker.min.js';
  const open = async (p) => lib.getDocument({data: new Uint8Array(
      await window.__TAURI_INTERNALS__.invoke('read_file_bytes', {path: p}))}).promise;
  const render = async (page) => {
    const vp = page.getViewport({scale: 1});
    const c = document.createElement('canvas'); c.width = vp.width; c.height = vp.height;
    const ctx = c.getContext('2d');
    const opts = {canvasContext: ctx, viewport: vp};
    if (lib.AnnotationMode) opts.annotationMode = lib.AnnotationMode.ENABLE;
    await page.render(opts).promise;
    return ctx.getImageData(0, 0, c.width, c.height).data;
  };
  const doc = await open(path), orig = await open(origPath);
  const page = await doc.getPage(1);
  const annots = (await page.getAnnotations()).map(a => a.subtype).filter(t => t !== 'Widget');
  const a = await render(page), b = await render(await orig.getPage(1));
  let diff = 0;
  for (let i = 0; i < Math.min(a.length, b.length); i += 4) if (a[i] !== b[i] || a[i+1] !== b[i+1] || a[i+2] !== b[i+2]) diff++;
  done({version: lib.version, pages: doc.numPages, markupAnnots: annots, diffPixelsVsOriginal: diff});
  doc.destroy(); orig.destroy();
})().catch(e => done({error: String(e && e.message || e)}));
"""


def run(flow):
    path = flow.corpus_copy("standard-form.pdf")
    n = common.pdf_pages(path)
    app = flow.launch([path], expect_pages=n)
    if not flow.check("document open", app.page_count() == n, f"{app.page_count()} pages"):
        return
    app.snap_off()
    pr = app.page_rect(0)
    X, Y = pr["x"] + 100, pr["y"] + 200
    app.select_tool("ink", "Technical Pen A")
    app.s.mouse_path([(X + i * 12, Y + (i % 3) * 8) for i in range(15)])
    app.select_tool("shapes", "Rectangle")
    app.s.mouse_path([(X, Y + 60), (X + 80, Y + 100), (X + 160, Y + 140)])
    app.select_tool("review", "Rich Text Box")
    app.s.mouse_click(X + 250, Y + 60)
    if app.s.wait("return !!document.querySelector('[data-testid=inline-pdf-text-editor] textarea')", 5):
        app.set_input("[data-testid=inline-pdf-text-editor] textarea", "E2E reopen note")
        app.click_button("Commit Text", "[data-testid=inline-pdf-text-editor] button")
    time.sleep(0.3)
    marks = app.vector_nodes()
    flow.check("three markups on the page before saving", marks == 3, f"{marks} vector nodes")

    out, msg = app.save()
    flow.info("save_message", msg)
    new_file = bool(out) and os.path.exists(out) and os.path.abspath(out) != os.path.abspath(path)
    flow.check("Save creates a NEW file", new_file, msg)
    flow.check("original's hash unchanged", flow.original_unchanged(path), "")
    errs = app.errors()
    flow.warn("no console errors while saving", not errs, " | ".join(errs)[:300])
    app.quit()
    if not new_file:
        return

    ok, detail = common.qpdf_check(out)
    flow.check("saved file passes qpdf --check", ok, detail)
    flow.check("pdfinfo reads it (same page count)", common.pdf_pages(out) == n, f"{common.pdf_pages(out)} pages")
    txt = common.pdftotext(out, 1, 1)
    flow.check("pdftotext reads it (page 1 text intact)", txt and "W-4" in txt, repr((txt or "")[:60]))
    annots = [a for a in common.page_annotations(out)[0] if a.get("/Subtype") != "/Widget"]
    flow.info("saved_markup_annotations", [a.get("/Subtype") for a in annots])
    flow.check("saved PDF carries the three markups as annotations", len(annots) == 3,
               f"{[a.get('/Subtype') for a in annots]} (form widgets excluded)")

    # reopen the saved file in our app (fresh process)
    app2 = flow.launch([out], expect_pages=n)
    time.sleep(1.0)
    again = app2.vector_nodes()
    flow.check("reopening in our app shows the markups", again == marks, f"{again} vector nodes vs {marks} saved")
    res = app2.s.js_async(PDFJS_ANNOTS_JS, out, path) or {}
    flow.info("pdfjs", res)
    flow.check("pdf.js lists the annotations of the saved file",
               not res.get("error") and len(res.get("markupAnnots", [])) == 3, str(res)[:300])
    flow.check("pdf.js renders the annotations (page 1 render differs from the original's)",
               (res.get("diffPixelsVsOriginal") or 0) > 50,
               f"{res.get('diffPixelsVsOriginal')} differing pixels (pdf.js {res.get('version')})")
    app2.quit()

    # the saved PDF on its own (no .studio.json sidecar next to it), as a recipient would open it
    alone_dir = os.path.join(flow.dir, "recipient")
    os.makedirs(alone_dir, exist_ok=True)
    alone = os.path.join(alone_dir, os.path.basename(out))
    shutil.copyfile(out, alone)
    app3 = flow.launch([alone], expect_pages=n)
    time.sleep(1.0)
    flow.info("sidecars_next_to_output", sorted(f for f in os.listdir(os.path.dirname(out)) if "studio" in f))
    flow.warn("PDF without its sidecar still reopens with editable markups",
              app3.vector_nodes() == marks, f"{app3.vector_nodes()} editable vector nodes (markups only "
              "come back from the *.studio.json sidecar, not from the PDF's own annotations)")
    app3.quit()
