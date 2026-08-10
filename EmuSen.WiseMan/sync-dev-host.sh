#!/usr/bin/env bash
# Pulls the machine-local inputs a second development host needs but git cannot
# carry - see EmuSen_Debugging_Tools_Reference_v5.md §3.54.
#
#   ./sync-dev-host.sh <user@host> [--roms] [--dry-run] [--repo <remote path>]
#
# Two trees, both gitignored and both derived from commercial game data, which is
# why they are synced between your own machines rather than committed:
#   Reference/dumps/   every ground-truth dump the probe has taken
#   the ROM library    only with --roms, into whatever RomDirectory says here
#
# Pull-only by construction. There is no push mode and no --delete on any rsync
# in this file, on either tree: the remote is treated as authoritative and this
# script can only ever add files to the machine it runs on. That is the whole
# safety argument, and it is why the direction is not a flag.
#
# appsettings.json is deliberately not synced. Its three directory settings are
# absolute paths belonging to the host that wrote them - a Fedora box's
# /home/red/Documents/Roms/ is not a path a Mac or a Windows machine has - so
# copying it would point a fresh install at nothing. The script prints what to
# set instead.
set -euo pipefail

REMOTE=""
REMOTE_REPO=""
WANT_ROMS=0
DRY=""

while [ $# -gt 0 ]; do
    case "$1" in
    --roms)    WANT_ROMS=1; shift ;;
    --dry-run) DRY="-n"; shift ;;
    --repo)    REMOTE_REPO="${2:?--repo needs a path}"; shift 2 ;;
    -*)        echo "unknown flag $1" >&2; exit 1 ;;
    *)         REMOTE="$1"; shift ;;
    esac
done

[ -n "$REMOTE" ] || { echo "usage: sync-dev-host.sh <user@host> [--roms] [--dry-run] [--repo <path>]" >&2; exit 1; }

# The only place a host is named - the same rule build-probe.sh follows. The one
# thing that differs is where .NET's SpecialFolder.ApplicationData points, which
# is ~/.config on Unix and %APPDATA% on Windows.
case "$(uname -s)" in
Linux|Darwin)         HOST=unix ;;
MINGW*|MSYS*|CYGWIN*) HOST=windows ;;
*) echo "sync-dev-host.sh knows Linux, macOS and Windows, not $(uname -s)" >&2; exit 1 ;;
esac

need() {
    for tool in "$@"; do
        command -v "$tool" >/dev/null || {
            echo "sync-dev-host.sh needs '$tool'." >&2
            [ "$HOST" = windows ] && echo "Git Bash does not ship it; MSYS2 has it (pacman -S rsync openssh)." >&2
            exit 1
        }
    done
}
need ssh rsync python3

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOCAL_REPO="$(cd "$HERE/.." && pwd)"
# Two Linux boxes share a layout; a Mac or a Windows box does not, hence --repo.
REMOTE_REPO="${REMOTE_REPO:-${EMUSEN_REMOTE_REPO:-$LOCAL_REPO}}"

legacy_config_dir() {
    case "$HOST" in
    windows) cygpath -u "${APPDATA:?APPDATA is not set}" ;;
    unix)    echo "${XDG_CONFIG_HOME:-$HOME/.config}" ;;
    esac
}

# Mirrors ConfigStore's own order - Directory, then PreviousDirectory, then
# LegacyDirectory - so this reads the file the app would actually read rather
# than the one that happens to exist. See EmuSen_Config_Reference.md §1.
appsettings_path() {
    local repo="$1"
    for candidate in \
        "$repo/home/etc/EmuSen/appsettings.json" \
        "$repo/etc/EmuSen/appsettings.json" \
        "$(legacy_config_dir)/EmuSen/appsettings.json"
    do
        [ -f "$candidate" ] && { echo "$candidate"; return 0; }
    done
    return 1
}

rom_directory() {
    python3 -c '
import json, sys
try:
    print(json.load(sys.stdin).get("RomDirectory", "").rstrip("/\\"))
except Exception:
    print("")
'
}

echo "== $REMOTE:$REMOTE_REPO -> $LOCAL_REPO"
ssh "$REMOTE" "test -d '$REMOTE_REPO/EmuSen.WiseMan/Reference'" || {
    echo "No EmuSen checkout at '$REMOTE_REPO' on $REMOTE - pass --repo <path>" >&2
    exit 1
}

# -s so the remote shell never expands the space in "EmuSen Project".
echo "== dumps"
mkdir -p "$LOCAL_REPO/EmuSen.WiseMan/Reference/dumps"
rsync -a -s $DRY --info=stats1 \
    "$REMOTE:$REMOTE_REPO/EmuSen.WiseMan/Reference/dumps/" \
    "$LOCAL_REPO/EmuSen.WiseMan/Reference/dumps/"

if [ "$WANT_ROMS" -eq 1 ]; then
    LOCAL_SETTINGS="$(appsettings_path "$LOCAL_REPO" || true)"
    REMOTE_ROMS="$(ssh "$REMOTE" "cat '$REMOTE_REPO/home/etc/EmuSen/appsettings.json' 2>/dev/null \
        || cat '$REMOTE_REPO/etc/EmuSen/appsettings.json' 2>/dev/null" | rom_directory)"
    LOCAL_ROMS=""
    [ -n "$LOCAL_SETTINGS" ] && LOCAL_ROMS="$(rom_directory < "$LOCAL_SETTINGS")"

    [ -n "$LOCAL_ROMS" ] || {
        echo "This machine has no RomDirectory set." >&2
        echo "Put one in ${LOCAL_SETTINGS:-$LOCAL_REPO/home/etc/EmuSen/appsettings.json}" >&2
        echo "The remote's is '$REMOTE_ROMS' - use a path that exists here, not that one." >&2
        exit 1
    }
    [ -n "$REMOTE_ROMS" ] || { echo "$REMOTE has no RomDirectory set" >&2; exit 1; }

    echo "== roms  $REMOTE_ROMS -> $LOCAL_ROMS  (additive; nothing is ever removed)"
    mkdir -p "$LOCAL_ROMS"
    rsync -a -s $DRY --info=stats1 "$REMOTE:$REMOTE_ROMS/" "$LOCAL_ROMS/"
fi

echo
echo "Done. Dumps discover themselves off the filesystem, so the differential"
echo "tests pick them up with no configuration:"
echo "  dotnet test EmuSen.WiseMan --filter ReferenceDumpTests"
[ "$WANT_ROMS" -eq 1 ] || echo "Pass --roms to bring the library across as well."
