#!/bin/bash
# The ES-DE captures of the watched states: each row of states.tsv, its system view at the SNES and its gamelist at
# entry 1, at 1280 x 800 and 1920 x 1200, through ../esde-oracle/run_esde.sh, one run at a time. A capture already
# taken is kept unless FRESH=1. See README.md.
# Usage: capture.sh [theme...]   (every theme of states.tsv when none is named)
set -u
here=$(cd "$(dirname "$0")" && pwd)
oracle=$HOME/.cache/emusen/esde-oracle
status=0
while IFS=$'\t' read -r theme variant scheme font; do
  [ "$theme" = theme ] && continue
  if [ $# -gt 0 ] && [[ " $* " != *" $theme "* ]]; then continue; fi
  [ "$variant" = - ] && variant=
  [ "$scheme" = - ] && scheme=
  for size in "1280 800" "1920 1200"; do
    read -r W H <<< "$size"
    for view in system gamelist:1; do
      out=$oracle/$theme/${view/:/-}-${W}x$H.png
      if [ -f "$out" ] && [ -z "${FRESH:-}" ]; then echo "kept $out"; continue; fi
      VARIANT=$variant SCHEME=$scheme FONT_SIZE=$font bash "$here/../esde-oracle/run_esde.sh" "$theme" "$view" "$W" "$H" || status=1
    done
  done
done < "$here/states.tsv"
exit $status
