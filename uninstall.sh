#!/bin/bash
set -e

PREFIX="${HOME}/.local"

echo "Removing Bookmarks Browser from ${PREFIX}..."

rm -f "${PREFIX}/bin/bookmarks-browser"
rm -f "${PREFIX}/share/applications/bookmark-browser.desktop"
rm -f "${PREFIX}/share/icons/hicolor/scalable/apps/bookmark-browser.svg"

# Update desktop database
update-desktop-database "${PREFIX}/share/applications" 2>/dev/null || true

echo "Uninstalled. Config files in ~/.config/bookmarks-browser/ were not removed."
