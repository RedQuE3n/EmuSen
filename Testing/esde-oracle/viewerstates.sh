#!/bin/bash
# Captures the media viewer under its settings (USERGUIDE.md, "Media viewer settings") on the NES's first game in
# CarAlt at both sizes, scanlines off so that only the setting differs, one run at a time; DRY=1 only prints.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
plain="MediaViewerVideoScanlines=false MediaViewerScreenshotScanlines=false"
while read -r tag view more; do
    for size in "1280 800" "1920 1200"; do
        if [ "${DRY:-0}" = 1 ]; then
            echo "SYSTEM=nes TAG=$tag ESDE_SETTINGS=\"$plain $more\" bash run_esde.sh caralt-es-de $view $size"
        else
            # shellcheck disable=SC2086
            SYSTEM=nes TAG=$tag ESDE_SETTINGS="$plain $more" bash "$here/run_esde.sh" caralt-es-de "$view" $size
        fi
    done
done <<'STATES'
types0 gamelist:0:media:0 MediaViewerShowTypes=true
types4 gamelist:0:media:4 MediaViewerShowTypes=true
bottom gamelist:0:media:0 MediaViewerHelpPrompts=bottom
nohelp gamelist:0:media:4 MediaViewerHelpPrompts=disabled
stretch gamelist:0:media:0 MediaViewerStretchVideos=true
blur gamelist:0:media:0 MediaViewerVideoBlur=true
plain gamelist:0:media:0
STATES
