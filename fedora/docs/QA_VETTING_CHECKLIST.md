# LocalPDF Studio — Enterprise QA Vetting Checklist & Test Plan (Release Candidate 1)

**Target Release:** `localpdf-studio-0.1.0-1.x86_64.rpm`  
**QA Auditor / Operator:** __________________________________  
**Date of Audit:** __________________________________  

---

## 1. System & Environment Record

Record the host under test prior to executing test cases:

- [ ] **Fedora Version:** `cat /etc/fedora-release` -> _________________________
- [ ] **Desktop Environment:** `echo $XDG_CURRENT_DESKTOP` (e.g. `KDE`) -> _________________________
- [ ] **Session Type:** `echo $XDG_SESSION_TYPE` (e.g. `wayland` / `x11`) -> _________________________
- [ ] **Kernel Release:** `uname -r` -> _________________________
- [ ] **Total Memory (RAM):** `free -h` -> _________________________
- [ ] **SELinux Enforcing State:** `getenforce` (Expected: `Enforcing`) -> _________________________
- [ ] **WebKit2GTK Version:** `rpm -q webkit2gtk4.1` -> _________________________

---

## 2. Installation & Lifecycle Checks

| Test ID | Action | Expected Result | Pass / Fail |
|---|---|---|---|
| **LIFECYCLE-01** | `sudo dnf install ./localpdf-studio-0.1.0-1.x86_64.rpm` | Installs cleanly without dependency breakage; binary placed at `/usr/bin/localpdf-studio`. | [ ] PASS  [ ] FAIL |
| **LIFECYCLE-02** | `rpm -V localpdf-studio` | Verification succeeds with zero checksum/attribute discrepancies. | [ ] PASS  [ ] FAIL |
| **LIFECYCLE-03** | FreeDesktop Desktop Entry Verification | `/usr/share/applications/localpdf-studio.desktop` exists; contains `Categories=Office;Utility;` and `MimeType=application/pdf;`. | [ ] PASS  [ ] FAIL |
| **LIFECYCLE-04** | KDE Dolphin Context Menu | Right-clicking any `.pdf` in Dolphin displays "LocalPDF Studio" in Open With menu. | [ ] PASS  [ ] FAIL |
| **LIFECYCLE-05** | KDE Dolphin Open-With Execution | Launching via Dolphin context menu passes file argument (`%F`) and loads document directly into studio without crash. | [ ] PASS  [ ] FAIL |
| **LIFECYCLE-06** | Application Launch under `unshare -n` | Executing `unshare -n /usr/bin/localpdf-studio` launches immediately with full functionality offline. | [ ] PASS  [ ] FAIL |
| **LIFECYCLE-07** | Clean Uninstallation | `sudo dnf remove localpdf-studio` cleanly removes binaries, icons, and `.desktop` entry without stray orphans. | [ ] PASS  [ ] FAIL |

---

## 3. Per-Tool Verification Test Cases (All 31 Primary Tools + Variants)

### Category 1: Organize PDF

- [ ] **ORG-01: Merge PDF**
  - *Input:* 3 distinct multi-page PDFs (`doc1.pdf`, `doc2.pdf`, `doc3.pdf`).
  - *Action:* Add to queue, drag to reorder, click Merge.
  - *Expected:* Output file created atomically with exact concatenated page order; originals intact.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **ORG-02: Split PDF (By Range)**
  - *Input:* 10-page document (`input.pdf`).
  - *Action:* Configure ranges `1-3`, `4-7`, `8-10`.
  - *Expected:* 3 distinct output PDFs created in target folder matching requested page intervals.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **ORG-03: Remove Pages**
  - *Input:* 5-page document (`input.pdf`).
  - *Action:* Mark pages 2 and 4 for removal.
  - *Expected:* Output has exactly 3 pages (formerly 1, 3, 5); pages 2 and 4 completely eliminated.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **ORG-04: Extract Pages**
  - *Input:* 20-page document (`input.pdf`).
  - *Action:* Extract pages `5,12,19`.
  - *Expected:* New 3-page document created containing selected pages in requested order.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **ORG-05: Organize PDF (Grid Visual Editor)**
  - *Input:* 12-page document.
  - *Action:* Reorder page tiles via pointer-event drag, rotate page 3 by 90°, delete page 6, click Save As.
  - *Expected:* Output accurately mirrors thumbnail grid arrangement; operations reflect in saved document.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **ORG-06: Scan to PDF (Image Import)**
  - *Input:* Folder of loose images (`scan1.jpg`, `scan2.png`, `scan3.webp`).
  - *Action:* Set page size to A4, apply auto-crop/deskew, compile to PDF.
  - *Expected:* Cohesive PDF generated with embedded images properly scaled to A4 bounds.
  - *Status:* [ ] PASS  [ ] FAIL

