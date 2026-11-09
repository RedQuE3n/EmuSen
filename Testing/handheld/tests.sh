#!/bin/bash
# On the handheld, inside box.sh: one of the Qt tests unpacked by pack.sh, with the prebuilt platform library and test
# core, its output to <scratch>/logs/<test>.log. acceptance makes its own `kwin_wayland --virtual` (Vulkan, SDL's
# in-process virtual pad); gate is the kit gallery's stage 0 gate, offscreen. See README.md.
# Usage: tests.sh <scratch> acceptance|gate
set -u
scratch=$(cd "$1" && pwd); test=$2
root=$(cat "$scratch/root")
mkdir -p "$scratch/logs" "$scratch/compare"
export EMUSEN_PLATFORM_LIBRARY=$root/Platform/target/test-fast/libemusen_platform.so
export EMUSEN_TEST_CORE=$root/Platform/target/test-fast/examples/libcrystal_test_core.so
export EMUSEN_QT_COMPARE=$scratch/compare LD_LIBRARY_PATH=$QT_BUNDLE_LIBS
case $test in
  acceptance|gate) ;;
  *) echo "no test $test" >&2; exit 2;;
esac
cd "$root/Frontend"
start=$(date +%s.%N)
"$root/Frontend/target/debug/deps/$test" --test-threads=1 > "$scratch/logs/$test.log" 2>&1
rc=$?
awk -v rc=$rc -v s=$start -v e=$(date +%s.%N) 'BEGIN { printf "exit %s after %.1f s\n", rc, e - s }' | tee -a "$scratch/logs/$test.log"
grep -E "^test result|panicked" "$scratch/logs/$test.log"
exit $rc
