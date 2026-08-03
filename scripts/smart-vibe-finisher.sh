#!/usr/bin/env bash
# scripts/smart-vibe-finisher.sh
# Bash vibe-finisher runner for agy CLI using Python JSON stream parser for real-time live output

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
python "$SCRIPT_DIR/vibe_finisher.py"
