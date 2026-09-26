#!/bin/bash
# Installs Mistress's icon and a desktop entry for this build, for the current user only.
# Usage: bin/install-desktop-entry.sh [--desktop] [--remove]
#   --desktop  also puts a shortcut on the desktop (~/Desktop, or the XDG desktop folder)
#   --remove   takes the entry, the shortcut and the icons away again
# The entry points at this copy of Mistress, so run it again after moving the folder.
# See EmuSen_Settings_Reference.md §4.55.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(dirname "$here")"
data="${XDG_DATA_HOME:-$HOME/.local/share}"
apps="$data/applications"
icons="$data/icons/hicolor"
name="emusen-mistress"
desktop_dir="$(xdg-user-dir DESKTOP 2>/dev/null || echo "$HOME/Desktop")"

want_desktop=0
remove=0
for arg in "$@"; do
  case "$arg" in
    --desktop) want_desktop=1 ;;
    --remove) remove=1 ;;
    -h|--help) sed -n '2,7p' "$0"; exit 0 ;;
    *) echo "unknown option: $arg" >&2; exit 2 ;;
  esac
done

refresh() {
  command -v update-desktop-database >/dev/null && update-desktop-database -q "$apps" 2>/dev/null || true
  command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q -t "$icons" 2>/dev/null || true
}

if [ "$remove" = 1 ]; then
  rm -f "$apps/$name.desktop" "$desktop_dir/$name.desktop"
  find "$icons" -path "*/apps/$name.*" -delete 2>/dev/null || true
  refresh
  echo "removed the EmuSen Mistress entry and icons"
  exit 0
fi

if [ ! -x "$root/bin/EmuSen.Mistress" ]; then
  echo "cannot find $root/bin/EmuSen.Mistress; run this from a published Mistress build" >&2
  exit 1
fi

for dir in "$root"/share/icons/hicolor/*/apps; do
  size="$(basename "$(dirname "$dir")")"
  mkdir -p "$icons/$size/apps"
  cp -f "$dir"/$name.* "$icons/$size/apps/"
done

mkdir -p "$apps"
cat > "$apps/$name.desktop" <<ENTRY
[Desktop Entry]
Type=Application
Name=EmuSen Mistress
GenericName=Emulator
Comment=Play your SNES, NES, Game Boy and Nintendo 64 library
Exec="$root/bin/EmuSen.Mistress" %f
TryExec=$root/bin/EmuSen.Mistress
Icon=$name
Terminal=false
Categories=Game;Emulator;
Keywords=emulator;snes;nes;gameboy;n64;
StartupWMClass=EmuSen.Mistress
ENTRY
chmod 644 "$apps/$name.desktop"

if [ "$want_desktop" = 1 ]; then
  mkdir -p "$desktop_dir"
  cp -f "$apps/$name.desktop" "$desktop_dir/$name.desktop"
  chmod 755 "$desktop_dir/$name.desktop"
  command -v gio >/dev/null && gio set "$desktop_dir/$name.desktop" metadata::trusted true 2>/dev/null || true
fi

refresh
echo "installed EmuSen Mistress for $USER: $apps/$name.desktop$([ "$want_desktop" = 1 ] && echo " and a desktop shortcut")"
