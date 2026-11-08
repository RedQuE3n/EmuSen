#!/usr/bin/python3 -I
"""One round's figures from gates.sh's runs: Home's first picture (the first grab with the window lit), the first press
(from the settled window to the focus moving), memory 10 s after Home in the idle run, and memory at the Pause Card
(median of the samples 3 to 6 s after the chord). See README.md.

Usage: figures.py <runs-dir> <round>
"""
import statistics
import sys

runs, n = sys.argv[1], sys.argv[2]


def home(run):
    start = float(open(f"{run}/start").read())
    rows = [l.split() for l in open(f"{run}/grabs.tsv") if l.split()[1] != "-"]
    final = float(rows[len(rows) // 2][1])
    first = next(float(r[0]) for r in rows if float(r[1]) > 0.01)
    full = next(float(r[0]) for r in rows if abs(float(r[1]) - final) < 0.005)
    return start, first, full


def rss(run):
    return [tuple(map(float, l.split())) for l in open(f"{run}/rss.tsv")]


start, first, full = home(f"{runs}/start-{n}")
print(f"home: window {first - start:.3f} s, picture whole {full - start:.3f} s")
start, window, full = home(f"{runs}/idle-{n}")
at = [r for r in rss(f"{runs}/idle-{n}") if r[0] >= window + 10]
print(f"idle: home {window - start:.3f} s; RSS 10 s after {at[0][1] / 1024:.0f} MiB, anonymous {at[0][2] / 1024:.0f} MiB")
run = f"{runs}/pause-{n}"
chord = next(float(l.split()[0]) for l in open(f"{run}/pad.log") if "hold:guide" in l)
at = [r for r in rss(run) if chord + 3 <= r[0] <= chord + 6]
print(f"pause card: RSS {statistics.median(r[1] for r in at) / 1024:.0f} MiB, anonymous {statistics.median(r[2] for r in at) / 1024:.0f} MiB ({len(at)} samples)")
