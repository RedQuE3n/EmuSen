#!/usr/bin/env python3
"""Renders EmuSen's icon from its two SVG sources into every size and format the platforms use.

emusen.svg draws 48 px and up; emusen-small.svg is the same mark redrawn on a 16-unit grid for 16-32 px,
where the full drawing's thin bars would blur. Needs ImageMagick 7 (`magick`) with librsvg.
Outputs, beside this file: png/emusen-<n>.png, emusen.ico (Windows) and emusen.icns (macOS).
See EmuSen_Settings_Reference.md, "The application icon".
"""
import os
import struct
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
MASTER = os.path.join(HERE, "emusen.svg")
SMALL = os.path.join(HERE, "emusen-small.svg")
PNG_DIR = os.path.join(HERE, "png")
SIZES = [16, 24, 32, 48, 64, 128, 256, 512, 1024]
ICO_SIZES = [16, 24, 32, 48, 64, 128, 256]

# The macOS icon types that hold a PNG, by pixel size (icp4..ic14; the @2x types reuse the larger render).
ICNS_TYPES = [(b"icp4", 16), (b"icp5", 32), (b"icp6", 64), (b"ic07", 128), (b"ic08", 256), (b"ic09", 512),
              (b"ic10", 1024), (b"ic11", 32), (b"ic12", 64), (b"ic13", 256), (b"ic14", 512)]


def png_path(size):
    return os.path.join(PNG_DIR, f"emusen-{size}.png")


def render(size):
    # Drawn at 256 px or more, then area-averaged down, so the small drawing's 16-unit grid lands on whole pixels.
    source = SMALL if size <= 32 else MASTER
    drawn = max(256, size)
    subprocess.run(["magick", "-background", "none", "-density", str(72 * drawn / 256), source,
                    "-filter", "Box", "-resize", f"{size}x{size}", "-strip", f"PNG32:{png_path(size)}"], check=True)


def write_ico():
    subprocess.run(["magick", *[png_path(s) for s in ICO_SIZES], os.path.join(HERE, "emusen.ico")], check=True)


def write_icns():
    chunks = b""
    for kind, size in ICNS_TYPES:
        data = open(png_path(size), "rb").read()
        chunks += kind + struct.pack(">I", len(data) + 8) + data
    with open(os.path.join(HERE, "emusen.icns"), "wb") as f:
        f.write(b"icns" + struct.pack(">I", len(chunks) + 8) + chunks)


def main():
    os.makedirs(PNG_DIR, exist_ok=True)
    for size in SIZES:
        render(size)
    write_ico()
    write_icns()
    print(f"rendered {len(SIZES)} PNGs, emusen.ico ({len(ICO_SIZES)} sizes) and emusen.icns ({len(ICNS_TYPES)} entries)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
