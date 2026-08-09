#!/usr/bin/env python3
"""A framebuffer normalised to RGB triples, so two emulators can be compared.

Raw screen bytes never compare directly: the reference writes one 16-bit palette
index per pixel for the NES, this project writes RGBA, and a libretro core may
write XRGB. All three are read into the same three-bytes-per-pixel row-major
form, and everything above this module works only in that form.

See EmuSen_Debugging_Tools_Reference_v5.md §3.48.

On the palette
--------------
The C# this replaces reached into the emulator core for the NES palette -
deliberately, so there would not be a second copy to keep in step. Crossing to
Python makes that impossible, so the table below is a copy, and the cost is paid
in a different currency: `PythonPaletteTests` in EmuSen.WiseMan parses this file
and asserts the 192 bytes equal `Ppu.NesPalette`. A copy that is checked is not
the same liability as a copy that is trusted, but it is not free either, and this
is the only piece of the analysis port that has to be kept in step by machinery
rather than by having one definition.

Emphasis bits are not applied, matching the core - see Moon_PPU.md §3.5.

On sizes
--------
Every format is read at 256x240 regardless of what the manifest claims, because
that is what the C# did and the verdicts were pinned against it. The manifest's
width and height are ingested into the `screen` table (see dumps-schema.sql) but
are not consulted here. That is a real limitation for any future non-NES-sized
reference and is recorded rather than quietly fixed, since fixing it changes
results.
"""
import array
import sys
from collections import Counter

WIDTH = 256
HEIGHT = 240

# The 64 NTSC colours as RGB triples. Checked against EmuSen.Cores.Nintendo.Moon
# .Video.Ppu.NesPalette by a C# test - see the module docstring.
NES_PALETTE = bytes((
    84, 84, 84,     0, 30, 116,     8, 16, 144,    48, 0, 136,
    68, 0, 100,     92, 0, 48,      84, 4, 0,      60, 24, 0,
    32, 42, 0,      8, 58, 0,       0, 64, 0,      0, 60, 0,
    0, 50, 60,      0, 0, 0,        0, 0, 0,       0, 0, 0,
    152, 150, 152,  8, 76, 196,     48, 50, 236,   92, 30, 228,
    136, 20, 176,   160, 20, 100,   152, 34, 32,   120, 60, 0,
    84, 90, 0,      40, 114, 0,     8, 124, 0,     0, 118, 40,
    0, 102, 120,    0, 0, 0,        0, 0, 0,       0, 0, 0,
    236, 238, 236,  76, 154, 236,   120, 124, 236, 176, 98, 236,
    228, 84, 236,   236, 88, 180,   236, 106, 100, 212, 136, 32,
    160, 170, 0,    116, 196, 0,    76, 208, 32,   56, 204, 108,
    56, 180, 204,   60, 60, 60,     0, 0, 0,       0, 0, 0,
    236, 238, 236,  168, 204, 236,  188, 188, 236, 212, 178, 236,
    236, 174, 236,  236, 174, 212,  236, 180, 176, 228, 196, 144,
    204, 210, 120,  180, 222, 120,  168, 226, 144, 152, 226, 180,
    160, 214, 228,  160, 162, 160,  0, 0, 0,       0, 0, 0,
))

_TRIPLES = [NES_PALETTE[i * 3:(i * 3) + 3] for i in range(64)]


class ScreenImage:

    def __init__(self, width, height, rgb):
        self.width = width
        self.height = height
        self.rgb = rgb
        self._packed = None

    @property
    def packed(self):
        """One 24-bit integer per pixel, built once - only Information() needs it."""
        if self._packed is None:
            rgb = self.rgb
            self._packed = [(rgb[i] << 16) | (rgb[i + 1] << 8) | rgb[i + 2]
                            for i in range(0, len(rgb), 3)]
        return self._packed

    def differing_pixels(self, other):
        """How many pixels differ, counting a size mismatch as everything."""
        if self.width != other.width or self.height != other.height:
            return self.width * self.height
        if self.rgb == other.rgb:
            return 0

        stride = self.width * 3
        mine, theirs = self.rgb, other.rgb
        differing = 0
        for y in range(self.height):
            start = y * stride
            stop = start + stride
            # Whole-row equality first: a differing frame is usually differing in
            # a minority of its rows, and this is a C-level compare.
            if mine[start:stop] == theirs[start:stop]:
                continue
            for i in range(start, stop, 3):
                if (mine[i] != theirs[i] or mine[i + 1] != theirs[i + 1]
                        or mine[i + 2] != theirs[i + 2]):
                    differing += 1
        return differing

    def information(self):
        """(colours, dominant fraction) - how much this frame could possibly prove.

        A near-uniform screen agrees with almost anything, and reporting that as a
        pass is the false positive the vacuity gate exists to stop.
        """
        counts = Counter(self.packed)
        return len(counts), max(counts.values()) / float(self.width * self.height)

    def vertical_shift(self, other, limit=16):
        """The whole-image vertical offset that explains the difference, if one does."""
        stride = self.width * 3
        for dy in range(-limit, limit + 1):
            if dy == 0:
                continue
            matched = 0
            for y in range(self.height):
                source = y + dy
                if source < 0 or source >= self.height:
                    continue
                a = source * stride
                b = y * other.width * 3
                if self.rgb[a:a + stride] == other.rgb[b:b + stride]:
                    matched += 1
            if matched >= self.height - 6:
                return dy
        return None


def read(raw, fmt, width=WIDTH, height=HEIGHT):
    """Decode raw probe bytes, or None when the format is unknown or the blob short."""
    if fmt == "PaletteIndex16":
        return _from_palette_index16(raw, width, height)
    if fmt == "Rgba8888":
        return _from_rgba(raw, width, height)
    if fmt == "Xrgb8888":
        return _from_xrgb(raw, width, height)
    return None


def _from_palette_index16(raw, width, height):
    """The reference's NES buffer: low six bits are the colour, the rest emphasis."""
    pixels = width * height
    if len(raw) < pixels * 2:
        return None

    indices = array.array("H")
    indices.frombytes(bytes(raw[:pixels * 2]))
    if sys.byteorder != "little":
        indices.byteswap()

    return ScreenImage(width, height, b"".join(_TRIPLES[v & 0x3F] for v in indices))


def _from_rgba(raw, width, height):
    pixels = width * height
    if len(raw) < pixels * 4:
        return None
    return ScreenImage(width, height,
                       b"".join(bytes(raw[i:i + 3]) for i in range(0, pixels * 4, 4)))


def _from_xrgb(raw, width, height):
    pixels = width * height
    if len(raw) < pixels * 4:
        return None
    return ScreenImage(width, height,
                       b"".join(bytes((raw[i + 2], raw[i + 1], raw[i]))
                                for i in range(0, pixels * 4, 4)))
