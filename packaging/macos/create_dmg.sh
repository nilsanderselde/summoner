#!/usr/bin/env bash
# Summoner DAW - macOS DMG Packaging Script
# Step 531: Create macOS .dmg packaging

set -euo pipefail

APP_NAME="Summoner"
VERSION="1.0.0"
STAGE_DIR="target/macOS_stage"
DMG_NAME="SummonerDAW-v${VERSION}-macOS.dmg"

echo "Building macOS release binary..."
if command -v lipo >/dev/null 2>&1; then
    cargo build --release --target aarch64-apple-darwin || true
    cargo build --release --target x86_64-apple-darwin || true
fi

echo "Creating bundle structure..."
mkdir -p "${STAGE_DIR}/${APP_NAME}.app/Contents/MacOS"
mkdir -p "${STAGE_DIR}/${APP_NAME}.app/Contents/Resources"

# Assemble universal binary or fallback to host build
AARCH64_BIN="target/aarch64-apple-darwin/release/summon"
X86_64_BIN="target/x86_64-apple-darwin/release/summon"
OUT_BIN="${STAGE_DIR}/${APP_NAME}.app/Contents/MacOS/summon"

if [[ -f "${AARCH64_BIN}" && -f "${X86_64_BIN}" ]] && command -v lipo >/dev/null 2>&1; then
    echo "Creating universal macOS binary with lipo..."
    lipo -create -output "${OUT_BIN}" "${AARCH64_BIN}" "${X86_64_BIN}"
elif [[ -f "${AARCH64_BIN}" ]]; then
    cp "${AARCH64_BIN}" "${OUT_BIN}"
elif [[ -f "${X86_64_BIN}" ]]; then
    cp "${X86_64_BIN}" "${OUT_BIN}"
else
    echo "Building release binary for host target..."
    cargo build --release
    cp target/release/summon "${OUT_BIN}"
fi
chmod +x "${OUT_BIN}"

# Generate valid Info.plist for macOS application bundle
cat <<EOF > "${STAGE_DIR}/${APP_NAME}.app/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>summon</string>
    <key>CFBundleIdentifier</key>
    <string>com.summonerdaw.summon</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSMicrophoneUsageDescription</key>
    <string>Summoner DAW requires access to the microphone and audio inputs for multi-track audio recording and live monitoring.</string>
</dict>
</plist>
EOF

if command -v codesign >/dev/null 2>&1; then
    echo "Signing application bundle with hardened runtime and entitlements..."
    codesign --force --options runtime --entitlements packaging/macos/entitlements.plist --deep -s - "${STAGE_DIR}/${APP_NAME}.app" || true
fi

rm -f "${STAGE_DIR}/Applications"
ln -s /Applications "${STAGE_DIR}/Applications"

echo "Creating DMG..."
if command -v hdiutil >/dev/null 2>&1; then
    hdiutil create -volname "${APP_NAME}" -srcfolder "${STAGE_DIR}" -ov -format UDZO "${DMG_NAME}"
    echo "DMG build complete: ${DMG_NAME}"
else
    echo "hdiutil not available on this platform; staged bundle ready at ${STAGE_DIR}"
fi
