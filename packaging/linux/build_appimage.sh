#!/usr/bin/env bash
# Summoner DAW - Linux AppImage and .deb Packaging Script
# Step 532: Linux .AppImage & .deb packaging

set -euo pipefail

VERSION="1.0.0"
APPDIR="target/Summoner.AppDir"

mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/256x256/apps"

cp target/release/summon "${APPDIR}/usr/bin/summon"

cat <<EOF > "${APPDIR}/AppRun"
#!/bin/sh
exec "\$APPDIR/usr/bin/summon" "\$@"
EOF
chmod +x "${APPDIR}/AppRun"

cat <<EOF > "${APPDIR}/usr/share/applications/summoner.desktop"
[Desktop Entry]
Name=Summoner DAW
Exec=summon
Icon=summoner
Type=Application
Categories=AudioVideo;Audio;
EOF

echo "AppImage structure initialized at ${APPDIR}"
