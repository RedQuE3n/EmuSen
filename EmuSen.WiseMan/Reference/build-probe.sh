#!/usr/bin/env bash
# Builds the reference probe against an emulator - see EmuSen_Debugging_Tools_Reference_v5.md
# §3.45 for the architecture, §3.50 for the Rust port, §3.52 for the Mesen C ABI
# and §3.54 for the host table below and what each host can and cannot do.
#
#   ./build-probe.sh <backend> [checkout] [work-dir]
#
# Backends:
#   mesen     <checkout> is a Mesen2 source tree. The probe is built from
#             probe-rs/ with cargo and linked against the checkout's
#             bin/pgohelperlib.so, which carries the C ABI added by
#             patches/mesen/probe-c-api.patch. Mesen's makefile wants an SDL2
#             toolchain, borrowed per host by deps_mesen_toolchain.
#             Linux and macOS only - see §3.54 for why Windows cannot, and use
#             WSL there if you need it.
#   libretro  No checkout: the core is chosen at *run* time with --core, and the
#             only build input is libretro.h, taken from a real distribution
#             rather than vendored so it cannot drift from the real ABI. Cores
#             come from the same place (libretro-nestopia, libretro-gambatte,
#             libretro-bsnes-mercury, libretro-mgba, ...). Every host.
# One binary per backend, because linking one is not free - Mesen's pulls in a
# 14 MB core library and libretro's needs nothing but dlopen.
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

# ---------------------------------------------------------------- the host table
# The only place a host is named. Everything below is written against these
# capabilities rather than against a host, so a host that cannot do something
# says so by declaring it here and is refused with a reason - never by failing
# somewhere deep in a toolchain that was never going to work. See §3.54.
#
#   DEPS      where headers and cores come from: a distribution, or upstream
#   CORE_EXT  what a libretro core is called here
#   EXE       what cargo calls the binary it just built
#   MESEN     whether the Mesen backend can be built at all
case "$(uname -s)" in
Linux)
    HOST=linux;   DEPS=distro;   CORE_EXT=so;    EXE=;     MESEN=yes; BUILDBOT_DIR= ;;
Darwin)
    HOST=macos;   DEPS=upstream; CORE_EXT=dylib; EXE=;     MESEN=yes; BUILDBOT_DIR="apple/osx/$(uname -m)" ;;
MINGW*|MSYS*|CYGWIN*)
    HOST=windows; DEPS=upstream; CORE_EXT=dll;   EXE=.exe; MESEN=no;  BUILDBOT_DIR="windows/$(uname -m)" ;;
*)
    echo "build-probe.sh knows Linux, macOS and Windows, not $(uname -s)" >&2; exit 1 ;;
esac

# Pinned because master is not a released ABI: v1.22.2's libretro.h is
# byte-identical to Fedora's retroarch-devel and master's is not - see §3.54.
RETROARCH_TAG=v1.22.2
BUILDBOT=https://buildbot.libretro.com/nightly

cpus() {
    case "$HOST" in
    linux)   nproc ;;
    macos)   sysctl -n hw.ncpu ;;
    windows) nproc 2>/dev/null || echo "${NUMBER_OF_PROCESSORS:-4}" ;;
    esac
}

# Names the missing tool instead of letting a pipeline fail halfway through it.
need() {
    for tool in "$@"; do
        command -v "$tool" >/dev/null || { echo "build-probe.sh needs '$tool' on $HOST" >&2; exit 1; }
    done
}

# Unpacks distro packages into <work-dir> without installing anything.
fetch_rpms() {
    local marker="$1"; shift
    [ -e "$WORK/deps/$marker" ] && return 0
    need dnf rpm2cpio cpio
    echo "== fetching $* into $WORK/deps"
    mkdir -p "$WORK/deps"
    ( cd "$WORK/deps" && dnf download "$@" && for f in *.rpm; do
        [ -e "$f" ] || continue
        rpm2cpio "$f" | cpio -idmu --quiet
    done )
}

