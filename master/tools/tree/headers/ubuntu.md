# ubuntu/ — Ubuntu / Debian build (.deb)

**Status (2026-11-05):** this is the current development machine (Zorin OS 18.1, based on Ubuntu 24.04).
- **Verified here:** frontend type check (0 errors), Vitest 127/127, frontend production build, the Rust workspace tests except the `app-tauri` crate, and every `cargo xtask` check.
- **Not verified yet:** the desktop binary and the `.deb` package. Both need the system packages below, and installing those needs `sudo`.

## One-time setup

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev libayatana-appindicator3-dev \
  libxdo-dev libssl-dev build-essential patchelf          # build the Tauri app
sudo apt-get install -y qpdf pandoc zenity                 # corpus generation, Markdown tools, file dialogs
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # Rust (user-level)
corepack enable                                            # provides `pnpm` (Tauri calls it during builds)
```

## Build the .deb

```bash
ubuntu/packaging/build-deb.sh        # from the repo root; output in master/target/release/bundle/deb/
```

The script builds `localpdf` (CLI) and `lpdf-worker`, then runs `tauri build --bundles deb --config ubuntu/packaging/tauri.ubuntu.conf.json`.

## Ubuntu-specific notes

- **`/bin/sh` is `dash`, not bash.** Shell snippets must be POSIX. For example, `printf` needs octal escapes (`\003`), not hex (`\x03`). Two mock-sidecar tests failed on Ubuntu for this reason; they're fixed.
- **AppArmor may block the network sandbox.** Stock Ubuntu 24.04 sets `kernel.apparmor_restrict_unprivileged_userns=1`, which blocks the `unshare -n -r` offline isolation used for sidecars and the worker. This machine has it set to `0`, so it works here. A `.deb` for stock Ubuntu will likely need an AppArmor profile that allows `userns`. That profile has not been written or tested yet.
- **File dialogs:** the app tries `kdialog` (KDE) first, then `zenity`. Ubuntu/GNOME needs `zenity`, so the `.deb` depends on it.
- **Package names:** LibreOffice components and `pandoc` are `recommends`, not hard dependencies. The feature map requires Office tools to show greyed out (not hidden) when LibreOffice is missing; that UI state is not yet verified on Ubuntu.
