#!/bin/bash
# Refreshes MarsRT's PGO profile for this machine's Rust target, pgo/marsrt.<target>.profdata, and its manifest,
# pgo/marsrt.<target>.pgo - see Mars_Native.md §6.17.
#
#   bash pgo/train.sh [training list, default pgo/training.txt]
#
# For each flavour below: an instrumented library, the training list run through the shim by pgo/driver, the counters merged;
# then every flavour merged into one profile. A flavour is a compiler, a cargo profile and a target, because each names the
# crates' symbols differently and a profile guides only the names it holds. The counters are updated atomically, because the
# rasteriser's workers share them and lost updates leave a function's counts inconsistent. Runs on Linux and macOS (bash 3.2 too).
#
# The list's lines are "<rom> <state|-> <frames> [Setting=value]... [rewind=1]", paths relative to GAMES. Its games are
# commercial and stay out of the repository; the profile holds counters about this crate's code, never a byte of them.
#
# Environment:
#   GAMES          where the list's files are (default ~/.cache/emusen/pgo/games)
#   TOOLCHAIN      the release CI pins for this target (default 1.98.1)
#   UPSTREAM       that release's sysroot, holding bin/cargo, bin/rustc and llvm-tools' llvm-profdata (default: rustup's
#                  toolchain of that name, else ~/.cache/emusen/toolchains/rust-<TOOLCHAIN>)
#   LLVM_PROFDATA  an llvm-profdata of the system rustc's LLVM (default: the system rustc's llvm-tools, else llvm-profdata)
#   WORK           scratch (default ~/.cache/emusen/pgo/work)
#   FLAVOURS       a subset of "release dist ci" (default all)
#   RUN            a command each build and each training run is wrapped in, such as "flock <lock>" to share a quiet machine
set -euo pipefail

CRATE="$(cd "$(dirname "$0")/.." && pwd)"
LIST="${1:-$CRATE/pgo/training.txt}"
LIST="$(cd "$(dirname "$LIST")" && pwd)/$(basename "$LIST")"
GAMES="${GAMES:-$HOME/.cache/emusen/pgo/games}"
TOOLCHAIN="${TOOLCHAIN:-1.98.1}"
WORK="${WORK:-$HOME/.cache/emusen/pgo/work}"
FLAVOURS="${FLAVOURS:-release dist ci}"
read -r -a RUN <<< "${RUN:-}"

die() { echo "train.sh: $*" >&2; exit 1; }
sum() { (sha256sum "$1" 2> /dev/null || shasum -a 256 "$1") | cut -c1-16; }
llvm_of_rustc() { "$1" -vV | sed -n 's/^LLVM version: //p'; }
llvm_of_profdata() { "$1" --version | sed -n 's/^ *LLVM version \([0-9.]*\).*/\1/p'; }

HOST="$(rustc -vV | sed -n 's/^host: //p')"
case "$HOST" in
  *-apple-darwin) LIBRARY=libmarsrt.dylib ;;
  *-linux-*) LIBRARY=libmarsrt.so ;;
  *) die "training on $HOST is not supported: the driver loads the library from beside itself as libmarsrt.so or .dylib" ;;
esac
tools="$(rustc --print sysroot)/lib/rustlib/$HOST/bin/llvm-profdata"
PROFDATA="${LLVM_PROFDATA:-$([ -x "$tools" ] && echo "$tools" || echo llvm-profdata)}"
if [ -z "${UPSTREAM:-}" ]; then
  if command -v rustup > /dev/null && rustup run "$TOOLCHAIN" rustc -V > /dev/null 2>&1; then
    UPSTREAM="$(rustup run "$TOOLCHAIN" rustc --print sysroot)"
  else
    UPSTREAM="$HOME/.cache/emusen/toolchains/rust-$TOOLCHAIN"
  fi
fi
OUT="$CRATE/pgo/marsrt.$HOST"

[ -f "$LIST" ] || die "no training list at $LIST"
[ -d "$GAMES" ] || die "no games at $GAMES; set GAMES"
command -v "$PROFDATA" > /dev/null || die "no llvm-profdata ($PROFDATA); set LLVM_PROFDATA"

mkdir -p "$WORK"
echo "== $HOST: the driver"
dotnet build "$CRATE/pgo/driver/MarsRtPgo.csproj" -c Release -p:EmuSenNative=false -o "$WORK/driver" -v quiet -nologo | grep -E "error|rror\(s\)" || true
[ -x "$WORK/driver/marsrt-pgo" ] || die "the driver did not build"

lines() { grep -v -E '^[[:space:]]*(#|$)' "$LIST"; }
manifest_runs=()
while read -r rom state frames rest; do
  line="$(basename "$rom") $(sum "$GAMES/$rom")"
  if [ "$state" = - ]; then line="$line power-on"; else line="$line $(basename "$state") $(sum "$GAMES/$state")"; fi
  manifest_runs+=("run = $line $frames ${rest:-}")
