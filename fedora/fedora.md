# fedora/ — Fedora build (.rpm)

**Status:** Fedora 44 KDE was the reference platform for the v0.1.0 release candidates. RPMs up to rc4 are tagged. The `v0.1.0-rc5` named in the old checkpoint was never tagged. The Fedora docs below describe that RPM. The manual QA checklist (`docs/QA_VETTING_CHECKLIST.md`) has not been executed.

## Build the .rpm (on Fedora)

```bash
sudo dnf install -y webkit2gtk4.1-devel openssl-devel librsvg2-devel libappindicator-gtk3-devel libxdo-devel gcc gcc-c++ qpdf rpm-build
fedora/packaging/build-rpm.sh                 # output in master/target/release/bundle/rpm/
SB_RELEASE=1 fedora/packaging/build-rpm.sh    # same, CPU/RAM-limited (low-memory hosts)
```

The script builds `localpdf` (CLI) and `lpdf-worker`, then runs `tauri build --bundles rpm --config fedora/packaging/tauri.fedora.conf.json`. The overlay keeps the RPM settings exactly as they were before the restructure (`libreoffice-core` and `pandoc` dependencies, and both extra binaries in `/usr/bin`).

## Map of `fedora/`

Every entry has a one-line description. `master/vendor/pdfcraft` is an upstream submodule and is listed as one entry. Build output (`target/`, `node_modules/`, `build/`) and downloaded corpus PDFs are not tracked and not listed.

```text
fedora/
├── docs/                        # Fedora install/QA/release documents
│   ├── FEDORA_INSTALL.md        # Install & user guide for Fedora (dnf, KDE)
│   ├── QA_VETTING_CHECKLIST.md  # RC1 manual QA checklist (not yet executed)
│   ├── RC1_INSTALL_REPORT.md    # RC1 RPM install/verification report
│   └── RELEASE_NOTES_v0.1.0.md  # v0.1.0 release notes (Fedora RPM release)
├── packaging/                   # Files used to build the .rpm
│   ├── build-rpm.sh             # Builds CLI + worker + app and bundles the .rpm with the Fedora overlay
│   ├── desktop.template         # Linux .desktop launcher template (Fedora copy)
│   └── tauri.fedora.conf.json   # Tauri config overlay: rpm target, Fedora dependency names, extra binaries
└── fedora.md                    # Map of fedora/ with build instructions and status
```
