#!/usr/bin/env bash
set -euo pipefail

echo "=== CHIRP installation check ==="

# Find Python
PYTHON=""
for candidate in python3 python; do
  if command -v "$candidate" &>/dev/null; then
    PYTHON="$candidate"
    break
  fi
done

if [ -z "$PYTHON" ]; then
  echo "ERROR: Python not found. Install Python 3.8+ from https://python.org"
  exit 1
fi
echo "Python: $($PYTHON --version)"

# Check CHIRP
if $PYTHON -m chirp --version &>/dev/null 2>&1; then
  echo "CHIRP: $($PYTHON -m chirp --version 2>&1 | head -1)"
elif command -v chirpw &>/dev/null; then
  echo "CHIRP: $(chirpw --version 2>&1 | head -1) (standalone)"
else
  echo ""
  echo "CHIRP not found. Install CHIRP-next:"
  echo "  pip install chirp"
  echo "  # or download from https://chirp.danplanet.com"
  echo ""
  echo "UV-5R serial programming (Phase 2) requires CHIRP."
  echo "Phase 1 (config management) works without it."
fi
