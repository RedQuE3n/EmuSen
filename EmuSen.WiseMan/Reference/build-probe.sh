#!/usr/bin/env bash
# Builds the reference probe against an emulator - see EmuSen_Debugging_Tools_Reference_v5.md
# §3.45 for the architecture, §3.50 for the Rust port and §3.52 for the Mesen C ABI.
#
#   ./build-probe.sh <backend> [checkout] [work-dir]
#
# Backends:
#   mesen     <checkout> is a Mesen2 source tree. The probe is built from
#             probe-rs/ with cargo and linked against the checkout's
#             bin/pgohelperlib.so, which carries the C ABI added by
#             patches/mesen/probe-c-api.patch. Mesen's makefile wants SDL2 and
#             X11 headers; rather than install them system-wide this fetches the
#             RPMs and unpacks them into <work-dir>, then points the compiler at
#             that tree with CPATH/LIBRARY_PATH plus a shim sdl2-config. Nothing
#             outside <work-dir> and the checkout is touched.
#   libretro  No checkout: the core is chosen at *run* time with --core, and the
#             only build input is libretro.h, taken from Fedora's retroarch-devel
#             rather than vendored so it cannot drift from the real ABI. Cores
#             are ordinary packages too (libretro-nestopia, libretro-gambatte,
#             libretro-bsnes-mercury, libretro-mgba, ...), unpacked the same way.
#             Built from probe-rs/ with cargo; bindgen reads that same header.
# One binary per backend, because linking one is not free - Mesen's pulls in a
# 14 MB MesenCore.so and libretro's needs nothing but dlopen.
#
# <work-dir> holds both the unpacked headers and the built probe, so it must
# outlive a reboot: defaulting it to /tmp made every session re-download the RPMs
# and relink, which was the whole of "we keep building it each time" - the
# emulator's own object tree was never the cost.
set -euo pipefail

BACKEND="${1:?usage: build-probe.sh <backend> [checkout] [work-dir]}"
CHECKOUT="${2:-}"
WORK="${3:-${XDG_CACHE_HOME:-$HOME/.cache}/emusen/probe/$BACKEND}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Unpacks distro packages into <work-dir> without installing anything, the same
# trick the Mesen path already uses for SDL2 - see the header comment.
fetch_rpms() {
    local marker="$1"; shift
    [ -e "$WORK/deps/$marker" ] && return 0
    echo "== fetching $* into $WORK/deps"
    mkdir -p "$WORK/deps"
    ( cd "$WORK/deps" && dnf download "$@" && for f in *.rpm; do
        [ -e "$f" ] || continue
        rpm2cpio "$f" | cpio -idmu --quiet
    done )
}

case "$BACKEND" in
mesen)
    [ -n "$CHECKOUT" ] || { echo "The mesen backend needs a Mesen2 checkout" >&2; exit 1; }
    [ -f "$CHECKOUT/makefile" ] || { echo "Not a Mesen2 checkout: $CHECKOUT" >&2; exit 1; }
    ;;
libretro)
    fetch_rpms usr/include/libretro-common/libretro.h retroarch-devel
    mkdir -p "$WORK"
    echo "== building the probe"
    # bindgen reads the same header the C++ backend included, so the Rust ABI
    # cannot drift from the real one either - see §3.50.
    LIBRETRO_INCLUDE_DIR="$WORK/deps/usr/include/libretro-common" \
        cargo build --release --manifest-path "$HERE/probe-rs/Cargo.toml" \
        --features libretro --target-dir "$WORK/target"
    cp "$WORK/target/release/probe" "$WORK/probe"
    echo
    echo "Built: $WORK/probe"
    echo "Cores are packages; fetch one without installing it, e.g.:"
    echo "  ./build-probe.sh libretro-core libretro-nestopia"
    echo "  '$WORK/probe' --core <core>_libretro.so <rom> <outDir> <start> <end> [stride]"
    exit 0
    ;;
libretro-core)
    # Fetches a core package into the libretro backend's work dir and says where
    # its .so landed. Nothing is installed system-wide.
    [ -n "$CHECKOUT" ] || { echo "usage: build-probe.sh libretro-core <package>" >&2; exit 1; }
    WORK="${3:-${XDG_CACHE_HOME:-$HOME/.cache}/emusen/probe/libretro}"
    fetch_rpms ".fetched-$CHECKOUT" "$CHECKOUT"
    touch "$WORK/deps/.fetched-$CHECKOUT"
    find "$WORK/deps" -name "*_libretro.so" -printf "core: %p\n" | sort
    exit 0
    ;;
*)
    echo "Unknown backend '$BACKEND' (known: mesen, libretro, libretro-core)" >&2
    exit 1
    ;;
esac

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
cd "$CHECKOUT"
grep -q g_gsuTraceOn    Core/SNES/Coprocessors/GSU/Gsu.cpp || git apply "$HERE/patches/mesen/gsu-trace.patch"
grep -q g_cpuTraceOn    Core/SNES/SnesCpu.cpp              || git apply "$HERE/patches/mesen/cpu-trace.patch"
grep -q g_nesApuTrace   Core/NES/NesMemoryManager.cpp      || git apply "$HERE/patches/mesen/nes-apu-trace.patch"
# Fills the same buffer cpu-trace.patch declares, so it must be applied after it.
grep -q g_cpuTraceOn    Core/NES/NesCpu.cpp                || git apply "$HERE/patches/mesen/nes-cpu-trace.patch"
# The sentinel is g_probeStopArmed, not g_probeStopFrame: a checkout carrying the
# pre-§3.45b patch has the latter already and would silently keep the old stop.
grep -q g_probeStopArmed Core/Shared/Emulator.cpp         || git apply "$HERE/patches/mesen/probe-frame-stop.patch"
# Add-only: it creates Core/Shared/EmuSenProbeApi.cpp, which makefile:138 globs
# in with no makefile edit, so it can never conflict on a rebase - see §3.52.
[ -f Core/Shared/EmuSenProbeApi.cpp ]                     || git apply "$HERE/patches/mesen/probe-c-api.patch"

# STATICLINK=false: the stock recipe wants libstdc++.a, which Fedora splits out.
echo "== building MesenCore.so (a few minutes the first time, a no-op after)"
LTO=false STATICLINK=false make core -j"$(nproc)"

echo "== building the probe"
# cargo does its own staleness tracking, so there is no hand-rolled -nt check
# here as there was for the C++ link step.
MESEN_CHECKOUT="$CHECKOUT" cargo build --release --manifest-path "$HERE/probe-rs/Cargo.toml" \
    --no-default-features --features mesen --target-dir "$WORK/target"
cp "$WORK/target/release/probe" "$WORK/probe"


echo
echo "Built: $WORK/probe"
echo "Run it from $CHECKOUT (its DT_NEEDED is the relative bin/pgohelperlib.so):"
echo "  cd '$CHECKOUT' && '$WORK/probe' <rom> <outDir> <startFrame> <endFrame> [stride] [anchorAddr] [traceUntilFrame]"
