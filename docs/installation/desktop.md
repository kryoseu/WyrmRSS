# Desktop app

A native desktop build (macOS, Windows, Linux) that runs the same backend and
frontend as the self-hosted version, embedded in a single app with no
separate database to set up — it uses SQLite instead of PostgreSQL.

## Install

Download the build for your platform from the [Releases](https://github.com/kryoseu/WyrmRSS/releases) page:

- **macOS**: `.dmg` (separate builds for Apple Silicon and Intel)
- **Windows**: `.exe` (NSIS installer)
- **Linux**: `.deb`, `.rpm`, or `.AppImage`

Data (feeds, posts, folders) is stored locally in the app's data directory
and isn't shared with any self-hosted instance.

## Unsigned builds

These builds aren't code-signed yet, so your OS will warn before opening them:

- **macOS**: Gatekeeper reports the app as "damaged." It isn't — this is
  the standard warning for an unsigned app downloaded from the internet.
  Clear the quarantine attribute to open it:

  ```bash
  xattr -cr /Applications/WyrmRSS.app
  ```

- **Windows**: SmartScreen will warn about an unrecognized publisher. Click
  "More info" → "Run anyway" to proceed.
  
> [!NOTE]
> The `.AppImage` fails to start on wlroots-based Wayland compositors (Hyprland,
> Sway, etc.) with `Could not create default EGL display: EGL_BAD_PARAMETER` —
> the bundle ships its own `libwayland-egl`/`libEGL` that conflict with these
> compositors. On Arch, or anywhere else none of the three packages fit, build
> from source instead (below).

> [!NOTE]
> YouTube videos failing with "Your browser can't play this video" on Linux
> usually means GStreamer has no codec plugins installed — WebKitGTK's media
> playback runs entirely through GStreamer, and Arch in particular doesn't
> pull these in as hard dependencies. Install them (adjust for your distro's
> package manager):
>
> ```bash
> sudo pacman -S gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly gst-libav
> ```

## Build from source

Prerequisites (no PostgreSQL needed — the desktop build uses SQLite):

- Rust (stable toolchain)
- Node.js and pnpm
- The Tauri CLI: `cargo install tauri-cli --locked`
- Linux only: the webview/packaging libraries `tauri build` links against —
  see [Tauri's prerequisites guide](https://v2.tauri.app/start/prerequisites/)
  for your distro's package names.

Then, from the repository root:

```bash
cd desktop/src-tauri
cargo tauri build --no-bundle
```

The resulting binary is under `target/release/` (or
`target/<triple>/release/` if cross-compiling). `--no-bundle` skips producing
a `.dmg`/`.msi`/`.deb`/etc and just builds the binary directly — the right
choice here, since it sidesteps both the wlroots AppImage issue and the
missing native package format on Arch.

## Uninstalling

Uninstalling the app does not remove its local data directory on any
platform — delete it manually if you want a clean removal.
