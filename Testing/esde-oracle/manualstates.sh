#!/bin/bash
# Captures the media viewer's PDF mode on the NES's manual game (library.py's MANUAL_GAME, entry 1) in CarAlt at both
# sizes, one run at a time (README.md, "States"); DRY=1 only prints.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
for size in "1280 800" "1920 1200"; do
    for view in gamelist:1:media:0 gamelist:1:manual:0 gamelist:1:manual:1 gamelist:1:manual:2 gamelist:1:manual:3 \
        gamelist:1:manual:0:zoom:1 gamelist:1:manual:0:zoom:2 gamelist:1:manual:0:zoom:2:pan:2; do
        if [ "${DRY:-0}" = 1 ]; then
            echo "SYSTEM=nes bash run_esde.sh caralt-es-de $view $size"
        else
            # shellcheck disable=SC2086
            SYSTEM=nes bash "$here/run_esde.sh" caralt-es-de "$view" $size
        fi
    done
done
