#!/bin/bash
# One build's gates of §9.3.5, by the rebuilt method (README.md): a sandbox made by sandbox.sh, one virtual KWin per run.
# Usage: gates.sh <root> <out-dir> <step>...   steps: warm (read the library once, snapshot the home), start (Home shown
# and the first press), idle (memory 10 s after Home), pause (memory at the Pause Card over ActRaiser)
#   ROUND=<n>  names the run directories
set -u
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$1" && pwd); out=$2; shift 2
mkdir -p "$out"; out=$(cd "$out" && pwd)
sb=$root/lib/EmuSen; round=${ROUND:-1}
roms=${ROMS:-/home/red/Documents/Roms}
export HOME=$root/hh XDG_CONFIG_HOME=$root/hh/config XDG_DATA_HOME=$root/hh/data XDG_CACHE_HOME=$root/hh/cache
export QT_QPA_PLATFORM=wayland SDL_AUDIODRIVER=dummy EMUSEN_PLATFORM_LIBRARY=$sb/libemusen_platform.so
export QT_LOGGING_RULES="qt.quick.focus=true" QT_MESSAGE_PATTERN="GATE %{time yyyy-MM-dd'T'HH:mm:ss.zzz} %{message}"
export TMPDIR=$HOME/.cache/emusen/tmp
program=(bwrap --die-with-parent --dev-bind / / --ro-bind "$roms" "$roms" -- "$sb/emusen-qt")
restore() { [ -d "$root/home.snap" ] && rsync -a --delete "$root/home.snap/" "$root/home/"; }
run() { # <name> <seconds>
  systemd-run --user --scope -p CPUWeight=20 -p MemoryMax=10G -p MemorySwapMax=2G -q \
    bash "$here/run.sh" "$out/$1" "$2" "${program[@]}"
}
# Home, the first console row's planets, Super Nintendo, ActRaiser (U) [!] on the grid's second row, Play, 10 s of play, the
# menu chord (Guide held 0.3 s), 6 s at the Pause Card; the card is seen in the run's last grabs.
route="wait:3 tap:a wait:2.5 tap:down wait:1 tap:right wait:0.8 tap:right wait:0.8 tap:right wait:1 tap:a wait:3 tap:down wait:0.8 tap:right wait:0.8 tap:right wait:1 tap:a wait:10 hold:guide:0.3 wait:6"
for step in "$@"; do
  case $step in
    warm) run warm 25; rsync -a --delete "$root/home/" "$root/home.snap/";;
    start) restore; PAD_FIRST=1 PAD="settle:$out/start-$round/grabs.tsv:0.1 tap:right wait:2" RSS=emusen-qt run start-$round 8
      python3 -I "$here/press.py" "$out/start-$round";;
    idle) restore; RSS=emusen-qt run idle-$round 16;;
    pause) restore; PAD_FIRST=1 PAD="$route" RSS=emusen-qt run pause-$round 40
      grep -c "hold:guide" "$out/pause-$round/pad.log" | sed 's/^/chords: /';;
  esac
done
