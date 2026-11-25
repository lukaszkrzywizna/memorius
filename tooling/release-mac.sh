#!/bin/sh
# Builds the signed universal macOS release on this Mac and uploads it to a
# draft GitHub release with `latest.json` for the in-app updater: no GitHub
# Actions minutes. Bump the version first; publish the draft to ship it.
set -eu
cd "$(dirname "$0")/.."

repo=lukaszkrzywizna/memorius
version=$(node -p 'require("./apps/desktop/src-tauri/tauri.conf.json").version')
tag="v$version"
[ -z "$(git status --porcelain)" ] || { echo "Commit your changes first." >&2; exit 1; }
[ "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)" ] || { echo "Push main first." >&2; exit 1; }
! gh release view "$tag" -R "$repo" >/dev/null 2>&1 || { echo "$tag already exists." >&2; exit 1; }
[ -f "$HOME/.memorius-release/memorius-release.p12" ] || { echo "No release certificate." >&2; exit 1; }

rustup target add aarch64-apple-darwin x86_64-apple-darwin
. tooling/mac-signing.sh
TAURI_SIGNING_PRIVATE_KEY=$(cat "$HOME/.memorius-release/updater.key")
TAURI_SIGNING_PRIVATE_KEY_PASSWORD=$(security find-generic-password -s "Memorius updater key password" -w)
export TAURI_SIGNING_PRIVATE_KEY TAURI_SIGNING_PRIVATE_KEY_PASSWORD

node tooling/build-cli-sidecar.mjs universal-apple-darwin
APPLE_SIGNING_IDENTITY="$identity" pnpm -C apps/desktop tauri build \
  --target universal-apple-darwin --bundles app,dmg \
  --config src-tauri/tauri.bundle.conf.json \
  --config '{"bundle":{"createUpdaterArtifacts":true}}'

bundle=target/universal-apple-darwin/release/bundle
out=$(mktemp -d)
cp "$bundle/dmg/Memorius_${version}_universal.dmg" "$out/"
update="Memorius_${version}_universal.app.tar.gz"
cp "$bundle/macos/Memorius.app.tar.gz" "$out/$update"
cp "$bundle/macos/Memorius.app.tar.gz.sig" "$out/$update.sig"
# One universal app serves both Mac architectures.
node -e '
const [out, version, url, signature] = process.argv.slice(1)
const mac = { signature, url }
require("fs").writeFileSync(out + "/latest.json", JSON.stringify({
  version, pub_date: new Date().toISOString(),
  platforms: { "darwin-aarch64": mac, "darwin-x86_64": mac },
}, null, 2))
' "$out" "$version" "https://github.com/$repo/releases/download/$tag/$update" "$(cat "$out/$update.sig")"

gh release create "$tag" -R "$repo" --target main --draft \
  --title "Memorius $tag" --notes "macOS only. See the README for installation steps." \
  "$out"/*
echo "Draft $tag is ready: https://github.com/$repo/releases"
