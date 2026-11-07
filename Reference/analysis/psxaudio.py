#!/usr/bin/env python3
"""Compares a PlayStation core's audio with a referee's, sample for sample, scene by scene (Diamond_SPU.md §10).

    psxaudio.py DIAMOND.wav REFEREE.wav [--cut FRAMES --fps FPS] [--quiet N] [--show K] [--gain]

Both are 16-bit stereo at 44,100 Hz, the SPU's own rate, so that, unlike `audio.py`'s feature streams, the samples
themselves compare. The program's scenes each start from exact silence (`programs/spu-voices.s`): a scene is a run of
the first file's samples between silences of at least --quiet samples (2,000 by default), and the second file's
scenes, found the same way, are taken in their order, each moved by the lag within ±8 of its start that leaves the
least difference over its first 4,096 samples; the scenes' lengths are printed, since a scene's tail may differ. For each scene the line gives its start in each file, the samples
equal, the largest difference and the mean absolute one, left and right together. --cut drops the second file's
first FRAMES frames at FPS unread, which is how a referee's recording loses whatever came before the program's
scenes; --show prints the first K differing samples of each scene. --gain compares the first file scaled by the gain
that fits each scene best, for a referee whose output stage scales the mix, and prints the gain.
"""
import struct
import sys


def read_wav(path):
    b = open(path, "rb").read()
    i = 12
    while i + 8 <= len(b):
        cid, n = b[i:i + 4], struct.unpack("<I", b[i + 4:i + 8])[0]
        if cid == b"data":
            d = b[i + 8:i + 8 + n]
            s = struct.unpack("<%dh" % (len(d) // 2), d)
            return list(zip(s[0::2], s[1::2]))
        i += 8 + n + (n & 1)
    raise SystemExit(f"{path}: no data chunk")


def scenes(frames, quiet):
    """(start, end) of each run between silences of `quiet` samples or more."""
    out, start, last = [], None, None
    for t, (l, r) in enumerate(frames):
        if l or r:
            if start is None:
                start = t
            elif t - last > quiet:
                out.append((start, last + 1))
                start = t
            last = t
    if start is not None:
        out.append((start, last + 1))
    return out


def cost(a, b, a0, b0, n):
    s = 0
    for k in range(n):
        x, y = a[a0 + k], b[b0 + k] if 0 <= b0 + k < len(b) else (0, 0)
        s += abs(x[0] - y[0]) + abs(x[1] - y[1])
    return s


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    a, b = read_wav(argv[0]), read_wav(argv[1])

    def opt(name, default):
        return float(argv[argv.index(name) + 1]) if name in argv else default

    cut = int(opt("--cut", 0) * 44100 / opt("--fps", 60))
    b = b[cut:]
    quiet, show = int(opt("--quiet", 2000)), int(opt("--show", 0))
    gain = "--gain" in argv
    print(f"{argv[0]}: {len(a)} frames; {argv[1]}: {len(b)} frames after {cut} cut")
    theirs = scenes(b, quiet)
    for n, (s, e) in enumerate(scenes(a, quiet)):
        if n >= len(theirs):
            print(f"scene {n}: at {s}, {e - s} samples, not found in the second file")
            break
        onset, end = theirs[n]
        length = min(e - s, 4096)
        bs = min(range(onset - 8, onset + 9), key=lambda t: cost(a, b, s, t, length))
        g = 1.0
        if gain:
            num = sum(a[s + k][0] * b[bs + k][0] + a[s + k][1] * b[bs + k][1] for k in range(e - s) if 0 <= bs + k < len(b))
            den = sum(a[s + k][0] ** 2 + a[s + k][1] ** 2 for k in range(e - s))
            g = num / den if den else 1.0
        diffs = []
        for k in range(e - s):
            x = tuple(round(v * g) for v in a[s + k])
            y = b[bs + k] if 0 <= bs + k < len(b) else (0, 0)
            diffs.append((abs(x[0] - y[0]), abs(x[1] - y[1]), k, x, y))
        equal = sum(1 for d in diffs if d[0] == 0 and d[1] == 0)
        worst = max(max(d[0], d[1]) for d in diffs)
        mean = sum(d[0] + d[1] for d in diffs) / (2 * len(diffs))
        scaled = f", gain {g:.4f}" if gain else ""
        print(f"scene {n}: at {s} and {bs + cut}, {e - s} and {end - onset} samples, {equal} equal ({100 * equal / (e - s):.1f}%), largest {worst}, mean {mean:.2f}{scaled}")
        for d in [d for d in diffs if d[0] or d[1]][:show]:
            print(f"    +{d[2]}: {d[3]} {d[4]}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
