"""Flow 2: open PDFs.

(a) small, 200+ page, scanned and encrypted files open (encrypted: a password prompt appears and
    `lpdf-test` opens it). (b) The Studio's page count equals `pdfinfo`. (c) Malformed / hostile
    files show a user-visible error or degraded-mode notice, and the app stays alive and responsive.
Files are opened the way the OS does it: as a command-line argument (get_startup_files).
"""
import time

import common
import wd

NUM = 2
TITLE = "Open PDFs"

GOOD = [  # (corpus file, password, layout timeout s)
    ("standard-text.pdf", None, 30),
    ("standard-form.pdf", None, 30),
    ("scanned.pdf", None, 30),
    ("synthetic-1k.pdf", None, 240),
]
HOSTILE = ["hostile-broken-xref.pdf", "hostile-bombs.pdf"]

USER_VISIBLE_ERROR_JS = r"""
const vis = (e) => { const r = e.getBoundingClientRect(); return r.width > 0 && r.height > 0; };
const hits = [];
for (const e of document.querySelectorAll('.toast, [role=alert], .studio-status-banner, .error, [role=status]')) {
  if (vis(e) && e.textContent.trim()) hits.push(e.textContent.trim().slice(0, 160));
}
const words = /error|could not|couldn't|cannot|failed|damaged|corrupt|degraded|repair|invalid|unsupported/i;
return hits.filter((t) => words.test(t));
"""

PASSWORD_PROMPT_JS = r"""
const vis = (e) => { const r = e.getBoundingClientRect(); return r.width > 0 && r.height > 0; };
const pw = [...document.querySelectorAll('input[type=password]')].find(vis);
return pw ? true : false;
"""


def _responsive(app):
    """The UI still reacts: toggle the Ledger drawer and see it open."""
    try:
        btn = app.s.find('button[aria-label^="Ledger"]')
        if not btn:
            return False, "ledger button missing"
        btn.click()
        ok = app.s.wait("return !!document.querySelector("
                        "'[aria-label=\"Annotation History and Quantity Takeoff Drawer\"]')", 5)
        return bool(ok), "ledger drawer opened" if ok else "ledger drawer did not open"
    except Exception as e:  # noqa: BLE001
        return False, f"{type(e).__name__}: {e}"


def run(flow):
    for name, pw, tmo in GOOD:
        path = flow.corpus_copy(name)
        expected = common.pdf_pages(path)
        t0 = time.monotonic()
        app = flow.launch([path])
        laid_out = app.wait_doc(expected, tmo)
        secs = time.monotonic() - t0
        flow.info(f"open_s[{name}]", round(secs, 2))
        n = app.page_count()
        flow.check(f"{name}: page count matches pdfinfo ({expected})", laid_out and n == expected,
                   f"studio shows {n} page(s), badge '{app.badge()}', after {secs:.1f} s")
        inked = app.canvas_inked(0)
        flow.check(f"{name}: first page rendered (non-blank canvas)", inked and inked > 0.001,
                   f"non-white fraction {inked}")
        errs = app.errors()
        flow.warn(f"{name}: no console errors while opening", not errs, " | ".join(errs)[:300])
        app.quit()

    # encrypted: must prompt for the password and open with it
    path = flow.corpus_copy("encrypted.pdf")
    expected = common.pdf_pages(path, common.ENC_PASSWORD)
    app = flow.launch([path])
    prompted = app.s.wait(PASSWORD_PROMPT_JS, 20)
    if flow.check("encrypted.pdf: password prompt appears", prompted,
                  f"no password field within 20 s; banner='{app.status()}', "
                  f"errors={' | '.join(app.errors())[:250]}"):
        app.set_input("input[type=password]", common.ENC_PASSWORD)
        app.s.keys(wd.KEY_ENTER)
        flow.check(f"encrypted.pdf: opens with lpdf-test, {expected} pages",
                   app.wait_doc(expected, 30), f"studio shows {app.page_count()} page(s)")
    else:
        flow.check(f"encrypted.pdf: opens with lpdf-test, {expected} pages", False,
                   f"blocked by missing prompt; studio shows {app.page_count()} page(s)")
    app.quit()

    # hostile: visible error, alive, responsive
    for name in HOSTILE:
        path = flow.corpus_copy(name)
        app = flow.launch([path])
        msgs = app.s.wait(USER_VISIBLE_ERROR_JS, 15) or []
        flow.check(f"{name}: user-visible error or degraded-mode notice", msgs,
                   f"none shown; console: {' | '.join(app.errors())[:250]}" if not msgs else msgs[0])
        flow.check(f"{name}: app alive", app.alive(), "")
        ok, why = _responsive(app)
        flow.check(f"{name}: app responsive", ok, why)
        app.quit()
