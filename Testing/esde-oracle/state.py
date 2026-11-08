#!/usr/bin/python3 -I
"""One oracle state as what reaches it: the settings that start ES-DE in the system's gamelist, the steps of qt-gates'
pad (pad.py) from there, and the run's length. The system is anchored by name (StartupSystem) and never walked to,
because a carousel's axis and the systems' order are the theme's and ES-DE's; the pad then walks the rest: "b" back to
the system view (which proves the pad reached ES-DE), or "down" j times to gamelist entry j as ES-DE lists it.
See README.md.

Usage: state.py <system> system|gamelist[:<j>]
Prints three lines: "settings Name=value...", "pad <step>...", "seconds <n>".
"""
import sys

STARTUP_S, STEP_S, SETTLE_S = 4.0, 0.4, 4.0


def state(system, view):
    name, _, entry = view.partition(":")
    if name == "system" and not entry:
        steps = ["tap:b"]
    elif name == "gamelist":
        j = int(entry or 0)
        if j < 0:
            raise SystemExit("state.py: the entry is counted from 0")
        steps = [f"tap:down wait:{STEP_S}"] * j
    else:
        raise SystemExit(f"state.py: not a view: {view}")
    pad = f"wait:{STARTUP_S} " + " ".join(steps) if steps else f"wait:{STARTUP_S}"
    seconds = int(1 + STARTUP_S + len(steps) * (STEP_S + 0.1) + SETTLE_S + 0.999)
    return [f"settings StartupSystem={system} StartupView=gamelist", f"pad {pad.strip()}", f"seconds {seconds}"]


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    print("\n".join(state(sys.argv[1], sys.argv[2])))
