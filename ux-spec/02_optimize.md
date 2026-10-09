# UX Specification: OPTIMIZE PDF

## 1. Compress PDF
- **Empty State**: Standard dropzone.
- **Loaded State**: File list in main area. Right option panel.
- **Option Panel**:
  - **Compression Levels**: 
    1. Extreme Compression (Less quality, high compression).
    2. Recommended Compression (Good quality, good compression). Default.
    3. Less Compression (High quality, less compression).
- **Processing State**: "Compressing PDF...".
- **Done State**: Success message showing compression ratio (e.g., "Your PDF is now 60% smaller!"). Shows before/after sizes. If never-grow F12 is triggered, says "File is already optimized." Download file.

## 2. Repair PDF
- **Empty State**: Standard dropzone.
- **Loaded State**: File list. No option panel (or just a "Repair" button).
- **Processing State**: "Repairing PDF...".
- **Done State**: Success message with a report of fixes applied. Degraded-mode notice if fallback adapter was used. Download file.

## 3. OCR PDF
- **Empty State**: Standard dropzone.
- **Loaded State**: File preview/list.
- **Option Panel**:
  - **Language**: Dropdown for document language.
  - Deskew toggle.
  - DPI setting for rendering before OCR.
- **Processing State**: "Running OCR...". Per-page progress indicator.
- **Done State**: "PDF is now searchable!". Download file.
