# 07 — FEATURE MAP (rev 2 · 2026-10-10) — 1:1 with the 7-column reference UI

Supersedes rev 1. Amended per user "System Update" (items 1, 3, 5). Decisions: ADR-003..ADR-006 in `logs/DECISIONS_ADR.md`.
Source legend: **adopt** = pdfcraft crate behind `PdfEngine` · **own** = we build · **ext** = optional external process (user-installed, never bundled) · **CI** = oracle only.
All API names below are *intentions*; every engine call must be verified from source and recorded in `protocol/ENGINE_NOTES.md` (rule DO-2). Verify the live iLovePDF column/tile list in sub-phase 0.3 and amend via PLAN_LOG.

## 1. Taxonomy rules (enforced by `cargo xtask feature-map-check` + UI snapshot test)
1. Home grid renders exactly **7 columns**, in this order, with these labels (i18n keys `cat.organize` … `cat.intelligence`):
   `ORGANIZE PDF` · `OPTIMIZE PDF` · `CONVERT TO PDF` · `CONVERT FROM PDF` · `EDIT PDF` · `PDF SECURITY` · `PDF INTELLIGENCE`.
2. Every tool belongs to exactly ONE column. Tile order inside a column = row order in the tables below.
3. A tool that is unavailable at runtime (LibreOffice missing, no AI model installed) stays **visible but greyed out** with a one-line reason and an "How to enable" link to local docs. Never hide tiles (parity).
4. Tool registry: `app-tauri/ui/src/tools/registry.ts` exports `{ id, category, labelKey, descKey, iconId, jobKind, route, availabilityProbe }[]`. `feature-map-check` fails if registry ids/categories differ from this file.
5. Non-reference extras (Markdown → PDF, Workflows) are shown in a separate "Extras" strip BELOW the 7-column grid, never inside a column.
6. **Assumption to confirm in 0.3:** the reference list given to us had no column labels per tool. Column placement below follows iLovePDF's public site as best known (Remove/Extract pages and Scan to PDF under Organize; Sign/Redact/Compare under Security; PDF Forms under Edit; PDF to Markdown under Convert From). If 0.3 shows a different live placement, change it here (one-line edits) and in the registry.

Coverage check: 31 reference tools → 4 + 3 + 5 + 6 + 6 + 5 + 2 = **31** (plus 2 tiles "Remove pages"/"Extract pages" that are deep links into Split modes, not separate engines).

## 2. Columns

Column format: Tool (exact tile label) · job kind · engine path · options shown in the option panel · offline notes · phase.

### 2.1 ORGANIZE PDF
| Tool | Job kind | Engine / source | Option panel (frontend) | Offline notes / limits | Phase |
|---|---|---|---|---|---|
| Merge PDF | `organize.merge` | `PdfEngine::combine` (adopt) | File cards, pointer-drag reorder + Alt+Arrow keyboard move, Sort A–Z/Z–A, add more files, per-file password prompt, bookmark policy (none / one per file) | No file-count cap; streaming read per tier budget | 4.1 |
| Split PDF | `organize.split` | page-range extract (adopt) | Modes: custom ranges · fixed ranges · every N pages · by target size · by bookmarks · extract all pages; "merge extracted into one file" toggle; output folder or ZIP | Size mode = greedy page packing then verify | 4.2 |
| Remove pages | `organize.remove_pages` | page delete (adopt) | Visual page select + range text (`1,3-5`) | Deep link: Split tool, "remove" mode | 4.3 |
| Extract pages | `organize.extract_pages` | page extract (adopt) | Visual page select + range text; one file or one-per-page | Deep link: Split tool, "extract" mode | 4.3 |
| Organize PDF | `organize.edit_pages` (+ editor session) | page ops + undo (adopt) + own UI | Grid editor: multi-select, drag-reorder, rotate L/R, delete, duplicate, insert blank / from file, undo/redo, live thumbs; Save / Save as | Editor holds a session in the worker; save = job (atomic) | 4.4 |
| Scan to PDF | `convert.scan_import` | own (`image` crate) | Import images from disk; auto-crop, deskew, enhance (colour / grayscale / B&W); reorder; page size; output PDF | **Offline adaptation (ADR-005):** NO mobile-to-browser sync, NO QR handoff. Scanner hardware support is a separate stretch item (§3.2) | 7.4a (core), 7.4b (stretch) |

### 2.2 OPTIMIZE PDF
| Tool | Job kind | Engine / source | Option panel | Offline notes | Phase |
|---|---|---|---|---|---|
| Compress PDF | `optimize.compress` | pdfcraft optimizer (adopt) + own profiles | Level: Extreme / Recommended / Less (verify names in 0.3); batch; before/after size | Never-grow (F12): returns original + "already optimized" | 5.1 |
| Repair PDF | `optimize.repair` | tolerant parse → rewrite → fallback adapter | Report of fixes; output new file | Degraded-mode notice when fallback used (F7) | 5.2 |
| OCR PDF | `optimize.ocr` | `ocrs` via pdfcraft (Latin); Tesseract (ext) for other languages | Language, deskew, DPI, per-page progress; output searchable PDF | Models CC-BY-SA → ATTRIBUTION; Tesseract optional/user-installed | 5.3 |

