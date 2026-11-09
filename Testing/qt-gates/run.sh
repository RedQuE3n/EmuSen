#!/bin/bash
# One gate run in a compositor of its own: KWin's virtual one at 1920 x 1200 (WIDTH, HEIGHT) with Xwayland, on a D-Bus
# of its own, every window made fullscreen by a rule; the program started with polled grabs (grab.py) and, when PAD
# names steps, the gate pad: pad.py writing presses down a FIFO into emusen-qt's own SDL virtual pad (its gate-pad feature),
# so no input device exists outside the program. Everything started is stopped when the run ends; the whole run is bounded
# by a watchdog.
# With DESK, the session's own compositor instead (desk.py): the program fullscreen on that output, no grabs, and the
# pad started only once the program holds the focus.
# Usage: run.sh <run-dir> <seconds> <program> [<arg>...]
#   PAD="wait:4 tap:start ..."  the pad's steps; none, no pad. The pad is made inside the program as it starts.
#   RSS=<comm>                  the program's memory sampled every 0.5 s to rss.tsv (rss.py; <comm> its process name)
#   BOX="r0-r1,g0-g1,b0-b1"     the game's colour box grab.py measures; INTERVAL grab interval in ms (35)
#   WIDTH, HEIGHT               the virtual compositor's size (1920, 1200); FULL=1 keeps each grab at full size
#   XWAYLAND=0                  no Xwayland in the virtual compositor (emusen-qt is a Wayland client; the handheld runs so)
#   DESK=<output>               the session's compositor, fullscreen on that output; MATCH the program's window class
#                               (a regex, "emusen"); the session's WAYLAND_DISPLAY, DISPLAY and XDG_RUNTIME_DIR as
#                               DESK_WAYLAND (wayland-0), DESK_DISPLAY (:0), DESK_RUNTIME (/run/user/<uid>)
# Output in <run-dir>: start (the program's exec, epoch s), grabs.tsv, grabs/, program.log, pad.log, kwin.log; with
# DESK, desk.log (KWin's windows and activations) and active (when the program first held the focus) in place of grabs.
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
if [ -z "\${DESK:-}" ]; then
  python3 -I "$here/grab.py" "$run/grabs" $((seconds + 2)) \${INTERVAL:-35} "\${BOX:-0-255,0-255,0-255}" > "$run/grabs.tsv" 2> "$run/grab.err" &
  grabber=\$!
fi
if [ -n "\${PAD:-}" ]; then rm -f "$run/pad.fifo"; mkfifo "$run/pad.fifo"; export EMUSEN_GATE_PAD="$run/pad.fifo"; fi
date +%s.%N > "$run/start"
end=\$((\$(date +%s%N) + $seconds * 1000000000))
$(cat "$run/command") > "$run/program.log" 2>&1 &
program=\$!
if [ -n "\${RSS:-}" ]; then
  python3 -I "$here/rss.py" "$run/rss.tsv" \$program \${RSS} &
  rsser=\$!
fi
if [ -n "\${PAD:-}" ]; then
  if [ -n "\${DESK:-}" ]; then for i in \$(seq 150); do [ -e "$run/active" ] && break; sleep 0.1; done; fi
  if [ -z "\${DESK:-}" ] || [ -e "$run/active" ]; then
    python3 -I "$here/pad.py" "$run/pad.fifo" \$PAD > "$run/pad.log" 2>&1 & pad=\$!
  else
    echo "no pad: the program never held the focus" > "$run/pad.log"
  fi
fi
left=\$(( (end - \$(date +%s%N)) / 1000000 ))
[ \$left -gt 0 ] && sleep \$((left / 1000)).\$(printf %03d \$((left % 1000)))
kill \$program 2>/dev/null; sleep 1; kill -9 \$program 2>/dev/null
[ -n "\${pad:-}" ] && kill \$pad 2>/dev/null && wait \$pad
[ -n "\${rsser:-}" ] && kill \$rsser 2>/dev/null
[ -n "\${grabber:-}" ] && wait \$grabber
true
EOF
if [ -n "${DESK:-}" ]; then
  export WAYLAND_DISPLAY=${DESK_WAYLAND:-wayland-0} DISPLAY=${DESK_DISPLAY:-:0} XDG_RUNTIME_DIR=${DESK_RUNTIME:-/run/user/$(id -u)}
  export DBUS_SESSION_BUS_ADDRESS=unix:path=$XDG_RUNTIME_DIR/bus
  rm -f "$run/active" "$run/desk-ready"
  python3 -I "$here/desk.py" "$run" "$DESK" "${MATCH:-emusen}" $((seconds + 30)) > "$run/desk.err" 2>&1 &
  desk=$!
  for i in $(seq 100); do [ -e "$run/desk-ready" ] && break; sleep 0.1; done
  if [ -e "$run/desk-ready" ]; then
    GATE_RUN="$run" timeout -k 5 $((seconds + 20)) /bin/bash "$run/launch.sh" > "$run/kwin.log" 2>&1
    echo "run ended: $?" >> "$run/kwin.log"
  else
    echo "run not started: KWin did not take the script (desk.err)" >> "$run/kwin.log"
  fi
  kill $desk 2>/dev/null; wait $desk
else
  socket=gate-$$
  GATE_RUN="$run" XDG_CONFIG_HOME="$run/kwin-config" KWIN_SCREENSHOT_NO_PERMISSION_CHECKS=1 KWIN_COMPOSE=O2 \
    timeout -k 5 $((seconds + 40)) env -u DISPLAY -u WAYLAND_DISPLAY dbus-run-session -- \
    kwin_wayland --virtual $([ "${XWAYLAND:-1}" = 0 ] || echo --xwayland) --no-lockscreen --no-global-shortcuts --no-kactivities --socket "$socket" \
    --width ${WIDTH:-1920} --height ${HEIGHT:-1200} --exit-with-session "/bin/bash $run/launch.sh" > "$run/kwin.log" 2>&1
  echo "run ended: $?" >> "$run/kwin.log"
fi
# Services the session activated (the desktop portal, ksecretd) are started by systemd and outlive it; each carries
# GATE_RUN, so this run's are stopped.
for p in $(pgrep -u "$USER" .); do grep -qaz "^GATE_RUN=$run$" /proc/$p/environ 2>/dev/null && kill $p && echo "stopped: $p $(cat /proc/$p/comm 2>/dev/null)" >> "$run/kwin.log"; done
# The pad lives inside the program; an input device of that name anywhere on the machine would be a fault.
grep -c "gate pad" /proc/bus/input/devices | sed 's/^/input devices named gate pad: /' >> "$run/kwin.log"
