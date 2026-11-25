#!/bin/sh
# Builds Memorius.app and installs it to ~/Applications, signed with a stable
# certificate so macOS keeps its permissions (Accessibility, browser control)
# across updates. With the release certificate in ~/.memorius-release the
# build is signed like GitHub releases, so both share one set of permissions;
# otherwise with a local "Memorius Local Signing" certificate.
set -eu
cd "$(dirname "$0")/.."

release_p12="$HOME/.memorius-release/memorius-release.p12"
identity="Memorius Local Signing"
if [ -f "$release_p12" ]; then
  keychain="$HOME/.memorius-release/signing.keychain-db"
  password=$(security find-generic-password -a memorius-release -s "Memorius signing keychain" -w 2>/dev/null || true)
  if [ ! -f "$keychain" ] || [ -z "$password" ]; then
    password=$(openssl rand -hex 16)
    rm -f "$keychain"
    security create-keychain -p "$password" "$keychain"
    security add-generic-password -U -a memorius-release -s "Memorius signing keychain" -w "$password"
    security import "$release_p12" -k "$keychain" -T /usr/bin/codesign \
      -P "$(security find-generic-password -a memorius-release -s "Memorius Release p12 password" -w)"
    security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$password" "$keychain" >/dev/null
  fi
  security unlock-keychain -p "$password" "$keychain"
  search=$(security list-keychains -d user | tr -d '"')
  security list-keychains -d user -s "$keychain" $search
  trap 'security list-keychains -d user -s $search' EXIT
  identity=AE22FD5CED0A778B1BC1D795220D5C6C416A13C1
fi
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
