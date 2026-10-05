#!/bin/bash
# Builds the referee benches with Verilator into work directories that also hold each core's microcode files and a copy
# of the bench (make cannot take a path with a space): fx68k from the MegaCD MiSTer checkout, Nuked-MD's m68kcpu from
# the Mega Drive MiSTer checkout. Beryl_M68k.md section 6. Usage: build-referees.sh [fx68k|nuked|both]
set -euo pipefail
WHICH="${1:-both}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/emusen"
FX="${FX68K_SRC:-$HOME/Projects/megacd-mister-reference/rtl/FX68K}"
NUKED="${NUKED_SRC:-$HOME/Projects/megadrive-mister-reference/rtl/nuked-md}"
VROOT="${VERILATOR_ROOT:-$CACHE/toolchains/verilator/usr/share/verilator}"
VBIN="${VERILATOR_BIN:-$CACHE/toolchains/verilator/usr/bin/verilator_bin}"
export VERILATOR_ROOT="$VROOT" LD_LIBRARY_PATH="${LD_LIBRARY_PATH:-}:$CACHE/toolchains/verilator/usr/lib64"
FLAGS=(--cc --exe --build -j 8 -O3 -Wno-fatal -Wno-lint -Wno-style -Wno-BLKANDNBLK -Wno-UNOPTFLAT)

build() {  # work-dir top-module extra-flags... -- sources...
    local work="$1" top="$2"; shift 2
    local extra=() srcs=()
    while [ "$1" != "--" ]; do extra+=("$1"); shift; done; shift
    srcs=("$@")
    cp "$HERE/tb68k.cpp" "$work/"
    "$VBIN" "${FLAGS[@]}" "${extra[@]}" --top-module "$top" -Mdir "$work/obj" -o "$work/tb68k" "${srcs[@]}" "$work/tb68k.cpp" \
        > "$work/build.log" 2>&1 || { tail -30 "$work/build.log"; exit 1; }
    echo "$work/tb68k"
}

if [ "$WHICH" = fx68k ] || [ "$WHICH" = both ]; then
    W="$CACHE/probe/fx68k"; mkdir -p "$W"; cp "$FX/microrom.mem" "$FX/nanorom.mem" "$W/"
    build "$W" fx68k -- "$FX/fx68k.sv" "$FX/fx68kAlu.sv" "$FX/uaddrPla.sv"
fi
if [ "$WHICH" = nuked ] || [ "$WHICH" = both ]; then
    W="$CACHE/probe/nuked68k"; mkdir -p "$W"; cp "$NUKED/68k_ucode.txt" "$NUKED/68k_ncode.txt" "$W/"
    build "$W" m68kcpu -CFLAGS -DNUKED -- "$NUKED/68k.v"
fi
