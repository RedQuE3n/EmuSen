#!/usr/bin/python3 -I
"""A test copy of a theme whose gamelist help is drawn for measuring: four entries with the same disc (a, b, x, y), at
full opacity over the theme's own flat ground, at a given fontSize, the rest of the theme unchanged. The copy is written
under <out> (outside the repository, never committed as a theme); run_esde.sh takes it with THEMES=<out>. It settles
how ES-DE spaces a help bar's entries when the icon's box is a fraction of a pixel (EmuSen_Qt.md §4.12).

Usage: helpcopy.py <theme-folder> <out> <name> <fontSize>
"""
import os
import re
import shutil
import sys


def main(src, out, name, size):
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
    help_ = help_.replace("</helpsystem>", "\t<entries>a,b,x,y</entries>\n\t\t</helpsystem>")
    open(path, "w", encoding="utf-8").write(text[: block.start()] + help_ + text[block.end():])
    print(dst)


if __name__ == "__main__":
    if len(sys.argv) != 5:
        sys.exit(__doc__)
    main(*sys.argv[1:])
