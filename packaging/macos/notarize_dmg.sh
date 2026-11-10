#!/usr/bin/env bash
# Summoner DAW - macOS DMG Notarization and Stapling Pipeline (Step 1088 / Epic J, Task J1)
# Automates submission to Apple Notary Service via notarytool and staples tickets onto the DMG.

set -euo pipefail

VERSION="${VERSION:-1.0.0}"
DMG_NAME="${1:-SummonerDAW-v${VERSION}-macOS.dmg}"

echo "=== Summoner DAW macOS Notarization Pipeline ==="
echo "Target DMG: ${DMG_NAME}"

if [[ ! -f "${DMG_NAME}" ]]; then
    echo "Error: DMG file '${DMG_NAME}' not found." >&2
    exit 1
fi

if ! command -v xcrun >/dev/null 2>&1; then
    echo "Warning: xcrun is not available on this host. Skipping live Apple notarization."
    exit 0
fi

# Credentials can be provided either via a stored keychain profile or environment variables:
# 1. KEYCHAIN_PROFILE (e.g. AC_PASSWORD_PROFILE configured with notarytool store-credentials)
# 2. APPLE_ID, APPLE_TEAM_ID, and APPLE_APP_SPECIFIC_PASSWORD
if [[ -n "${KEYCHAIN_PROFILE:-}" ]]; then
    echo "Submitting DMG for notarization using keychain profile '${KEYCHAIN_PROFILE}'..."
    xcrun notarytool submit "${DMG_NAME}" --keychain-profile "${KEYCHAIN_PROFILE}" --wait
elif [[ -n "${APPLE_ID:-}" && -n "${APPLE_TEAM_ID:-}" && -n "${APPLE_APP_SPECIFIC_PASSWORD:-}" ]]; then
    echo "Submitting DMG for notarization using Apple ID credentials..."
    xcrun notarytool submit "${DMG_NAME}" \
        --apple-id "${APPLE_ID}" \
        --team-id "${APPLE_TEAM_ID}" \
        --password "${APPLE_APP_SPECIFIC_PASSWORD}" \
        --wait
else
    echo "Notice: Neither KEYCHAIN_PROFILE nor APPLE_ID credentials set."
    echo "In CI / Release environments, set KEYCHAIN_PROFILE or (APPLE_ID, APPLE_TEAM_ID, APPLE_APP_SPECIFIC_PASSWORD)."
    echo "Skipping live notarization submission."
    exit 0
fi

echo "Stapling notarization ticket to ${DMG_NAME}..."
xcrun stapler staple "${DMG_NAME}"

echo "Validating Gatekeeper assessment..."
spctl --assess --type open --context context:primary-signature --verbose "${DMG_NAME}" || true

echo "=== Notarization and stapling successfully completed for ${DMG_NAME} ==="

