#!/bin/bash
# One oracle capture: ES-DE 3.4.1 (the AppImage whose sha256 README.md pins) in qt-gates' compositor at W x H, in a
# home of its own under ~/.cache/emusen/esde-oracle, with no network and the synthetic library, walked to one state by
# the pad and grabbed at full size. ES-DE runs inside bwrap: the root read-only, the run's home the only place written,
# its own network and PID namespaces, so stopping it by PID stops all of it. See README.md.
# Usage: run_esde.sh <theme> system|gamelist[:<j>] <W> <H>
#   VARIANT, SCHEME, FONT_SIZE (medium), ASPECT (from W x H when exact: 16:10, 16:9, 4:3), SYSTEM (snes)
#   THEMES   the folder holding <theme> (~/.cache/emusen/bigpicture/corpus/whole)
#   DRY=1    write the library, the home and the settings, print the command, start nothing
# Output: ~/.cache/emusen/esde-oracle/<theme>/<view>-<W>x<H>.png; the run's logs in runs/<theme>-<view>-<W>x<H>/.
set -u
here=$(cd "$(dirname "$0")" && pwd)
[ $# -eq 4 ] || { sed -n '6,10p' "$0"; exit 2; }
theme=$1; view=$2; W=$3; H=$4
appimage=${ESDE_APPIMAGE:-$HOME/.cache/emusen/bigpicture/esde/ES-DE_x64.AppImage}
oracle=$HOME/.cache/emusen/esde-oracle
themes=${THEMES:-$HOME/.cache/emusen/bigpicture/corpus/whole}
pinned=$(sed -n 's/^- sha256: \([0-9a-f]\{64\}\)$/\1/p' "$here/README.md")
[ ${#pinned} -eq 64 ] || { echo "run_esde.sh: README.md pins no sha256"; exit 1; }
case $theme in */*|.*|'') echo "run_esde.sh: a theme is a folder name"; exit 2;; esac
[ -d "$themes/$theme" ] || { echo "run_esde.sh: no theme at $themes/$theme"; exit 1; }
case $W$H in *[!0-9]*|'') echo "run_esde.sh: W and H are numbers"; exit 2;; esac
if [ -z "${ASPECT:-}" ]; then
  if [ $((W * 10)) -eq $((H * 16)) ]; then ASPECT=16:10
  elif [ $((W * 9)) -eq $((H * 16)) ]; then ASPECT=16:9
  elif [ $((W * 3)) -eq $((H * 4)) ]; then ASPECT=4:3
  else echo "run_esde.sh: give ASPECT for $W x $H"; exit 2; fi
fi

if [ -z "${DRY:-}" ]; then
  [ -f "$appimage" ] || { echo "run_esde.sh: no ES-DE binary at $appimage (README.md says where it comes from)"; exit 1; }
  sum=$(sha256sum "$appimage" | cut -d' ' -f1)
  [ "$sum" = "$pinned" ] || { echo "run_esde.sh: $appimage has sha256 $sum, not the pinned $pinned; refused"; exit 1; }
fi

python3 -I "$here/library.py" "$oracle/library" || exit 1
name=${view/:/-}
run=$oracle/runs/$theme-$name-${W}x$H
home=$run/home
rm -rf "$run"
mkdir -p "$home/ES-DE/settings" "$home/ES-DE/themes"
ln -s "$themes/$theme" "$home/ES-DE/themes/$theme"
cp -r "$oracle/library/gamelists" "$home/ES-DE/gamelists"
mapfile -t walk < <(python3 -I "$here/state.py" "${SYSTEM:-snes}" "$view") || exit 1
[ ${#walk[@]} -eq 3 ] || exit 1
read -r -a state_settings <<< "${walk[0]#settings }"
pad=${walk[1]#pad }
seconds=${walk[2]#seconds }
python3 -I "$here/settings.py" "$home/ES-DE/settings/es_settings.xml" "ROMDirectory=$oracle/library/roms/" \
  "MediaDirectory=$oracle/library/downloaded_media/" "Theme=$theme" "ThemeVariant=${VARIANT:-}" \
  "ThemeColorScheme=${SCHEME:-}" "ThemeFontSize=${FONT_SIZE:-medium}" "ThemeAspectRatio=$ASPECT" "${state_settings[@]}" || exit 1

tree=$oracle/bin/$pinned/squashfs-root
hide=()
for d in "$HOME/Documents" "$HOME/ES-DE" "$HOME/.emulationstation" /mnt/nas-roms; do [ -e "$d" ] && hide+=(--tmpfs "$d"); done
sandbox=(bwrap --ro-bind / / --dev-bind /dev /dev --proc /proc --tmpfs /tmp --ro-bind /tmp/.X11-unix /tmp/.X11-unix
  --bind "$home" "$home" "${hide[@]}" --unshare-net --unshare-pid --die-with-parent
  --setenv HOME "$home" --setenv XDG_CACHE_HOME "$home/.cache" --setenv XDG_CONFIG_HOME "$home/.config"
  --setenv XDG_DATA_HOME "$home/.local/share" --setenv TMPDIR /tmp --setenv SDL_AUDIODRIVER dummy
  --setenv SDL_VIDEODRIVER x11 --setenv SDL_JOYSTICK_DISABLE_UDEV 1
  --setenv SDL_GAMECONTROLLER_IGNORE_DEVICES_EXCEPT 0x1209/0x0001 --)
command=("${sandbox[@]}" "$tree/AppRun" --home "$home" --resolution "$W" "$H" --fullscreen-padding off --no-update-check
  --no-splash)
if [ -n "${DRY:-}" ]; then
  echo "settings: $home/ES-DE/settings/es_settings.xml"
  echo "pad: $pad"
  echo "command: WIDTH=$W HEIGHT=$H FULL=1 INTERVAL=1000 PAD=\"$pad\" bash $here/../qt-gates/run.sh $run $seconds $(printf '%q ' "${command[@]}")"
  exit 0
fi

links=$("${sandbox[@]}" /bin/cat /proc/net/dev | awk -F: 'NR > 2 && $1 !~ /^ *lo$/ {print $1}' | tr -d ' \n')
[ -z "$links" ] || { echo "run_esde.sh: the sandbox reaches the network through $links; refused"; exit 1; }
if [ ! -x "$tree/AppRun" ]; then
  rm -rf "$oracle/bin/$pinned.part"; mkdir -p "$oracle/bin/$pinned.part"
  (cd "$oracle/bin/$pinned.part" && "$appimage" --appimage-extract > /dev/null) || { echo "run_esde.sh: extraction failed"; exit 1; }
  mv "$oracle/bin/$pinned.part" "$oracle/bin/$pinned"
fi

WIDTH=$W HEIGHT=$H FULL=1 INTERVAL=1000 PAD=$pad bash "$here/../qt-gates/run.sh" "$run" "$seconds" "${command[@]}"
cp "$home/ES-DE/logs/es_log.txt" "$run/es_log.txt" 2>/dev/null
status=0
python3 -I "$here/pick.py" "$run" "$seconds" "$oracle/$theme/$name-${W}x$H.png" || status=1
rm -rf "$run/grabs"
left=$(ps -eo pid=,args= | grep -a "[e]sde-oracle/bin/" | awk '{print $1}' | tr '\n' ' ')
if [ -n "$left" ]; then echo "run_esde.sh: ES-DE still running: $left"; status=1; else echo "ES-DE closed"; fi
grep -h "pads left" "$run/kwin.log"
if ! grep -aq 'Added controller.*"gate pad"' "$run/es_log.txt"; then echo "run_esde.sh: ES-DE never saw the pad"; status=1; fi
grep -aiE "updat|http|scrap|network|curl" "$run/es_log.txt" | sed 's/^/es_log: /' | head -5
exit $status
