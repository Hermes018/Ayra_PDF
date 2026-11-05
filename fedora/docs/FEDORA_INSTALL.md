# LocalPDF Studio — Fedora Installation & User Guide

Enterprise deployment, installation, upgrade, uninstallation, and CLI integration guide for Fedora Linux (Fedora 40/41/42/43/44 KDE Plasma Wayland & X11).

---

## 1. System Requirements

| Parameter | Minimum | Recommended |
|---|---|---|
| **OS** | Fedora 40+ (Workstation / KDE Spin / Silverblue) | Fedora 44 (KDE Plasma 6, Wayland) |
| **Architecture** | x86_64 | x86_64 (AVX2 supported) |
| **Memory (RAM)** | 2 GB (Tier 1 low-memory mode) | 8 GB+ (Tier 3 full in-memory mode) |
| **Storage** | 200 MB free space | 1 GB free space (for large spooling) |
| **Display** | 1024x768 | 1920x1080 or HiDPI / 4K with Wayland scaling |

---

## 2. Installing the RPM Package

### Recommended: Via DNF (Resolves Recommended Sidecars)
```bash
sudo dnf install ./localpdf-studio-0.1.0-1.x86_64.rpm
```

### Alternative: Direct RPM Tool
```bash
sudo rpm -ivh ./localpdf-studio-0.1.0-1.x86_64.rpm
```

### Offline / Air-Gapped Environments
LocalPDF Studio contains **zero external network dependencies** in its runtime binary. To install in an air-gapped system:
```bash
# Verify integrity
sha256sum localpdf-studio-0.1.0-1.x86_64.rpm

# Install without requiring network access
sudo dnf install --disablerepo="*" ./localpdf-studio-0.1.0-1.x86_64.rpm
```

---

## 3. Package Upgrade & Uninstallation

### Upgrading to a New Release
```bash
sudo dnf upgrade ./localpdf-studio-0.2.0-1.x86_64.rpm
```

### Uninstallation
To completely remove LocalPDF Studio from your system:
```bash
sudo dnf remove localpdf-studio
```

---

## 4. KDE Plasma & Dolphin File Manager Integration

Upon installation, LocalPDF Studio registers standard FreeDesktop desktop entries and MIME associations:

- **Desktop File:** `/usr/share/applications/localpdf-studio.desktop`
- **Application Categories:** `Office;Utility;`
- **MIME Association:** `application/pdf`
- **Icons:** Installed into `/usr/share/icons/hicolor/*/apps/localpdf-studio.png`

### KDE Dolphin "Open With..." Workflow
1. Open **KDE Dolphin** and navigate to any `.pdf` document.
2. Right-click the file and select **Open with...** -> **LocalPDF Studio**.
3. To set LocalPDF Studio as the default PDF handler:
   - Right-click file -> **Properties** -> **File Type Options**.
   - Move **LocalPDF Studio** to the top of the *Application Preference Order* list.
   - Alternatively via CLI:
     ```bash
     xdg-mime default localpdf-studio.desktop application/pdf
     ```

### Wayland & HiDPI Scaling
LocalPDF Studio runs natively on Wayland via `webkit2gtk-4.1`. Fractional scaling (e.g., 125%, 150%, 175%) configured in KDE System Settings (*Display Configuration*) is honored automatically with sharp vector rendering and subpixel anti-aliasing.

---

## 5. External Sidecar Dependencies

LocalPDF Studio executes all core operations (Organize, Compress, Protect, Redact, OCR, Annotate, Forms) **natively in Rust** via `pdfcraft` without any external binaries.

For specialized legacy office conversions and pandoc markdown processing, external sidecars are leveraged:

| Tool Domain | External Sidecar | Required Fedora Packages | Purpose |
|---|---|---|---|
| **Office Conversion** | LibreOffice Headless | `libreoffice-core`, `libreoffice-writer`, `libreoffice-calc`, `libreoffice-impress` | DOCX/XLSX/PPTX bidirectional translation |
| **Markdown Conversion** | Pandoc | `pandoc` | Markdown to/from PDF, DOCX, HTML batch conversion |

### Installing Sidecars Manually (If Not Automatically Installed)
```bash
sudo dnf install libreoffice-core libreoffice-writer libreoffice-calc libreoffice-impress pandoc
```

> **Security Note:** All sidecar invocations are strictly contained on Linux via kernel namespaces (`unshare -n -r`). Sidecar processes have zero network access and run with unprivileged user namespaces.

---

## 6. LocalPDF Command-Line Interface (`localpdf`)

In addition to the GUI studio, LocalPDF provides a headless high-performance CLI utility `localpdf`.

If you build or install the companion CLI to `/usr/bin/localpdf`:
```bash
sudo cp target/release/localpdf /usr/bin/localpdf
```

### Common CLI Workflows

#### 1. Inspect PDF Metadata
```bash
localpdf info document.pdf
localpdf info document.pdf --json
```

#### 2. Merge Multiple PDFs
```bash
localpdf merge combined.pdf doc1.pdf doc2.pdf doc3.pdf
```

#### 3. Split Document
```bash
# Split every page into separate files in output directory
localpdf split large.pdf ./split_output/

# Split at specific cut pages
localpdf split large.pdf ./split_output/ --at 5,12,20
```

#### 4. Rotate Pages
```bash
# Rotate entire document
localpdf rotate input.pdf rotated.pdf --degrees 90

# Rotate specific pages only
localpdf rotate input.pdf rotated.pdf --degrees 180 --pages 1,3,5
```

#### 5. High-Efficiency Compression
```bash
localpdf compress raw.pdf optimized.pdf --profile high
```

#### 6. Document Repair (Corrupt XREF)
```bash
localpdf repair broken.pdf recovered.pdf
```

#### 7. Password Protection & Decryption
```bash
# Encrypt with restrictions
localpdf protect public.pdf protected.pdf --password "EnterpriseSecure2026" --allow-print

# Decrypt
localpdf unlock protected.pdf decrypted.pdf --password "EnterpriseSecure2026"
```

#### 8. PDF/A Archival Conversion
```bash
localpdf pdfa input.pdf archival.pdf --conformance 2b
```

#### 9. Automated Workflow Pipeline Execution
```bash
# List available pipeline presets
localpdf pipeline presets

# Execute multi-step pipeline (e.g. compress + strip + metadata normalize)
localpdf pipeline run web-optimize source.pdf dest.pdf
```
