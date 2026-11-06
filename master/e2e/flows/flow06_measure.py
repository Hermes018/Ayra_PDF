"""Flow 6: measure.

Two-point calibration sets the scale (300 screen px == 100 mm at a fixed zoom). Length, area and
angle measurements, drawn with exact pointer coordinates, match the values computed from those
coordinates within 0.5 %. They appear in the Ledger. CSV export writes a file whose rows match.
"""
import csv
import math
import os
import re
import time

import common

NUM = 6
TITLE = "Measure"
TOL = 0.005
DIALOG = '[data-testid="calibration-dialog"]'


def _first_number(text):
    m = re.search(r"-?\d+(?:\.\d+)?", text.replace(",", ""))
    return float(m.group(0)) if m else None


def _ledger_rows(app):
    return app.s.js("return [...document.querySelectorAll('.ledger-row')].map(r => "
                    "[...r.querySelectorAll('td')].map(td => td.textContent.trim()))")


def run(flow):
    path = flow.corpus_copy("standard-text.pdf")
    app = flow.launch([path], expect_pages=1)
    if not flow.check("document open", app.page_count() == 1, f"{app.page_count()} pages"):
        return
    app.snap_off()
    app.set_select('select[aria-label="Zoom Level Preset"]', "75")
    time.sleep(0.3)
    pr = app.page_rect(0)
    X, Y = int(pr["x"]) + 60, int(pr["y"]) + 60

    # --- calibration: 300 px == 100 mm -------------------------------------------------------
    flow.check("select calibrate tool", app.select_tool("measure", "Calibrate Page Scale"), "")
    app.s.mouse_path([(X, Y), (X + 150, Y), (X + 300, Y)])
    dlg = app.s.wait(f"return !!document.querySelector('{DIALOG}')", 5)
    if not flow.check("calibration dialog opens after a two-point drag", dlg, ""):
        app.quit()
        return
    app.set_input(f'{DIALOG} [data-testid="calibration-distance"]', 100, 0)
    unit_ok = app.s.js(f"""
        const s = [...document.querySelectorAll('{DIALOG} select')].find(s => [...s.options].some(o => o.value === 'mm'));
        if (!s) return false; s.value = 'mm'; s.dispatchEvent(new Event('change', {{bubbles: true}})); return true;
    """)
    app.click_button("Save calibrated scale")
    time.sleep(0.3)
    closed = not app.s.js(f"return !!document.querySelector('{DIALOG}')")
    flow.check("calibration saved (100 mm over 300 px)", unit_ok and closed, f"unit set {unit_ok}, closed {closed}")
    k = 100.0 / 300.0  # mm per screen px at this zoom

    # --- measurements ---------------------------------------------------------------------
    expected = []  # (kind, value, unit)
    app.select_tool("measure", "Linear Length")
    app.s.mouse_path([(X, Y + 60), (X + 120, Y + 60), (X + 240, Y + 60)])
    expected.append(("length", 240 * k, "mm"))
    time.sleep(0.2)
    app.s.mouse_path([(X, Y + 100), (X + 90, Y + 220), (X + 180, Y + 340)])
    expected.append(("length", math.hypot(180, 240) * k, "mm"))
    time.sleep(0.2)
    flow.check("select rectangle-area tool", app.select_tool("measure", "Rectangle Area"), "")
    app.s.mouse_path([(X + 300, Y + 100), (X + 380, Y + 180), (X + 450, Y + 250)])
    expected.append(("area", (150 * k) * (150 * k), "mm²"))
    time.sleep(0.2)
    flow.check("select protractor tool", app.select_tool("measure", "3-Point Protractor"), "")
    vx, vy = X + 250, Y + 450
    p1, p2 = (vx - 150, vy), (vx - 75, vy - 130)
    for x, y in (p1, (vx, vy), p2):
        app.s.mouse_click(x, y)
        time.sleep(0.15)
    a1 = math.atan2(p1[1] - vy, p1[0] - vx)
    a2 = math.atan2(p2[1] - vy, p2[0] - vx)
    ang = abs(math.degrees(a1 - a2))
    expected.append(("angle", min(ang, 360 - ang), "deg"))
    time.sleep(0.3)

    # --- ledger ---------------------------------------------------------------------------
    app.s.find('button[aria-label^="Ledger"]').click()
    app.s.wait("return document.querySelectorAll('.ledger-row').length", 5)
    rows = [r for r in _ledger_rows(app) if len(r) >= 3 and r[1].startswith("measure.")]
    flow.info("ledger_rows", rows)
    flow.check("all 4 measurements appear in the Ledger", len(rows) == len(expected),
               f"{len(rows)} measurement rows: {rows}")
    want_types = {"length": "measure.length", "area": "measure.area.rect", "angle": "measure.protractor"}
    for idx, (kind, val, unit) in enumerate(expected):
        row = rows[idx] if idx < len(rows) else None
        got = _first_number(row[2]) if row else None
        ok = row is not None and got is not None and abs(got - val) <= TOL * val and row[1] == want_types[kind]
        flow.check(f"{kind} #{idx + 1} = {val:.2f} {unit} within 0.5 %", ok,
                   f"ledger row {row}" if row else "missing")

    # --- CSV ------------------------------------------------------------------------------
    csv_path = os.path.splitext(path)[0] + ".takeoff.csv"
    if os.path.exists(csv_path):
        os.remove(csv_path)
    clicked = app.click_button("Export takeoff (CSV)")
    for _ in range(50):
        if os.path.exists(csv_path):
            break
        time.sleep(0.1)
    if not flow.check("CSV export writes the takeoff file", clicked and os.path.exists(csv_path),
                      f"expected {csv_path}; banner '{app.status()}'"):
        app.quit()
        return
    with open(csv_path, newline="", encoding="utf-8") as f:
        table = [r for r in csv.reader(f) if r]
    header, body = table[0], []
    for r in table[1:]:
        if r[0] in ("SUMMARY", "TOTAL"):
            break
        body.append(dict(zip(header, r)))
    flow.info("csv_rows", body)
    flow.check("CSV has one row per Ledger measurement", len(body) == len(rows),
               f"{len(body)} CSV rows vs {len(rows)} ledger rows")
    mism = []
    for r, lr in zip(body, rows):
        try:
            cv = float(r["Calibrated Value"])
        except (KeyError, TypeError, ValueError):
            cv = None  # a missing column or non-numeric cell is a mismatch, never NaN-passes
        lv = _first_number(lr[2])
        if (cv is None or not math.isfinite(cv) or lv is None
                or abs(cv - lv) > TOL * max(abs(lv), 1e-9) or r.get("Type") != lr[1].replace("measure.", "")):
            mism.append((r.get("Type"), r.get("Calibrated Value"), lr[1], lv))
    flow.check("CSV rows match the Ledger (type and value)", not mism, f"mismatches {mism}")
    errs = app.errors()
    flow.warn("no console errors", not errs, " | ".join(errs)[:300])
    app.quit()
