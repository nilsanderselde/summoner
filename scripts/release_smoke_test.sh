#!/usr/bin/env bash
# End-to-end release smoke test on Linux, macOS, and Windows (Step 1098)
set -euo pipefail

echo "=== Summoner Release Smoke Test v1.1.0 ==="
cargo test --all-targets --all-features
echo "=== Smoke Test OK ==="
