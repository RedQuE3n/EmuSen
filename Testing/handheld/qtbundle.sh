#!/bin/bash
# On the desktop: the desktop's Qt for programs built against it, laid out for another machine, since SteamOS's Qt
# exports no private API (`Qt_6.11_PRIVATE_API`, which cxx-qt's generated code needs). <dest>/lib holds every library
# the binaries, the plugins and the QML modules need that is Qt's or ICU's, or that the other machine lacks by soname;
# <dest>/plugins the Wayland and offscreen platforms with Wayland's integrations and the common image formats;
# <dest>/qml the modules the front end imports. A program reads them through a qt.conf beside it naming <dest> and
# LD_LIBRARY_PATH naming <dest>/lib. See README.md.
# Usage: qtbundle.sh <dest> <sonames-file> <binary>...    <sonames-file>: the other machine's library file names, one a line
#   (`ldconfig -p`'s and every lib*.so* under /usr/lib: a library it holds off ldconfig's list is still never carried; README.md)
set -eu
dest=$1; sonames=$2; shift 2
qt=/usr/lib64/qt6
mkdir -p "$dest/lib" "$dest/plugins" "$dest/qml"
for dir in platforms wayland-shell-integration wayland-graphics-integration-client wayland-decoration-client; do
  cp -a "$qt/plugins/$dir" "$dest/plugins/"
done
rm -f "$dest"/plugins/platforms/lib{qeglfs,qlinuxfb,qminimalegl,qvkkhrdisplay,qvnc,qxcb}.so
mkdir -p "$dest/plugins/imageformats"
for f in gif ico jpeg svg webp; do cp -a "$qt/plugins/imageformats/libq$f.so" "$dest/plugins/imageformats/"; done
for m in QtQuick QtQml QtMultimedia QtCore builtins.qmltypes jsroot.qmltypes; do cp -a "$qt/qml/$m" "$dest/qml/"; done
objects=("$@")
mapfile -t -O ${#objects[@]} objects < <(find "$dest/plugins" "$dest/qml" -name '*.so')
ldd "${objects[@]}" 2>/dev/null | awk '$2 == "=>" && $3 ~ /^\// { print $1, $3 }' | sort -u | while read -r name path; do
  if [[ $name == libQt6* || $name == libicu* ]] || ! grep -qxF "$name" "$sonames"; then
    cp -L "$path" "$dest/lib/$name"
  fi
done
echo "bundled $(ls "$dest/lib" | wc -l) libraries, $(find "$dest/plugins" -name '*.so' | wc -l) plugins, $(du -sh "$dest" | cut -f1)"
