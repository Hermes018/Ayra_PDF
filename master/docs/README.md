# LocalPDF Studio (v0.2.0-rc1)

**Offline, Local-Only PDF Markup & CAD Studio**  
*Built with Tauri v2, Svelte 5, and Native Rust.*

LocalPDF Studio is a high-performance, private, zero-telemetry desktop workstation for viewing, editing, organizing, annotating, measuring, redacting, and converting PDF documents natively on your device.

---

## Key Guarantees

1. **100% Offline & Local**: Zero runtime network connections, zero telemetry, no external font/asset downloads. Works completely air-gapped.
2. **Atomic Safe Outputs**: Edits are written atomically to a new output file with verify-after-write. Your original documents are never altered or overwritten in place.
3. **Enterprise Supply Chain**: CycloneDX 1.5 SBOM generated for all Rust crates and npm transitive dependencies. Permissive MIT/Apache-2.0 licensing with zero linked GPL/AGPL libraries.
4. **CAD Precision & Spatial Snapping**: High-precision vector geometry, object snapping (OSNAP), calibrated dimensional measurement, and hatch patterns.

---

## Installation & Running

Packages are available for 64-bit Linux (`x86_64`):

### Fedora / RHEL / openSUSE (`.rpm`)
```bash
sudo dnf install ./localpdf-studio-0.2.0-1.x86_64.rpm
# Or with rpm:
sudo rpm -Uvh ./localpdf-studio-0.2.0-1.x86_64.rpm
```

### Ubuntu / Debian / Linux Mint (`.deb`)
```bash
sudo dpkg -i ./localpdf-studio_0.2.0_amd64.deb
sudo apt-get install -f # resolve any missing runtime system dependencies (WebKitGTK)
```

### Standalone AppImage
```bash
chmod +x ./localpdf-studio_0.2.0_amd64.AppImage
./localpdf-studio_0.2.0_amd64.AppImage
```

---

## Integrity Verification

Verify the SHA256 checksums of release bundles before launching:
```bash
sha256sum -c SHA256SUMS.txt
```

---

## System Requirements

- **Operating System**: Linux (Kernel 5.15+, glibc 2.34+, WebKitGTK 4.1 / 2.40+)
- **Memory**: Minimum 4 GiB RAM (8 GiB recommended)
- **Disk Space**: ~250 MB for installation
