# UX Specification: PDF SECURITY

## 1. Unlock PDF
- **Empty State**: Standard dropzone.
- **Loaded State**: File list.
- **Interaction**: If the file has an open-password, a prominent password input field appears. User must type the password to unlock. No brute-force.
- **Processing State**: "Unlocking PDF...".
- **Done State**: Success message. Download unlocked file.

## 2. Protect PDF
- **Empty State**: Standard dropzone.
- **Loaded State**: Option panel.
- **Option Panel**:
  - User password (required to open).
  - Owner password (required to change permissions).
  - Permissions checkboxes: Allow printing, copying, editing.
- **Processing State**: "Encrypting PDF...".
- **Done State**: Success message. Download protected file.

## 3. Sign PDF
- **Empty State**: Standard dropzone.
- **Loaded State**: Two-step flow. 1) Configure signature, 2) Place on document.
- **Option Panel (Configure)**:
  - Visual: Draw signature, Type signature (with font choices), or Upload image.
  - Digital: Upload local PKCS#12 (.p12/.pfx) certificate, input password.
- **Interaction**: Drag and drop visual signature onto the page preview. Resize.
- **Processing State**: "Signing PDF...".
- **Done State**: Success message. Download signed file.

## 4. Redact PDF
- **Empty State**: Standard dropzone.
- **Loaded State**: Page preview.
- **Interaction**: Draw rectangles to mark areas for redaction. Search bar to find and mark text.
- **Processing State**: "Applying redactions...".
- **Done State**: Success message. Download redacted file.

## 5. Compare PDF
- **Empty State**: Two distinct dropzones ("Original File" and "Modified File").
- **Loaded State**: Side-by-side or overlay comparison view.
- **Option Panel**: View modes (side-by-side, visual diff overlay, text diff).
- **Processing State**: "Comparing...".
- **Done State**: Shows comparison UI. Option to export report.