### 2.3 CONVERT TO PDF
| Tool | Job kind | Engine / source | Option panel | Offline notes | Phase |
|---|---|---|---|---|---|
| JPG to PDF | `convert.images_to_pdf` | create-from-images (adopt) | Page size (fit image / A4 / Letter), orientation, margin (none/small/big), merge all into one PDF or one each; formats JPG/PNG/TIFF/BMP/GIF/WebP | — | 7.1 |
| Word to PDF | `convert.office_to_pdf` | LibreOffice headless (ext) | Input .doc/.docx/.odt | Needs user-installed LibreOffice; probe at startup; grey-out if absent. Runs in sandbox, no network, timeout (F20) | 7.2 |
| PowerPoint to PDF | `convert.office_to_pdf` | same | .ppt/.pptx/.odp | same | 7.2 |
| Excel to PDF | `convert.office_to_pdf` | same | .xls/.xlsx/.ods; fit-to-width option | same | 7.2 |
| HTML to PDF | `convert.html_to_pdf` | native webview print path behind `HtmlPrinter` trait (own) | Pick local `.html` file (or paste HTML); page size, margins, orientation, print backgrounds | **Offline adaptation (ADR-005):** local files only. NO URL field in the UI. Remote subresources blocked, JS disabled. If a URL is needed it explicitly requires network and is OUT of scope | 7.3 (stretch) |

### 2.4 CONVERT FROM PDF
| Tool | Job kind | Engine / source | Option panel | Offline notes | Phase |
|---|---|---|---|---|---|
| PDF to JPG | `export.pdf_to_jpg` | render export (adopt) | Mode: pages → JPG, or extract embedded images; DPI/quality; page range; ZIP | — | 8.1 |
| PDF to Word | `export.pdf_to_docx` | `lpdf-layout` + own OOXML writer; pdfcraft export for baseline | Mode: **Flow (editable)** / **Exact layout**; OCR-first toggle for scans | **ADR-006:** no perfect 1:1 reflow promised. Quality report shown after conversion | 8.2 |
| PDF to PowerPoint | `export.pdf_to_pptx` | `lpdf-layout` + own OOXML writer | One slide per page; Exact layout (default) / Text-and-images | ADR-006 | 8.3 |
| PDF to Excel | `export.pdf_to_xlsx` | table detection (`lpdf-layout`) + `rust_xlsxwriter` | One sheet per table / per page; pages without tables are reported, not faked | ADR-006 | 8.3 |
| PDF to PDF/A | `export.pdf_to_pdfa` | pdfcraft preflight (verify) ; veraPDF = CI oracle | Conformance: PDF/A-1b / 2b / 3b | Fonts must be embeddable; failures listed per rule, never claim compliance unverified | 8.4 |
| PDF to Markdown | `export.pdf_to_md` | `lpdf-layout` heuristics (own) | Page-break policy, image extraction to `assets/`, table style, OCR fallback | Headings/lists/tables/links preserved per ADR-006 heuristics | 9.2 |

### 2.5 EDIT PDF
| Tool | Job kind | Engine / source | Option panel | Offline notes | Phase |
|---|---|---|---|---|---|
| Rotate PDF | `edit.rotate` | page ops (adopt) | Rotate all or per-page selection, 90/180/270, multiple files | — | 4.5 |
| Page numbers | `edit.page_numbers` | header/footer (adopt) | Position (3×3), margin, first number, page range, font family/size/colour, format (`1`, `Page 1 of N`) | Font = bundled subset or system via fontdb (vetted) | 6.2 |
| Watermark | `edit.watermark` | watermark (adopt) | Text or image; font/size/colour; transparency; rotation; position; layer over/under; page range | — | 6.2 |
| Crop PDF | `edit.crop` | page boxes (adopt) | Draw crop rectangle; apply to this page or all pages; numeric margins | Sets CropBox; "flatten crop" option rewrites content | 6.2 |
| Edit PDF | `edit.session` | annot + content writer (adopt) + own canvas UI (Svelte) | Add text, images, shapes, highlight, freehand draw; edit size/font/colour of ADDED items; undo/redo | Editing *existing* page text = experimental flag only | 6.3 |
| PDF Forms | `edit.forms` | `pdfcraft-forms` (adopt, verify) | Auto-detect fields; create fillable (text, checkbox, radio, dropdown/list); fill and save; flatten | XFDF import/export if crate supports | 6.5 |

