#!/bin/bash
# The stale-core case of EmuSen_RustPlatform.md §28.10, on a project already built once:
# an output folder holding a different core of the same size, newer than the build's copy.
# The check must refuse that folder, and a build must end with the core its sidecar describes.
# usage: stale-core-copy-test.sh [project]   (default EmuSen/EmuSen.csproj)
set -u
here=$(cd "$(dirname "$0")" && pwd)
proj=$(realpath "${1:-$here/../EmuSen.csproj}")
export MSBUILDDISABLENODEREUSE=1
fail() { echo "FAIL: $*"; exit 1; }

out=$(dotnet msbuild "$proj" -getProperty:OutDir -nologo | tr '\\' /) || fail "cannot read OutDir of $proj"
case "$out" in /*) ;; *) out="$(dirname "$proj")/$out" ;; esac
sidecar=$(ls "$out"/*.core.json 2>/dev/null | head -1)
[ -n "$sidecar" ] || { echo "SKIP: no sidecar in $out; build $proj first"; exit 2; }
lib="$out/$(grep -o '"library" *: *"[^"]*"' "$sidecar" | head -1 | sed 's/.*"\([^"]*\)"$/\1/')"
want=$(grep -o '"sha256" *: *"[0-9a-fA-F]*"' "$sidecar" | head -1 | sed 's/.*"\([^"]*\)"$/\1/' | tr A-F a-f)
[ -f "$lib" ] || fail "$lib is missing"

size=$(stat -c %s "$lib")
byte=$(od -An -tu1 -j $((size / 2)) -N1 "$lib" | tr -d ' ')
printf "\\$(printf %03o $(( (byte + 1) % 256 )))" | dd of="$lib" bs=1 seek=$((size / 2)) conv=notrunc status=none
touch -d '+1 hour' "$lib"
[ "$(sha256sum "$lib" | cut -c1-64)" != "$want" ] || fail "could not plant a different core"
echo "planted: $(basename "$lib"), same size, one byte changed, an hour newer"

log=$(dotnet msbuild "$here/CheckCoreSidecars.proj" -t:Check -p:Folder="$out" -nologo -v:m 2>&1)
[ $? -ne 0 ] && grep -q "is not the library its sidecar describes" <<<"$log" || fail "the check passed a folder with a stale core"
echo "ok: the check refuses the folder"

log=$(dotnet build "$proj" -p:UseSharedCompilation=false -nologo -v:m 2>&1)
[ $? -eq 0 ] || { grep -m3 "error" <<<"$log"; fail "the build failed with the stale core in place"; }
[ "$(sha256sum "$lib" | cut -c1-64)" = "$want" ] || fail "$(basename "$lib") still differs from its sidecar after the build"
echo "ok: the build copied the core its sidecar describes"
echo "PASS"