---

### Category 2: Optimize PDF

- [ ] **OPT-01: Compress PDF (High / Medium / Low Profiles)**
  - *Input:* 25 MB graphic-heavy presentation PDF.
  - *Action:* Execute High compression profile.
  - *Expected:* Output file size reduced significantly; text and vectors remain sharp and legible.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **OPT-02: Never-Grow Compression Rule (F12)**
  - *Input:* Pre-optimized, already compressed 1-page PDF.
  - *Action:* Run compression pass.
  - *Expected:* Engine detects compressed output >= input; returns original untouched with "Already optimized" notice.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **OPT-03: Repair PDF**
  - *Input:* Corrupted PDF with damaged XREF table and missing EOF trailer (`hostile_broken_xref.pdf`).
  - *Action:* Run Repair tool.
  - *Expected:* Tolerant fallback parser reconstructs object dictionary and outputs readable valid PDF.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **OPT-04: OCR PDF (Searchable PDF Generation)**
  - *Input:* Scanned document containing pure rasterized text without font layer.
  - *Action:* Run OCR tool (English profile).
  - *Expected:* Output PDF contains invisible text layer matching underlying glyphs; text is selectable and searchable.
  - *Status:* [ ] PASS  [ ] FAIL

---

### Category 3: Convert to PDF

- [ ] **CNV-TO-01: JPG/PNG to PDF**
  - *Input:* Multiple high-resolution JPEG and PNG photographs.
  - *Action:* Convert with "Fit to Page" and small margins.
  - *Expected:* Valid PDF document created with embedded image streams without JPEG re-compression degradation.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **CNV-TO-02: Word to PDF (Sidecar Bridge)**
  - *Input:* Sample `.docx` document with tables and headers.
  - *Action:* Convert to PDF.
  - *Expected:* Headless LibreOffice sidecar invoked under `unshare -n -r`; pixel-accurate PDF rendered.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **CNV-TO-03: PowerPoint to PDF (Sidecar Bridge)**
  - *Input:* Sample `.pptx` presentation.
  - *Action:* Convert to PDF.
  - *Expected:* Accurate slide translation; vector graphics and slide dimensions preserved.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **CNV-TO-04: Excel to PDF (Sidecar Bridge)**
  - *Input:* Sample `.xlsx` spreadsheet with multi-column sheets.
  - *Action:* Convert to PDF with fit-to-width option enabled.
  - *Expected:* Spreadsheets rendered into paginated PDF tables without truncation.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **CNV-TO-05: HTML to PDF (Offline Webview Print)**
  - *Input:* Local `index.html` file referencing local CSS and images.
  - *Action:* Render to PDF via offline `HtmlPrinter`.
  - *Expected:* Layout cleanly paginated; zero network requests attempted (verified under `unshare -n`).
  - *Status:* [ ] PASS  [ ] FAIL

---

### Category 4: Convert from PDF

- [ ] **CNV-FROM-01: PDF to JPG / Images**
  - *Input:* 5-page PDF with raster and vector graphics.
  - *Action:* Render pages to PNG at 150 DPI.
  - *Expected:* 5 crisp image files rendered in target directory; dimensions match aspect ratios.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **CNV-FROM-02: PDF to Word (Structural DOCX)**
  - *Input:* Multi-column text PDF.
  - *Action:* Convert to Word.
  - *Expected:* Generates valid `.docx` with editable paragraphs and tables; accompanied by ADR-006 quality report.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **CNV-FROM-03: PDF to PowerPoint**
  - *Input:* Slide deck PDF.
  - *Action:* Convert to PowerPoint.
  - *Expected:* Valid `.pptx` generated with individual slide canvases corresponding to PDF pages.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **CNV-FROM-04: PDF to Excel**
  - *Input:* PDF document containing structured data tables.
  - *Action:* Convert to Excel.
  - *Expected:* Table extraction produces `.xlsx` file with data populated in distinct grid cells.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **CNV-FROM-05: PDF to PDF/A (Archival Standard)**
  - *Input:* Standard PDF 1.7 document.
  - *Action:* Convert to PDF/A-2b.
  - *Expected:* Fonts embedded, device-independent colorspaces defined, preflight report passes compliance.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **CNV-FROM-06: PDF to Markdown**
  - *Input:* Research paper PDF with headings, lists, tables, and figures.
  - *Action:* Export to Markdown via Pandoc bridge.
  - *Expected:* Clean CommonMark output with extracted figures in `assets/` subfolder.
  - *Status:* [ ] PASS  [ ] FAIL

