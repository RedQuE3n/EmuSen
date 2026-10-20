#!/bin/bash
# Builds the YM2413 referee bench (Endou_Opll.md section 7): Nuked-OPLL's opll.c taken from the tarball pinned in
# Endou_References.md section 3, never opened, compiled beside tb_opll.c, which includes only opll.h. Usage:
# build-opll.sh; then `cargo run --release --example opllref -- voices | <bench> > referee.txt` in Endou's crate and
# `cargo run --release --example opllref -- compare referee.txt`.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/emusen/probe/endou/refs/nuked-opll"
TAR="$CACHE/Nuked-OPLL-1269cf5.tar.gz"
SUM=a0c8f63205f85c745627cfe5f4dc8fbe7d506ea63b2f086ccab62c631e714833
echo "$SUM  $TAR" | sha256sum -c --quiet
W="$CACHE/bench"
mkdir -p "$W"
tar xzf "$TAR" -C "$W" --strip-components=1 --wildcards '*/opll.c' '*/opll.h' '*/LICENSE'
cc -O2 -std=c99 -I"$W" -o "$W/tb_opll" "$HERE/tb_opll.c" "$W/opll.c"
echo "$W/tb_opll"
