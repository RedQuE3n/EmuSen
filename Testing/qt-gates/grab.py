#!/usr/bin/python3 -I
"""Polled grabs of a gate's own compositor through KWin's ScreenShot2 (CaptureWorkspace, raw pixels down a pipe), each
kept as a small PNG (full size with FULL=1) and measured: the share of pixels that are not black, and the share in a
colour box (the game's own, given as r0-r1,g0-g1,b0-b1). One line a grab to stdout: wall-clock time, the two shares, the PNG's name.
The pictures are scaled by GdkPixbuf, which the desktop and the handheld both carry, and written as PNG here, since the
handheld's GdkPixbuf saves through a sandboxed loader that cannot start inside the handheld's box. See README.md.

Usage: grab.py <out-dir> <seconds> <interval-ms> [<colour-box>]
"""
import os
import struct
import sys
import time
import zlib

import dbus
import gi

gi.require_version("GdkPixbuf", "2.0")
from gi.repository import GdkPixbuf, GLib  # noqa: E402

out, seconds, interval = sys.argv[1], float(sys.argv[2]), int(sys.argv[3]) / 1000
box = [tuple(int(v) for v in part.split("-")) for part in (sys.argv[4] if len(sys.argv) > 4 else "0-255,0-255,0-255").split(",")]
os.makedirs(out, exist_ok=True)
shots = dbus.Interface(dbus.SessionBus().get_object("org.kde.KWin", "/org/kde/KWin/ScreenShot2"), "org.kde.KWin.ScreenShot2")
# Byte tables giving 1 where a channel passes: dark (24 or under), and inside the box per channel (B, G, R as KWin sends them).
DARK = bytes(int(v <= 24) for v in range(256))
INSIDE = [bytes(int(lo <= v <= hi) for v in range(256)) for lo, hi in reversed(box)]


def grab():
    r, w = os.pipe()
    try:
        meta = shots.CaptureWorkspace({"native-resolution": True}, dbus.types.UnixFd(w))
    finally:
        os.close(w)
    data = bytearray()
    while chunk := os.read(r, 1 << 20):
        data += chunk
    os.close(r)
    width, height, stride = int(meta["width"]), int(meta["height"]), int(meta["stride"])
    return GdkPixbuf.Pixbuf.new_from_bytes(GLib.Bytes.new(bytes(data)), GdkPixbuf.Colorspace.RGB, True, 8, width, height, stride)


def pixels(image, width, height, interp):
    """BGRA bytes at a size, rows packed."""
    small = image.scale_simple(width, height, interp)
    raw, stride = small.get_pixels(), small.get_rowstride()
    return b"".join(raw[y * stride:y * stride + width * 4] for y in range(height))


def ones(channels):
    """How many pixels have every channel's byte 1, from each channel's 0/1 bytes."""
    joined = int.from_bytes(channels[0], "little")
    for c in channels[1:]:
        joined &= int.from_bytes(c, "little")
    return joined.bit_count()


def shares(image):
    small = pixels(image, 192, 120, GdkPixbuf.InterpType.NEAREST)
    n = len(small) // 4
    dark = ones([small[i::4].translate(DARK) for i in range(3)])
    inside = ones([small[i::4].translate(INSIDE[i]) for i in range(3)])
    return (n - dark) / n, inside / n


def save(image, path):
    full = os.environ.get("FULL") == "1"
    w, h = (image.get_width(), image.get_height()) if full else (480, 300)
    bgra = pixels(image, w, h, GdkPixbuf.InterpType.BILINEAR)
    rgb = bytearray(w * h * 3)
    rgb[0::3], rgb[1::3], rgb[2::3] = bgra[2::4], bgra[1::4], bgra[0::4]
    rows = b"".join(b"\x00" + bytes(rgb[y * w * 3:(y + 1) * w * 3]) for y in range(h))
    chunk = lambda kind, data: struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(rows, 1)) + chunk(b"IEND", b""))


end, n = time.time() + seconds, 0
while (now := time.time()) < end:
    try:
        image = grab()
    except dbus.DBusException as e:
        print(f"{now:.3f} - - none {e.get_dbus_name()}", flush=True)
        time.sleep(interval)
        continue
    lit, inside = shares(image)
    name = f"{n:04d}.png"
    save(image, os.path.join(out, name))
    print(f"{now:.3f} {lit:.3f} {inside:.3f} {name}", flush=True)
    n += 1
    time.sleep(max(0.0, interval - (time.time() - now)))
