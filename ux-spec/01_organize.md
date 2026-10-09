# UX Specification: ORGANIZE PDF

## 1. Merge PDF
- **Empty State**: Centered dropzone "Drop PDFs here or Select files".
- **Loaded State**: Horizontal/Grid list of file cards. Each card shows a thumbnail, filename, and page count. Hovering a card reveals a "remove" (X) button.
  - **Interaction**: Pointer-drag to reorder cards. Keyboard Alt+Arrow to move selected card. Sort A-Z / Z-A buttons available in the toolbar.
  - **Add More**: Large '+' button at the end of the list or in the side panel.
  - **Option Panel**: Shows list of files, total page count. Options to sort. If any file is encrypted, a password prompt appears on its card.
- **Processing State**: Progress bar "Merging files...", showing percentage. Cancel button.
- **Done State**: "PDFs have been merged!". Download button, "Save to Drive" (hidden/removed for offline), "Open Folder".
- **Error State**: Toast or dialog with typed error code and "Copy diagnostics".
- **Responsive**: Grid reflows to 2 columns on tablet, 1 column on mobile. Options panel moves to bottom sheet on mobile.

## 2. Split PDF
- **Empty State**: Standard dropzone.
- **Loaded State**: File preview on the left. Options panel on the right.
- **Option Panel Modes**:
  - **Custom ranges**: Input fields to define ranges (e.g., 1-5, 8-10). "Add range" button. Toggle "Merge extracted into one file".
  - **Fixed ranges**: Input "Split in ranges of [ N ] pages".
  - **Every N pages**: "Extract every [ N ] pages".
  - **By size**: "Split by file size (Max MB)".
  - **By bookmarks**: Split at top-level bookmarks.
  - **Extract all pages**: Splits every page into a separate file.
- **Processing State**: Progress bar "Splitting PDF...".
- **Done State**: Success message. "Download ZIP" (or folder output).
- **Deep Links (Remove pages / Extract pages)**: Pre-selects the "Extract" mode or a "Remove" mode variant of the UI.

## 3. Organize PDF
- **Empty State**: Standard dropzone.
- **Loaded State**: Grid of all pages from the PDF(s). Each page is a card.
- **Interaction**: Multi-select, pointer-drag to reorder pages. Rotate Left/Right and Delete icons on hover or via toolbar for selected. Insert blank page or insert from another file via '+' button between pages.
- **Option Panel**: Tools to Sort (A-Z, Z-A).
- **Processing State**: "Saving changes...".
- **Done State**: Success message. "Download file".

## 4. Scan to PDF
- **Empty State**: Standard dropzone for images.
- **Loaded State**: Grid of imported images. 
- **Option Panel**: Auto-crop toggle, Deskew toggle. Color enhancement options (Original, Grayscale, B&W). Page size and orientation.
- **Processing State**: "Processing images...".
- **Done State**: Success message. Download file.
