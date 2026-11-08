#!/usr/bin/python3 -I
"""A run's first press: from run.sh's start to the settled window (pad.log's "settle" line), to the press (the pad's
"tap" line, printed 0.1 s after the button went down) and to the first grab whose picture differs from the last grab
before the press (file hash), the focus ring having moved. The grabs are late by up to one grab interval.
See README.md.

Usage: press.py <run-dir>
"""
import hashlib
import sys

run = sys.argv[1]
start = float(open(f"{run}/start").read())
pad = [line.split() for line in open(f"{run}/pad.log")]
settled = next(float(p[0]) for p in pad if "settle" in " ".join(p))
down = next(float(p[0]) for p in pad if len(p) > 1 and p[1].startswith("tap:")) - 0.1
rows = [line.split() for line in open(f"{run}/grabs.tsv") if line.split()[1] != "-"]
digest = lambda name: hashlib.md5(open(f"{run}/grabs/{name}", "rb").read()).hexdigest()
before = [r for r in rows if float(r[0]) < down][-1]
changed = next((r for r in rows if float(r[0]) >= down and digest(r[3]) != digest(before[3])), None)
print(f"settled {settled - start:.3f} s, pressed {down - start:.3f} s" + (
    f", the picture changed at {float(changed[0]) - start:.3f} s: answered in {float(changed[0]) - down:.3f} s" if changed
    else ", the picture never changed"))
