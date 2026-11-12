#!/usr/bin/python3 -I
"""One oracle state as what reaches it: the settings that start ES-DE in the system's gamelist, the steps of qt-gates'
pad (pad.py) from there, and the run's length. The system is anchored by name (StartupSystem) and never walked to,
because a carousel's axis and the systems' order are the theme's and ES-DE's; the pad then walks the rest: "b" back to
the system view (which proves the pad reached ES-DE), or "down" j times to gamelist entry j as ES-DE lists it, and with
"media:<n>" then "x" (the media viewer) and "right" n times, or with "manual:<n>" then "x", "up" (the game's PDF
manual) and "right" n times (a page each), and after it "zoom:<k>" the right shoulder k times, "zoom-out:<k>" the left
shoulder k times, "pan:<m>" "right" m times, or "pan-y:<m>" "down" m times, in the order written.
See README.md.

Usage: state.py <system> system|gamelist[:<j>[:media:<n>|:manual:<n>[:zoom|zoom-out|pan|pan-y:<k>]...]]
Prints three lines: "settings Name=value...", "pad <step>...", "seconds <n>".
"""
import sys

STARTUP_S, STEP_S, SETTLE_S = 4.0, 0.4, 4.0
# The viewer opens before the first "right" is pressed.
VIEWER_S = 1.0


def state(system, view):
    name, _, rest = view.partition(":")
    entry, _, media = rest.partition(":")
    if name == "system" and not entry:
        steps = ["tap:b"]
    elif name == "gamelist":
        j = int(entry or 0)
        if j < 0:
            raise SystemExit("state.py: the entry is counted from 0")
        steps = [f"tap:down wait:{STEP_S}"] * j
        if media:
            words = media.split(":")
            counts = words[1::2]
            if words[0] not in ("media", "manual") or len(words) % 2 or not all(c.isdigit() for c in counts) \
                    or any(k not in ("zoom", "zoom-out", "pan", "pan-y") for k in words[2::2]) \
                    or (words[0] == "media" and len(words) > 2):
                raise SystemExit(f"state.py: after the entry, media:<n> or manual:<n> with zoom, zoom-out, pan or pan-y steps: {view}")
            n = int(counts[0])
            steps += [f"tap:x wait:{VIEWER_S}"]
            if words[0] == "manual":
                steps += [f"tap:up wait:{VIEWER_S}"]
            steps += [f"tap:right wait:{STEP_S}"] * n
            buttons = {"zoom": "r", "zoom-out": "l", "pan": "right", "pan-y": "down"}
            for operation, count in zip(words[2::2], counts[1:]):
                steps += [f"tap:{buttons[operation]} wait:{STEP_S}"] * int(count)
    else:
        raise SystemExit(f"state.py: not a view: {view}")
    pad = f"wait:{STARTUP_S} " + " ".join(steps) if steps else f"wait:{STARTUP_S}"
    seconds = int(1 + STARTUP_S + len(steps) * (STEP_S + 0.1) + (VIEWER_S * (1 + media.startswith("manual")) if media else 0) + SETTLE_S + 0.999)
    return [f"settings StartupSystem={system} StartupView=gamelist", f"pad {pad.strip()}", f"seconds {seconds}"]


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    print("\n".join(state(sys.argv[1], sys.argv[2])))
