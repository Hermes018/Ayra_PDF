# DOGFOOD_AUTO (real-world PDFs, DOGFOOD AppImage 0.1.0, SHA verified)

Run 2026-11-06 via master/e2e harness code (Flow/App/wd) + ~/lpdf-dogfood-run/scripts/dogfood_one.py. IDs only; no content.

Class: all 8 = public by heuristic (blank forms/instructions, a published paper, a sample file): no AcroForm field values, no SSN-pattern text, browser print-to-PDF or generator output. Owner may confirm.

| ID | class | pages | size MB | 1 Open | 2 Render | 3 Text | 4 Forms | 5 Markup | 6 Redact | 7 Large | open s | peak RSS MB | notes |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| DF-01 | public | 2 | 0.2 | PASS | PASS | PASS | N/A (no fields) | PASS | N/A | PASS | 1.8 | 675 | text overlap prec 0.953 cov 0.661, 594 blocks p1 |
| DF-02 | public | 16 | 1.0 | PASS | PASS | PARTIAL | N/A (no fields) | PASS | N/A | PASS | 1.79 | 714 | text overlap prec 0.844 cov 0.258, 27 blocks p1 |
| DF-03 | public | 648 | 2.2 | PASS | PASS | PARTIAL | N/A (no fields) | PASS | N/A | FLAG | 1.63 | 2930 | text overlap prec 0.8 cov 0.032, 1 blocks p1; render sampled 45/648 pages; peak RSS > 2 GiB |
| DF-04 | public | 37 | 3.9 | PASS | PASS | PARTIAL | N/A (no fields) | PASS | PASS | PASS | 1.99 | 943 | text overlap prec 0.755 cov 0.305, 74 blocks p1 |
| DF-05 | public | 30 | 26.5 | PASS | PASS | PARTIAL | N/A (no fields) | PASS | N/A | PASS | 3.72 | 1166 | text overlap prec 0.87 cov 0.435, 24 blocks p1 |
| DF-06 | public | 6 | 0.5 | PASS | PASS | PARTIAL | N/A (no fields) | PASS | N/A | PASS | 1.79 | 656 | text overlap prec 0.896 cov 0.455, 49 blocks p1 |
| DF-07 | public | 18 | 1.5 | PASS | PASS | PARTIAL | N/A (no fields) | PASS | N/A | PASS | 1.33 | 743 | text overlap prec 0.891 cov 0.149, 18 blocks p1 |
| DF-08 | public | 20 | 5.7 | PASS | PASS | PARTIAL | N/A (no fields) | PASS | N/A | PASS | 2.06 | 950 | text overlap prec 0.825 cov 0.15, 33 blocks p1 |

Legend: PARTIAL = Edit Text hotspot labels vs pdftotext page 1 word overlap below thresholds (precision >= 0.90 and coverage >= 0.60); ratios shown, see bugs.md. N/A = not applicable to the file.
Step 5 detail (all 8 files, 12 checks each): new file created, original hash unchanged, qpdf --check ok, poppler page count equal, Ink x2 + Square + FreeText annotations present in the saved file, text box text in /Contents and pdftotext, reopen in our app restores all markups, pdf.js lists >= 3 annotations and renders them differently from the original.
Step 1: no encrypted or damaged file in this set, so the clear-message path is manual-needed (covered by E2E flow 2). Step 4: none of the files has form fields (all flattened), so not exercised: manual-needed with a real fillable form.
Step 6 (one public research paper, DF-04): sentence gone: yes; qpdf --check ok; rest of text kept; original unchanged.
Step 7: only DF-03 (648 pages, 2.2 MB) exceeded 2 GiB (2930 MB peak across the harness sessions). Max open time 3.7 s (DF-05, 26 MB); none above 5 s.
Process hygiene: one app instance at a time, reaped after each file; 0 leaked/surviving harness processes in all runs (BUG-031 post-run check).

Corpus candidates (public, owner must approve; none added): DF-01, DF-02, DF-03, DF-04, DF-05, DF-06, DF-07, DF-08 (DF-04 and DF-05 are the larger / image-heavy; confirm licence/redistribution for each before any addition).
