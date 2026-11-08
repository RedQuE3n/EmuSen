#!/bin/bash
# One gate run in a compositor of its own: KWin's virtual one at 1920 x 1200 with Xwayland, on a D-Bus of its own,
# every window made fullscreen by a rule; the program started with polled grabs (grab.py) and, when PAD names steps, a
# uinput pad (pad.py). Everything started is stopped when the run ends; the whole run is bounded by a watchdog.
# Usage: run.sh <run-dir> <seconds> <program> [<arg>...]
#   PAD="wait:4 tap:start ..."  the pad's steps; none, no pad
#   BOX="r0-r1,g0-g1,b0-b1"     the game's colour box grab.py measures; INTERVAL grab interval in ms (35)
# Output in <run-dir>: start (the program's exec, epoch s), grabs.tsv, grabs/, program.log, pad.log, kwin.log.
# See README.md.
set -u
here=$(cd "$(dirname "$0")" && pwd)
run=$1; seconds=$2; shift 2
mkdir -p "$run/kwin-config"
run=$(cd "$run" && pwd)
cat > "$run/kwin-config/kwinrulesrc" <<'EOF'
[General]
count=1
rules=gate

[gate]
Description=every window fullscreen
fullscreen=true
fullscreenrule=2
EOF
printf '%q ' "$@" > "$run/command"
cat > "$run/launch.sh" <<EOF
#!/bin/bash
export XDG_CONFIG_HOME=${XDG_CONFIG_HOME:-$HOME/.config}
date +%s.%N > "$run/start"
python3 -I "$here/grab.py" "$run/grabs" $((seconds + 2)) \${INTERVAL:-35} "\${BOX:-0-255,0-255,0-255}" > "$run/grabs.tsv" 2> "$run/grab.err" &
grabber=\$!
if [ -n "\${PAD:-}" ]; then python3 -I "$here/pad.py" \$PAD > "$run/pad.log" 2>&1 & pad=\$!; fi
$(cat "$run/command") > "$run/program.log" 2>&1 &
program=\$!
sleep $seconds
kill \$program 2>/dev/null; sleep 1; kill -9 \$program 2>/dev/null
[ -n "\${pad:-}" ] && kill \$pad 2>/dev/null
wait \$grabber
EOF
socket=gate-$$
GATE_RUN="$run" XDG_CONFIG_HOME="$run/kwin-config" KWIN_SCREENSHOT_NO_PERMISSION_CHECKS=1 KWIN_COMPOSE=O2 \
  timeout -k 5 $((seconds + 40)) env -u DISPLAY -u WAYLAND_DISPLAY dbus-run-session -- \
  kwin_wayland --virtual --xwayland --no-lockscreen --no-global-shortcuts --no-kactivities --socket "$socket" \
  --width 1920 --height 1200 --exit-with-session "/bin/bash $run/launch.sh" > "$run/kwin.log" 2>&1
echo "run ended: $?" >> "$run/kwin.log"
# Services the session activated (the desktop portal, ksecretd) are started by systemd and outlive it; each carries
# GATE_RUN, so this run's are stopped.
for p in $(pgrep -u "$USER" .); do grep -qaz "^GATE_RUN=$run$" /proc/$p/environ 2>/dev/null && kill $p && echo "stopped: $p $(cat /proc/$p/comm 2>/dev/null)" >> "$run/kwin.log"; done
grep -c "gate pad" /proc/bus/input/devices | sed 's/^/pads left: /' >> "$run/kwin.log"
