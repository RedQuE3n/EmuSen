#!/bin/bash
# Writes the help copies of helpstates.tsv and captures each, one at a time (README.md, "Help copies"); DRY=1 only prints.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
theme=${CARALT:-$HOME/.cache/emusen/bigpicture/corpus/whole/caralt-es-de}
out=$HOME/.cache/emusen/esde-oracle/helpcopies
grep -v '^#' "$here/helpstates.tsv" | while IFS=$'\t' read -r name size props; do
    copy="caralt-help-$name-es-de"
    # shellcheck disable=SC2086
    python3 -I "$here/helpcopy.py" "$theme" "$out" "$copy" "$size" $props > /dev/null
    if [ "${DRY:-0}" = 1 ]; then
        echo "THEMES=$out bash run_esde.sh $copy gamelist:1 1280 800"
    else
        THEMES=$out bash "$here/run_esde.sh" "$copy" gamelist:1 1280 800
    fi
done
