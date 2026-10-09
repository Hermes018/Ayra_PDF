# UX Specification: EXTRAS

## 1. Markdown to PDF
- **Empty State**: Dropzone for `.md` files.
- **Loaded State**: File list.
- **Option Panel**:
  - Theme selection (e.g., GitHub, Light, Dark).
  - Page size and orientation.
  - Toggles: "Generate Table of Contents", "Include Code Highlighting", "Render Math".
- **Processing State**: "Converting to PDF...".
- **Done State**: Success message. Download file.

## 2. Markdown ↔ Word / HTML
- **Empty State**: Dropzone for `.md`, `.docx`, or `.html` files.
- **Loaded State**: File list.
- **Option Panel**: Target format dropdown.
- **Processing State**: "Converting format...".
- **Done State**: Success message. Download file(s).

## 3. Workflows
- **Empty State**: List of available workflow presets (e.g., "Merge then Compress", "Scan then OCR").
- **Loaded State**: Workflow builder / runner view. Dropzone for initial files.
- **Processing State**: Pipeline progress (shows each step completing).
- **Done State**: Success message. Download final output.