---

### Category 5: Edit PDF

- [ ] **EDIT-01: Rotate PDF**
  - *Input:* Landscape document misoriented as portrait.
  - *Action:* Rotate 90° clockwise.
  - *Expected:* Page rotation dictionary modified cleanly; views correctly in PDF readers.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **EDIT-02: Page Numbers**
  - *Input:* 15-page unnumbered document.
  - *Action:* Add "Page {n} of {total}" in bottom-right corner, margin 20pt, starting at page 1.
  - *Expected:* Page numbers overlaid consistently across all pages using embedded fonts.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **EDIT-03: Watermark**
  - *Input:* Sensitive draft agreement.
  - *Action:* Apply "CONFIDENTIAL" text watermark at 45° angle, 30% opacity, centered.
  - *Expected:* Watermark rendered beneath or over text cleanly without obliterating readability.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **EDIT-04: Crop PDF**
  - *Input:* Scanned document with excessive printer margin bleed.
  - *Action:* Draw crop box and apply to all pages.
  - *Expected:* `CropBox` updated in page tree; viewer displays cropped view accurately.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **EDIT-05: Edit PDF (Canvas Annotator)**
  - *Input:* Sample contract PDF.
  - *Action:* Add sticky note, highlight paragraph, draw rectangle over clause.
  - *Expected:* Standard PDF annotation elements created (`/Highlight`, `/Text`, `/Square`); editable in external readers.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **EDIT-06: PDF Forms (Fill & Flatten)**
  - *Input:* IRS W-9 or interactive AcroForm PDF.
  - *Action:* Fill text fields, toggle checkboxes, click Flatten Form.
  - *Expected:* Form inputs embedded permanently into page content stream; form fields become static vector text.
  - *Status:* [ ] PASS  [ ] FAIL

---

### Category 6: PDF Security

- [ ] **SEC-01: Protect PDF (AES-256 Encryption)**
  - *Input:* Unencrypted document.
  - *Action:* Protect with password `Enterprise2026!`, restrict copying and editing.
  - *Expected:* PDF encrypted with AES-256 (standard security handler R=6); password prompt required on open.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **SEC-02: Unlock PDF (Decryption)**
  - *Input:* Encrypted document from SEC-01.
  - *Action:* Provide correct password and unlock.
  - *Expected:* Decrypted document generated with encryption dictionaries stripped; opens freely.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **SEC-03: Sign PDF (Self-Signed PAdES & Visual Stamp)**
  - *Input:* Agreement PDF.
  - *Action:* Stamp handwritten vector signature and attach local `.p12` digital certificate.
  - *Expected:* Visual signature positioned correctly; digital signature valid with unbroken cryptographic digest.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **SEC-04: Redact PDF (True Semantic Removal)**
  - *Input:* Document with sensitive PII (SSN, credit card, names).
  - *Action:* Mark PII regions and apply Redact.
  - *Expected:* Underlying text glyphs, vector paths, and content streams are destroyed at the byte level. Verified via `pdftotext` and binary grep (zero residual text).
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **SEC-05: Compare PDF**
  - *Input:* `v1_contract.pdf` and `v2_contract.pdf`.
  - *Action:* Execute visual and text comparison.
  - *Expected:* Red/green side-by-side highlighting showing added and removed paragraphs.
  - *Status:* [ ] PASS  [ ] FAIL

---

### Category 7: PDF Intelligence

- [ ] **INT-01: AI Summarizer (Basic Zero-Model Mode)**
  - *Input:* 10-page academic paper or corporate whitepaper.
  - *Action:* Select Extractive Summary (Medium length).
  - *Expected:* Generates structured Markdown report with key extracted sentences without requiring any downloaded model.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **INT-02: AI Translate PDF (Offline Quality Guards)**
  - *Input:* 2-page document with headings and paragraphs.
  - *Action:* Execute translation job with local model.
  - *Expected:* Block-level replacement maintains bounding box coordinates; outputs translation report.
  - *Status:* [ ] PASS  [ ] FAIL

---

