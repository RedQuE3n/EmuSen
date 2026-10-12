#!/usr/bin/env python3
"""Drives a disc by a press script on Diamond's run tool (open kernel) or on SwanStation through psxreferee.py, with a
picture every K frames, the card in and out and the sound, so that a game's play is compared by the screen and the
sound alone (Diamond_Players.md §13).

    psxplay.py diamond|swanstation DISC SCRIPT OUTDIR --anchor T [--frames N] [--every K] [--card FILE]
                [--wav FILE] [--run BIN] [--bios FILE] [--extra "ARGS"]

SCRIPT holds one press a line, `+N BUTTONS DUR` for N frames after the anchor T (the frame the game's title shows on
that emulator, which differs by the referee's BIOS intro) or `@N BUTTONS DUR` at an absolute frame; BUTTONS are joined
by `+` from Up Down Left Right Cross Circle Square Triangle L1 R1 L2 R2 Start Select, or `stick:AXIS:VALUE` (AXIS 0-1
the left stick's x and y, 2-3 the right's, VALUE -1 to 1); `#` starts a comment. `--card FILE` is the first card,
written back as the run leaves it. Diamond's run tool is `examples/run` of Diamond's crate, built in release; SwanStation
is run on the player's BIOS named by `--bios` (passed by path, never read). L2 and R2 reach Diamond only.
"""
import os
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REFEREE = HERE / "psxreferee.py"
RUN = HERE.parents[2] / "EmuSen/Cores/Sony/Diamond - PlayStation/target/release/examples/run"
BITS = {"Select": 0x1, "L3": 0x2, "R3": 0x4, "Start": 0x8, "Up": 0x10, "Right": 0x20, "Down": 0x40, "Left": 0x80, "L2": 0x100,
        "R2": 0x200, "L1": 0x400, "R1": 0x800, "Triangle": 0x1000, "Circle": 0x2000, "Cross": 0x4000, "Square": 0x8000}
RETRO = {"Select": "Select", "Start": "Start", "Up": "Up", "Right": "Right", "Down": "Down", "Left": "Left", "L1": "L", "R1": "R",
         "Triangle": "X", "Circle": "A", "Cross": "B", "Square": "Y"}


def presses(script, anchor):
    out = []
    for line in Path(script).read_text().splitlines():
        line = line.split("#", 1)[0].strip()
        if not line:
            continue
        at, buttons, duration = line.split()
        frame = anchor + int(at[1:]) if at[0] == "+" else int(at[1:])
        out.append((frame, buttons.split("+"), int(duration)))
    return out


def main(argv):
    if len(argv) < 4 or argv[0] not in ("diamond", "swanstation"):
        sys.exit(__doc__)
    who, disc, script, outdir = argv[:4]
    rest = argv[4:]

    def option(name, default=None):
        if name in rest:
            i = rest.index(name)
            value = rest[i + 1]
            del rest[i:i + 2]
            return value
        return default

    anchor = int(option("--anchor", "0"))
    frames, every = option("--frames", "18000"), option("--every", "60")
    card, wav = option("--card"), option("--wav")
    extra = (option("--extra") or "").split()
    Path(outdir).mkdir(parents=True, exist_ok=True)
    if who == "diamond":
        cmd = [option("--run", str(RUN)), disc, frames, "--pictures", outdir, every]
        for f, buttons, n in presses(script, anchor):
            mask = 0
            for b in buttons:
                if b.startswith("stick:"):
                    _, axis, value = b.split(":")
                    cmd += ["--stick", f"{f}:{axis}:{value}:{n}"]
                else:
                    mask |= BITS[b]
            if mask:
                cmd += ["--press", f"{f}:{mask:X}:{n}"]
        cmd += (["--card", card] if card else []) + (["--wav", wav] if wav else [])
    else:
        bios = option("--bios") or sys.exit("--bios names the player's own BIOS file")
        cmd = [sys.executable, str(REFEREE), "pictures", "swanstation", disc, "--bios", bios, "--for", frames, "--every", every]
        for f, buttons, n in presses(script, anchor):
            for b in buttons:
                if b.startswith("stick:"):
                    _, axis, value = b.split(":")
                    cmd += ["--stick", f"{f}:{axis}:{value}:{n}"]
                elif b in RETRO:
                    cmd += ["--press", f"{f}:{RETRO[b]}:{n}"]
        cmd += (["--card-in", card, "--card-out", card] if card else []) + (["--wav", wav] if wav else [])
    cmd += extra + ([outdir] if who == "swanstation" else [])
    with open(Path(outdir) / "run.log", "w") as log:
        p = subprocess.run(cmd, stdout=log, stderr=subprocess.STDOUT, env=os.environ)
    sys.exit(p.returncode)


if __name__ == "__main__":
    main(sys.argv[1:])
