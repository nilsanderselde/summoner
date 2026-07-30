#!/usr/bin/env bash
# Summoner DAW - Debian/Ubuntu PPA & APT Repository Publisher (Step 1089)
set -euo pipefail

VERSION="1.1.0"
PPA_REPO="ppa:summoner/daw"

echo "=== Building Debian Package summoner-daw_${VERSION}_amd64.deb ==="
mkdir -p build/debian/DEBIAN
mkdir -p build/debian/usr/local/bin

cp target/release/summon build/debian/usr/local/bin/
cp target/release/summoner_gui build/debian/usr/local/bin/

cat <<EOF > build/debian/DEBIAN/control
Package: summoner-daw
Version: ${VERSION}
Section: sound
Priority: optional
Architecture: amd64
Maintainer: Summoner Team <dev@summoner.audio>
Description: Deterministic microtonal DAW and headless synthesizer engine.
EOF

dpkg-deb --build build/debian "packaging/linux/summoner-daw_${VERSION}_amd64.deb"
echo "=== Successfully generated Debian PPA package packaging/linux/summoner-daw_${VERSION}_amd64.deb ==="
