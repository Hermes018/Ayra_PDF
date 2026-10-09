# UX Specification: PDF INTELLIGENCE

## 1. AI Summarizer
- **Empty State**: Standard dropzone.
- **Loaded State**: File list, summary output area, option panel.
- **Option Panel**:
  - Length: Short, Medium, Long.
  - Format: Bullet points, Paragraphs.
  - Language of summary.
  - Mode indicator (Basic/extractive vs LLM). Automatically uses LLM if installed, otherwise Basic.
- **Processing State**: "Summarizing..." (with cancel button).
- **Done State**: Summary text displayed in UI. Option to copy to clipboard or export as text/PDF.

## 2. Translate PDF
- **Disabled State**: Tile is greyed out if no LLM model is installed locally. Message: "Requires local AI model."
- **Empty State**: Standard dropzone.
- **Loaded State**: Option panel.
- **Option Panel**:
  - Source Language (Auto-detect or list).
  - Target Language.
  - Warning: "Layout/formatting intact is best effort."
- **Processing State**: "Translating...". Per-page progress.
- **Done State**: Success message showing blocks translated and any fallback fonts used. Download translated file.
