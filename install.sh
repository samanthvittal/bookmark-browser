#!/bin/bash
set -e

PREFIX="${HOME}/.local"

echo "Building release binary..."
cargo build --release

echo "Installing to ${PREFIX}..."

# Install binary
install -Dm755 target/release/bookmarks-browser "${PREFIX}/bin/bookmarks-browser"

# Install desktop file
install -Dm644 assets/bookmark-browser.desktop "${PREFIX}/share/applications/bookmark-browser.desktop"

# Install icon
install -Dm644 assets/bookmark-browser.svg "${PREFIX}/share/icons/hicolor/scalable/apps/bookmark-browser.svg"

# Update desktop database
update-desktop-database "${PREFIX}/share/applications" 2>/dev/null || true

echo "Installed to ${PREFIX}. Ensure ${PREFIX}/bin is in your PATH."
