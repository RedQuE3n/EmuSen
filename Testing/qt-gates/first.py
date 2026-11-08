#!/usr/bin/python3 -I
"""A run's start to its first picture, from run.sh's grabs: the program's exec to the first grab with its window (more
than 1% of the screen lit), and to the first grab whose share in the game's colour box passes a threshold (the game's
picture). Each time is the grab's own, so it is late by up to one grab interval, which is printed. See README.md.

Usage: first.py <run-dir> [<threshold>]
"""
import sys

run, threshold = sys.argv[1], float(sys.argv[2]) if len(sys.argv) > 2 else 0.2
start = float(open(f"{run}/start").read())
rows = [line.split() for line in open(f"{run}/grabs.tsv") if line.split()[1] != "-"]
times = [float(r[0]) - start for r in rows]
window = next((t, r[3]) for t, r in zip(times, rows) if float(r[1]) > 0.01)
picture = next((t, r[3]) for t, r in zip(times, rows) if float(r[2]) > threshold)
gaps = sorted(b - a for a, b in zip(times, times[1:]))
print(f"window {window[0]:.3f} s ({window[1]}) picture {picture[0]:.3f} s ({picture[1]}) window-to-picture "
      f"{picture[0] - window[0]:.3f} s; grabs {len(rows)}, interval median {gaps[len(gaps) // 2] * 1000:.0f} ms")
