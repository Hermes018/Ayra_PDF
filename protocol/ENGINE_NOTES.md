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
