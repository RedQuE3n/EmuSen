#!/bin/bash
# Refreshes MarsRT's PGO profile, pgo/marsrt.profdata, and its manifest, pgo/marsrt.pgo - see Mars_Native.md §6.17.
#
#   bash pgo/train.sh [training list, default ~/.cache/emusen/pgo/training.txt]
#
# For each flavour below: an instrumented library, the training list run through the shim by pgo/driver, the counters merged;
# then every flavour merged into one profile. A flavour is a compiler, a cargo profile and a target, because each names the
# crates' symbols differently and a profile guides only the names it holds.
#
# The list's lines are "<rom> <state|-> <frames> [Setting=value]... [rewind=1]", paths relative to the list. Its games are
# commercial and stay out of the repository; the profile holds counters about this crate's code, never a byte of them.
#
# Environment: LLVM_PROFDATA (the system compiler's llvm-profdata, same LLVM version), UPSTREAM (a rustup-free install of
# the toolchain CI pins, default ~/.cache/emusen/toolchains/rust-<toolchain>), WORK (scratch, default ~/.cache/emusen/pgo/work),
# FLAVOURS (a subset of "release dist ci", default all).
set -euo pipefail

CRATE="$(cd "$(dirname "$0")/.." && pwd)"
LIST="$(realpath "${1:-$HOME/.cache/emusen/pgo/training.txt}")"
TOOLCHAIN=1.98.1
UPSTREAM="${UPSTREAM:-$HOME/.cache/emusen/toolchains/rust-$TOOLCHAIN}"
WORK="${WORK:-$HOME/.cache/emusen/pgo/work}"
PROFDATA="${LLVM_PROFDATA:-llvm-profdata}"
FLAVOURS="${FLAVOURS:-release dist ci}"
CI_TARGET=x86_64-unknown-linux-gnu

die() { echo "train.sh: $*" >&2; exit 1; }
[ -f "$LIST" ] || die "no training list at $LIST"
command -v "$PROFDATA" > /dev/null || die "no llvm-profdata ($PROFDATA); set LLVM_PROFDATA"
llvm_of_rustc() { "$1" -vV | sed -n 's/^LLVM version: //p'; }
llvm_of_profdata() { "$1" --version | sed -n 's/^ *LLVM version \([0-9.]*\).*/\1/p'; }

mkdir -p "$WORK"
echo "== the driver"
dotnet build "$CRATE/pgo/driver/MarsRtPgo.csproj" -c Release -p:EmuSenNative=false -o "$WORK/driver" -v quiet -nologo | grep -E "error|rror\(s\)" || true
[ -x "$WORK/driver/marsrt-pgo" ] || die "the driver did not build"

manifest_runs=()
while read -r rom state frames rest; do
  [ -z "${rom:-}" ] || [ "${rom:0:1}" = "#" ] && continue
  base="$(dirname "$LIST")"
  sum() { sha256sum "$1" | cut -c1-16; }
  line="$(basename "$rom") $(sum "$base/$rom")"
  [ "$state" = - ] && line+=" power-on" || line+=" $(basename "$state") $(sum "$base/$state")"
  manifest_runs+=("run = $line $frames ${rest:-}")
done < "$LIST"

flavours=()
sources=()
merged=()
for f in $FLAVOURS; do
  case $f in
    release) cargo=cargo; rustc=rustc; profile=release; target=(); profdata="$PROFDATA" ;;
    dist) cargo=cargo; rustc=rustc; profile=dist; target=(); profdata="$PROFDATA" ;;
    ci) cargo="$UPSTREAM/bin/cargo"; rustc="$UPSTREAM/bin/rustc"; profile=dist; target=(--target "$CI_TARGET")
        profdata="$UPSTREAM/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-profdata"
        [ -x "$cargo" ] && [ -x "$profdata" ] || die "flavour ci needs Rust $TOOLCHAIN with llvm-tools at $UPSTREAM" ;;
    *) die "flavour $f?" ;;
  esac
  rustc="$(command -v "$rustc")"
  [ "$(llvm_of_rustc "$rustc")" = "$(llvm_of_profdata "$profdata")" ] ||
    die "flavour $f: $rustc uses LLVM $(llvm_of_rustc "$rustc") and $profdata is LLVM $(llvm_of_profdata "$profdata"); the raw counters need the same version"

  echo "== flavour $f: instrumented build"
  tdir="$WORK/target-$f"; raw="$WORK/raw-$f"
  rm -rf "$raw"; mkdir -p "$raw" "$WORK/junk"
  if ! (cd "$CRATE" && LLVM_PROFILE_FILE="$WORK/junk/%p.profraw" RUSTC="$rustc" "$cargo" build --lib --profile "$profile" "${target[@]}" \
    --target-dir "$tdir" --config "build.rustflags=['-Cprofile-generate=$raw']" > "$WORK/build-$f.log" 2>&1); then
    tail -20 "$WORK/build-$f.log"; die "flavour $f: the instrumented build failed"
  fi
  out="$tdir/${target[1]:-}/$profile"; out="${out//\/\//\/}"
  status="$out/marsrt-pgo.txt"
  grep -q "^state = instrumented" "$status" || die "flavour $f: the build script did not see an instrumented build ($status)"
  sources+=("$(sed -n 's/^source = //p' "$status")")
  flavours+=("$(sed -n 's/^flavour = //p' "$status")")
  cp "$out/libmarsrt.so" "$WORK/driver/libmarsrt.so"

  echo "== flavour $f: training"
  while read -r rom state frames rest; do
    [ -z "${rom:-}" ] || [ "${rom:0:1}" = "#" ] && continue
    base="$(dirname "$LIST")"
    [ "$state" = - ] && st=- || st="$base/$state"
    # shellcheck disable=SC2086
    LLVM_PROFILE_FILE="$raw/%p-%m.profraw" "$WORK/driver/marsrt-pgo" "$base/$rom" "$st" "$frames" ${rest:-} || die "flavour $f: $rom failed"
  done < "$LIST"
  n=$(find "$raw" -name '*.profraw' | wc -l)
  [ "$n" -gt 0 ] || die "flavour $f: no counters were written"
  "$profdata" merge -o "$WORK/flavour-$f.profdata" "$raw"/*.profraw
  echo "   $n runs merged: $("$profdata" show "$WORK/flavour-$f.profdata" | grep -E 'Total functions')"
  merged+=("$WORK/flavour-$f.profdata")
  rm -f "$WORK/driver/libmarsrt.so"
done

for s in "${sources[@]}"; do [ "$s" = "${sources[0]}" ] || die "the flavours saw different sources: ${sources[*]}"; done

echo "== the profile"
"$PROFDATA" merge -o "$CRATE/pgo/marsrt.profdata" "${merged[@]}"
{
  echo "# MarsRT's PGO profile: written by pgo/train.sh, read by build.rs, MSBuild and CI - see Mars_Native.md §6.17. Not edited by hand."
  echo "llvm = $(llvm_of_profdata "$PROFDATA")"
  echo "source = ${sources[0]}"
  echo "toolchain = $TOOLCHAIN"
  echo "trained = $(date -u +%Y-%m-%dT%H:%MZ)"
  for fl in "${flavours[@]}"; do echo "flavour = $fl"; done
  for r in "${manifest_runs[@]}"; do echo "$r"; done
} > "$CRATE/pgo/marsrt.pgo"
"$PROFDATA" show "$CRATE/pgo/marsrt.profdata" | grep -E "Total functions|Maximum function count"
ls -la "$CRATE/pgo/marsrt.profdata"
cat "$CRATE/pgo/marsrt.pgo"
