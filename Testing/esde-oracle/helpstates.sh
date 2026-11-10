#!/bin/bash
# Writes the help copies of a table (helpstates.tsv by default) and captures each, one at a time; DRY=1 only prints.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
theme=${CARALT:-$HOME/.cache/emusen/bigpicture/corpus/whole/caralt-es-de}
out=$HOME/.cache/emusen/esde-oracle/helpcopies
states=${1:-$here/helpstates.tsv}
grep -v '^#' "$states" | while IFS=$'\t' read -r name size props; do
    copy="caralt-help-$name-es-de"
    # shellcheck disable=SC2086
    python3 -I "$here/helpcopy.py" "$theme" "$out" "$copy" "$size" $props > /dev/null
    if [ "${DRY:-0}" = 1 ]; then
        echo "THEMES=$out bash run_esde.sh $copy gamelist:1 1280 800"
    else
        THEMES=$out bash "$here/run_esde.sh" "$copy" gamelist:1 1280 800
    fi
done
