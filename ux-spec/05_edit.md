# UX Specification: EDIT PDF

## 1. Rotate PDF
- **Empty State**: Standard dropzone.
- **Loaded State**: File cards or page grid.
- **Interaction**: "Rotate All" button in option panel (Right/Left), or hover on individual pages/files to rotate.
- **Processing State**: "Rotating PDF...".
- **Done State**: Success message. Download file.

## 2. Page Numbers
- **Empty State**: Standard dropzone.
- **Loaded State**: File preview.
- **Option Panel**:
  - Position grid (3x3).
  - Margin sizes.
  - Page range to apply numbers.
  - First number to start with.
  - Text format (e.g., `Page {n} of {p}`).
  - Typography settings (font, size, color).
- **Processing State**: "Adding page numbers...".
- **Done State**: Success message. Download file.

## 3. Watermark
- **Empty State**: Standard dropzone.
- **Loaded State**: File preview.
- **Option Panel**:
  - **Type**: Text or Image.
  - **Text**: Input field, typography settings.
  - **Image**: Upload button.
  - **Position**: 3x3 grid, rotation angle, transparency slider.
  - **Layer**: Over or Under page content.
- **Processing State**: "Adding watermark...".
- **Done State**: Success message. Download file.

## 4. Crop PDF
- **Empty State**: Standard dropzone.
- **Loaded State**: Single page preview with draggable crop handles.
- **Option Panel**: 
  - Margin inputs (Top, Bottom, Left, Right).
  - Apply to: "All pages" or "Current page".
  - Toggle: "Flatten crop" (permanently rewrite content vs setting CropBox).
- **Processing State**: "Cropping PDF...".
- **Done State**: Success message. Download file.

## 5. Edit PDF
- **Empty State**: Standard dropzone.
- **Loaded State**: Canvas overlay editor.
- **UI Components**: Top toolbar with Add Text, Add Image, Add Shape, Highlight, Draw. Sidebar for editing properties of the selected element (font, color, stroke). 
- **Processing State**: "Applying edits..." (Commit to Rust backend).
- **Done State**: Success message. Download file.

## 6. PDF Forms
- **Empty State**: Standard dropzone.
- **Loaded State**: Canvas view.
- **Option Panel**: Auto-detect fields button. Toolbar to insert Text Field, Checkbox, Radio, Dropdown. Fill mode vs Edit mode.
- **Processing State**: "Saving form...".
- **Done State**: Success message. Download file.
