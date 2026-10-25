#!/bin/bash
# Builds the chdman that chdman.pin names, for linux-x64, from MAME's own source archive, following only MAME's
# documentation "Compiling MAME" of the pinned release: its Fedora prerequisites and its TOOLS=1 EMULATOR=0 options.
# MAME's source is unpacked and built as a black box and never read. The build runs in a throwaway Fedora container
# (rootless podman), so nothing is installed on the machine and no sudo is used. See EmuSen_GamePackages.md §4.9, §12.1.
#
# Usage: bash build-chdman.sh [<work-dir>]      default ~/.cache/emusen/toolchains/mame<release>
# Output: ~/.cache/emusen/toolchains/chdman/linux-x64/chdman, where Platform.targets finds it, and its SHA-256 compared
# with the pin.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
pin="$here/chdman.pin"
field() { awk -v k="$1" '$1 == k { print $2 }' "$pin"; }
release=$(field release); tag=$(field tag); url=$(field source); want=$(field source-sha256); pinned=$(field linux-x64)
work=${1:-$HOME/.cache/emusen/toolchains/$tag}
out=$HOME/.cache/emusen/toolchains/chdman/linux-x64
mkdir -p "$work" "$out"
cd "$work"

archive=$(basename "$url")
[ -f "$archive" ] || curl -sSL -o "$archive" "$url"
got=$(sha256sum "$archive" | cut -d' ' -f1)
[ "$got" = "$want" ] || { echo "FAIL: $archive has SHA-256 $got; chdman.pin names $want"; exit 1; }
rm -rf src && 7z x -y -osrc "$archive" > /dev/null

cat > build-in-container.sh <<'EOF'
#!/bin/bash
# Inside registry.fedoraproject.org/fedora:44: "Compiling MAME"'s Fedora prerequisites, libstdc++-static so that the
# binary carries its own C++ library and runs on SteamOS too, then the tools alone at low priority.
set -e
dnf install -y -q git gcc gcc-c++ SDL2-devel SDL2_ttf-devel libXi-devel libXinerama-devel qt6-qtbase-devel expat-devel fontconfig-devel alsa-lib-devel pulseaudio-libs-devel python3 make which libstdc++-static
cd /src
nice -n 19 make TOOLS=1 EMULATOR=0 NOWERROR=1 ARCHOPTS="-static-libstdc++ -static-libgcc" -j"$(nproc)"
EOF
podman run --rm --cpu-shares=205 -v "$PWD/src:/src:Z" -v "$PWD/build-in-container.sh:/build.sh:ro,Z" registry.fedoraproject.org/fedora:44 bash /build.sh > build.log 2>&1 \
  || { echo "FAIL: the build stopped; see $work/build.log"; exit 1; }

cp -p src/chdman "$out/chdman"
built=$(sha256sum "$out/chdman" | cut -d' ' -f1)
"$out/chdman" 2>&1 | head -1
echo "built  $built  $out/chdman"
echo "pinned $pinned"
[ "$built" = "$pinned" ] && echo "ok: the binary is the pinned one" || echo "NOTE: a rebuild differs from the pin; the tool refuses it until chdman.pin records this hash"
