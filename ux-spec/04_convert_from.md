# UX Specification: CONVERT FROM PDF

## 1. PDF to JPG
- **Empty State**: Standard dropzone.
- **Loaded State**: File list.
- **Option Panel**:
  - **Modes**: "Page to JPG" (each page as an image) OR "Extract images" (extract embedded images).
  - Image quality/DPI slider or dropdown.
- **Processing State**: "Converting to JPG...".
- **Done State**: Success message. Download ZIP or Folder.

## 2. PDF to Word / PowerPoint / Excel
- **Empty State**: Standard dropzone.
- **Loaded State**: File list.
- **Option Panel (Word)**: Mode toggle between "Flow (editable)" and "Exact layout". OCR-first toggle for scanned documents.
- **Option Panel (PowerPoint)**: Default to Exact layout.
- **Option Panel (Excel)**: One sheet per page / table.
- **Processing State**: "Converting document...".
- **Done State**: Success message + Conversion Quality Report shown. Download file.

## 3. PDF to PDF/A
- **Empty State**: Standard dropzone.
- **Loaded State**: File list.
- **Option Panel**: Conformance level dropdown (PDF/A-1b, PDF/A-2b, PDF/A-3b).
- **Processing State**: "Validating and Converting...".
- **Done State**: Success message or list of validation failures. Download file.

## 4. PDF to Markdown
- **Empty State**: Standard dropzone.
- **Loaded State**: File list.
- **Option Panel**: Page-break policy, image extraction (yes/no), table style. OCR fallback toggle.
- **Processing State**: "Extracting Markdown...".
- **Done State**: Success message. Download ZIP (with `assets/`).
