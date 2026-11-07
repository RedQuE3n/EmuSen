#!/usr/bin/env python3
"""Compares a game's anchor on Sapphire with the referee's (Sapphire_Native.md §39): pictures, the game's RAM, and the
change between two states for rewind's reach. Outputs are counts only, never a memory's contents.

    ps2anchor.py pic REFERENCE.png FRAME.ppm... [--box X0,Y0,X1,Y1]
    ps2anchor.py ram SAPPHIRE.bin REFEREE.ram [PROGRAM_END]
    ps2anchor.py delta STATE.sph2 STATE.sph2...

`pic` scales each of Sapphire's frames (examples/s28.rs's PPM pictures) bilinearly to the referee's picture (a PNG
from a state PCSX2 saved), and counts the pixels whose largest channel differs by more than 8, 32 and 64, inside the box
when one is given (in the referee's coordinates); the frame that differs least is named, for screens that move.
`ram` compares the game's memory from 00100000h, Sapphire's `ram` dump against `ps2referee.py game`'s, by words and by
4 KiB pages, then megabyte by megabyte with each side's zero pages, and over the game's program up to PROGRAM_END
(hexadecimal) where given; the kernel's memory below 00100000h is in neither file. `delta` counts the runs of 64 bytes
that differ between consecutive states, the measure Sapphire_Native.md §38.7 prices the rewind chain's deltas by.
"""
import sys

from PIL import Image, ImageChops

BASE = 0x0010_0000


def pic(reference, frames, box=None):
    ref = Image.open(reference).convert("RGB")
    best = None
    for f in frames:
        d = ImageChops.difference(Image.open(f).convert("RGB").resize(ref.size, Image.BILINEAR), ref)
        if box:
            d = d.crop(box)
        m = [max(p) for p in d.get_flattened_data()] if hasattr(d, "get_flattened_data") else [max(p) for p in d.getdata()]
        counts = [sum(1 for v in m if v > t) for t in (8, 32, 64)]
        print(f"{f}: {len(m)} pixels, {counts[0]} differ by more than 8, {counts[1]} by more than 32, {counts[2]} by more than 64")
        if best is None or counts[1] < best[1][1]:
            best = (f, counts)
    print(f"least different: {best[0]} ({best[1][1]} by more than 32, {best[1][2]} by more than 64)")


def ram(sapphire, referee, program_end=None):
    x, y = open(sapphire, "rb").read(), open(referee, "rb").read()
    n = min(len(x), len(y))
    words = sum(1 for i in range(0, n, 4) if x[i:i + 4] == y[i:i + 4])
    pages = sum(1 for i in range(0, n, 4096) if x[i:i + 4096] == y[i:i + 4096])
    print(f"{n} bytes from {BASE:08X}h: words equal {words} of {n // 4} ({100 * words / (n // 4):.2f} %), 4 KiB pages equal {pages} of {n // 4096}")
    zero = bytes(4096)
    for mb in range(0, n, 1 << 20):
        a, b = x[mb:mb + (1 << 20)], y[mb:mb + (1 << 20)]
        eq = sum(1 for i in range(0, len(a), 4096) if a[i:i + 4096] == b[i:i + 4096])
        za = sum(1 for i in range(0, len(a), 4096) if a[i:i + 4096] == zero)
        zb = sum(1 for i in range(0, len(b), 4096) if b[i:i + 4096] == zero)
        print(f"  {BASE + mb:08X}h: pages equal {eq:3d} of 256; zero pages Sapphire {za:3d}, referee {zb:3d}")
    if program_end:
        m = int(program_end, 16) - BASE
        e = sum(1 for i in range(0, m, 4) if x[i:i + 4] == y[i:i + 4])
        print(f"the program's segment {BASE:08X}h-{program_end}h: words equal {e} of {m // 4}")


def delta(states):
    for a, b in zip(states, states[1:]):
        x, y = open(a, "rb").read(), open(b, "rb").read()
        n = min(len(x), len(y))
        runs = sum(1 for i in range(0, n, 64) if x[i:i + 64] != y[i:i + 64])
        print(f"{a} -> {b}: {len(x)} bytes, {runs} runs of 64 bytes differ ({100 * runs * 64 / n:.2f} % of the state)")


def main(argv):
    if len(argv) < 2 or argv[0] not in ("pic", "ram", "delta"):
        sys.exit(__doc__)
    box = None
    if "--box" in argv:
        i = argv.index("--box")
        box = tuple(int(v) for v in argv[i + 1].split(","))
        del argv[i:i + 2]
    if argv[0] == "pic":
        pic(argv[1], argv[2:], box)
    elif argv[0] == "ram":
        ram(*argv[1:4])
    else:
        delta(argv[1:])


if __name__ == "__main__":
    main(sys.argv[1:])
