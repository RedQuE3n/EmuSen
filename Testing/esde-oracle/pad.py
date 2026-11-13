#!/usr/bin/python3 -I
"""A uinput pad, "gate pad" (vendor 1209, product 0001), that ES-DE reads through SDL, pressed by a script of steps:
wait:<s>, tap:<button>, hold:<button>:<s>. Buttons: a b x y l r select start guide up down left right. The pad is
removed when the script ends or the process is stopped. Every program on the desk sees it, so it is refused while
Steam runs (`pgrep -x steam`), and run_esde.sh refuses the run before ES-DE starts. See README.md.

Usage: pad.py <step>...
"""
import fcntl
import os
import signal
import struct
import subprocess
import sys
import time

EV_SYN, EV_KEY, EV_ABS = 0, 1, 3
BUTTONS = {"a": 0x130, "b": 0x131, "x": 0x133, "y": 0x134, "l": 0x136, "r": 0x137, "select": 0x13A, "start": 0x13B,
           "guide": 0x13C, "up": 0x220, "down": 0x221, "left": 0x222, "right": 0x223}
STICKS = (0, 1, 3, 4)  # ABS_X, ABS_Y, ABS_RX, ABS_RY
UI_SET_EVBIT, UI_SET_KEYBIT, UI_SET_ABSBIT, UI_DEV_CREATE, UI_DEV_DESTROY = 0x40045564, 0x40045565, 0x40045567, 0x5501, 0x5502

def steam_running():
    return subprocess.run(["pgrep", "-x", "steam"], stdout=subprocess.DEVNULL).returncode == 0


# A uinput pad is seen by every program on the desk: Steam acts on its presses (it launched a game, 2026-11-08).
if steam_running():
    sys.exit("pad.py: Steam is running and would read this pad; close Steam first (README.md)")

fd = os.open("/dev/uinput", os.O_WRONLY | os.O_NONBLOCK)
for ev in (EV_KEY, EV_ABS):
    fcntl.ioctl(fd, UI_SET_EVBIT, ev)
for code in BUTTONS.values():
    fcntl.ioctl(fd, UI_SET_KEYBIT, code)
for axis in STICKS:
    fcntl.ioctl(fd, UI_SET_ABSBIT, axis)
absmax, absmin = [0] * 64, [0] * 64
for axis in STICKS:
    absmax[axis], absmin[axis] = 32767, -32768
device = struct.pack("80sHHHHI64i64i64i64i", b"gate pad", 0x03, 0x1209, 0x0001, 1, 0, *absmax, *absmin, *([0] * 64), *([0] * 64))
os.write(fd, device)
fcntl.ioctl(fd, UI_DEV_CREATE)


def gone(*_):
    fcntl.ioctl(fd, UI_DEV_DESTROY)
    os.close(fd)
    sys.exit(0)


signal.signal(signal.SIGTERM, gone)
signal.signal(signal.SIGINT, gone)


def send(kind, code, value):
    for t, c, v in ((kind, code, value), (EV_SYN, 0, 0)):
        now = time.time()
        os.write(fd, struct.pack("llHHi", int(now), int(now % 1 * 1e6), t, c, v))


time.sleep(1.0)  # the readers see the pad arrive before its first press
for step in sys.argv[1:]:
    word, *rest = step.split(":")
    if word == "wait":
        time.sleep(float(rest[0]))
    elif word in ("tap", "hold"):
        send(EV_KEY, BUTTONS[rest[0]], 1)
        time.sleep(float(rest[1]) if word == "hold" else 0.1)
        send(EV_KEY, BUTTONS[rest[0]], 0)
        print(f"{time.time():.3f} {step}", flush=True)
gone()
