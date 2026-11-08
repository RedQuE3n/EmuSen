#!/usr/bin/python3 -I
"""Polled grabs of a gate's own compositor through KWin's ScreenShot2 (CaptureWorkspace, raw pixels down a pipe), each
kept as a small PNG (full size with FULL=1) and measured: the share of pixels that are not black, and the share in a
colour box (the game's own, given as r0-r1,g0-g1,b0-b1). One line a grab to stdout: wall-clock time, the two shares, the PNG's name.
See README.md.

Usage: grab.py <out-dir> <seconds> <interval-ms> [<colour-box>]
"""
import os
import sys
import time

import dbus
from PIL import Image

out, seconds, interval = sys.argv[1], float(sys.argv[2]), int(sys.argv[3]) / 1000
box = [tuple(int(v) for v in part.split("-")) for part in (sys.argv[4] if len(sys.argv) > 4 else "0-255,0-255,0-255").split(",")]
os.makedirs(out, exist_ok=True)
shots = dbus.Interface(dbus.SessionBus().get_object("org.kde.KWin", "/org/kde/KWin/ScreenShot2"), "org.kde.KWin.ScreenShot2")


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
    return Image.frombuffer("RGBA", (width, height), bytes(data), "raw", "BGRA", stride, 1)


def shares(image):
    small = image.convert("RGB").resize((192, 120), Image.NEAREST)
    pixels = list(small.get_flattened_data()) if hasattr(small, "get_flattened_data") else list(small.getdata())
    lit = sum(1 for p in pixels if max(p) > 24) / len(pixels)
    inside = sum(1 for p in pixels if all(lo <= c <= hi for c, (lo, hi) in zip(p, box))) / len(pixels)
    return lit, inside


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
    kept = image.convert("RGB") if os.environ.get("FULL") == "1" else image.convert("RGB").resize((480, 300))
    kept.save(os.path.join(out, name), compress_level=1)
    print(f"{now:.3f} {lit:.3f} {inside:.3f} {name}", flush=True)
    n += 1
    time.sleep(max(0.0, interval - (time.time() - now)))
