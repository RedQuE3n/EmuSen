#!/bin/bash
# On the handheld: a command in a box of its own, so a test or gate leaves Game Mode, Steam and the player's files as they
# were. The root is read-only; /home is empty but for the scratch folder and the desktop checkout's path, bound to the
# unpacked tree (the tests name their files by the path they were built at); /dev holds the GPU's nodes alone, so no
# pad, hidraw or input device on the handheld is seen; the runtime folder, /tmp and the D-Bus session are the box's own.
# Nothing in it reaches gamescope, Steam or the session's bus. The bundled Qt's libraries are named in QT_BUNDLE_LIBS, for
# the programs built against it alone; shim/kwin_wayland keeps them from the handheld's own KWin. The bus starts no
# services (session.conf). It refuses to start a command if any input device is visible in it. See README.md.
# Usage: box.sh <scratch> <command> [<arg>...]     <scratch> holds root, tree/ (from pack.sh), home/ and tmp/
set -eu
scratch=$(cd "$1" && pwd); shift
root=$(cat "$scratch/root")
runtime=/run/user/$(id -u)
mkdir -p "$scratch/home/.cache" "$scratch/home/.config" "$scratch/tmp"
exec bwrap --die-with-parent --ro-bind / / --tmpfs /home --bind "$scratch" "$scratch" --bind "$scratch/tree" "$root" \
  --dev /dev --dev-bind /dev/dri /dev/dri --tmpfs /dev/shm --tmpfs /tmp --perms 0700 --tmpfs "$runtime" \
  --unsetenv DISPLAY --unsetenv WAYLAND_DISPLAY --unsetenv DBUS_SESSION_BUS_ADDRESS --unsetenv XAUTHORITY \
  --setenv HOME "$scratch/home" --setenv XDG_RUNTIME_DIR "$runtime" --setenv TMPDIR "$scratch/tmp" \
  --setenv XDG_CACHE_HOME "$scratch/home/.cache" --setenv XDG_CONFIG_HOME "$scratch/home/.config" \
  --setenv LANG C.UTF-8 --setenv SDL_AUDIODRIVER dummy --setenv BOXED 1 --setenv QT_BUNDLE_LIBS "$root/qt/lib" \
  --setenv PATH "$root/Testing/handheld/shim:$PATH" \
  -- /bin/bash -c 'ls -A /dev/input /dev/uinput /dev/hidraw* >/dev/null 2>&1 && { echo "box.sh: an input device is visible in the box" >&2; exit 3; }
    exec dbus-run-session --config-file="$0" -- "$@"' "$root/Testing/handheld/session.conf" "$@"
