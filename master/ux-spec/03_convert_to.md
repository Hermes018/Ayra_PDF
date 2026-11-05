# UX Specification: CONVERT TO PDF

## 1. JPG to PDF
- **Empty State**: Standard image dropzone.
- **Loaded State**: Image grid. Reorder via drag-and-drop.
- **Option Panel**:
  - Page orientation (Portrait, Landscape).
  - Page size (Fit, A4, Letter).
  - Margin (No margin, Small, Big).
  - Toggle: "Merge all images in one PDF file".
- **Processing State**: "Converting images...".
- **Done State**: Success message. Download file.

## 2. Word/PowerPoint/Excel to PDF
- **Empty State**: Standard dropzone for Office formats.
- **Loaded State**: File list.
- **Option Panel**: (For Excel) "Fit to width" toggle.
- **Processing State**: "Converting to PDF...".
- **Done State**: Success message. Download file.
- **Disabled State**: If LibreOffice is not found locally, the tile on the home screen is greyed out with a message "LibreOffice required".

## 3. HTML to PDF
- **Empty State**: Dropzone for `.html` files. Also accepts pasted HTML text.
- **Loaded State**: List of HTML files/snippets.
- **Option Panel**: Page size, orientation, margins. Toggle "Print backgrounds".
- **Processing State**: "Converting HTML to PDF...".
- **Done State**: Success message. Download file.
