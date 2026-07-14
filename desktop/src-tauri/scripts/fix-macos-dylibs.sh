#!/bin/bash
# Runs via tauri.macos.conf.json's build.beforeBundleCommand — the only
# point in tauri-action's flow that's after `cargo build` produces the
# binary but before Tauri copies/codesigns it into the .app. Modifying the
# binary after that point would invalidate Tauri's own signature.
#
# libpq.5.dylib/libssl.3.dylib/libcrypto.3.dylib get copied into
# external-libs/ by the "Install libpq (macOS)" workflow step and bundled
# into Contents/Frameworks/ via bundle.macOS.frameworks. As copied, though,
# they (and our own binary) still reference each other via this CI runner's
# absolute Homebrew paths (e.g. /opt/homebrew/Cellar/libpq/18.4/lib/...) —
# rewrite every such reference to @rpath/<name> so the app resolves them
# from its own bundled Frameworks/ dir (via the -rpath the RUSTFLAGS in
# that workflow step already baked into the binary) on any machine,
# Homebrew or not.
set -euo pipefail

# Invoked via beforeBundleCommand with an explicit absolute path (see
# tauri.macos.conf.json) specifically to sidestep guessing this hook's
# actual default working directory, which doesn't match the beforeDevCommand/
# beforeBuildCommand convention (relative to the projectPath's parent).
BIN="$GITHUB_WORKSPACE/target/release/desktop"
LIBS_DIR="$GITHUB_WORKSPACE/desktop/src-tauri/external-libs"

LIBPQ_PREFIX="$(brew --prefix libpq)"
OPENSSL_PREFIX="$(brew --prefix openssl@3)"

fix_refs() {
  local file="$1"
  for dep in "$LIBPQ_PREFIX"/lib/*.dylib "$OPENSSL_PREFIX"/lib/*.dylib; do
    [ -e "$dep" ] || continue
    local name
    name="$(basename "$dep")"
    install_name_tool -change "$dep" "@rpath/$name" "$file" 2>/dev/null || true
  done
}

for dylib in "$LIBS_DIR"/*.dylib; do
  install_name_tool -id "@rpath/$(basename "$dylib")" "$dylib"
  fix_refs "$dylib"
done

fix_refs "$BIN"

echo "=== Fixed dylib references ==="
otool -L "$BIN"
for dylib in "$LIBS_DIR"/*.dylib; do
  otool -L "$dylib"
done