# Every host lands libretro.h at deps/usr/include/libretro-common/, which is a
# path build.rs's find_include_dir() already searched, so the Rust side has no
# host knowledge at all.
deps_libretro_header() {
    local dir="$WORK/deps/usr/include/libretro-common"
    [ -f "$dir/libretro.h" ] && return 0
    case "$DEPS" in
    distro)
        fetch_rpms usr/include/libretro-common/libretro.h retroarch-devel
        ;;
    upstream)
        need curl
        echo "== fetching libretro.h ($RETROARCH_TAG) into $WORK/deps"
        mkdir -p "$dir"
        curl -fsSL -o "$dir/libretro.h" \
            "https://raw.githubusercontent.com/libretro/RetroArch/$RETROARCH_TAG/libretro-common/include/libretro.h"
        ;;
    esac
}

# Named in Fedora's spelling on every host: libretro-bsnes-mercury reaches the
# buildbot's bsnes_mercury, so one documented command works everywhere.
deps_libretro_core() {
    local want="$1"
    case "$DEPS" in
    distro)
        fetch_rpms ".fetched-$want" "$want"
        touch "$WORK/deps/.fetched-$want"
        ;;
    upstream)
        local stem="${want#libretro-}"; stem="${stem//-/_}"
        [ -f "$WORK/deps/${stem}_libretro.$CORE_EXT" ] && return 0
        need curl unzip
        echo "== fetching ${stem}_libretro.$CORE_EXT from the libretro buildbot"
        mkdir -p "$WORK/deps"
        curl -fsSL -o "$WORK/deps/${stem}_libretro.$CORE_EXT.zip" \
            "$BUILDBOT/$BUILDBOT_DIR/latest/${stem}_libretro.$CORE_EXT.zip"
        unzip -oq "$WORK/deps/${stem}_libretro.$CORE_EXT.zip" -d "$WORK/deps"
        ;;
    esac
}

# Puts an SDL2 toolchain in front of Mesen's makefile, which reads sdl2-config.
deps_mesen_toolchain() {
    case "$HOST" in
    linux)
        mkdir -p "$WORK/deps/bin"
        if [ ! -f "$WORK/deps/usr/include/SDL2/SDL.h" ] || [ ! -f "$WORK/deps/usr/include/X11/Xlib.h" ]; then
            need dnf rpm2cpio cpio
            echo "== fetching SDL2/X11 headers into $WORK/deps"
            ( cd "$WORK/deps" && dnf download SDL2-devel SDL2 libX11-devel xorg-x11-proto-devel libxcb-devel libXau-devel
              for f in *.x86_64.rpm *.noarch.rpm; do
                  [ -e "$f" ] || continue
                  rpm2cpio "$f" | cpio -idmu --quiet
              done
              # sdl2-compat's libSDL2.so links against the system SDL3 at runtime.
              [ -e usr/lib64/libX11.so ] || ln -sf /usr/lib64/libX11.so.6 usr/lib64/libX11.so )
        fi
        cat > "$WORK/deps/bin/sdl2-config" <<EOF
#!/bin/sh
case "\$1" in
--cflags) echo "-I$WORK/deps/usr/include/SDL2 -D_REENTRANT";;
--libs)   echo "-L$WORK/deps/usr/lib64 -lSDL2";;
esac
EOF
        chmod +x "$WORK/deps/bin/sdl2-config"
        export PATH="$WORK/deps/bin:$PATH"
        export CPATH="$WORK/deps/usr/include"
        export LIBRARY_PATH="$WORK/deps/usr/lib64"
        ;;
    macos)
        # Homebrew has no sdl2 formula at all, and a bottle cannot be unpacked
        # into <work-dir> the way an RPM can - §3.54 says why this one dependency
        # is installed rather than borrowed. The makefile needs no X11 or
        # libevdev here; it empties both off Linux.
        need brew
        local prefix
        prefix="$(brew --prefix sdl2-compat 2>/dev/null || true)"
        [ -n "$prefix" ] && [ -x "$prefix/bin/sdl2-config" ] || {
            echo "Mesen's makefile needs sdl2-config: brew install sdl2-compat" >&2
            exit 1
        }
        export PATH="$prefix/bin:$PATH"
        export CPATH="$prefix/include"
        export LIBRARY_PATH="$prefix/lib"
        ;;
    esac
}

