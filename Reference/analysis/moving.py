#!/usr/bin/env python3
"""Whether a game reaches a moving picture, in a reference and in Nephrite (Nephrite_Native.md §46.3).

Each game is run from power-on with no input, and its picture taken every 50 frames from 600 to 900, in the reference
through the probe and in Nephrite through its `dump` example. A picture that is the same at all seven is still; one of
a single colour is blank. A game is `moving` when Nephrite's pictures differ among themselves; `still` when they do not
and the reference's do not either, a title that waits; `STILL` when Nephrite's do not and the reference's do, which is
a game to look at. The last column is how many bytes of the 68000's RAM are the reference's at frame 900.

  moving.py --probe PROBE --core REFERENCE.so --dump NEPHRITE_DUMP GAME...

One line a game, tab-separated: verdict, Nephrite's distinct pictures and the colours of its last, the reference's
two, the RAM bytes equal, the name. The reference is never read, only run.
"""
import argparse, hashlib, json, os, subprocess, sys, tempfile

FRAMES = list(range(600, 901, 50))


def colours(pixels, size):
    return len({pixels[i:i + size] for i in range(0, len(pixels), size)})


def reference(a, game, out):
    subprocess.run([a.probe, game, out, str(FRAMES[0]), str(FRAMES[-1]), "50", "--backend", "libretro", "--core", a.core, "--system", "megadrive"],
                   check=True, capture_output=True)
    pictures = []
    for f in FRAMES:
        manifest = json.load(open(os.path.join(out, [n for n in os.listdir(out) if n.endswith(f"manifest_f{f:05d}.json")][0])))
        pictures.append(open(os.path.join(out, manifest["screen"]["file"]), "rb").read())
    size = manifest["screen"]["bytes"] // (manifest["screen"]["width"] * manifest["screen"]["height"])
    words = open(os.path.join(out, [s["file"] for s in manifest["spaces"] if s["name"] == "ram"][0]), "rb").read()
    # The reference keeps the 68000's RAM as host words; put back in the 68000's byte order.
    return pictures, size, bytes(words[i ^ 1] for i in range(len(words)))


def nephrite(a, game, out):
    subprocess.run([a.dump, game, out] + [str(f) for f in FRAMES], check=True, capture_output=True)
    pictures = [open(os.path.join(out, [n for n in os.listdir(out) if n.startswith(f"nephrite_screen_f{f:05d}_")][0]), "rb").read() for f in FRAMES]
    return pictures, 4, open(os.path.join(out, f"nephrite_wram_f{FRAMES[-1]:05d}.bin"), "rb").read()


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--probe", required=True); p.add_argument("--core", required=True); p.add_argument("--dump", required=True)
    p.add_argument("games", nargs="+")
    a = p.parse_args()
    for game in a.games:
        with tempfile.TemporaryDirectory() as tmp:
            try:
                (own, own_size, own_ram), (ref, ref_size, ref_ram) = nephrite(a, game, os.path.join(tmp, "n")), reference(a, game, os.path.join(tmp, "r"))
            except (subprocess.CalledProcessError, IndexError, KeyError) as e:
                print(f"error\t-\t-\t-\t-\t-\t{os.path.basename(game)}\t{type(e).__name__}")
                continue
            distinct = lambda pictures: len({hashlib.md5(x).digest() for x in pictures})
            n, r = distinct(own), distinct(ref)
            verdict = "moving" if n > 1 else "still" if r == 1 else "STILL"
            equal = sum(x == y for x, y in zip(own_ram, ref_ram))
            print(f"{verdict}\t{n}\t{colours(own[-1], own_size)}\t{r}\t{colours(ref[-1], ref_size)}\t{equal}\t{os.path.basename(game)}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
