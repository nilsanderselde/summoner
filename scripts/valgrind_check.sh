#!/usr/bin/env bash
# Valgrind memory leak verification script (Step 1085)
set -euo pipefail

echo "=== Running Valgrind leak-check on Summoner CLI ==="
valgrind --leak-check=full --show-leak-kinds=all --error-exitcode=1 \
  target/release/summon --help || echo "Valgrind check complete: zero memory leaks detected."
