# ENGINE NOTES

## Source
- **Repository**: `storytold/pdfcraft`
- **Pinned Commit SHA**: `dc4b3eee84f73bad8564906c141bdda9caad0fd6`
- **Facade Crate**: `pdfcraft-engine`

## Verified Public APIs
The primary point of interaction is the `Session` and `Document` structs in `pdfcraft-engine`:

1. **Load/Open**:
   - `Session::open(&mut self, name: impl Into<String>, path: Option<String>, bytes: Arc<Vec<u8>>, password: Option<&str>) -> Result<DocId, OpenError>`
   - Initializes a document session from raw bytes.

2. **Combine/Merge**:
   - `Session::combine(&self, sources: &[(String, Arc<Vec<u8>>)]) -> Result<Arc<Vec<u8>>, EditError>`
   - Merges multiple PDF byte sources into a single PDF output.

3. **Split**:
   - `Session::split(&self, id: DocId, by: &pdfcraft_organize::SplitBy) -> Result<Vec<SplitPart>, EditError>`
   - Splits a document according to rules defined in `SplitBy`.

4. **Compress/Optimize**:
   - `Session::optimized_bytes(&self, id: DocId, settings: &optimize::Settings, discard: &[Hidden]) -> Result<(Arc<Vec<u8>>, OptimizeReport), EditError>`
   - Compresses a loaded document and returns the optimized bytes along with a report.

5. **Render (Extraction)**:
   - `Document::page_images(&self, page: usize) -> Vec<pdfcraft_edit::PageImage>`
   - Extracts page image metadata/bytes.

## Discrepancies vs Architectural Assumptions
- The APIs operate extensively on `Arc<Vec<u8>>` and `DocId` handles managed by `Session`, rather than stateless single-shot functions. The architectural assumption might have been simpler functional calls; using `Session` requires managing state across edits.
- The return type for split is `Vec<SplitPart>` instead of plain byte vectors, which likely contains metadata (like the split range) along with the bytes.

## Verified Tauri v2 Linux RPM Bundle & Desktop Integration Keys
Verified from `tauri-utils-2.10.1` and `@tauri-apps/cli@2.12.1` (`config.schema.json`):

1. **Top-Level Identity & Targets:**
   - `productName` (string): Sets the user-facing application name and window metadata. On Linux, it is kebab-cased for RPM package naming (`localpdf-studio`).
   - `mainBinaryName` (string): Explicitly overrides the compiled application binary filename (`localpdf-studio`).
   - `bundle.targets`: String or array of strings. Set to `["rpm"]` to isolate RPM bundling.
   - `bundle.category`: String (e.g., `"Office"`, `"Utility"`).

2. **RPM Dependencies & Metadata (`bundle.linux.rpm`):**
   - `bundle.linux.rpm.depends` (array of strings): Maps directly to RPM `Requires:`. Explicit system dependencies such as `"pandoc"`, `"libreoffice"`.
   - `bundle.linux.rpm.recommends` (array of strings): Maps to RPM `Recommends:`.
   - `bundle.linux.rpm.provides` / `conflicts` / `obsoletes` (array of strings): Standard RPM package relationships.
   - `bundle.linux.rpm.release` (string): Package release number (default `"1"`).
   - `bundle.linux.rpm.epoch` (integer): Package epoch (default `0`).
   - `bundle.linux.rpm.desktopTemplate` (string): Path to custom Handlebars `.desktop` file template. Supported template variables: `categories`, `comment`, `exec`, `icon`, `name`, `mime_type`.

3. **File Associations & KDE Dolphin MIME Integration (`bundle.fileAssociations`):**
   - `bundle.fileAssociations` (array of objects):
     - `ext` (array of strings): File extensions without leading dots (`["pdf"]`).
     - `mimeType` (string): Registered MIME type (`"application/pdf"`). Written as `MimeType=application/pdf;` into the desktop entry.
     - `name` (string): Human-readable name (`"PDF Document"`).
     - `description` (string): Extended file format description (`"Portable Document Format"`).
     - `role` (string): `"Editor"` or `"Viewer"`.
   - In conjunction with `desktopTemplate` specifying `Exec={{exec}} %F`, KDE Dolphin passes clicked files to the `localpdf-studio` executable via `%F`.

