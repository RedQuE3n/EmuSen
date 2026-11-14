#!/bin/bash
# On the desktop: one tarball of what the handheld's runs need, built beforehand by cargo in this checkout: the Qt
# acceptance and the kit gallery's gate (test executables), emusen-qt in release with gate-pad, the platform library and
# the test core of the test-fast profile, the Frontend's sources the tests read, Testing's mockups, Moonrise references
# and scripts, and the published tree's program folder for the gates. See README.md.
# Usage: SONAMES=<file> pack.sh <out.tar.zst>
#   SONAMES    the handheld's library file names (`ldconfig -p`'s and those under /usr/lib; README.md), for qtbundle.sh
#   PUBLISHED  the published desktop tree (the main checkout's out/linux-x64/Mistress)
set -eu
out=$(realpath -m "$1")
root=$(cd "$(dirname "$0")/../.." && pwd)
published=${PUBLISHED:-/home/red/Projects/EmuSen Project/out/linux-x64/Mistress}
newest() { ls -t "$root"/Frontend/target/debug/deps/"$1"-* | grep -v '\.d$' | head -1; }
stage=$(mktemp -d "${TMPDIR:-/tmp}/hh-pack.XXXXXX")
trap 'rm -rf "$stage"' EXIT
mkdir -p "$stage/tree/Frontend/target/debug/deps" "$stage/tree/Frontend/target/release" "$stage/tree/out/linux-x64/Mistress/lib" \
  "$stage/tree/Platform/target/test-fast/examples" "$stage/tree/Testing"
(cd "$root" && tar cf - --exclude=target Frontend) | tar xf - -C "$stage/tree"
cp -a "$root/Frontend/target/cxxqt" "$stage/tree/Frontend/target/"
# The tests' executables without their debug sections, a third of the size; a panic still names its message.
for t in acceptance gate; do objcopy --strip-debug "$(newest $t)" "$stage/tree/Frontend/target/debug/deps/$t"; done
cp -a "$root/Frontend/target/release/emusen-qt" "$stage/tree/Frontend/target/release/"
cp -a "$root/Platform/target/test-fast/libemusen_platform.so" "$stage/tree/Platform/target/test-fast/"
cp -a "$root/Platform/target/test-fast/examples/libcrystal_test_core.so" "$stage/tree/Platform/target/test-fast/examples/"
cp -a "$root"/Testing/{mockups,moonrise,qt-gates,handheld} "$stage/tree/Testing/"
bash "$root/Testing/handheld/qtbundle.sh" "$stage/tree/qt" "$SONAMES" "$stage"/tree/Frontend/target/debug/deps/{acceptance,gate} \
  "$stage/tree/Frontend/target/release/emusen-qt" "$stage/tree/Platform/target/test-fast/libemusen_platform.so"
for dir in debug/deps release; do printf '[Paths]\nPrefix=%s/qt\nPlugins=plugins\nQmlImports=qml\n' "$root" > "$stage/tree/Frontend/target/$dir/qt.conf"; done
cp -a --reflink=auto "$published/lib/EmuSen" "$stage/tree/out/linux-x64/Mistress/lib/"
cp -a "$published/.dianashroot" "$stage/tree/out/linux-x64/Mistress/"
printf '%s\n' "$root" > "$stage/root"
git -C "$root" rev-parse HEAD > "$stage/commit"
tar -C "$stage" --zstd -cf "$out" root commit tree
echo "packed $out ($(du -h "$out" | cut -f1)) from $(cat "$stage/commit")"
