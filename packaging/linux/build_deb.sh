#!/usr/bin/env bash
# Summoner DAW - Linux Debian (.deb) Packaging Script (Epic J, Task J3)
# Builds a standard .deb package for Ubuntu/Debian x86_64 distributions.

set -euo pipefail

VERSION="${VERSION:-1.0.0}"
PKG_NAME="summoner-daw"
DEB_DIR="target/${PKG_NAME}_${VERSION}_amd64"
OUTPUT_DEB="target/${PKG_NAME}_${VERSION}_amd64.deb"

echo "=== Building Debian Package: ${OUTPUT_DEB} ==="

mkdir -p "${DEB_DIR}/DEBIAN"
mkdir -p "${DEB_DIR}/usr/bin"
mkdir -p "${DEB_DIR}/usr/share/applications"
mkdir -p "${DEB_DIR}/usr/share/icons/hicolor/256x256/apps"
mkdir -p "${DEB_DIR}/usr/share/mime/packages"
mkdir -p "${DEB_DIR}/usr/share/doc/${PKG_NAME}"

# Ensure release binary
if [[ ! -f "target/release/summon" ]]; then
    echo "Building release binary..."
    cargo build --release --features gui
fi

cp target/release/summon "${DEB_DIR}/usr/bin/summon"
chmod +x "${DEB_DIR}/usr/bin/summon"

# Control file
cat <<EOF > "${DEB_DIR}/DEBIAN/control"
Package: ${PKG_NAME}
Version: ${VERSION}
Section: sound
Priority: optional
Architecture: amd64
Depends: libasound2 (>= 1.1.0), libudev1, libx11-6, libxcb1, libxcursor1, libxrandr2, libxi6
Recommends: pipewire | jackd2 | jackd
Maintainer: Summoner DAW Maintainers <maintainers@summoner-daw.org>
Description: Deterministic, Headless-First Digital Audio Workstation
 High-performance cross-platform DAW featuring microtonal N-EDO tuning,
 real-time DSP, git-based session versioning, and live performance Stage View.
 Homepage: https://summoner.audio
EOF

# Desktop file
cat <<EOF > "${DEB_DIR}/usr/share/applications/summoner.desktop"
[Desktop Entry]
Name=Summoner DAW
Comment=Deterministic, Headless-First Digital Audio Workstation
Exec=/usr/bin/summon %F
Icon=summoner
Type=Application
Categories=AudioVideo;Audio;AudioVideoEditing;Midi;Sequencer;
MimeType=application/x-summoner-project;audio/x-wav;audio/flac;audio/ogg;audio/aiff;
Terminal=false
StartupWMClass=summoner
Keywords=daw;audio;music;synth;sequencer;microtonal;
EOF

# MIME file
cat <<EOF > "${DEB_DIR}/usr/share/mime/packages/summoner.xml"
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="application/x-summoner-project">
    <comment>Summoner DAW Project File</comment>
    <glob pattern="*.summoner"/>
    <glob pattern="*.summoner_proj"/>
    <icon name="summoner"/>
  </mime-type>
</mime-info>
EOF

# Doc files
if [[ -f "README.md" ]]; then
    cp README.md "${DEB_DIR}/usr/share/doc/${PKG_NAME}/README.md"
fi
if [[ -f "LICENSE" ]]; then
    cp LICENSE "${DEB_DIR}/usr/share/doc/${PKG_NAME}/copyright"
fi

# Build .deb with dpkg-deb if available
if command -v dpkg-deb >/dev/null 2>&1; then
    echo "Running dpkg-deb to package ${OUTPUT_DEB}..."
    dpkg-deb --build --root-owner-group "${DEB_DIR}" "${OUTPUT_DEB}"
    echo "Debian package created: ${OUTPUT_DEB}"
else
    echo "Notice: dpkg-deb is not available. Debian staging structure is ready at ${DEB_DIR}."
fi

