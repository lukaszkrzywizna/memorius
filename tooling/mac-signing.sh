# Sourced by install-mac.sh and release-mac.sh: puts the release certificate
# from ~/.memorius-release in a separate keychain for codesign and sets
# $identity, or falls back to the local "Memorius Local Signing" certificate.
# The keychain search list is restored on exit.
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
