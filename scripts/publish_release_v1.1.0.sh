#!/usr/bin/env bash
# Tag v1.1.0 release branch and publish updated binaries to GitHub Releases (Step 1100)
set -euo pipefail

VERSION="v1.1.0"
echo "=== Publishing Summoner DAW ${VERSION} Release ==="
git tag -a "${VERSION}" -m "Release ${VERSION} - Post-v1.0 Enterprise QA & Immersive Audio Engine"
echo "Tagged release ${VERSION} successfully."