## 4. Sidecar Degradation & Graceful Error Handling

- [ ] **SIDECAR-01: Missing LibreOffice Binary**
  - *Condition:* Temporarily simulate absent LibreOffice by unsetting or renaming binary path.
  - *Action:* Open "Word to PDF" tool.
  - *Expected:* Tile displays greyed out or returns clean `ExternalDependencyError` explaining how to install LibreOffice via DNF. Host process remains stable without crashing.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **SIDECAR-02: Missing Pandoc Binary**
  - *Condition:* Temporarily simulate absent Pandoc binary.
  - *Action:* Execute "PDF to Markdown" export.
  - *Expected:* UI catches error and surfaces friendly message: *"Pandoc is required for Markdown export. Please install via `sudo dnf install pandoc`"*.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **SIDECAR-03: Hostile Sidecar Input Protection**
  - *Condition:* File path containing shell metacharacters (`test; rm -rf /; $(id).docx`).
  - *Action:* Invoke Word to PDF conversion.
  - *Expected:* Rust subprocess API executes direct argv array without shell interpolation. Metacharacters treated literally; no shell injection possible.
  - *Status:* [ ] PASS  [ ] FAIL

---

## 5. UI/UX Responsiveness & Performance Stress Tests

- [ ] **PERF-01: 1,000-Page Document Grid Virtualization**
  - *Input:* Synthetic 1,000-page PDF document.
  - *Action:* Load into Organize Grid Editor, scroll rapidly through thumbnail list.
  - *Expected:* Memory remains bounded under tier limit; virtualized list renders visible items smoothly without freezing UI event loop.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **PERF-02: Wayland Fractional Scaling & HiDPI**
  - *Condition:* KDE Display Settings set to 150% fractional scale on Wayland session.
  - *Action:* Inspect icons, text nodes, thumbnail previews, and dialogs.
  - *Expected:* Vector assets and fonts render with crisp integer/fractional rasterization; zero blurriness or layout clipping.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **PERF-03: Keyboard Navigation & A11y Landmarks**
  - *Action:* Navigate entire home grid and modal dialogs using Tab, Shift+Tab, Enter, Space, and Arrow keys.
  - *Expected:* Logical focus rings visible at all times; all 31 tools accessible and triggerable without mouse interaction.
  - *Status:* [ ] PASS  [ ] FAIL

---

## 6. Hostile Files & Malformed Input Handling

- [ ] **HOSTILE-01: Zero-Byte File**
  - *Input:* Empty 0-byte file named `empty.pdf`.
  - *Action:* Attempt to open or process.
  - *Expected:* Clean error prompt: *"Invalid PDF: file is empty"*; zero panics.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **HOSTILE-02: Decompression Flate Bomb**
  - *Input:* High compression ratio recursive stream (`flate_bomb.pdf`).
  - *Action:* Load and attempt render.
  - *Expected:* Resource guard (F4) trips decompression ratio cap; job terminates gracefully with error code `L004_RESOURCE_LIMIT`. UI survives intact.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **HOSTILE-03: Cyclic Page Tree**
  - *Input:* Malformed PDF with circular `/Parent` / `/Kids` reference loops.
  - *Action:* Open in document viewer.
  - *Expected:* Depth-counter guard prevents infinite recursion and stack overflow; returns `InvalidStructure` error cleanly.
  - *Status:* [ ] PASS  [ ] FAIL

---

## 7. Air-Gapped Network Isolation (F9 Guarantee)

- [ ] **NET-01: Runtime Network Socket Verification**
  - *Action:* Launch LocalPDF Studio under `strace -e trace=network /usr/bin/localpdf-studio`.
  - *Expected:* Zero `connect()`, `sendto()`, or `getaddrinfo()` calls made to external remote IPs. Sockets restricted to local Unix domain sockets for IPC and Wayland display compositor.
  - *Status:* [ ] PASS  [ ] FAIL

- [ ] **NET-02: Strict CSP Enforcement**
  - *Action:* Inspect browser DevTools console in debug build.
  - *Expected:* CSP active: `default-src 'self'; connect-src ipc: http://ipc.localhost`. Any injected remote script or font fails immediately with CSP block violation.
  - *Status:* [ ] PASS  [ ] FAIL

---

### Sign-off & Recommendation

**Overall Assessment:** [ ] ACCEPT (Ready for Release)  [ ] REJECT (Blockers Encountered)  

**Auditor Signature:** __________________________________  
**Date:** ___________________  