done < <(lines)

flavours=()
sources=()
merged=()
for f in $FLAVOURS; do
  target=()
  case $f in
    release) cargo=cargo; rustc=rustc; profile=release; profdata="$PROFDATA" ;;
    dist) cargo=cargo; rustc=rustc; profile=dist; profdata="$PROFDATA" ;;
    ci) cargo="$UPSTREAM/bin/cargo"; rustc="$UPSTREAM/bin/rustc"; profile=dist; target=(--target "$HOST")
        profdata="$UPSTREAM/lib/rustlib/$HOST/bin/llvm-profdata"
        [ -x "$cargo" ] && [ -x "$profdata" ] || die "flavour ci needs Rust $TOOLCHAIN with llvm-tools at $UPSTREAM" ;;
    *) die "flavour $f?" ;;
  esac
  rustc="$(command -v "$rustc")"
  [ "$(llvm_of_rustc "$rustc")" = "$(llvm_of_profdata "$profdata")" ] ||
    die "flavour $f: $rustc uses LLVM $(llvm_of_rustc "$rustc") and $profdata is LLVM $(llvm_of_profdata "$profdata"); the raw counters need the same version"

  echo "== flavour $f: instrumented build"
  tdir="$WORK/target-$f"; raw="$WORK/raw-$f"
  rm -rf "$raw" "$WORK/junk"; mkdir -p "$raw" "$WORK/junk"
  if ! (cd "$CRATE" && LLVM_PROFILE_FILE="$WORK/junk/%p.profraw" RUSTC="$rustc" ${RUN[@]+"${RUN[@]}"} "$cargo" build --lib --profile "$profile" ${target[@]+"${target[@]}"} \
    --target-dir "$tdir" --config "build.rustflags=['-Cprofile-generate=$raw', '-Cllvm-args=-instrprof-atomic-counter-update-all']" > "$WORK/build-$f.log" 2>&1); then
    tail -20 "$WORK/build-$f.log"; die "flavour $f: the instrumented build failed"
  fi
  if [ "$f" = ci ]; then out="$tdir/$HOST/$profile"; else out="$tdir/$profile"; fi
  status="$out/marsrt-pgo.txt"
  grep -q "^state = instrumented" "$status" || die "flavour $f: the build script did not see an instrumented build ($status)"
  sources+=("$(sed -n 's/^source = //p' "$status")")
  flavours+=("$(sed -n 's/^flavour = //p' "$status")")
  cp "$out/$LIBRARY" "$WORK/driver/$LIBRARY"

  echo "== flavour $f: training"
  while read -r rom state frames rest; do
    if [ "$state" = - ]; then st=-; else st="$GAMES/$state"; fi
    # shellcheck disable=SC2086
    LLVM_PROFILE_FILE="$raw/%p-%m.profraw" ${RUN[@]+"${RUN[@]}"} "$WORK/driver/marsrt-pgo" "$GAMES/$rom" "$st" "$frames" ${rest:-} < /dev/null || die "flavour $f: $rom failed"
  done < <(lines)
  n=$(find "$raw" -name '*.profraw' | wc -l | tr -d ' ')
  [ "$n" -gt 0 ] || die "flavour $f: no counters were written"
  "$profdata" merge -o "$WORK/flavour-$f.profdata" "$raw"/*.profraw
  echo "   $n runs merged: $("$profdata" show "$WORK/flavour-$f.profdata" | grep -E 'Total functions')"
  merged+=("$WORK/flavour-$f.profdata")
  rm -f "$WORK/driver/$LIBRARY"
done

for s in "${sources[@]}"; do [ "$s" = "${sources[0]}" ] || die "the flavours saw different sources: ${sources[*]}"; done

echo "== the profile"
"$PROFDATA" merge -o "$OUT.profdata" "${merged[@]}"
{
  echo "# MarsRT's PGO profile for $HOST: written by pgo/train.sh, read by build.rs, MSBuild and CI - see Mars_Native.md §6.17. Not edited by hand."
  echo "llvm = $(llvm_of_profdata "$PROFDATA")"
  echo "source = ${sources[0]}"
  echo "toolchain = $TOOLCHAIN"
  echo "target = $HOST"
  echo "trained = $(date -u +%Y-%m-%dT%H:%MZ)"
  for fl in "${flavours[@]}"; do echo "flavour = $fl"; done
  for r in "${manifest_runs[@]}"; do echo "$r"; done
} > "$OUT.pgo"
"$PROFDATA" show "$OUT.profdata" | grep -E "Total functions|Maximum function count"
ls -la "$OUT.profdata"
cat "$OUT.pgo"
