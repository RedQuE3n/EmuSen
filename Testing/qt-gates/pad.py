#!/usr/bin/python3 -I
"""A uinput pad, "gate pad" (vendor 1209, product 0001), that both programs read through SDL, pressed by a script of
steps: wait:<s>, tap:<button>, hold:<button>:<s>. Buttons: a b x y l r select start guide up down left right. The pad
is removed when the script ends or the process is stopped. See README.md.

Usage: pad.py <step>...
"""
import fcntl
import os
import signal
import struct
import sys
import time

EV_SYN, EV_KEY, EV_ABS = 0, 1, 3
BUTTONS = {"a": 0x130, "b": 0x131, "x": 0x133, "y": 0x134, "l": 0x136, "r": 0x137, "select": 0x13A, "start": 0x13B,
           "guide": 0x13C, "up": 0x220, "down": 0x221, "left": 0x222, "right": 0x223}
STICKS = (0, 1, 3, 4)  # ABS_X, ABS_Y, ABS_RX, ABS_RY
UI_SET_EVBIT, UI_SET_KEYBIT, UI_SET_ABSBIT, UI_DEV_CREATE, UI_DEV_DESTROY = 0x40045564, 0x40045565, 0x40045567, 0x5501, 0x5502

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


def settle(tsv, still):
    """Waits until the window is lit (over 1% of the screen) and its lit share has not changed for `still` seconds in the
    polled grabs of run.sh (grabs.tsv), or 15 s; prints the epoch time it stopped waiting."""
    give_up = time.time() + 15
    while time.time() < give_up:
        try:
            rows = [line.split() for line in open(tsv) if line.split() and line.split()[1] != "-"]
        except OSError:
            rows = []
        lit = [(float(r[0]), float(r[1])) for r in rows if float(r[1]) > 0.01]
        if lit:
            changed = lit[0][0]
            for (_, a), (t, b) in zip(lit, lit[1:]):
                if abs(a - b) > 0.0005:
                    changed = t
            if lit[-1][0] - changed >= still:
                break
        time.sleep(0.02)
    print(f"{time.time():.3f} {step} (stopped waiting)", flush=True)


time.sleep(1.0)  # the readers see the pad arrive before its first press
print("ready", flush=True)
for step in sys.argv[1:]:
    word, *rest = step.split(":")
    if word == "wait":
        time.sleep(float(rest[0]))
    elif word == "settle":
        settle(rest[0], float(rest[1]))
    elif word in ("tap", "hold"):
        send(EV_KEY, BUTTONS[rest[0]], 1)
        time.sleep(float(rest[1]) if word == "hold" else 0.1)
        send(EV_KEY, BUTTONS[rest[0]], 0)
        print(f"{time.time():.3f} {step}", flush=True)
gone()
