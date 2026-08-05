#!/bin/bash
# Runs via tauri.macos.conf.json's build.beforeBundleCommand — the only
# point in tauri-action's flow that's after `cargo build` produces the
# binary but before Tauri copies/codesigns it into the .app. Modifying the
# binary after that point would invalidate Tauri's own signature.
#
# The "Install libpq (macOS)" workflow step walks libpq.5.dylib's full
# dependency closure (libssl, libcrypto, krb5's libgssapi_krb5 and whatever
# *that* pulls in, etc.) and copies all of it into external-libs/, which
# gets bundled into Contents/Frameworks/ via bundle.macOS.frameworks. As
# copied, though, they (and our own binary) still reference each other via
# this CI runner's absolute Homebrew paths (e.g.
# /opt/homebrew/Cellar/libpq/18.4/lib/...) — rewrite every such reference to
# @rpath/<name> so the app resolves them from its own bundled Frameworks/
# dir (Tauri adds the @executable_path/../Frameworks rpath automatically
# once bundle.macOS.frameworks is non-empty) on any machine, Homebrew or
# not. Driven off whatever's actually in external-libs/ rather than a fixed
# set of Homebrew prefixes, since the exact dependency closure isn't fixed.
set -euo pipefail

# Invoked via beforeBundleCommand with an explicit absolute path (see
# tauri.macos.conf.json) specifically to sidestep guessing this hook's
# actual default working directory, which doesn't match the beforeDevCommand/
# beforeBuildCommand convention (relative to the projectPath's parent).
BIN="$GITHUB_WORKSPACE/target/release/desktop"
LIBS_DIR="$GITHUB_WORKSPACE/desktop/src-tauri/external-libs"

fix_refs() {
  local file="$1"
  while IFS= read -r dep; do
    local name
    name="$(basename "$dep")"
    if [ -f "$LIBS_DIR/$name" ]; then
      install_name_tool -change "$dep" "@rpath/$name" "$file"
    fi
  done < <(otool -L "$file" | tail -n +2 | awk '{print $1}')
}

for dylib in "$LIBS_DIR"/*.dylib; do
  install_name_tool -id "@rpath/$(basename "$dylib")" "$dylib"
  fix_refs "$dylib"
  # install_name_tool edits invalidate whatever signature Homebrew's build
  # shipped (a real one, in this case) without stripping it — on Apple
  # Silicon an *invalid* signature gets a hard SIGKILL from the kernel the
  # moment dyld maps the file, unlike a missing one. Ad-hoc re-sign so it's
  # valid again; Tauri's own later codesign pass re-signs the whole .app
  # over this anyway, so this only has to hold until then.
  codesign --force -s - "$dylib"
done

fix_refs "$BIN"
codesign --force -s - "$BIN"

echo "=== Fixed dylib references ==="
otool -L "$BIN"
for dylib in "$LIBS_DIR"/*.dylib; do
  otool -L "$dylib"
done
