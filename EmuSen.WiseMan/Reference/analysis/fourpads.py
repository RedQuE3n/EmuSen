#!/usr/bin/env python3
"""A game's own pad variables with an adapter, in a reference and in Nephrite (Nephrite_Native.md §45).

For each pad the reference is run twice, the pad holding a button and not; the RAM bytes that differ are where the
game keeps what it read of that pad. Nephrite is run the same two ways, and its bytes at those addresses are compared
with the reference's. A pad whose press changes nothing in the reference is one the game does not read at that frame.

Each pad is one of: `exact`, every byte the press moves in the reference holding the reference's two values in
Nephrite; `read`, some of them do and the rest are the game's course, which the two run a frame or so apart; `unread`,
the press moves nothing in the reference; `MISSING`, the press moves the reference and nothing of it is in Nephrite.

  fourpads.py --probe PROBE --core REFERENCE.so --dump NEPHRITE_DUMP --adapter teamplayer|4way [--six] [--pads 4]
              [--frame 600] [--from 300] GAME...

The reference is never read, only run.
"""
import argparse, os, subprocess, sys, tempfile

BUTTONS = [("Up", 0), ("Down", 1), ("Left", 2), ("Right", 3)]
REFERENCE = {("teamplayer", False): "MD Joypad 3 Button + Teamplayer", ("teamplayer", True): "MD Joypad 6 Button + Teamplayer",
             ("4way", False): "MD Joypad 3 Button + 4-WayPlay", ("4way", True): "MD Joypad 6 Button + 4-WayPlay",
             ("none", False): "MD Joypad 3 Button", ("none", True): "MD Joypad 6 Button"}
NEPHRITE = {("teamplayer", False): "md.teamplayer3", ("teamplayer", True): "md.teamplayer6", ("4way", False): "md.4way3",
            ("4way", True): "md.4way6", ("none", False): "md.pad3", ("none", True): "md.pad6"}


def reference_ram(a, game, out, pad):
    cmd = [a.probe, game, out, str(a.frame), str(a.frame), "--backend", "libretro", "--core", a.core, "--system", "megadrive",
           "--device", "1=" + REFERENCE[(a.adapter, a.six)]]
    # The reference's 4 Way Play is chosen on both of its ports; the Team Player on one, a pad on the other.
    cmd += ["--device", "2=" + REFERENCE[(a.adapter if a.adapter == "4way" else "none", a.six)]]
    if pad is not None:
        cmd += ["--presson", f"{pad + 1}:{a.start}:{BUTTONS[pad % 4][0]}:{a.frame}"]
    subprocess.run(cmd, check=True, capture_output=True)
    name = [f for f in os.listdir(out) if "_ram_f" in f][0]
    words = open(os.path.join(out, name), "rb").read()
    # The reference keeps the 68000's RAM as host words; put back in the 68000's byte order.
    return bytes(words[i ^ 1] for i in range(len(words)))


def nephrite_ram(a, game, out, pad):
    env = dict(os.environ, NEPHRITE_PLUGS=NEPHRITE[(a.adapter, a.six)] + "," + NEPHRITE[("none", a.six)])
    if pad is not None:
        env["NEPHRITE_PRESS"] = f"{a.start}:{1 << BUTTONS[pad % 4][1]}:{a.frame}:{pad + 1}"
    subprocess.run([a.dump, game, out, str(a.frame)], check=True, capture_output=True, env=env)
    return open(os.path.join(out, f"nephrite_wram_f{a.frame:05d}.bin"), "rb").read()


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--probe", required=True); p.add_argument("--core", required=True); p.add_argument("--dump", required=True)
    p.add_argument("--adapter", choices=["teamplayer", "4way", "none"], required=True); p.add_argument("--six", action="store_true")
    p.add_argument("--pads", type=int, default=4); p.add_argument("--frame", type=int, default=600)
    p.add_argument("--from", dest="start", type=int, default=300); p.add_argument("--verbose", action="store_true"); p.add_argument("games", nargs="+")
    a = p.parse_args()
    verified = 0
    counts = {}
    for game in a.games:
        with tempfile.TemporaryDirectory() as tmp:
            run = lambda f, pad, tag: f(a, game, os.path.join(tmp, tag), pad)
            ref0, own0 = run(reference_ram, None, "r"), run(nephrite_ram, None, "n")
            line, classes = [], []
            for pad in range(a.pads):
                ref1, own1 = run(reference_ram, pad, f"r{pad}"), run(nephrite_ram, pad, f"n{pad}")
                moved = [i for i in range(len(ref0)) if ref0[i] != ref1[i]]
                same = [i for i in moved if own0[i] == ref0[i] and own1[i] == ref1[i]]
                kind = "unread" if not moved else "exact" if len(same) == len(moved) else "read" if same else "MISSING"
                classes.append(kind)
                line.append(f"pad {pad + 1} {kind}: {len(moved)} bytes move in the reference, {len(same)} of them the same in Nephrite")
                if len(moved) <= 4:
                    line.append("       " + ", ".join(f"${0xFF0000 + i:06X} {ref0[i]:02X} to {ref1[i]:02X}" for i in moved))
            good = all(c in ("exact", "read") for c in classes)
            verdict = "all" if good else "MISSING" if "MISSING" in classes else "unread" if all(c == "unread" for c in classes) else "some"
            counts[verdict] = counts.get(verdict, 0) + 1
            verified += good
            print(f"{verdict:8}" + os.path.basename(game) + "  [" + " ".join(classes) + "]")
            if a.verbose:
                for l in line:
                    print("     " + l)
    print(f"{verified} of {len(a.games)} games read every pad as the reference does; {counts}")
    return 0 if "MISSING" not in counts else 1


if __name__ == "__main__":
    sys.exit(main())
