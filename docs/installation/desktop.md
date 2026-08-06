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

## Uninstalling

Uninstalling the app does not remove its local data directory on any
platform — delete it manually if you want a clean removal.
