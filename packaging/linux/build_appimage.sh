#!/usr/bin/env bash
# Summoner DAW - Linux AppImage Packaging Script (Epic J, Task J3)
# Builds a self-contained portable AppImage for Linux x86_64.

set -euo pipefail

VERSION="${VERSION:-1.0.0}"
APP_NAME="SummonerDAW"
APPDIR="target/Summoner.AppDir"
OUTPUT_APPIMAGE="target/${APP_NAME}-v${VERSION}-x86_64.AppImage"

echo "=== Building Linux AppImage for ${APP_NAME} v${VERSION} ==="

mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/256x256/apps"
mkdir -p "${APPDIR}/usr/share/mime/packages"

# Ensure binary is built
if [[ ! -f "target/release/summon" ]]; then
    echo "Building release binary..."
    cargo build --release --features gui
fi

cp target/release/summon "${APPDIR}/usr/bin/summon"
chmod +x "${APPDIR}/usr/bin/summon"

# Create AppRun launcher
cat <<'EOF' > "${APPDIR}/AppRun"
#!/bin/sh
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH:-}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
exec "${HERE}/usr/bin/summon" "$@"
EOF
chmod +x "${APPDIR}/AppRun"

# Desktop entry
cat <<EOF > "${APPDIR}/usr/share/applications/summoner.desktop"
[Desktop Entry]
Name=Summoner DAW
Comment=Deterministic, Headless-First Digital Audio Workstation
Exec=summon %F
Icon=summoner
Type=Application
Categories=AudioVideo;Audio;AudioVideoEditing;Midi;Sequencer;
MimeType=application/x-summoner-project;audio/x-wav;audio/flac;audio/ogg;audio/aiff;
Terminal=false
StartupWMClass=summoner
Keywords=daw;audio;music;synth;sequencer;microtonal;
EOF

ln -sf usr/share/applications/summoner.desktop "${APPDIR}/summoner.desktop"

# MIME type definition
cat <<EOF > "${APPDIR}/usr/share/mime/packages/summoner.xml"
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

# Placeholder icon if none exists
ICON_PATH="${APPDIR}/usr/share/icons/hicolor/256x256/apps/summoner.png"
if [[ ! -f "${ICON_PATH}" ]]; then
    # Generate a simple 1x1 png or copy if present
    echo -n "" > "${ICON_PATH}"
fi
ln -sf usr/share/icons/hicolor/256x256/apps/summoner.png "${APPDIR}/summoner.png"

# Packaging with appimagetool if available
if command -v appimagetool >/dev/null 2>&1; then
    echo "Running appimagetool to create ${OUTPUT_APPIMAGE}..."
    ARCH=x86_64 appimagetool "${APPDIR}" "${OUTPUT_APPIMAGE}"
    echo "AppImage created successfully: ${OUTPUT_APPIMAGE}"
else
    echo "Notice: appimagetool is not in PATH. AppDir structure is staged at ${APPDIR}."
    echo "To produce final AppImage: ARCH=x86_64 appimagetool ${APPDIR} ${OUTPUT_APPIMAGE}"
fi