# Mach-O records a dylib's own install name, not the path it was linked against,
# so the checkout-relative load that -l: gives the ELF probe is written in after
# the fact. Read rather than assumed, because Mesen sets no -install_name.
fix_mesen_load_path() {
    need otool install_name_tool
    local recorded
    recorded="$(otool -D "$CHECKOUT/bin/pgohelperlib.so" | tail -n 1)"
    [ -n "$recorded" ] || { echo "otool -D found no install name to rewrite" >&2; exit 1; }
    install_name_tool -change "$recorded" bin/pgohelperlib.so "$WORK/probe"
}

case "$BACKEND" in
mesen)
    [ "$MESEN" = yes ] || {
        echo "The mesen backend cannot be built on $HOST - see EmuSen_Debugging_Tools_Reference_v5.md §3.54." >&2
        echo "Mesen's MSVC projects list their sources, so the add-only probe-c-api.patch is never" >&2
        echo "compiled, and a PE import names a DLL rather than a path. Use the libretro backend" >&2
        echo "here, or the mesen backend under WSL." >&2
        exit 1
    }
    [ -n "$CHECKOUT" ] || { echo "The mesen backend needs a Mesen2 checkout" >&2; exit 1; }
    [ -f "$CHECKOUT/makefile" ] || { echo "Not a Mesen2 checkout: $CHECKOUT" >&2; exit 1; }
    ;;
libretro)
    need cargo
    deps_libretro_header
    mkdir -p "$WORK"
    echo "== building the probe"
    # bindgen reads the same header the C++ backend included, so the Rust ABI
    # cannot drift from the real one either - see §3.50.
    LIBRETRO_INCLUDE_DIR="$WORK/deps/usr/include/libretro-common" \
        cargo build --release --manifest-path "$HERE/probe-rs/Cargo.toml" \
        --features libretro --target-dir "$WORK/target"
    cp "$WORK/target/release/probe$EXE" "$WORK/probe$EXE"
    echo
    echo "Built: $WORK/probe$EXE"
    echo "Cores come from a distribution, not this repo; fetch one with:"
    echo "  ./build-probe.sh libretro-core libretro-nestopia"
    echo "  '$WORK/probe$EXE' --core <core>_libretro.$CORE_EXT <rom> <outDir> <start> <end> [stride]"
    exit 0
    ;;
libretro-core)
    # Fetches a core into the libretro backend's work dir and says where it
    # landed. Nothing is installed system-wide.
    [ -n "$CHECKOUT" ] || { echo "usage: build-probe.sh libretro-core <package>" >&2; exit 1; }
    WORK="${3:-${XDG_CACHE_HOME:-$HOME/.cache}/emusen/probe/libretro}"
    deps_libretro_core "$CHECKOUT"
    # Not find -printf: that is GNU-only and this script runs on three hosts.
    find "$WORK/deps" -name "*_libretro.$CORE_EXT" | sed 's/^/core: /' | sort
    exit 0
    ;;
*)
    echo "Unknown backend '$BACKEND' (known: mesen, libretro, libretro-core)" >&2
    exit 1
    ;;
esac

need cargo git make
deps_mesen_toolchain

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
# That glob is also exactly why this backend has no Windows host - see §3.54.
[ -f Core/Shared/EmuSenProbeApi.cpp ]                     || git apply "$HERE/patches/mesen/probe-c-api.patch"

# STATICLINK=false: the stock recipe wants libstdc++.a, which Fedora splits out.
# The makefile forces both off on Darwin anyway, so passing them is harmless.
echo "== building the Mesen core (a few minutes the first time, a no-op after)"
LTO=false STATICLINK=false make core -j"$(cpus)"

echo "== building the probe"
# cargo does its own staleness tracking, so there is no hand-rolled -nt check
# here as there was for the C++ link step.
MESEN_CHECKOUT="$CHECKOUT" cargo build --release --manifest-path "$HERE/probe-rs/Cargo.toml" \
    --no-default-features --features mesen --target-dir "$WORK/target"
cp "$WORK/target/release/probe" "$WORK/probe"

[ "$HOST" = macos ] && fix_mesen_load_path

echo
echo "Built: $WORK/probe"
echo "Run it from $CHECKOUT (it loads the relative bin/pgohelperlib.so):"
echo "  cd '$CHECKOUT' && '$WORK/probe' <rom> <outDir> <startFrame> <endFrame> [stride] [anchorAddr] [traceUntilFrame]"
