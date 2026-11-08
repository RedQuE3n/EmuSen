#!/usr/bin/python3 -I
"""Keeps a run's capture: the last full-size grab qt-gates' grab.py took while ES-DE still ran (before <seconds> after
its start, less a margin), copied to <out>. Prints the grab, its time after the start, its size, and its mean absolute
difference from the grab before (0 when the picture had settled). See README.md.

Usage: pick.py <run-dir> <seconds> <out.png>
"""
import os
import shutil
import sys

from PIL import Image, ImageChops, ImageStat

MARGIN_S = 0.5


def pick(run, seconds):
    start = float(open(os.path.join(run, "start")).read())
    taken = []
    for line in open(os.path.join(run, "grabs.tsv")):
        when, _, _, name = line.split()[:4]
        if name != "none" and float(when) < start + seconds - MARGIN_S:
            taken.append((float(when) - start, os.path.join(run, "grabs", name)))
    if len(taken) < 2:
        raise SystemExit(f"pick.py: {len(taken)} grabs while ES-DE ran in {run}")
    (_, before), (after_s, last) = taken[-2], taken[-1]
    a, b = Image.open(before).convert("RGB"), Image.open(last).convert("RGB")
    settled = sum(ImageStat.Stat(ImageChops.difference(a, b)).mean) / 3
    return last, after_s, b.size, settled


if __name__ == "__main__":
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    last, after_s, size, settled = pick(sys.argv[1], float(sys.argv[2]))
    os.makedirs(os.path.dirname(os.path.abspath(sys.argv[3])), exist_ok=True)
    shutil.copyfile(last, sys.argv[3])
    print(f"{os.path.basename(last)} at {after_s:.1f} s, {size[0]}x{size[1]}, difference from the grab before {settled:.3f}")
