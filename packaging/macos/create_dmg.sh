#!/usr/bin/env bash
# Summoner DAW - macOS DMG Packaging Script
# Step 531: Create macOS .dmg packaging

set -euo pipefail

APP_NAME="Summoner"
VERSION="1.0.0"
STAGE_DIR="target/macOS_stage"
DMG_NAME="SummonerDAW-v${VERSION}-macOS.dmg"

echo "Building macOS release binary..."
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

echo "Creating bundle structure..."
mkdir -p "${STAGE_DIR}/${APP_NAME}.app/Contents/MacOS"
mkdir -p "${STAGE_DIR}/${APP_NAME}.app/Contents/Resources"

cp target/release/summon "${STAGE_DIR}/${APP_NAME}.app/Contents/MacOS/summon"
ln -s /Applications "${STAGE_DIR}/Applications"

echo "Creating DMG..."
hdiutil create -volname "${APP_NAME}" -srcfolder "${STAGE_DIR}" -ov -format UDZO "${DMG_NAME}"
echo "DMG build complete: ${DMG_NAME}"
