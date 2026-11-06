#!/usr/bin/env bash
# Generates and installs the Linux .desktop shortcut using the current repo location.
set -euo pipefail
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEMPLATE="$REPO_DIR/desktop/titan-hft.desktop.in"
OUT_APPS="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
mkdir -p "$OUT_APPS"
sed "s|@REPO_DIR@|$REPO_DIR|g" "$TEMPLATE" > "$OUT_APPS/titan-hft.desktop"
chmod +x "$OUT_APPS/titan-hft.desktop"
if [ -d "$HOME/Desktop" ]; then
  cp "$OUT_APPS/titan-hft.desktop" "$HOME/Desktop/titan-hft.desktop"
  chmod +x "$HOME/Desktop/titan-hft.desktop"
fi
echo "✔ Installed titan-hft.desktop pointing to $REPO_DIR"