### 2.6 PDF SECURITY
| Tool | Job kind | Engine / source | Option panel | Offline notes | Phase |
|---|---|---|---|---|---|
| Unlock PDF | `security.unlock` | crypt crate (adopt) | Password field (user-supplied) | No cracking (DON'T-8). Files needing an open-password without it are refused with a clear message | 6.1 |
| Protect PDF | `security.protect` | AES-256 (adopt) | User/owner password, confirm; permissions (print/copy/edit) | Passwords in `secrecy` types, zeroized | 6.1 |
| Sign PDF | `security.sign` | visual stamp (adopt) + PAdES B-B (adopt) + own UI | **Sign yourself only.** Draw / type / image signature, place & resize on pages; optional digital signature with local PKCS#12 (.p12/.pfx) | **ADR-005:** NO "request signatures from others", no e-mail, no signer tracking, no timestamp authority (needs network → PAdES B-T/LTV out of scope) | 6.4 |
| Redact PDF | `security.redact` | redaction (adopt) | Draw boxes / search-and-mark text; apply | True content removal verified by re-extraction + byte grep (gate 6.4) | 6.4 |
| Compare PDF | `security.compare` | compare (adopt) | Two files; side-by-side, text diff + visual diff overlay | Local only; report exportable as PDF | 6.4 |

### 2.7 PDF INTELLIGENCE
| Tool | Job kind | Engine / source | Option panel | Offline notes | Phase |
|---|---|---|---|---|---|
| AI Summarizer | `ai.summarize` | **Basic mode:** own extractive (TextRank-style, ~200 LOC, no model). **Local-LLM mode:** `lpdf-ai-worker` + user-supplied GGUF model | Length (short/medium/long), language of summary, bullet / paragraph; mode auto: LLM if a model is installed, else Basic | **ADR-004:** default OFF for LLM; Basic mode always available. Never network | 10.5a (Basic) · 10.5c (LLM) |
| Translate PDF | `ai.translate` | `lpdf-ai-worker` + user-supplied model; block-level in-place replacement via redaction + re-typeset | Source/target language, model picker, quality warning | Greyed out until a model is installed. "Fonts/layout intact" is **best effort** (ADR-004 §Translate): text blocks keep position and size, font reused only if glyph coverage exists else fallback | 10.5d |

## 3. Offline adaptations (explicit)
### 3.1 Sign PDF (ADR-005)
- Included: draw/type/image visual signature stamping; sign-yourself digital signature with a **local PKCS#12 file**; self-signed certificate generator (own, via a vetted `rcgen`-class crate — vet in 0.5) for users who have no certificate; verification of existing signatures (status shown, trust chain = user-imported roots only).
- Excluded: request/collect signatures from others, signer e-mail flows, audit trail server, remote timestamping (TSA), OS keystore/PKCS#11 hardware tokens (revisit later via ADR).
### 3.2 Scan to PDF (ADR-005)
- **7.4a (committed):** import images/files from disk + auto-crop, deskew, enhance → PDF. Fully portable.
- **7.4b (stretch, default = DROPPED unless user approves):** scanner hardware. Linux: drive user-installed `scanimage` (SANE) as an external process (GPL tool, never bundled/linked; needs user approval + ADR because DON'T-9). Windows: WIA via COM in a single allow-listed `unsafe` module (needs ADR for the unsafe exception). macOS: ImageCaptureCore (later). If not approved, the tile still works with 7.4a only and the UI says "import scans from files".
### 3.3 HTML to PDF (ADR-005)
- Input: local `.html` file (+ its local relative assets) or pasted HTML. No URL input anywhere.
- Enforcement: webview created with network blocked (custom scheme resolver serving only the chosen directory), JS disabled, remote URLs rewritten to blank, CSP `default-src 'none'; img-src file-scope-only`. Test: run under `unshare -n` with a page that references `https://example.com/x.png` → output has no request attempts and shows placeholder.
- Print path per OS behind trait `HtmlPrinter`: Linux WebKitGTK print-to-file, Windows WebView2 `PrintToPdf`, macOS WKWebView `createPDF`. If an OS path fails the ADR-required spike (0.5), tile is greyed out on that OS.

## 4. Extras strip (not part of the 7 columns)
| Tool | Job kind | Source | Phase |
|---|---|---|---|
| Markdown to PDF | `convert.md_to_pdf` | backend chosen by ADR in 0.5 (Typst-as-library vs krilla-based own layout vs comrak+HTML print) | 9.1 |
| MD ↔ DOCX / HTML, DOCX → MD, HTML → MD | `convert.md_*` | comrak + own writers | 9.3 |
| Workflows | `pipeline.run` | own pipeline engine + presets | 10.2 |

## 5. Out of scope (offline product)
Cloud storage import, e-mail/signature requests, accounts, premium tiers, usage limits, URL fetching of any kind, mobile-browser handoff, telemetry.
