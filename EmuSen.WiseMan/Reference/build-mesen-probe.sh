#!/usr/bin/env bash
# Builds MesenProbe against a Mesen2 checkout - see EmuSen_Debugging_Tools_Reference_v5.md §3.39.
#
#   ./build-mesen-probe.sh <path-to-mesen2-checkout> [work-dir]
#
# Mesen's own makefile wants SDL2 and X11 headers. Rather than install them
# system-wide, this fetches the RPMs and unpacks them into <work-dir>, then
# points the compiler at that tree with CPATH/LIBRARY_PATH plus a shim
# sdl2-config. Nothing outside <work-dir> and the Mesen checkout is touched.
set -euo pipefail

MESEN="${1:?usage: build-mesen-probe.sh <mesen2-checkout> [work-dir]}"
WORK="${2:-${TMPDIR:-/tmp}/emusen-mesen-probe}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

[ -f "$MESEN/makefile" ] || { echo "Not a Mesen2 checkout: $MESEN" >&2; exit 1; }

mkdir -p "$WORK/deps/bin"
cd "$WORK/deps"

if [ ! -f usr/include/SDL2/SDL.h ] || [ ! -f usr/include/X11/Xlib.h ]; then
    echo "== fetching SDL2/X11 headers into $WORK/deps"
    dnf download SDL2-devel SDL2 libX11-devel xorg-x11-proto-devel libxcb-devel libXau-devel
    for f in *.x86_64.rpm *.noarch.rpm; do
        [ -e "$f" ] || continue
        rpm2cpio "$f" | cpio -idmu --quiet
    done
    # sdl2-compat's libSDL2.so links against the system SDL3 at runtime.
    [ -e usr/lib64/libX11.so ] || ln -sf /usr/lib64/libX11.so.6 usr/lib64/libX11.so
fi

cat > bin/sdl2-config <<EOF
#!/bin/sh
case "\$1" in
--cflags) echo "-I$WORK/deps/usr/include/SDL2 -D_REENTRANT";;
--libs)   echo "-L$WORK/deps/usr/lib64 -lSDL2";;
esac
EOF
chmod +x bin/sdl2-config

export PATH="$WORK/deps/bin:$PATH"
export CPATH="$WORK/deps/usr/include"
export LIBRARY_PATH="$WORK/deps/usr/lib64"

echo "== applying trace instrumentation (idempotent)"
cd "$MESEN"
if ! grep -q g_gsuTraceOn Core/SNES/Coprocessors/GSU/Gsu.cpp; then
    git apply "$HERE/mesen-gsu-trace.patch"
fi
if ! grep -q g_cpuTraceOn Core/SNES/SnesCpu.cpp; then
    git apply "$HERE/mesen-cpu-trace.patch"
fi

# STATICLINK=false: the stock recipe wants libstdc++.a, which Fedora splits out.
echo "== building MesenCore.so (this takes a few minutes the first time)"
LTO=false STATICLINK=false make core -j"$(nproc)"

echo "== building the probe"
clang++ -fPIC -Wall --std=c++17 -m64 -O2 \
    -I"$WORK/deps/usr/include/SDL2" -I. -ICore -IUtilities -ISdl -ILinux \
    -o "$WORK/mesenprobe" "$HERE/MesenProbe.cpp" bin/pgohelperlib.so \
    -pthread -lstdc++fs -L"$WORK/deps/usr/lib64" -lSDL2 -lX11

echo
echo "Built: $WORK/mesenprobe"
echo "Run it from $MESEN (its DT_NEEDED is the relative bin/pgohelperlib.so):"
echo "  cd '$MESEN' && '$WORK/mesenprobe' <rom> <outDir> <startFrame> <endFrame> [stride] [gsuRamAddr] [traceUntilFrame]"
