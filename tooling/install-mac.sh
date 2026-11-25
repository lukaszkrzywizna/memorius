#!/bin/sh
# Builds Memorius.app and installs it to ~/Applications, signed with a stable
# certificate so macOS keeps its permissions (Accessibility, browser control)
# across updates. With the release certificate in ~/.memorius-release the
# build is signed like GitHub releases, so both share one set of permissions;
# otherwise with a local "Memorius Local Signing" certificate.
set -eu
cd "$(dirname "$0")/.."

. tooling/mac-signing.sh
# The memorius CLI ships inside the app, next to its executable.
node tooling/build-cli-sidecar.mjs
APPLE_SIGNING_IDENTITY="$identity" pnpm -C apps/desktop tauri build --bundles app \
  --config src-tauri/tauri.bundle.conf.json
pkill -f 'Applications/Memorius.app' || true
sleep 1
rm -rf "$HOME/Applications/Memorius.app"
ditto target/release/bundle/macos/Memorius.app "$HOME/Applications/Memorius.app"
codesign --verify --deep --strict "$HOME/Applications/Memorius.app"
# Register the copy again so the Dock and Finder pick up a new app icon.
touch "$HOME/Applications/Memorius.app"
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$HOME/Applications/Memorius.app"
# Keep one Memorius on this Mac: the build copy would register as a second app.
rm -rf target/release/bundle/macos/Memorius.app
open "$HOME/Applications/Memorius.app"
