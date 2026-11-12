#!/bin/bash
# Writes the help copies of a table (helpstates.tsv by default) and captures each, one at a time; DRY=1 only prints.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
# THEME and PREFIX take another theme's copies (Artflix: its folder, artflix-help, and file=default.xml in the table).
theme=${THEME:-${CARALT:-$HOME/.cache/emusen/bigpicture/corpus/whole/caralt-es-de}}
prefix=${PREFIX:-caralt-help}
out=$HOME/.cache/emusen/esde-oracle/helpcopies
states=${1:-$here/helpstates.tsv}
grep -v '^#' "$states" | while IFS=$'\t' read -r name size props; do
    copy="$prefix-$name-es-de"
    # shellcheck disable=SC2086
    python3 -I "$here/helpcopy.py" "$theme" "$out" "$copy" "$size" $props > /dev/null
    if [ "${DRY:-0}" = 1 ]; then
        echo "THEMES=$out bash run_esde.sh $copy gamelist:1 1280 800"
    else
        THEMES=$out bash "$here/run_esde.sh" "$copy" gamelist:1 1280 800
    fi
done
