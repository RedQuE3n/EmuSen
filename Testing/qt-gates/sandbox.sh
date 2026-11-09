#!/bin/bash
# A sandbox for the gates: a copy of the published desktop tree's program folder with emusen-qt (and emusen-qt-play)
# placed beside the platform library, and a home of its own, so no run reads or writes the player's settings.
# Usage: sandbox.sh <root> <qt-bin-dir>
#   <root>        the sandbox (made fresh): <root>/lib/EmuSen is the program folder, <root>/home the tree's home
#   <qt-bin-dir>  the folder holding the release builds of emusen-qt and emusen-qt-play
#   TREE          the published tree (out/linux-x64/Mistress of the checkout); ROMS the library (the player's, read-only)
# Settings written: the big screen in the reference look, Moonrise off, scraping off, the ROM folder the library, saves
# and states inside the sandbox. After the first start has read the library, snapshot.sh keeps the home as it stands.
# See README.md.
set -eu
root=$1; bins=$2
here=$(cd "$(dirname "$0")" && pwd)
tree=${TREE:-$here/../../out/linux-x64/Mistress}
roms=${ROMS:-/home/red/Documents/Roms}
[ -d "$tree/lib/EmuSen" ] || { echo "no published tree at $tree" >&2; exit 1; }
[ -e "$root" ] && { echo "$root exists" >&2; exit 1; }
mkdir -p "$root/lib" "$root/home/etc/EmuSen" "$root/hh/config" "$root/hh/data" "$root/hh/cache"
cp -a --reflink=auto "$tree/lib/EmuSen" "$root/lib/EmuSen"
cp -a "$tree/.dianashroot" "$root/.dianashroot"
cp -a --reflink=auto "$bins/emusen-qt" "$root/lib/EmuSen/"
[ -e "$bins/emusen-qt-play" ] && cp -a --reflink=auto "$bins/emusen-qt-play" "$root/lib/EmuSen/"
[ -e "$bins/qt.conf" ] && cp -a "$bins/qt.conf" "$root/lib/EmuSen/"  # the handheld's bundled Qt (../handheld/README.md)
cat > "$root/home/etc/EmuSen/appsettings.json" <<EOF
{
  "RomDirectory": "$roms/",
  "BigScreen": true,
  "StartupSplash": false,
  "ReducedMotion": "system",
  "Scraping": false,
  "ScrapeCovers": false,
  "OpenEmuFallback": false,
  "OnlineCovers": false,
  "LibraryStyle": "Theme",
  "Mute": true
}
EOF
echo "sandbox $root ready: program folder $root/lib/EmuSen"
