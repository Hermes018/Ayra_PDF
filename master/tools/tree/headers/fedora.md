# fedora/ — Fedora build (.rpm)

**Status:** Fedora 44 KDE was the reference platform for the v0.1.0 release candidates. RPMs up to rc4 are tagged. The `v0.1.0-rc5` named in the old checkpoint was never tagged. The Fedora docs below describe that RPM. The manual QA checklist (`docs/QA_VETTING_CHECKLIST.md`) has not been executed.

## Build the .rpm (on Fedora)

```bash
sudo dnf install -y webkit2gtk4.1-devel openssl-devel librsvg2-devel libappindicator-gtk3-devel libxdo-devel gcc gcc-c++ qpdf rpm-build
fedora/packaging/build-rpm.sh                 # output in master/target/release/bundle/rpm/
SB_RELEASE=1 fedora/packaging/build-rpm.sh    # same, CPU/RAM-limited (low-memory hosts)
```

The script builds `localpdf` (CLI) and `lpdf-worker`, then runs `tauri build --bundles rpm --config fedora/packaging/tauri.fedora.conf.json`. The overlay keeps the RPM settings exactly as they were before the restructure (`libreoffice-core` and `pandoc` dependencies, and both extra binaries in `/usr/bin`).
