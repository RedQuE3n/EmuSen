#!/usr/bin/python3 -I
"""A test copy of a theme whose gamelist help is drawn for measuring: four entries with the same disc (a, b, x, y), at
full opacity over the theme's own flat ground, at a given fontSize, the rest of the theme unchanged. Further help
properties of THEMES.md (entryRelativeScale, entrySpacing, iconTextSpacing, letterCase, entries) may be given as
name=value, each replacing the theme's own, so that one capture differs from another by one property. The copy is
written under <out> (outside the repository, never committed as a theme); run_esde.sh takes it with THEMES=<out>. It
settles how ES-DE spaces a help bar's entries when its sizes are fractions of a pixel (EmuSen_Qt.md §4.12).

Usage: helpcopy.py <theme-folder> <out> <name> <fontSize> [property=value...]
"""
import os
import re
import shutil
import sys


PROPERTIES = ("entryRelativeScale", "entrySpacing", "iconTextSpacing", "letterCase", "entries")


def main(src, out, name, size, *properties):
    given = dict(p.split("=", 1) for p in properties)
    unknown = sorted(set(given) - set(PROPERTIES))
    if unknown:
        sys.exit(f"helpcopy.py: not a help property it sets: {', '.join(unknown)}")
    given.setdefault("entries", "a,b,x,y")
    dst = os.path.join(out, name)
    if os.path.exists(dst):
        shutil.rmtree(dst)
    shutil.copytree(src, dst, symlinks=True)
    path = os.path.join(dst, "16-10.xml")
    text = open(path, encoding="utf-8").read()
    block = re.search(r'<helpsystem name="help">.*?</helpsystem>', text, re.S)
    if not block:
        sys.exit("helpcopy.py: no help element in 16-10.xml")
    help_ = block.group(0)
    help_ = re.sub(r"<fontSize>[^<]*</fontSize>", f"<fontSize>{size}</fontSize>", help_)
    help_ = re.sub(r"<opacity>[^<]*</opacity>", "<opacity>1</opacity>", help_)
    for prop, value in given.items():
        help_ = re.sub(rf"\s*<{prop}>[^<]*</{prop}>", "", help_)
        help_ = help_.replace("</helpsystem>", f"\t<{prop}>{value}</{prop}>\n\t\t</helpsystem>")
    open(path, "w", encoding="utf-8").write(text[: block.start()] + help_ + text[block.end():])
    print(dst)


if __name__ == "__main__":
    if len(sys.argv) < 5 or any("=" not in a for a in sys.argv[5:]):
        sys.exit(__doc__)
    main(*sys.argv[1:])
