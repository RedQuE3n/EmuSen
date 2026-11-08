#!/usr/bin/python3 -I
"""The C# painter against ES-DE in each watched state: the mean absolute error of the C# grab against the oracle's
capture over R, G and B (0 to 255), the help bar's strip left out, written as mae.tsv beside this script; and for a
human look, the capture, the grab and their difference side by side. The strip is the rows the state's helpsystem
element covers in its .spec (EmuSen_BigPicture.md 36.3: the help bar's words and icons are Mistress's own). See README.md.

Usage: compare.py
  reads   states.tsv, <theme>-<state>.spec, ~/.cache/emusen/esde-oracle/<theme>/<state>.png,
          ~/.cache/emusen/tmp/qt-s4/compare/csharp/<theme>-<state>.png
  writes  mae.tsv; ~/.cache/emusen/tmp/qt-s4/compare/<theme>-<state>.png (ES-DE | C# | difference)
"""
import math
import os
import sys

from PIL import Image, ImageChops

HERE = os.path.dirname(os.path.abspath(__file__))
HOME = os.path.expanduser("~")
ORACLE = os.path.join(HOME, ".cache", "emusen", "esde-oracle")
COMPARE = os.path.join(HOME, ".cache", "emusen", "tmp", "qt-s4", "compare")
SIZES = [(1280, 800), (1920, 1200)]
STATES = ["system", "gamelist-1"]


def strip(spec, w):
    """The rows, in window pixels, that the spec's helpsystem elements cover; None when it has none."""
    unit, rows = w / 1280.0, []
    for line in open(spec, encoding="utf-8"):
        if line.startswith("element helpsystem_"):
            box = dict(p.split("=") for p in line.split()[2:])
            rows.append((math.floor(float(box["y"]) * unit), math.ceil((float(box["y"]) + float(box["h"])) * unit)))
    return (min(r[0] for r in rows), max(r[1] for r in rows)) if rows else None


def mae(a, b, rows):
    """Mean absolute error over R, G, B of the rows outside the strip."""
    w, h = a.size
    keep = [y for y in range(h) if rows is None or not rows[0] <= y < rows[1]]
    diff = ImageChops.difference(a, b)
    total = 0
    for y in keep:
        total += sum(diff.crop((0, y, w, y + 1)).tobytes())
    return total / (len(keep) * w * 3), diff


def side_by_side(esde, cs, diff, rows, out):
    w, h = esde.size
    shown = diff.point(lambda v: min(255, v * 4))
    if rows is not None:
        shown.paste((0, 0, 96), (0, max(0, rows[0]), w, min(h, rows[1])))
    sheet = Image.new("RGB", (w * 3, h))
    for i, image in enumerate((esde, cs, shown)):
        sheet.paste(image, (w * i, 0))
    sheet.save(out)


def main():
    rows_out = ["theme\tstate\twidth\theight\tstrip_top\tstrip_bottom\tmae"]
    themes = [l.split("\t")[0] for l in open(os.path.join(HERE, "states.tsv"), encoding="utf-8").read().splitlines()[1:] if l]
    for theme in themes:
        for w, h in SIZES:
            for state in STATES:
                name = f"{state}-{w}x{h}"
                spec = os.path.join(HERE, f"{theme}-{name}.spec")
                esde = Image.open(os.path.join(ORACLE, theme, name + ".png")).convert("RGB")
                cs = Image.open(os.path.join(COMPARE, "csharp", f"{theme}-{name}.png")).convert("RGB")
                if esde.size != (w, h) or cs.size != (w, h):
                    sys.exit(f"compare.py: {theme} {name}: ES-DE {esde.size}, C# {cs.size}, not {w}x{h}")
                band = strip(spec, w)
                error, diff = mae(esde, cs, band)
                side_by_side(esde, cs, diff, band, os.path.join(COMPARE, f"{theme}-{name}.png"))
                top, bottom = band if band else ("-", "-")
                rows_out.append(f"{theme}\t{state}\t{w}\t{h}\t{top}\t{bottom}\t{error:.2f}")
                print(rows_out[-1])
    with open(os.path.join(HERE, "mae.tsv"), "w", encoding="utf-8") as f:
        f.write("\n".join(rows_out) + "\n")


if __name__ == "__main__":
    if len(sys.argv) != 1:
        sys.exit(__doc__)
    main()
