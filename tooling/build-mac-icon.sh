#!/bin/sh
# Compiles the Icon Composer icon into Assets.car (needs Xcode). macOS 26 and
# later show it through CFBundleIconName; without it the Dock puts icon.icns
# on a light tile. Run after editing icons/AppIcon.icon.
set -eu
cd "$(dirname "$0")/../apps/desktop/src-tauri/icons"
out=$(mktemp -d)
xcrun actool AppIcon.icon --compile "$out" --output-partial-info-plist "$out/partial.plist" \
  --app-icon AppIcon --include-all-app-icons --enable-on-demand-resources NO \
  --development-region en --target-device mac --minimum-deployment-target 11.0 \
  --platform macosx >/dev/null
cp "$out/Assets.car" Assets.car
rm -r "$out"
